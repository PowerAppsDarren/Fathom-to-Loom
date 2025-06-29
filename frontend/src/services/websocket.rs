use futures::{SinkExt, StreamExt};
use ws_stream_wasm::{WsMeta, WsMessage};
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::time::Duration;
use crate::config::get_config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueUpdate {
    pub update_type: String,
    pub queue: Vec<crate::services::api::Meeting>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub worker_id: String,
    pub status: String,
    pub progress: f32,
    pub current_task: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    QueueUpdate(QueueUpdate),
    WorkerStatus(WorkerStatus),
    Ping,
    Pong,
}

pub struct WebSocketService {
    connection: Option<WsMeta>,
    url: String,
}

impl WebSocketService {
    pub fn new() -> Result<Self> {
        let config = get_config().ok_or_else(|| anyhow!("Configuration not loaded"))?;
        let ws_url = config.api.base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = format!("{}/queue_updates", ws_url);
        
        Ok(Self {
            connection: None,
            url,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        if self.connection.is_some() {
            return Ok(());
        }

        let (ws, wsio) = WsMeta::connect(&self.url, None).await
            .map_err(|e| anyhow!("Failed to connect to WebSocket: {:?}", e))?;

        self.connection = Some(ws);
        
        // Spawn background task to handle incoming messages
        wasm_bindgen_futures::spawn_local(async move {
            let (_sink, mut stream) = wsio.split();
            
            while let Some(msg) = stream.next().await {
                match msg {
                    WsMessage::Text(text) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                            tracing::info!("Received WebSocket message: {:?}", ws_msg);
                            // TODO: Send message to subscribers
                        }
                    }
                    WsMessage::Binary(_) => {
                        tracing::warn!("Received unexpected binary WebSocket message");
                    }
                    // Note: WsMessage doesn't have Error variant, errors come from stream.next()
                }
            }
            
            tracing::info!("WebSocket connection closed");
        });

        Ok(())
    }

    pub async fn disconnect(&mut self) {
        if let Some(connection) = self.connection.take() {
            let _ = connection.close().await;
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub async fn send_message(&mut self, message: WebSocketMessage) -> Result<()> {
        if let Some(_connection) = &self.connection {
            let text = serde_json::to_string(&message)
                .map_err(|e| anyhow!("Failed to serialize message: {}", e))?;
            
            // TODO: Send message through connection
            tracing::info!("Sending WebSocket message: {}", text);
            Ok(())
        } else {
            Err(anyhow!("WebSocket not connected"))
        }
    }

    pub async fn reconnect_with_backoff(&mut self, max_retries: u32) -> Result<()> {
        let mut retries = 0;
        let mut delay = Duration::from_millis(1000);

        while retries < max_retries {
            match self.connect().await {
                Ok(()) => {
                    tracing::info!("WebSocket reconnected successfully");
                    return Ok(());
                }
                Err(e) => {
                    retries += 1;
                    tracing::warn!("WebSocket reconnection attempt {} failed: {}", retries, e);
                    
                    if retries < max_retries {
                        TimeoutFuture::new(delay.as_millis() as u32).await;
                        delay = Duration::from_millis((delay.as_millis() as u64 * 2).min(30000));
                    }
                }
            }
        }

        Err(anyhow!("Failed to reconnect after {} attempts", max_retries))
    }
}
