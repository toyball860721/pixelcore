pub mod client;
pub mod error;
pub mod types;
pub mod mcp_types;
pub mod stdio_transport;
pub mod local_mcp;

pub use client::ClawClient;
pub use error::ClawError;
pub use types::{LlmRequest, LlmResponse, Tool, ToolCall, ToolResult, OpenAiRequest, OpenAiResponse};
pub use mcp_types::*;
pub use stdio_transport::StdioTransport;
pub use local_mcp::LocalMcpClient;

// Back-compat re-exports
pub use client::ClawClient as McpClient;
pub use error::ClawError as McpError;
