use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

mod config;
mod email;
mod pocketbase;
mod security;

use config::Config;
use email::{EmailService, EmailRequest};
use pocketbase::PocketBaseClient;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    pocketbase: Arc<PocketBaseClient>,
    email_service: Arc<EmailService>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    pocketbase_connected: bool,
    smtp_configured: bool,
}

#[derive(Deserialize)]
struct SendEmailRequest {
    to_email: String,
    to_name: Option<String>,
    subject: String,
    body_html: Option<String>,
    body_text: Option<String>,
}

#[derive(Serialize)]
struct SendEmailResponse {
    success: bool,
    message: String,
    queue_id: Option<String>,
}

#[derive(Deserialize)]
struct TestSmtpRequest {
    test_email: String,
}

#[derive(Serialize)]
struct TestSmtpResponse {
    success: bool,
    message: String,
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let pocketbase_connected = state.pocketbase.health_check().await.unwrap_or(false);
    let smtp_configured = state.email_service.is_configured().await;

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        pocketbase_connected,
        smtp_configured,
    })
}

async fn send_email(
    State(state): State<AppState>,
    Json(request): Json<SendEmailRequest>,
) -> Result<Json<SendEmailResponse>, StatusCode> {
    let email_request = EmailRequest {
        to_email: request.to_email,
        to_name: request.to_name,
        subject: request.subject,
        body_html: request.body_html,
        body_text: request.body_text,
    };

    match state.email_service.queue_email(email_request).await {
        Ok(queue_id) => Ok(Json(SendEmailResponse {
            success: true,
            message: "Email queued successfully".to_string(),
            queue_id: Some(queue_id),
        })),
        Err(e) => {
            error!("Failed to queue email: {}", e);
            Ok(Json(SendEmailResponse {
                success: false,
                message: format!("Failed to queue email: {}", e),
                queue_id: None,
            }))
        }
    }
}

async fn test_smtp_connection(
    State(state): State<AppState>,
    Json(request): Json<TestSmtpRequest>,
) -> Json<TestSmtpResponse> {
    match state.email_service.test_connection(&request.test_email).await {
        Ok(_) => Json(TestSmtpResponse {
            success: true,
            message: "SMTP connection test successful".to_string(),
        }),
        Err(e) => Json(TestSmtpResponse {
            success: false,
            message: format!("SMTP connection test failed: {}", e),
        }),
    }
}

async fn process_email_queue(state: AppState) {
    let mut interval = interval(Duration::from_secs(30)); // Process queue every 30 seconds

    loop {
        interval.tick().await;
        
        match state.email_service.process_queue().await {
            Ok(processed_count) => {
                if processed_count > 0 {
                    info!("Processed {} emails from queue", processed_count);
                }
            }
            Err(e) => {
                error!("Error processing email queue: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    info!("Configuration loaded successfully");

    // Initialize PocketBase client
    let pocketbase = Arc::new(PocketBaseClient::new(&config.pocketbase_url)?);
    info!("PocketBase client initialized");

    // Initialize email service
    let email_service = Arc::new(EmailService::new(config.clone(), pocketbase.clone()).await?);
    info!("Email service initialized");

    // Create app state
    let state = AppState {
        config: config.clone(),
        pocketbase,
        email_service,
    };

    // Start email queue processor
    let queue_state = state.clone();
    tokio::spawn(async move {
        process_email_queue(queue_state).await;
    });

    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/send-email", post(send_email))
        .route("/test-smtp", post(test_smtp_connection))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.smtp_service_host, config.smtp_service_port
    ))
    .await?;

    info!(
        "SMTP service listening on {}:{}",
        config.smtp_service_host, config.smtp_service_port
    );

    axum::serve(listener, app).await?;

    Ok(())
}
