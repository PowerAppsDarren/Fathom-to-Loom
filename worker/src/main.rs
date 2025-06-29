mod config;
mod queue;

use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use common::broadcast::BroadcastServiceFactory;
use std::sync::Arc;

use config::WorkerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Load configuration
    let config = WorkerConfig::from_env()?;

    // Initialize tracing with level from config
    let log_level = match config.logging.level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => {
            warn!("Invalid log level '{}', defaulting to 'info'", config.logging.level);
            tracing::Level::INFO
        }
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::LevelFilter::from_level(log_level))
        .init();

    info!("Worker configuration loaded successfully");
    info!("Log level: {}", config.logging.level);
    info!("Database URL: {}", config.database.url);
    info!("Worker concurrency: {}", config.worker.concurrency);
    info!("Queue concurrency: {}", config.worker.queue_concurrency);
    info!("Poll interval: {}s", config.worker.poll_interval);

    info!("Starting Fathom to Loom worker");
    
    // Initialize shared broadcast service
    let broadcast_service = BroadcastServiceFactory::create_shared(1000);
    info!("Broadcast service initialized");

    // Main worker loop
    loop {
        match process_tasks(&config, broadcast_service.clone()).await {
            Ok(_) => {
                info!("Worker cycle completed successfully");
            }
            Err(e) => {
                error!("Worker error: {}", e);
            }
        }

        // Wait before next cycle based on configuration
        sleep(Duration::from_secs(config.worker.poll_interval)).await;
    }
}

async fn process_tasks(
    config: &WorkerConfig,
    broadcast_service: Arc<common::broadcast::BroadcastService>
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Processing tasks with {} concurrency", config.worker.concurrency);
    info!("Broadcast service subscribers: {}", broadcast_service.subscriber_count());

    // TODO: Replace with actual user authentication/lookup
    let dummy_user = common::User {
        id: uuid::Uuid::new_v4(),
        email: "worker@example.com".to_string(),
        username: "worker".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Process tasks with broadcasting
    match queue::process_task(config, &dummy_user, broadcast_service).await {
        Ok(_) => info!("Task processing completed"),
        Err(e) => error!("Task processing failed: {}", e),
    }

    Ok(())
}
