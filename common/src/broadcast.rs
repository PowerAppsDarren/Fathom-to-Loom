//! Shared broadcasting service for real-time queue updates
//! This module provides a centralized service for broadcasting queue changes
//! between the backend API and worker processes.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;

/// Represents a queue update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueUpdate {
    pub update_type: QueueUpdateType,
    pub affected_user_id: Option<String>,
    pub global_position: Option<usize>,
    pub task_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of queue updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueUpdateType {
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskRetried,
    PositionUpdated,
    QueueCleared,
}

/// Shared broadcasting service
pub struct BroadcastService {
    sender: broadcast::Sender<QueueUpdate>,
}

impl BroadcastService {
    /// Create a new broadcast service with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to queue updates
    pub fn subscribe(&self) -> broadcast::Receiver<QueueUpdate> {
        self.sender.subscribe()
    }

    /// Broadcast a queue update
    pub async fn broadcast(&self, update: QueueUpdate) {
        if let Err(e) = self.sender.send(update.clone()) {
            error!("Failed to broadcast queue update: {}", e);
        } else {
            info!("Broadcasted queue update: {:?}", update.update_type);
        }
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for BroadcastService {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Factory for creating shared broadcast services
pub struct BroadcastServiceFactory;

impl BroadcastServiceFactory {
    /// Create a shared broadcast service instance
    pub fn create_shared(capacity: usize) -> Arc<BroadcastService> {
        Arc::new(BroadcastService::new(capacity))
    }
}
