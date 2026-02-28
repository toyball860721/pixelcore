// 性能测试脚本
// 测试多 Agent 并发处理能力
// 运行: cargo run --example performance_test

use pixelcore_runtime::{Agent, AgentConfig, Message};
use pixelcore_agents::ClaudeAgent;
use pixelcore_skills::builtins::{create_compute_skills, create_data_skills};
use pixelcore_claw::ClawClient;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PixelCore 性能测试\n");

    // 测试 1: 单 Agent 响应时间
    println!("📊 测试 1: 单 Agent 响应时间");
    test_single_agent_latency().await?;
    println!();

    // 测试 2: 多 Agent 并发处理
    println!("📊 测试 2: 多 Agent 并发处理");
    test_concurrent_agents().await?;
    println!();

    // 测试 3: 长对话历史性能
    println!("📊 测试 3: 长对话历史性能");
    test_long_conversation().await?;
    println!();

    println!("✅ 所有性能测试完成！\n");

    Ok(())
}

async fn test_single_agent_latency() -> Result<(), Box<dyn std::error::Error>> {
    let mut agent = create_agent("latency-test")?;
    agent.start().await?;

    let start = Instant::now();
    let message = Message::user("你好");

    match agent.process(message).await {
        Ok(_) => {
            let duration = start.elapsed();
            println!("   响应时间: {:?}", duration);
        }
        Err(e) => {
            println!("   ❌ 测试失败: {}", e);
        }
    }

    Ok(())
}

async fn test_concurrent_agents() -> Result<(), Box<dyn std::error::Error>> {
    let agent_count = 3;
    let mut tasks = Vec::new();

    let start = Instant::now();

    for i in 0..agent_count {
        let task = tokio::spawn(async move {
            let api_key = std::env::var("SILICONFLOW_API_KEY").ok()?;
            let config = AgentConfig::new(&format!("concurrent-{}", i), "你是一个测试助手")
                .with_model("deepseek-ai/DeepSeek-V3");
            let client = ClawClient::siliconflow(api_key);
            let mut agent = ClaudeAgent::with_client(config, client);

            for skill in create_compute_skills() {
                agent.register_skill(skill);
            }

            agent.start().await.ok()?;
            let message = Message::user("计算 10 + 20");
            agent.process(message).await.ok()
        });
        tasks.push(task);
    }

    let mut success = 0;
    let mut failed = 0;

    for task in tasks {
        match task.await {
            Ok(Some(_)) => success += 1,
            _ => failed += 1,
        }
    }

    let duration = start.elapsed();
    println!("   并发 Agent 数: {}", agent_count);
    println!("   成功: {}, 失败: {}", success, failed);
    println!("   总耗时: {:?}", duration);
    println!("   平均耗时: {:?}", duration / agent_count);

    Ok(())
}

async fn test_long_conversation() -> Result<(), Box<dyn std::error::Error>> {
    let mut agent = create_agent("long-conversation")?;
    agent.start().await?;
    let message_count = 5;

    let start = Instant::now();

    for i in 0..message_count {
        let message = Message::user(format!("这是第 {} 条消息", i + 1));
        if let Err(e) = agent.process(message).await {
            println!("   ❌ 消息 {} 失败: {}", i + 1, e);
            break;
        }
    }

    let duration = start.elapsed();
    println!("   消息数: {}", message_count);
    println!("   总耗时: {:?}", duration);
    println!("   平均耗时: {:?}", duration / message_count);

    Ok(())
}

fn create_agent(name: &str) -> Result<ClaudeAgent, Box<dyn std::error::Error>> {
    let api_key = std::env::var("SILICONFLOW_API_KEY")?;

    let config = AgentConfig::new(name, "你是一个测试助手")
        .with_model("deepseek-ai/DeepSeek-V3");

    let client = ClawClient::siliconflow(api_key);
    let mut agent = ClaudeAgent::with_client(config, client);

    for skill in create_compute_skills() {
        agent.register_skill(skill);
    }

    for skill in create_data_skills() {
        agent.register_skill(skill);
    }

    Ok(agent)
}
