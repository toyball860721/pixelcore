use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegionError {
    #[error("Region not found: {0}")]
    RegionNotFound(String),

    #[error("Region already exists: {0}")]
    RegionAlreadyExists(String),

    #[error("Region is unhealthy: {0}")]
    RegionUnhealthy(String),

    #[error("No healthy regions available")]
    NoHealthyRegions,

    #[error("Replication error: {0}")]
    ReplicationError(String),

    #[error("Load balancer error: {0}")]
    LoadBalancerError(String),

    #[error("Failover error: {0}")]
    FailoverError(String),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, RegionError>;
