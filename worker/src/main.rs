mod config;

use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // Main worker loop
    loop {
        match process_tasks(&config).await {
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

async fn process_tasks(config: &WorkerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement actual task processing
    info!("Processing tasks with {} concurrency", config.worker.concurrency);

    // Placeholder for actual work
    sleep(Duration::from_secs(1)).await;

    Ok(())
}
