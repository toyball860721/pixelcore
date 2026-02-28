use anyhow::Result;
use pixelcore_skills::{McpSkillProvider, SkillInput};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("=== Testing All MCP Servers ===\n");

    // 1. 测试文件系统服务器
    println!("--- Testing Filesystem Server ---");
    test_filesystem_server().await?;

    // 2. 测试时间服务器
    println!("\n--- Testing Time Server ---");
    test_time_server().await?;

    // 3. 测试 HTTP 服务器（可选，需要网络）
    println!("\n--- Testing HTTP Server ---");
    if let Err(e) = test_http_server().await {
        println!("HTTP server test skipped: {}", e);
    }

    println!("\n=== All Tests Completed ===");
    Ok(())
}

async fn test_filesystem_server() -> Result<()> {
    let provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/filesystem_server.py", "/tmp"]
    ).await?;

    let skills = provider.skills();
    println!("Loaded {} filesystem skills", skills.len());

    // 测试 write_file
    let write_skill = skills.iter().find(|s| s.name() == "write_file").unwrap();
    let input = SkillInput {
        name: "write_file".to_string(),
        args: serde_json::json!({
            "path": "test_mcp.txt",
            "content": "Hello from MCP!"
        }),
    };
    let result = write_skill.execute(input).await?;
    println!("write_file: {:?}", result);

    // 测试 read_file
    let read_skill = skills.iter().find(|s| s.name() == "read_file").unwrap();
    let input = SkillInput {
        name: "read_file".to_string(),
        args: serde_json::json!({"path": "test_mcp.txt"}),
    };
    let result = read_skill.execute(input).await?;
    println!("read_file: {:?}", result);

    // 测试 file_exists
    let exists_skill = skills.iter().find(|s| s.name() == "file_exists").unwrap();
    let input = SkillInput {
        name: "file_exists".to_string(),
        args: serde_json::json!({"path": "test_mcp.txt"}),
    };
    let result = exists_skill.execute(input).await?;
    println!("file_exists: {:?}", result);

    provider.shutdown().await?;
    Ok(())
}

async fn test_time_server() -> Result<()> {
    let provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/time_server.py"]
    ).await?;

    let skills = provider.skills();
    println!("Loaded {} time skills", skills.len());

    // 测试 get_current_time
    let time_skill = skills.iter().find(|s| s.name() == "get_current_time").unwrap();
    let input = SkillInput {
        name: "get_current_time".to_string(),
        args: serde_json::json!({}),
    };
    let result = time_skill.execute(input).await?;
    println!("get_current_time: {:?}", result);

    // 测试 format_time
    let format_skill = skills.iter().find(|s| s.name() == "format_time").unwrap();
    let input = SkillInput {
        name: "format_time".to_string(),
        args: serde_json::json!({
            "time": "2024-01-01T12:00:00",
            "format": "%Y年%m月%d日 %H:%M:%S"
        }),
    };
    let result = format_skill.execute(input).await?;
    println!("format_time: {:?}", result);

    // 测试 add_time
    let add_skill = skills.iter().find(|s| s.name() == "add_time").unwrap();
    let input = SkillInput {
        name: "add_time".to_string(),
        args: serde_json::json!({
            "time": "2024-01-01T00:00:00",
            "days": 7,
            "hours": 3
        }),
    };
    let result = add_skill.execute(input).await?;
    println!("add_time: {:?}", result);

    provider.shutdown().await?;
    Ok(())
}

async fn test_http_server() -> Result<()> {
    let provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/http_server.py"]
    ).await?;

    let skills = provider.skills();
    println!("Loaded {} HTTP skills", skills.len());

    // 测试 http_get（使用公共 API）
    let get_skill = skills.iter().find(|s| s.name() == "http_get").unwrap();
    let input = SkillInput {
        name: "http_get".to_string(),
        args: serde_json::json!({
            "url": "https://httpbin.org/get"
        }),
    };
    let result = get_skill.execute(input).await?;
    println!("http_get: success={}", result.success);

    provider.shutdown().await?;
    Ok(())
}
