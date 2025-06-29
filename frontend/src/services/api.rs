use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use anyhow::{Result, anyhow};
use crate::config::get_config;
use crate::services::auth::AuthService;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: Uuid,
    pub user_id: String,
    pub topic: String,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRequest {
    pub user_id: String,
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<Vec<Meeting>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FathomMeeting {
    pub id: String,
    pub title: String,
    pub start_time: String,
    pub duration: u32,
    pub participants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingsResponse {
    pub success: bool,
    pub meetings: Vec<FathomMeeting>,
    pub total: u32,
    pub cached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub encrypted_value: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRequest {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiService {
    auth_service: AuthService,
}

impl ApiService {
    pub fn new(auth_service: AuthService) -> Self {
        Self { auth_service }
    }

    fn get_base_url(&self) -> Result<String> {
        let config = get_config().ok_or_else(|| anyhow!("Configuration not loaded"))?;
        Ok(config.api.base_url.clone())
    }

    fn create_authenticated_request(&self, method: &str, endpoint: &str) -> Result<Request> {
        let base_url = self.get_base_url()?;
        let url = format!("{}/api{}", base_url, endpoint);
        
        let mut request = match method {
            "GET" => Request::get(&url),
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };

        if let Some(auth_header) = self.auth_service.get_auth_header() {
            request = request.header("Authorization", &auth_header);
        }

        Ok(request)
    }

    // Queue management
    pub async fn get_queue(&self) -> Result<QueueResponse> {
        let request = self.create_authenticated_request("GET", "/queue")?;
        let response = request.send().await
            .map_err(|e| anyhow!("Failed to get queue: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Get queue failed: {}", response.status()));
        }

        response.json().await
            .map_err(|e| anyhow!("Failed to parse queue response: {}", e))
    }

    pub async fn add_to_queue(&self, meeting_request: MeetingRequest) -> Result<QueueResponse> {
        let request = self.create_authenticated_request("POST", "/queue")?
            .json(&meeting_request)
            .map_err(|e| anyhow!("Failed to serialize meeting request: {}", e))?;

        let response = request.send().await
            .map_err(|e| anyhow!("Failed to add to queue: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Add to queue failed: {}", response.status()));
        }

        response.json().await
            .map_err(|e| anyhow!("Failed to parse add queue response: {}", e))
    }

    pub async fn remove_from_queue(&self, meeting_id: Uuid) -> Result<QueueResponse> {
        let endpoint = format!("/queue/{}", meeting_id);
        let request = self.create_authenticated_request("DELETE", &endpoint)?;

        let response = request.send().await
            .map_err(|e| anyhow!("Failed to remove from queue: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Remove from queue failed: {}", response.status()));
        }

        response.json().await
            .map_err(|e| anyhow!("Failed to parse remove queue response: {}", e))
    }

    // Meetings (Fathom proxy)
    pub async fn get_meetings(&self, limit: Option<u32>, offset: Option<u32>) -> Result<MeetingsResponse> {
        let mut endpoint = "/meetings".to_string();
        let mut params = Vec::new();
        
        if let Some(limit) = limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = offset {
            params.push(format!("offset={}", offset));
        }
        
        if !params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&params.join("&"));
        }

        let request = self.create_authenticated_request("GET", &endpoint)?;
        let response = request.send().await
            .map_err(|e| anyhow!("Failed to get meetings: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Get meetings failed: {}", response.status()));
        }

        response.json().await
            .map_err(|e| anyhow!("Failed to parse meetings response: {}", e))
    }

    // API Keys management
    pub async fn get_api_keys(&self) -> Result<Vec<ApiKey>> {
        let request = self.create_authenticated_request("GET", "/keys")?;
        let response = request.send().await
            .map_err(|e| anyhow!("Failed to get API keys: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Get API keys failed: {}", response.status()));
        }

        response.json().await
            .map_err(|e| anyhow!("Failed to parse API keys response: {}", e))
    }

    pub async fn save_api_key(&self, api_key_request: ApiKeyRequest) -> Result<()> {
        let request = self.create_authenticated_request("PUT", "/keys")?
            .json(&api_key_request)
            .map_err(|e| anyhow!("Failed to serialize API key request: {}", e))?;

        let response = request.send().await
            .map_err(|e| anyhow!("Failed to save API key: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Save API key failed: {}", response.status()));
        }

        Ok(())
    }
}
