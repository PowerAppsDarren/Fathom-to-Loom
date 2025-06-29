use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use crate::api::websocket::{QueueUpdate, QueueUpdateType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::pocketbase_manager::PocketBaseManager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Meeting {
    pub id: Uuid,
    pub user_id: String,
    pub topic: String,
    pub position: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetingRequest {
    pub user_id: String,
    pub topic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<Vec<Meeting>>,
}

/// In-memory storage for meetings queue (replace with persistent storage in prod)
type MeetingsQueue = Arc<RwLock<Vec<Meeting>>>;

/// Create router for queue management
pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/queue", post(add_meetings))
        .route("/queue", get(get_queue))
        .route("/queue/:id", delete(remove_meeting))
}

/// POST /api/queue - Add meetings to the queue
pub async fn add_meetings(
    axum::extract::State(app_state): axum::extract::State<crate::api::AppState>,
    Json(payload): Json<MeetingRequest>,
) -> Result<Json<QueueResponse>, StatusCode> {
    let queue = &app_state.meetings_queue;
    let mut queue = queue.write().await;
    let position = queue.len() + 1;

    let meeting = Meeting {
        id: Uuid::new_v4(),
        user_id: payload.user_id.clone(),
        topic: payload.topic.clone(),
        position,
    };

    queue.push(meeting.clone());
    let queue_clone = queue.clone();
    drop(queue); // Release the write lock before broadcasting
    
    // Broadcast update to WebSocket clients
    app_state.ws_manager.broadcast_queue_update(QueueUpdate {
        update_type: QueueUpdateType::MeetingAdded,
        queue: queue_clone.clone(),
        affected_user_id: Some(meeting.user_id.clone()),
        global_position: Some(meeting.position),
        timestamp: chrono::Utc::now(),
    }).await;

    Ok(Json(QueueResponse {
        success: true,
        message: "Meeting added to queue".into(),
        data: Some(queue_clone),
    }))
}

/// GET /api/queue - Get all meetings in the queue
pub async fn get_queue(
    axum::extract::State(app_state): axum::extract::State<crate::api::AppState>
) -> Result<Json<QueueResponse>, StatusCode> {
    let queue = &app_state.meetings_queue;
    let queue = queue.read().await;
    Ok(Json(QueueResponse {
        success: true,
        message: "Current queue".into(),
        data: Some(queue.clone()),
    }))
}

/// DELETE /api/queue/:id - Remove a meeting from the queue
pub async fn remove_meeting(
    axum::extract::State(app_state): axum::extract::State<crate::api::AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<QueueResponse>, StatusCode> {
    let queue = &app_state.meetings_queue;
    let mut queue = queue.write().await;
    if let Some(pos) = queue.iter().position(|m| m.id == id) {
        let removed_meeting = queue.remove(pos);
        for (i, meeting) in queue.iter_mut().enumerate() {
            meeting.position = i + 1;
        }
        
        let queue_clone = queue.clone();
        drop(queue); // Release the write lock before broadcasting
        
        // Broadcast update to WebSocket clients
        app_state.ws_manager.broadcast_queue_update(QueueUpdate {
            update_type: QueueUpdateType::MeetingRemoved,
            queue: queue_clone.clone(),
            affected_user_id: Some(removed_meeting.user_id.clone()),
            global_position: Some(pos + 1), // Previous position
            timestamp: chrono::Utc::now(),
        }).await;

        Ok(Json(QueueResponse {
            success: true,
            message: "Meeting removed from queue".into(),
            data: Some(queue_clone),
        }))
    } else {
        let queue_clone = queue.clone();
        drop(queue);
        
        Ok(Json(QueueResponse {
            success: false,
            message: "Meeting not found".into(),
            data: Some(queue_clone),
        }))
    }
}
