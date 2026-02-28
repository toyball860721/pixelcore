use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use anyhow::{Context, Result};
use serde_json::Value;
use crate::stdio_transport::StdioTransport;
use crate::mcp_types::*;

/// 本地 MCP 客户端
pub struct LocalMcpClient {
    transport: Arc<StdioTransport>,
    request_id: AtomicU64,
    server_info: Option<ServerInfo>,
    capabilities: Option<ServerCapabilities>,
}

impl LocalMcpClient {
    /// 创建并初始化一个新的本地 MCP 客户端
    pub async fn new(command: &str, args: &[&str]) -> Result<Self> {
        let transport = StdioTransport::spawn(command, args).await?;
        let mut client = Self {
            transport: Arc::new(transport),
            request_id: AtomicU64::new(1),
            server_info: None,
            capabilities: None,
        };

        // 初始化连接
        client.initialize().await?;

        Ok(client)
    }

    /// 生成下一个请求 ID
    fn next_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// 初始化 MCP 连接
    async fn initialize(&mut self) -> Result<()> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                experimental: None,
            },
            client_info: ClientInfo {
                name: "pixelcore".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let request = JsonRpcRequest::new(
            self.next_id(),
            "initialize",
            Some(serde_json::to_value(params)?),
        );

        let response = self.transport.call(request).await?;

        if let Some(error) = response.error {
            anyhow::bail!("MCP initialization failed: {} (code: {})", error.message, error.code);
        }

        let result: InitializeResult = serde_json::from_value(
            response.result.context("Missing result in initialize response")?
        )?;

        self.server_info = Some(result.server_info);
        self.capabilities = Some(result.capabilities);

        tracing::info!(
            "MCP server initialized: {} v{}",
            self.server_info.as_ref().unwrap().name,
            self.server_info.as_ref().unwrap().version
        );

        // 发送 initialized 通知
        let notification = JsonRpcRequest::new(
            Value::Null,
            "notifications/initialized",
            None,
        );
        self.transport.send_request(&notification).await?;

        Ok(())
    }

    /// 获取服务器信息
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// 获取服务器能力
    pub fn capabilities(&self) -> Option<&ServerCapabilities> {
        self.capabilities.as_ref()
    }

    /// 列出所有可用的工具
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let request = JsonRpcRequest::new(
            self.next_id(),
            "tools/list",
            None,
        );

        let response = self.transport.call(request).await?;

        if let Some(error) = response.error {
            anyhow::bail!("Failed to list tools: {} (code: {})", error.message, error.code);
        }

        let result: ListToolsResult = serde_json::from_value(
            response.result.context("Missing result in tools/list response")?
        )?;

        Ok(result.tools)
    }

    /// 调用一个工具
    pub async fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult> {
        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        let request = JsonRpcRequest::new(
            self.next_id(),
            "tools/call",
            Some(serde_json::to_value(params)?),
        );

        let response = self.transport.call(request).await?;

        if let Some(error) = response.error {
            anyhow::bail!("Tool call failed: {} (code: {})", error.message, error.code);
        }

        let result: CallToolResult = serde_json::from_value(
            response.result.context("Missing result in tools/call response")?
        )?;

        Ok(result)
    }

    /// 检查服务器是否还在运行
    pub async fn is_alive(&self) -> bool {
        self.transport.is_alive().await
    }

    /// 关闭连接
    pub async fn shutdown(&self) -> Result<()> {
        self.transport.kill().await
    }
}

impl Drop for LocalMcpClient {
    fn drop(&mut self) {
        // 尝试优雅关闭
        // 注意：这里不能使用 async，所以只能尽力而为
    }
}
