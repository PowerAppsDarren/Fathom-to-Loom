use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use gloo_net::http::Request;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub cors: CorsConfig,
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub auth_enabled: bool,
    pub encryption_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                base_url: get_api_base_url(),
                version: "0.1.0".to_string(),
            },
            database: DatabaseConfig {
                url: "http://pb_global:8090".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
            cors: CorsConfig {
                origins: vec![
                    "http://localhost:8080".to_string(),
                    "http://localhost:3000".to_string(),
                ],
            },
            features: FeaturesConfig {
                auth_enabled: true,
                encryption_enabled: true,
            },
        }
    }
}

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub async fn load_config() -> Result<&'static AppConfig, Box<dyn std::error::Error>> {
    if let Some(config) = CONFIG.get() {
        return Ok(config);
    }

    // Try to fetch config from backend
    let config = match fetch_config_from_backend().await {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to fetch config from backend: {}, using defaults", e);
            AppConfig::default()
        }
    };

    CONFIG.set(config).map_err(|_| "Failed to set global config")?;
    Ok(CONFIG.get().unwrap())
}

pub fn get_config() -> Option<&'static AppConfig> {
    CONFIG.get()
}

async fn fetch_config_from_backend() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let api_base = get_api_base_url();
    let url = format!("{}/api/env", api_base);
    
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch config: {}", e))?;

    if !response.ok() {
        return Err(format!("Backend returned error: {}", response.status()).into());
    }

    let config: AppConfig = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

fn get_api_base_url() -> String {
    // Try to get from environment variable compiled at build time
    if let Some(api_url) = option_env!("API_BASE_URL") {
        return api_url.to_string();
    }

    // Check if we're running in development or production
    let hostname = web_sys::window()
        .and_then(|w| w.location().hostname().ok())
        .unwrap_or_else(|| "localhost".to_string());

    if hostname == "localhost" || hostname == "127.0.0.1" {
        // Development mode
        "http://localhost:3000".to_string()
    } else {
        // Production mode - assume backend is on the same host
        format!("http://{}:3000", hostname)
    }
}

// Environment variables that can be baked in at build time
pub struct BuildConfig;

impl BuildConfig {
    pub fn api_base_url() -> Option<&'static str> {
        option_env!("API_BASE_URL")
    }

    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn environment() -> &'static str {
        option_env!("ENVIRONMENT").unwrap_or("development")
    }

    pub fn is_development() -> bool {
        Self::environment() == "development"
    }

    pub fn is_production() -> bool {
        Self::environment() == "production"
    }
}
