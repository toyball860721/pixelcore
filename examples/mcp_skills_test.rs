use anyhow::Result;
use pixelcore_skills::{McpSkillProvider, Skill, SkillInput};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("=== MCP Skills Provider Test ===\n");

    // 1. 启动 MCP 服务器并加载 Skills
    println!("Starting MCP server and loading skills...");
    let provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_server/server.py"]
    ).await?;

    let skills = provider.skills();
    println!("Loaded {} MCP skills:\n", skills.len());

    // 2. 显示每个 Skill 的信息
    for skill in skills {
        println!("Skill: {}", skill.name());
        println!("  Description: {}", skill.description());
        println!("  Schema: {}", serde_json::to_string_pretty(&skill.input_schema())?);
        println!();
    }

    // 3. 测试直接调用 Skills
    println!("=== Testing Skills ===\n");

    // 测试 add
    println!("Test 1: add(5, 3)");
    let add_skill = &skills[0];
    let input = SkillInput {
        name: "add".to_string(),
        args: serde_json::json!({"a": 5, "b": 3}),
    };
    let result = add_skill.execute(input).await?;
    println!("Result: {:?}\n", result);

    // 测试 multiply
    println!("Test 2: multiply(4, 7)");
    let multiply_skill = &skills[1];
    let input = SkillInput {
        name: "multiply".to_string(),
        args: serde_json::json!({"a": 4, "b": 7}),
    };
    let result = multiply_skill.execute(input).await?;
    println!("Result: {:?}\n", result);

    // 测试 echo
    println!("Test 3: echo('Hello MCP Skills!')");
    let echo_skill = &skills[2];
    let input = SkillInput {
        name: "echo".to_string(),
        args: serde_json::json!({"text": "Hello MCP Skills!"}),
    };
    let result = echo_skill.execute(input).await?;
    println!("Result: {:?}\n", result);

    // 4. 清理
    println!("Shutting down...");
    provider.shutdown().await?;

    println!("Done!");
    Ok(())
}
