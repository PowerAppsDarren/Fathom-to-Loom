use axum::{response::Html, routing::get, Router};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting SMTP service");

    // Build application router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check));

    // Start server
    let port = std::env::var("SMTP_PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("SMTP service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> Html<&'static str> {
    Html("<h1>Fathom to Loom SMTP Service</h1><p>Service is running!</p>")
}

async fn health_check() -> &'static str {
    "OK"
}
