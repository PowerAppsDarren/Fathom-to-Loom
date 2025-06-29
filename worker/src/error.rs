#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Queue error: {0}")]
    Queue(String),

    #[error("Fathom API error: {0}")]
    Fathom(String),

    #[error("Loom API error: {0}")]
    Loom(String),

    #[error("PocketBase error: {0}")]
    PocketBase(String),

    #[error("Video processing error: {0}")]
    Video(String),

    #[error("Email error: {0}")]
    Email(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type WorkerResult<T> = Result<T, WorkerError>;

impl WorkerError {
    pub fn is_retryable(&self) -> bool {
        match self {
            WorkerError::Network(_) => true,
            WorkerError::Queue(_) => true,
            WorkerError::Fathom(_) => true,
            WorkerError::Loom(_) => true,
            WorkerError::PocketBase(_) => true,
            WorkerError::Video(_) => false, // Video processing errors usually aren't retryable
            WorkerError::Email(_) => true,
            WorkerError::Io(_) => true,
            WorkerError::Serde(_) => false,
            WorkerError::Config(_) => false,
            WorkerError::Internal(_) => false,
        }
    }
}
