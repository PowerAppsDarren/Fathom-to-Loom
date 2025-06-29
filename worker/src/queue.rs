use std::time::Duration;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use common::{JobStatus, User};
use common::broadcast::{BroadcastService, QueueUpdate, QueueUpdateType};
use std::sync::Arc;
use crate::{WorkerConfig, WorkerResult, WorkerError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueTask {
    pub id: Uuid,
    pub user_id: String,
    pub meeting_id: String,
    pub topic: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl Default for QueueTask {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id: String::new(),
            meeting_id: String::new(),
            topic: String::new(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            retry_count: 0,
            max_retries: 3,
            error_message: None,
        }
    }
}

/// Entry point for processing a single task in the queue
pub async fn process_task(
    config: &WorkerConfig, 
    user: &User, 
    broadcast_service: Arc<BroadcastService>
) -> WorkerResult<()> {
    loop {
        if let Some(task) = claim_oldest_unclaimed_task().await? {
            // Broadcast task started
            broadcast_service.broadcast(QueueUpdate {
                update_type: QueueUpdateType::TaskStarted,
                affected_user_id: Some(task.user_id.clone()),
                global_position: None, // Will be updated based on queue position
                task_id: Some(task.id),
                timestamp: Utc::now(),
            }).await;
            
            update_status(TaskStatus::InProgress).await?;

            let result = process_pipeline(config, &task, broadcast_service.clone()).await;

            match result {
                Ok(_) => {
                    update_status(TaskStatus::Completed).await?;
                    
                    // Broadcast task completed
                    broadcast_service.broadcast(QueueUpdate {
                        update_type: QueueUpdateType::TaskCompleted,
                        affected_user_id: Some(task.user_id.clone()),
                        global_position: None,
                        task_id: Some(task.id),
                        timestamp: Utc::now(),
                    }).await;
                }
                Err(e) => {
                    update_status(TaskStatus::Failed).await?;
                    
                    // Broadcast task failed
                    broadcast_service.broadcast(QueueUpdate {
                        update_type: QueueUpdateType::TaskFailed,
                        affected_user_id: Some(task.user_id.clone()),
                        global_position: None,
                        task_id: Some(task.id),
                        timestamp: Utc::now(),
                    }).await;
                    
                    return_task_to_queue(&task).await?;
                    email_user_failure(&e, user).await?;
                }
            }
        } else {
            sleep(Duration::from_secs(2)).await;
        }
    }
}

async fn claim_oldest_unclaimed_task() -> WorkerResult<Option<QueueTask>> {
    // Placeholder: simulate task retrieval
    Ok(Some(QueueTask::default()))
}

async fn return_task_to_queue(task: &QueueTask) -> WorkerResult<()> {
    // Placeholder: implement queue return logic
    Ok(())
}

async fn update_status(status: TaskStatus) -> WorkerResult<()> {
    // Placeholder: implement status update logic
    Ok(())
}

async fn email_user_failure(error: &WorkerError, user: &User) -> WorkerResult<()> {
    // Placeholder: implement email logic
    Ok(())
}

async fn process_pipeline(
    config: &WorkerConfig, 
    task: &QueueTask, 
    _broadcast_service: Arc<BroadcastService>
) -> WorkerResult<()> {
    // Simulate metadata fetching
    fetch_meeting_data(task).await?;
    store_metadata_in_user_db(task).await?;
    
    // Simulate video processing
    download_video(task).await?;
    upload_to_loom(task).await?;

    Ok(())
}

async fn fetch_meeting_data(task: &QueueTask) -> WorkerResult<()> {
    // Placeholder for meeting data fetching
    Ok(())
}

async fn store_metadata_in_user_db(task: &QueueTask) -> WorkerResult<()> {
    // Placeholder for metadata storage
    Ok(())
}

async fn download_video(task: &QueueTask) -> WorkerResult<()> {
    // Placeholder for video downloading
    Ok(())
}

async fn upload_to_loom(task: &QueueTask) -> WorkerResult<()> {
    // Placeholder for uploading to Loom
    Ok(())
}
