pub mod client;
pub mod error;
pub mod types;

pub use client::ClawClient;
pub use error::ClawError;
pub use types::{LlmRequest, LlmResponse, Tool, ToolCall, ToolResult, OpenAiRequest, OpenAiResponse};

// Back-compat re-exports
pub use client::ClawClient as McpClient;
pub use error::ClawError as McpError;
