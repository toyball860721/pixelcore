use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Channel closed")]
    ChannelClosed,

    #[error("Send error: {0}")]
    Send(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
