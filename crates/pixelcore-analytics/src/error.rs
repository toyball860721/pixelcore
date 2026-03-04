use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Warehouse error: {0}")]
    WarehouseError(String),

    #[error("ETL error: {0}")]
    EtlError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    PoolError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;
