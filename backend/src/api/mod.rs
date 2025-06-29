pub mod adapters;
pub mod auth;
pub mod extractors;
pub mod keys;
pub mod meetings;
pub mod pocketbase;
pub mod queue;
pub mod websocket;

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{config::Config, pocketbase_manager::PocketBaseManager};
use websocket::WebSocketManager;

/// Application state combining all managers and config
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pb_manager: Arc<PocketBaseManager>,
    pub ws_manager: Arc<WebSocketManager>,
    pub meetings_queue: Arc<RwLock<Vec<queue::Meeting>>>,
}

/// Create the main API router with all endpoints
pub fn create_api_router(app_state: AppState) -> Router {
    Router::new()
        // Health checks
        .route("/health/pb", get(pocketbase::pb_health_check))
        .route("/health/ws", get(websocket_health_check))
        
        // WebSocket endpoint for real-time updates
        .route("/queue_updates", get(websocket::websocket_handler))
        
        // API routes with authentication
        .nest("/api", create_authenticated_api_router())
        
        // Legacy PocketBase management routes
        .nest("/api", pocketbase::router().with_state(app_state.pb_manager.clone()))
        
        .with_state(app_state.clone())
        // Authentication routes (proxied to global PB) - separate router with config state
        .merge(
            Router::new()
                .nest("/auth", auth::router())
                .with_state(app_state.config.clone())
        )
}

/// Create authenticated API router
fn create_authenticated_api_router() -> Router<AppState> {
    Router::new()
        // Key management with encryption
        .route("/keys", axum::routing::get(keys::get_keys))
        .route("/keys", axum::routing::put(keys::put_key))
        
        // Queue management
        .route("/queue", axum::routing::post(queue::add_meetings))
        .route("/queue", axum::routing::get(queue::get_queue))
        .route("/queue/:id", axum::routing::delete(queue::remove_meeting))
        
        // Meetings proxy to Fathom with caching
        .route("/meetings", axum::routing::get(meetings::get_meetings))
}

/// WebSocket health check
async fn websocket_health_check(
    axum::extract::State(app_state): axum::extract::State<AppState>,
) -> axum::response::Json<serde_json::Value> {
    let connection_count = app_state.ws_manager.connection_count().await;
    
    axum::response::Json(serde_json::json!({
        "status": "ok",
        "websocket_connections": connection_count,
        "timestamp": chrono::Utc::now()
    }))
}
