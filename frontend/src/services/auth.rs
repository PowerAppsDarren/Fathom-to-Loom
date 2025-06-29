use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};
use gloo_net::http::Request;
use anyhow::{Result, anyhow};
use crate::config::get_config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub username: String,
}

const TOKEN_KEY: &str = "auth_token";
const USER_KEY: &str = "user_info";

#[derive(Debug, Clone, PartialEq)]
pub struct AuthService {
    token: Option<String>,
    user: Option<UserInfo>,
}

impl AuthService {
    pub fn new() -> Self {
        let token = LocalStorage::get(TOKEN_KEY).ok();
        let user = LocalStorage::get(USER_KEY).ok();
        
        Self { token, user }
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some() && self.user.is_some()
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    pub fn get_user(&self) -> Option<&UserInfo> {
        self.user.as_ref()
    }

    pub async fn login(&mut self, request: LoginRequest) -> Result<()> {
        let config = get_config().ok_or_else(|| anyhow!("Configuration not loaded"))?;
        let url = format!("{}/auth/login", config.api.base_url);

        let response = Request::post(&url)
            .json(&request)?
            .send()
            .await
            .map_err(|e| anyhow!("Login request failed: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Login failed: {}", response.status()));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse login response: {}", e))?;

        // Store token and user info
        LocalStorage::set(TOKEN_KEY, &auth_response.token)
            .map_err(|e| anyhow!("Failed to store token: {:?}", e))?;
        LocalStorage::set(USER_KEY, &auth_response.user)
            .map_err(|e| anyhow!("Failed to store user info: {:?}", e))?;

        self.token = Some(auth_response.token);
        self.user = Some(auth_response.user);

        Ok(())
    }

    pub async fn register(&mut self, request: RegisterRequest) -> Result<()> {
        let config = get_config().ok_or_else(|| anyhow!("Configuration not loaded"))?;
        let url = format!("{}/auth/register", config.api.base_url);

        let response = Request::post(&url)
            .json(&request)?
            .send()
            .await
            .map_err(|e| anyhow!("Registration request failed: {}", e))?;

        if !response.ok() {
            return Err(anyhow!("Registration failed: {}", response.status()));
        }

        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse registration response: {}", e))?;

        // Store token and user info
        LocalStorage::set(TOKEN_KEY, &auth_response.token)
            .map_err(|e| anyhow!("Failed to store token: {:?}", e))?;
        LocalStorage::set(USER_KEY, &auth_response.user)
            .map_err(|e| anyhow!("Failed to store user info: {:?}", e))?;

        self.token = Some(auth_response.token);
        self.user = Some(auth_response.user);

        Ok(())
    }

    pub fn logout(&mut self) {
        // Clear stored data
        let _ = LocalStorage::delete(TOKEN_KEY);
        let _ = LocalStorage::delete(USER_KEY);
        
        self.token = None;
        self.user = None;
    }

    pub fn get_auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|token| format!("Bearer {}", token))
    }
}
