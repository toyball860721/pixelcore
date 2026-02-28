use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use std::sync::Arc;
use anyhow::{Context, Result};
use crate::mcp_types::{JsonRpcRequest, JsonRpcResponse};

/// Stdio 传输层，用于与本地 MCP 服务器通信
pub struct StdioTransport {
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl StdioTransport {
    /// 启动一个新的 MCP 服务器进程
    pub async fn spawn(command: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn MCP server process")?;

        let stdin = child.stdin.take()
            .context("Failed to get stdin")?;
        let stdout = child.stdout.take()
            .context("Failed to get stdout")?;

        Ok(Self {
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
        })
    }

    /// 发送 JSON-RPC 请求
    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<()> {
        let json = serde_json::to_string(request)?;
        let mut stdin = self.stdin.lock().await;

        // MCP 使用换行符分隔的 JSON
        stdin.write_all(json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        tracing::debug!("Sent request: {}", json);
        Ok(())
    }

    /// 接收 JSON-RPC 响应
    pub async fn receive_response(&self) -> Result<JsonRpcResponse> {
        let mut stdout = self.stdout.lock().await;
        let mut line = String::new();

        stdout.read_line(&mut line).await
            .context("Failed to read response from MCP server")?;

        if line.is_empty() {
            anyhow::bail!("MCP server closed connection");
        }

        tracing::debug!("Received response: {}", line.trim());

        let response: JsonRpcResponse = serde_json::from_str(&line)
            .context("Failed to parse JSON-RPC response")?;

        Ok(response)
    }

    /// 发送请求并等待响应
    pub async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        self.send_request(&request).await?;
        self.receive_response().await
    }

    /// 检查进程是否还在运行
    pub async fn is_alive(&self) -> bool {
        let mut child = self.child.lock().await;
        child.try_wait().ok().flatten().is_none()
    }

    /// 终止进程
    pub async fn kill(&self) -> Result<()> {
        let mut child = self.child.lock().await;
        child.kill().await.context("Failed to kill MCP server process")
    }

    /// 等待进程退出
    pub async fn wait(&self) -> Result<std::process::ExitStatus> {
        let mut child = self.child.lock().await;
        child.wait().await.context("Failed to wait for MCP server process")
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // 尝试终止进程（非阻塞）
        if let Ok(mut child) = self.child.try_lock() {
            let _ = child.start_kill();
        }
    }
}
