use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwarmError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Runtime error: {0}")]
    Runtime(#[from] pixelcore_runtime::error::RuntimeError),

    #[error("{0}")]
    Other(String),
}
