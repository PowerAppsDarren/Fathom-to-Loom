mod config;
mod pocketbase_manager;
mod api;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::get,
    Router,
};
use std::{path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde_json::{json, Value};

use config::Config;
use pocketbase_manager::PocketBaseManager;
use api::{AppState, websocket::WebSocketManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Arc::new(Config::from_env()?);

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

    info!("Configuration loaded successfully");
    info!("Log level: {}", config.logging.level);
    info!("Database URL: {}", config.database.url);

    // Initialize PocketBase manager
    let user_dbs_path = PathBuf::from(&config.pocketbase.user_dbs_path);
    let pb_manager = Arc::new(PocketBaseManager::new(
        user_dbs_path, 
        config.pocketbase.base_port,
        config.pocketbase.binary_path.clone()
    ));
    
    // Start health monitoring for PocketBase instances
    pb_manager.start_health_monitoring().await;
    info!("PocketBase manager initialized with base path: {}", config.pocketbase.user_dbs_path);

    // Initialize shared broadcast service
    let broadcast_service = common::broadcast::BroadcastServiceFactory::create_shared(1000);
    info!("Shared broadcast service initialized");
    
    // Initialize WebSocket manager with external broadcast integration
    let ws_manager = Arc::new(WebSocketManager::with_external_broadcast(broadcast_service.clone()));
    info!("WebSocket manager initialized with broadcast integration");

    // Initialize meeting queue
    let meetings_queue = Arc::new(RwLock::new(Vec::new()));
    info!("Meetings queue initialized");

    // Create application state
    let app_state = AppState {
        config: config.clone(),
        pb_manager,
        ws_manager,
        meetings_queue,
    };

    // Build our application with unified state
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/env", get(env_endpoint))
        .with_state(config.clone())
        .merge(api::create_api_router(app_state));  // Add unified API routes

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("Starting Fathom to Loom backend server on {}", addr);
    info!("PocketBase API endpoints available under /api/users/{{id}}/init_pb");

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> Html<&'static str> {
    Html("<h1>Fathom to Loom Backend</h1><p>API server is running!</p>")
}

async fn health_check() -> &'static str {
    "OK"
}

/// Endpoint to expose safe environment configuration to frontend
async fn env_endpoint(State(config): State<Arc<Config>>) -> Result<Json<Value>, StatusCode> {
    // Only expose safe, non-sensitive configuration values to the frontend
    let safe_config = json!({
        "api": {
            "base_url": format!("http://{}:{}", config.server.host, config.server.port),
            "version": env!("CARGO_PKG_VERSION")
        },
        "database": {
            "url": config.database.url
        },
        "logging": {
            "level": config.logging.level
        },
        "cors": {
            "origins": config.cors.origins
        },
        "features": {
            "auth_enabled": true,
            "encryption_enabled": true
        }
    });

    Ok(Json(safe_config))
}
