use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info};

use crate::pocketbase_manager::{PocketBaseManager, PocketBaseInstance, PocketBaseError};

/// Response for PocketBase initialization
#[derive(Debug, Serialize)]
pub struct InitPbResponse {
    pub success: bool,
    pub message: String,
    pub instance: Option<PocketBaseInstance>,
}

/// Response for PocketBase status
#[derive(Debug, Serialize)]
pub struct PbStatusResponse {
    pub user_id: String,
    pub instance: Option<PocketBaseInstance>,
}

/// Request body for PocketBase initialization
#[derive(Debug, Deserialize)]
pub struct InitPbRequest {
    pub force_restart: Option<bool>,
}

/// Create router for PocketBase API endpoints
pub fn router() -> Router<Arc<PocketBaseManager>> {
    Router::new()
        .route("/users/:id/init_pb", post(init_user_pocketbase))
        .route("/users/:id/pb_status", get(get_user_pocketbase_status))
        .route("/users/:id/stop_pb", post(stop_user_pocketbase))
        .route("/pb_instances", get(list_all_instances))
}

/// POST /api/users/{id}/init_pb
/// Initialize PocketBase instance for a user
async fn init_user_pocketbase(
    Path(user_id): Path<String>,
    State(pb_manager): State<Arc<PocketBaseManager>>,
    Json(request): Json<InitPbRequest>,
) -> Result<Json<InitPbResponse>, StatusCode> {
    info!("Received request to initialize PocketBase for user: {}", user_id);

    // Validate user ID
    if user_id.trim().is_empty() {
        return Ok(Json(InitPbResponse {
            success: false,
            message: "Invalid user ID".to_string(),
            instance: None,
        }));
    }

    // If force_restart is true, stop existing instance first
    if request.force_restart.unwrap_or(false) {
        if let Err(e) = pb_manager.stop_user_instance(&user_id).await {
            error!("Failed to stop existing instance for user {}: {}", user_id, e);
            return Ok(Json(InitPbResponse {
                success: false,
                message: format!("Failed to stop existing instance: {}", e),
                instance: None,
            }));
        }
        
        // Wait a moment for cleanup
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Initialize the PocketBase instance
    match pb_manager.init_user_instance(&user_id).await {
        Ok(instance) => {
            info!("Successfully initialized PocketBase for user {} on port {}", user_id, instance.port);
            
            Ok(Json(InitPbResponse {
                success: true,
                message: format!("PocketBase instance initialized on port {}", instance.port),
                instance: Some(instance),
            }))
        }
        Err(e) => {
            error!("Failed to initialize PocketBase for user {}: {}", user_id, e);
            
            let error_message = match e {
                PocketBaseError::NoPortsAvailable => "No available ports for PocketBase instance".to_string(),
                PocketBaseError::ProcessError(msg) => format!("Process error: {}", msg),
                PocketBaseError::IoError(msg) => format!("IO error: {}", msg),
                _ => format!("Initialization failed: {}", e),
            };
            
            Ok(Json(InitPbResponse {
                success: false,
                message: error_message,
                instance: None,
            }))
        }
    }
}

/// GET /api/users/{id}/pb_status
/// Get PocketBase instance status for a user
async fn get_user_pocketbase_status(
    Path(user_id): Path<String>,
    State(pb_manager): State<Arc<PocketBaseManager>>,
) -> Result<Json<PbStatusResponse>, StatusCode> {
    info!("Checking PocketBase status for user: {}", user_id);

    let instance = pb_manager.get_user_instance(&user_id).await;
    
    Ok(Json(PbStatusResponse {
        user_id,
        instance,
    }))
}

/// POST /api/users/{id}/stop_pb
/// Stop PocketBase instance for a user
async fn stop_user_pocketbase(
    Path(user_id): Path<String>,
    State(pb_manager): State<Arc<PocketBaseManager>>,
) -> Result<Json<Value>, StatusCode> {
    info!("Received request to stop PocketBase for user: {}", user_id);

    match pb_manager.stop_user_instance(&user_id).await {
        Ok(()) => {
            info!("Successfully stopped PocketBase for user: {}", user_id);
            Ok(Json(json!({
                "success": true,
                "message": "PocketBase instance stopped successfully"
            })))
        }
        Err(e) => {
            error!("Failed to stop PocketBase for user {}: {}", user_id, e);
            Ok(Json(json!({
                "success": false,
                "message": format!("Failed to stop PocketBase instance: {}", e)
            })))
        }
    }
}

/// GET /api/pb_instances
/// List all PocketBase instances
async fn list_all_instances(
    State(pb_manager): State<Arc<PocketBaseManager>>,
) -> Result<Json<Value>, StatusCode> {
    let instances = pb_manager.get_all_instances().await;
    
    Ok(Json(json!({
        "instances": instances,
        "count": instances.len()
    })))
}

/// Health check endpoint for PocketBase API
pub async fn pb_health_check(
    State(pb_manager): State<Arc<PocketBaseManager>>,
) -> Result<Json<Value>, StatusCode> {
    let instances = pb_manager.get_all_instances().await;
    let running_count = instances.values()
        .filter(|instance| instance.status == crate::pocketbase_manager::InstanceStatus::Running)
        .count();
    
    Ok(Json(json!({
        "status": "ok",
        "total_instances": instances.len(),
        "running_instances": running_count,
        "timestamp": chrono::Utc::now()
    })))
}
