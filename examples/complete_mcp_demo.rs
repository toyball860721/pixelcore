use anyhow::Result;
use pixelcore_runtime::{Agent, AgentConfig, Message};
use pixelcore_agents::ClaudeAgent;
use pixelcore_skills::McpSkillProvider;
use pixelcore_storage::Storage;
use pixelcore_claw::ClawClient;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                                                              ║");
    println!("║           完整的 MCP 服务器集成示例                           ║");
    println!("║                                                              ║");
    println!("║  演示如何将多个 MCP 服务器集成到 Agent 中                     ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // 加载环境变量
    dotenvy::dotenv().ok();

    // 检查 API key
    let api_key = match std::env::var("SILICONFLOW_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("⚠️  未找到 SILICONFLOW_API_KEY 环境变量");
            println!("   将只测试 MCP 服务器，不测试 Agent 集成");
            println!("\n   如需测试 Agent 集成，请设置环境变量：");
            println!("   export SILICONFLOW_API_KEY=your-api-key\n");

            // 只测试 MCP 服务器
            test_mcp_servers_only().await?;
            return Ok(());
        }
    };

    // 完整测试（包括 Agent 集成）
    test_with_agent(&api_key).await?;

    Ok(())
}

/// 只测试 MCP 服务器（不需要 API key）
async fn test_mcp_servers_only() -> Result<()> {
    println!("═══ 第一部分：启动 MCP 服务器 ═══\n");

    // 1. 启动文件系统服务器
    println!("1. 启动文件系统服务器...");
    let fs_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/filesystem_server.py", "/tmp"]
    ).await?;
    println!("   ✅ 文件系统服务器已启动，提供 {} 个工具", fs_provider.skills().len());
    for skill in fs_provider.skills() {
        println!("      - {}: {}", skill.name(), skill.description());
    }

    // 2. 启动时间服务器
    println!("\n2. 启动时间工具服务器...");
    let time_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/time_server.py"]
    ).await?;
    println!("   ✅ 时间服务器已启动，提供 {} 个工具", time_provider.skills().len());
    for skill in time_provider.skills() {
        println!("      - {}: {}", skill.name(), skill.description());
    }

    // 3. 尝试启动 HTTP 服务器
    println!("\n3. 启动 HTTP API 服务器...");
    match McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/http_server.py"]
    ).await {
        Ok(http_provider) => {
            println!("   ✅ HTTP 服务器已启动，提供 {} 个工具", http_provider.skills().len());
            for skill in http_provider.skills() {
                println!("      - {}: {}", skill.name(), skill.description());
            }
            http_provider.shutdown().await?;
        }
        Err(e) => {
            println!("   ⚠️  HTTP 服务器启动失败: {}", e);
            println!("      提示: 运行 'pip install requests' 安装依赖");
        }
    }

    println!("\n═══ 第二部分：测试工具调用 ═══\n");

    // 测试文件系统工具
    test_filesystem_tools(&fs_provider).await?;

    // 测试时间工具
    test_time_tools(&time_provider).await?;

    // 清理
    println!("\n═══ 清理资源 ═══\n");
    fs_provider.shutdown().await?;
    time_provider.shutdown().await?;
    println!("✅ 所有服务器已关闭");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  测试完成！                                                   ║");
    println!("║                                                              ║");
    println!("║  如需测试 Agent 集成，请设置 SILICONFLOW_API_KEY             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}

/// 测试文件系统工具
async fn test_filesystem_tools(provider: &McpSkillProvider) -> Result<()> {
    use pixelcore_skills::SkillInput;

    println!("测试文件系统工具:");

    // 写入文件
    let write_skill = provider.skills().iter()
        .find(|s| s.name() == "write_file")
        .unwrap();

    let input = SkillInput {
        name: "write_file".to_string(),
        args: serde_json::json!({
            "path": "mcp_demo.txt",
            "content": "Hello from PixelCore MCP!\n这是一个测试文件。\nTest complete."
        }),
    };

    let result = write_skill.execute(input).await?;
    println!("  ✅ write_file: {}", if result.success { "成功" } else { "失败" });

    // 读取文件
    let read_skill = provider.skills().iter()
        .find(|s| s.name() == "read_file")
        .unwrap();

    let input = SkillInput {
        name: "read_file".to_string(),
        args: serde_json::json!({"path": "mcp_demo.txt"}),
    };

    let result = read_skill.execute(input).await?;
    println!("  ✅ read_file: {}", if result.success { "成功" } else { "失败" });
    if result.success {
        println!("     内容: {:?}", result.result);
    }

    Ok(())
}

/// 测试时间工具
async fn test_time_tools(provider: &McpSkillProvider) -> Result<()> {
    use pixelcore_skills::SkillInput;

    println!("\n测试时间工具:");

    // 获取当前时间
    let time_skill = provider.skills().iter()
        .find(|s| s.name() == "get_current_time")
        .unwrap();

    let input = SkillInput {
        name: "get_current_time".to_string(),
        args: serde_json::json!({}),
    };

    let result = time_skill.execute(input).await?;
    println!("  ✅ get_current_time: {}", result.result);

    // 格式化时间
    let format_skill = provider.skills().iter()
        .find(|s| s.name() == "format_time")
        .unwrap();

    let input = SkillInput {
        name: "format_time".to_string(),
        args: serde_json::json!({
            "time": "2024-12-25T18:30:00",
            "format": "%Y年%m月%d日 %H时%M分"
        }),
    };

    let result = format_skill.execute(input).await?;
    println!("  ✅ format_time: {}", result.result);

    Ok(())
}

/// 完整测试（包括 Agent 集成）
async fn test_with_agent(api_key: &str) -> Result<()> {
    println!("═══ 第一部分：启动 MCP 服务器 ═══\n");

    // 1. 启动文件系统服务器
    println!("1. 启动文件系统服务器...");
    let fs_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/filesystem_server.py", "/tmp"]
    ).await?;
    println!("   ✅ 文件系统服务器已启动，提供 {} 个工具", fs_provider.skills().len());

    // 2. 启动时间服务器
    println!("\n2. 启动时间工具服务器...");
    let time_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/time_server.py"]
    ).await?;
    println!("   ✅ 时间服务器已启动，提供 {} 个工具", time_provider.skills().len());

    println!("\n═══ 第二部分：创建 Agent 并集成 MCP 工具 ═══\n");

    // 创建 Storage
    let storage = Storage::new();

    // 创建 ClawClient (使用 SiliconFlow API)
    let client = ClawClient::siliconflow(api_key);

    // 创建 Agent 配置
    let config = AgentConfig::new(
        "MCP Demo Agent",
        "你是一个智能助手，可以使用文件系统和时间工具来帮助用户。\n\
         当用户需要文件操作时，使用 filesystem 工具。\n\
         当用户需要时间信息时，使用 time 工具。"
    ).with_model("deepseek-ai/DeepSeek-V3");

    // 创建 Agent
    let mut agent = ClaudeAgent::with_client(config, client)
        .with_storage(storage);

    // 注册 MCP 技能
    println!("注册 MCP 技能到 Agent...");
    for skill in fs_provider.skills() {
        agent.register_skill(skill.clone());
    }
    for skill in time_provider.skills() {
        agent.register_skill(skill.clone());
    }
    println!("✅ 已注册 {} 个技能\n", fs_provider.skills().len() + time_provider.skills().len());

    // 启动 Agent
    agent.start().await?;

    println!("═══ 第三部分：测试 Agent 使用 MCP 工具 ═══\n");

    // 测试 1: 文件操作
    println!("【测试 1】让 Agent 创建并读取文件\n");
    let message = Message::user(
        "请帮我创建一个文件 /tmp/agent_test.txt，内容是：\n\
         这是由 Agent 创建的测试文件。\n\
         创建时间：2024-12-25\n\
         然后读取这个文件的内容给我看。"
    );

    match agent.process(message).await {
        Ok(response) => {
            println!("Agent 响应:\n{}\n", response.content);
        }
        Err(e) => {
            println!("❌ Agent 处理失败: {}\n", e);
        }
    }

    // 测试 2: 时间操作
    println!("\n【测试 2】让 Agent 处理时间信息\n");
    let message = Message::user(
        "请告诉我现在的时间，并将 2024-12-25T18:30:00 格式化为中文格式。"
    );

    match agent.process(message).await {
        Ok(response) => {
            println!("Agent 响应:\n{}\n", response.content);
        }
        Err(e) => {
            println!("❌ Agent 处理失败: {}\n", e);
        }
    }

    // 测试 3: 组合操作
    println!("\n【测试 3】让 Agent 执行组合任务\n");
    let message = Message::user(
        "请创建一个文件 /tmp/time_log.txt，内容包含当前时间和一条日志信息：'系统运行正常'。"
    );

    match agent.process(message).await {
        Ok(response) => {
            println!("Agent 响应:\n{}\n", response.content);
        }
        Err(e) => {
            println!("❌ Agent 处理失败: {}\n", e);
        }
    }

    // 清理
    println!("\n═══ 清理资源 ═══\n");
    fs_provider.shutdown().await?;
    time_provider.shutdown().await?;
    println!("✅ 所有服务器已关闭");

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  完整测试成功！                                               ║");
    println!("║                                                              ║");
    println!("║  Agent 成功使用了 MCP 工具来完成任务                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
