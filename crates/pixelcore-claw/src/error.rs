use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClawError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Missing API key for provider: {0}")]
    MissingApiKey(String),

    #[error("Rate limited, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),

    #[error("{0}")]
    Other(String),
}
