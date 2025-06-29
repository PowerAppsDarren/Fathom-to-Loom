use axum::{
    routing::{get, post, delete},
    Router,
    extract::State,
    response::{Json, IntoResponse},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

use crate::config::Config;
use common::crypto::{EncryptedApiKey, generate_master_key, encrypt, decrypt};
use crate::pocketbase_manager::PocketBaseManager;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyEntry {
    pub service: String,
    pub key_id: String,
    pub encrypted_key: EncryptedApiKey,
}

/// Create router for keys management
pub fn router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/keys", get(get_keys).put(put_key))
}

/// GET /api/keys - Retrieve encrypted API keys
pub async fn get_keys(
    axum::extract::State(app_state): axum::extract::State<crate::api::AppState>
) -> impl IntoResponse {
    info!("Retrieving API keys");
    
    // Dummy implementation, replace with actual logic fetching from storage
    let keys = vec![
        KeyEntry {
            service: "pocketbase".to_string(),
            key_id: "default-key-id".to_string(),
            encrypted_key: EncryptedApiKey::new(
                "pocketbase".to_string(),
                "default-key-id".to_string(),
                "fake-api-key",
                &generate_master_key(),
                None,
            ),
        }
    ];

    (StatusCode::OK, Json(keys))
}

/// PUT /api/keys - Add or update an encrypted API key
pub async fn put_key(
    axum::extract::State(app_state): axum::extract::State<crate::api::AppState>,
    Json(entry): Json<KeyEntry>
) -> impl IntoResponse {
    info!("Updating API key for service: {}", entry.service);

    // Dummy implementation, replace with actual logic for storing key to secure storage
    let encrypted_key = encrypt(&generate_master_key(), entry.encrypted_key.decrypt_key(&generate_master_key()).unwrap().as_bytes());

    let stored_entry = KeyEntry {
        service: entry.service.clone(),
        key_id: entry.key_id.clone(),
        encrypted_key: EncryptedApiKey {
            service: entry.service.clone(),
            key_id: entry.key_id.clone(),
            encrypted_key,
            created_at: chrono::Utc::now(),
            expires_at: None,
        }
    };

    // Mock inserting to database or storage
    info!("API key for service '{}' stored.", stored_entry.service);

    (StatusCode::OK, Json(stored_entry))
}
