use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::config::Config;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<Value>,
    pub message: Option<String>,
}

/// Create router for authentication endpoints
pub fn router() -> Router<Arc<Config>> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

/// POST /auth/login - proxy to global PocketBase
async fn login(
    State(config): State<Arc<Config>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("Login attempt for email: {}", request.email);

    // Validate input
    if request.email.trim().is_empty() || request.password.is_empty() {
        return Ok(Json(AuthResponse {
            success: false,
            token: None,
            user: None,
            message: Some("Email and password are required".to_string()),
        }));
    }

    // Make request to global PocketBase
    let client = reqwest::Client::new();
    let auth_url = format!("{}/api/collections/users/auth-with-password", config.database.url);
    
    let pb_request = json!({
        "identity": request.email,
        "password": request.password
    });

    match client
        .post(&auth_url)
        .json(&pb_request)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if status.is_success() {
                match serde_json::from_str::<Value>(&response_text) {
                    Ok(pb_response) => {
                        info!("Successful login for user: {}", request.email);
                        Ok(Json(AuthResponse {
                            success: true,
                            token: pb_response.get("token").and_then(|t| t.as_str()).map(String::from),
                            user: pb_response.get("record").cloned(),
                            message: Some("Login successful".to_string()),
                        }))
                    }
                    Err(e) => {
                        error!("Failed to parse PocketBase response: {}", e);
                        Ok(Json(AuthResponse {
                            success: false,
                            token: None,
                            user: None,
                            message: Some("Authentication server error".to_string()),
                        }))
                    }
                }
            } else {
                warn!("Failed login attempt for {}: {}", request.email, status);
                Ok(Json(AuthResponse {
                    success: false,
                    token: None,
                    user: None,
                    message: Some("Invalid email or password".to_string()),
                }))
            }
        }
        Err(e) => {
            error!("Failed to connect to PocketBase: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// POST /auth/register - proxy to global PocketBase
async fn register(
    State(config): State<Arc<Config>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("Registration attempt for email: {}", request.email);

    // Validate input
    if request.email.trim().is_empty() || request.password.is_empty() {
        return Ok(Json(AuthResponse {
            success: false,
            token: None,
            user: None,
            message: Some("Email and password are required".to_string()),
        }));
    }

    if request.password != request.password_confirm {
        return Ok(Json(AuthResponse {
            success: false,
            token: None,
            user: None,
            message: Some("Passwords do not match".to_string()),
        }));
    }

    // Make request to global PocketBase
    let client = reqwest::Client::new();
    let register_url = format!("{}/api/collections/users/records", config.database.url);
    
    let pb_request = json!({
        "email": request.email,
        "password": request.password,
        "passwordConfirm": request.password_confirm,
        "name": request.name.unwrap_or_else(|| request.email.split('@').next().unwrap_or("User").to_string())
    });

    match client
        .post(&register_url)
        .json(&pb_request)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if status.is_success() {
                match serde_json::from_str::<Value>(&response_text) {
                    Ok(pb_response) => {
                        info!("Successful registration for user: {}", request.email);
                        
                        // After successful registration, attempt to login
                        let login_request = LoginRequest {
                            email: request.email,
                            password: request.password,
                        };
                        
                        // Recursively call login to get the token
                        match login(State(config), Json(login_request)).await {
                            Ok(login_response) => Ok(login_response),
                            Err(_) => Ok(Json(AuthResponse {
                                success: true,
                                token: None,
                                user: Some(pb_response),
                                message: Some("Registration successful, please login".to_string()),
                            }))
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse PocketBase registration response: {}", e);
                        Ok(Json(AuthResponse {
                            success: false,
                            token: None,
                            user: None,
                            message: Some("Registration server error".to_string()),
                        }))
                    }
                }
            } else {
                warn!("Failed registration attempt for {}: {} - {}", request.email, status, response_text);
                Ok(Json(AuthResponse {
                    success: false,
                    token: None,
                    user: None,
                    message: Some("Registration failed. Email may already be in use.".to_string()),
                }))
            }
        }
        Err(e) => {
            error!("Failed to connect to PocketBase for registration: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}
