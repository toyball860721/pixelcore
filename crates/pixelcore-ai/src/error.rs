use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("Recommendation error: {0}")]
    RecommendationError(String),

    #[error("Model not trained: {0}")]
    ModelNotTrained(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, AiError>;
