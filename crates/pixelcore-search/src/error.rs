use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Search error: {0}")]
    SearchError(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Query parse error: {0}")]
    QueryParseError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Tantivy error: {0}")]
    TantivyError(#[from] tantivy::TantivyError),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, SearchError>;
