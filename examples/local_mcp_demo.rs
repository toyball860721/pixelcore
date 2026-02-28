use pixelcore_claw::LocalMcpClient;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("=== Local MCP Client Example ===\n");

    // 启动 MCP 服务器
    println!("Starting MCP server...");
    let client = LocalMcpClient::new(
        "python3",
        &["examples/mcp_server/server.py"]
    ).await?;

    // 显示服务器信息
    if let Some(info) = client.server_info() {
        println!("Server: {} v{}", info.name, info.version);
    }

    // 列出所有工具
    println!("\nListing available tools...");
    let tools = client.list_tools().await?;
    for tool in &tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // 测试 add 工具
    println!("\nCalling 'add' tool with a=5, b=3...");
    let result = client.call_tool(
        "add",
        Some(serde_json::json!({"a": 5, "b": 3}))
    ).await?;
    println!("Result: {:?}", result);

    // 测试 multiply 工具
    println!("\nCalling 'multiply' tool with a=4, b=7...");
    let result = client.call_tool(
        "multiply",
        Some(serde_json::json!({"a": 4, "b": 7}))
    ).await?;
    println!("Result: {:?}", result);

    // 测试 echo 工具
    println!("\nCalling 'echo' tool with text='Hello MCP!'...");
    let result = client.call_tool(
        "echo",
        Some(serde_json::json!({"text": "Hello MCP!"}))
    ).await?;
    println!("Result: {:?}", result);

    // 关闭连接
    println!("\nShutting down...");
    client.shutdown().await?;

    println!("Done!");
    Ok(())
}
