//! Common types and utilities shared across Fathom to Loom workspace

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cryptographic utilities for secure storage
pub mod crypto;

/// Application-wide error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Common API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

/// User representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Job status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Background job representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub status: JobStatus,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Configuration utilities
pub mod config {
    use std::env;

    pub fn get_env_var(key: &str) -> Result<String, crate::AppError> {
        env::var(key).map_err(|_| {
            crate::AppError::Internal(format!("Environment variable {} not found", key))
        })
    }

    pub fn get_env_var_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<()> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}
