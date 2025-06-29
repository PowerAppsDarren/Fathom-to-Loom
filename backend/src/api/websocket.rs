use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Query,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api::queue::Meeting;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueUpdate {
    pub update_type: QueueUpdateType,
    pub queue: Vec<Meeting>,
    pub affected_user_id: Option<String>,
    pub global_position: Option<usize>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueUpdateType {
    MeetingAdded,
    MeetingRemoved,
    PositionUpdated,
    QueueCleared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub meeting_id: Uuid,
    pub progress_type: ProgressType,
    pub details: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressType {
    Processing,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    QueueUpdate(QueueUpdate),
    ProgressUpdate(ProgressUpdate),
    Ping,
    Pong,
}

/// WebSocket connection manager
pub struct WebSocketManager {
    queue_sender: broadcast::Sender<QueueUpdate>,
    progress_sender: broadcast::Sender<ProgressUpdate>,
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    user_id: String,
    connected_at: chrono::DateTime<chrono::Utc>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (queue_sender, _) = broadcast::channel(1000);
        let (progress_sender, _) = broadcast::channel(1000);
        
        Self {
            queue_sender,
            progress_sender,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create WebSocket manager with external broadcast service integration
    pub fn with_external_broadcast(external_broadcast: Arc<common::broadcast::BroadcastService>) -> Self {
        let manager = Self::new();
        
        // Spawn task to forward external broadcasts to WebSocket clients
        let queue_sender = manager.queue_sender.clone();
        tokio::spawn(async move {
            let mut rx = external_broadcast.subscribe();
            while let Ok(update) = rx.recv().await {
                // Convert external broadcast format to WebSocket format
                let ws_update = QueueUpdate {
                    update_type: match update.update_type {
                        common::broadcast::QueueUpdateType::TaskStarted => QueueUpdateType::PositionUpdated,
                        common::broadcast::QueueUpdateType::TaskCompleted => QueueUpdateType::MeetingRemoved,
                        common::broadcast::QueueUpdateType::TaskFailed => QueueUpdateType::PositionUpdated,
                        common::broadcast::QueueUpdateType::TaskRetried => QueueUpdateType::PositionUpdated,
                        common::broadcast::QueueUpdateType::PositionUpdated => QueueUpdateType::PositionUpdated,
                        common::broadcast::QueueUpdateType::QueueCleared => QueueUpdateType::QueueCleared,
                    },
                    queue: vec![], // Will be populated by the queue state
                    affected_user_id: update.affected_user_id,
                    global_position: update.global_position,
                    timestamp: update.timestamp,
                };
                
                if let Err(e) = queue_sender.send(ws_update) {
                    error!("Failed to forward external broadcast to WebSocket clients: {}", e);
                }
            }
        });
        
        manager
    }

    /// Handle new WebSocket connection
    pub async fn handle_socket(
        &self,
        socket: WebSocket,
        user_id: String,
    ) {
        let connection_id = Uuid::new_v4().to_string();
        let user_id_clone = user_id.clone();
        
        // Register connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), ConnectionInfo {
                user_id: user_id.clone(),
                connected_at: chrono::Utc::now(),
            });
        }
        
        info!("New WebSocket connection established for user: {}", user_id);

        let (sender, mut receiver) = socket.split();
        
        // Subscribe to updates
        let mut queue_rx = self.queue_sender.subscribe();
        let mut progress_rx = self.progress_sender.subscribe();
        
        // Clone connection manager for cleanup
        let connections_cleanup = Arc::clone(&self.connections);
        let connection_id_cleanup = connection_id.clone();

        // Use channels to communicate between tasks
        let (pong_tx, mut pong_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
        let sender = Arc::new(tokio::sync::Mutex::new(sender));
        let sender_clone = Arc::clone(&sender);

        // Spawn task to handle incoming messages
        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Handle incoming text messages (like ping/pong)
                        if text == "ping" {
                            if pong_tx.send(Message::Text("pong".to_string())).is_err() {
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket connection closed by client");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Spawn task to handle outgoing messages
        let outgoing_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    pong_msg = pong_rx.recv() => {
                        if let Some(msg) = pong_msg {
                            let mut sender_guard = sender_clone.lock().await;
                            if sender_guard.send(msg).await.is_err() {
                                break;
                            }
                        }
                    }
                    queue_update = queue_rx.recv() => {
                        match queue_update {
                            Ok(update) => {
                                // Filter updates by user_id and global position
                                let should_send = match (&update.affected_user_id, &update.global_position) {
                                    // If update has specific user_id, only send to that user or show global view
                                    (Some(affected_user), _) => {
                                        affected_user == &user_id_clone || user_id_clone == "admin" // Admin sees all
                                    },
                                    // If no specific user but has position info, send to all (global updates)
                                    (None, Some(_)) => true,
                                    // Send all other updates
                                    (None, None) => true,
                                };
                                
                                if should_send {
                                    let message = WebSocketMessage::QueueUpdate(update);
                                    if let Ok(json) = serde_json::to_string(&message) {
                                        let mut sender_guard = sender_clone.lock().await;
                                        if sender_guard.send(Message::Text(json)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                warn!("WebSocket lagged behind on queue updates");
                            }
                            Err(_) => break,
                        }
                    }
                    progress_update = progress_rx.recv() => {
                        match progress_update {
                            Ok(update) => {
                                let message = WebSocketMessage::ProgressUpdate(update);
                                if let Ok(json) = serde_json::to_string(&message) {
                                    let mut sender_guard = sender_clone.lock().await;
                                    if sender_guard.send(Message::Text(json)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                warn!("WebSocket lagged behind on progress updates");
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });

        // Wait for either task to complete (connection closed)
        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }

        // Cleanup connection
        {
            let mut connections = connections_cleanup.write().await;
            connections.remove(&connection_id_cleanup);
        }
        
        info!("WebSocket connection cleaned up for user: {}", user_id);
    }

    /// Broadcast queue update to all connected clients
    pub async fn broadcast_queue_update(&self, update: QueueUpdate) {
        if let Err(e) = self.queue_sender.send(update) {
            error!("Failed to broadcast queue update: {}", e);
        }
    }

    /// Broadcast progress update to all connected clients
    pub async fn broadcast_progress_update(&self, update: ProgressUpdate) {
        if let Err(e) = self.progress_sender.send(update) {
            error!("Failed to broadcast progress update: {}", e);
        }
    }

    /// Get number of active connections
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

#[derive(Deserialize)]
pub struct WebSocketQuery {
    pub user_id: Option<String>,
    pub token: Option<String>,
}

/// WebSocket upgrade handler with user authentication
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    State(app_state): State<crate::api::AppState>,
) -> Response {
    // Extract user_id from query params or authentication token
    let user_id = match (params.user_id, params.token) {
        (Some(uid), _) => uid,
        (None, Some(token)) => {
            // TODO: Validate token and extract user_id
            // For now, use a placeholder
            format!("user_from_token_{}", token.chars().take(8).collect::<String>())
        },
        (None, None) => "anonymous".to_string(),
    };
    
    let ws_manager = app_state.ws_manager.clone();
    
    ws.on_upgrade(move |socket| async move {
        ws_manager.handle_socket(socket, user_id).await
    })
}
