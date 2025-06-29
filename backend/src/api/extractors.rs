use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, warn};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = StatusCode::UNAUTHORIZED;
        let body = Json(json!({
            "error": self.error,
            "message": self.message
        }));
        (status, body).into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<Config>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract authorization header
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AuthError {
                error: "missing_authorization".to_string(),
                message: "Authorization header is required".to_string(),
            })?;

        // Extract bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthError {
                error: "invalid_authorization".to_string(),
                message: "Authorization header must be a Bearer token".to_string(),
            })?;

        if token.is_empty() {
            return Err(AuthError {
                error: "empty_token".to_string(),
                message: "Token cannot be empty".to_string(),
            });
        }

        let token = token.to_string(); // Convert to owned string to avoid lifetime issues

        // Get config to validate token with PocketBase
        let config = Arc::<Config>::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError {
                error: "server_error".to_string(),
                message: "Failed to access server configuration".to_string(),
            })?;

        // Validate token with global PocketBase
        match validate_pb_token(&token, &config).await {
            Ok(user) => Ok(user),
            Err(e) => {
                warn!("Token validation failed: {}", e);
                Err(AuthError {
                    error: "invalid_token".to_string(),
                    message: "Invalid or expired token".to_string(),
                })
            }
        }
    }
}

/// Validate PocketBase token and extract user information
async fn validate_pb_token(token: &str, config: &Config) -> Result<AuthUser, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let auth_refresh_url = format!("{}/api/collections/users/auth-refresh", config.database.url);

    let response = client
        .post(&auth_refresh_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status().is_success() {
        let auth_data: Value = response.json().await?;
        
        let record = auth_data
            .get("record")
            .ok_or("No user record in response")?;

        let user = AuthUser {
            id: record
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or("Missing user ID")?
                .to_string(),
            email: record
                .get("email")
                .and_then(|v| v.as_str())
                .ok_or("Missing user email")?
                .to_string(),
            name: record
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from),
            token: auth_data
                .get("token")
                .and_then(|v| v.as_str())
                .unwrap_or(token)
                .to_string(),
        };

        Ok(user)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!("PocketBase token validation failed: {}", error_text);
        Err(format!("Token validation failed: {}", status).into())
    }
}

/// Optional authentication extractor - doesn't fail if no auth provided
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
    Arc<Config>: FromRequestParts<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
