use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use pixelcore_runtime::{Agent, AgentConfig, Message};
use pixelcore_agents::ClaudeAgent;
use pixelcore_skills::McpSkillProvider;
use pixelcore_swarm::Swarm;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("=== MCP Skills Integration Demo ===\n");

    // 加载环境变量
    dotenvy::dotenv().ok();

    // 1. 启动 MCP 服务器并获取 Skills
    println!("Starting MCP server and loading skills...");
    let mcp_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_server/server.py"]
    ).await?;

    let skills = mcp_provider.skills();
    println!("Loaded {} MCP skills:", skills.len());
    for skill in skills {
        println!("  - {}: {}", skill.name(), skill.description());
    }

    // 2. 创建 Agent 并注册 MCP Skills
    println!("\nCreating agent with MCP skills...");
    let api_key = std::env::var("SILICONFLOW_API_KEY")
        .expect("SILICONFLOW_API_KEY not found");
    let model_name = std::env::var("MODEL_NAME")
        .unwrap_or_else(|_| "Pro/MiniMaxAI/MiniMax-M2.5".to_string());

    let config = AgentConfig::new(
        "mcp-demo-agent",
        "You are a helpful assistant with access to MCP tools. Use them to help the user.",
    ).with_model(&model_name);

    let client = pixelcore_claw::ClawClient::siliconflow(&api_key);
    let mut agent = ClaudeAgent::with_client(config, client);

    // 注册所有 MCP Skills
    for skill in skills {
        agent.register_skill(Arc::clone(skill));
    }

    agent.start().await?;

    // 3. 使用 Swarm 管理 Agent
    let swarm = Swarm::new();
    let agent_id = agent.id();
    swarm.add(Arc::new(Mutex::new(agent))).await;

    // 4. 测试 MCP 工具调用
    println!("\n=== Testing MCP Tools ===\n");

    let test_cases = vec![
        "Please add 15 and 27 using the add tool.",
        "Multiply 8 by 9 using the multiply tool.",
        "Echo back the message: 'MCP integration works!'",
    ];

    for (i, input) in test_cases.iter().enumerate() {
        println!("[Test {}] User: {}", i + 1, input);
        let reply = swarm.route(&agent_id, Message::user(*input)).await?;
        println!("[Test {}] Assistant: {}\n", i + 1, reply.content);
    }

    // 5. 清理
    println!("Shutting down...");
    let agent_arc = swarm.get(&agent_id).await?;
    agent_arc.lock().await.stop().await?;

    mcp_provider.shutdown().await?;

    println!("Done!");
    Ok(())
}
