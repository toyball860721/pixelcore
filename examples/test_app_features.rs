// Tauri 应用功能测试
// 运行: cargo run --example test_app_features

use pixelcore_runtime::{Agent, AgentConfig, Message};
use pixelcore_agents::ClaudeAgent;
use pixelcore_skills::builtins::{create_compute_skills, create_data_skills};
use pixelcore_claw::ClawClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 开始测试 PixelCore 应用功能...\n");

    // 测试 1: 创建 Agent
    println!("📝 测试 1: 创建 Agent");
    let mut agent = create_test_agent("测试助手")?;
    agent.start().await?;
    println!("✅ Agent 创建并启动成功\n");

    // 测试 2: 获取可用技能
    println!("📝 测试 2: 获取可用技能");
    let compute_skills = create_compute_skills();
    let data_skills = create_data_skills();
    let total_skills = compute_skills.len() + data_skills.len();
    println!("✅ 找到 {} 个技能:", total_skills);
    println!("   计算技能: {}", compute_skills.len());
    println!("   数据处理技能: {}", data_skills.len());
    println!();

    // 测试 3: 发送简单消息
    println!("📝 测试 3: 发送简单消息");
    let message = Message::user("你好，请用一句话介绍你自己");

    match agent.process(message).await {
        Ok(response) => {
            println!("✅ 收到回复:");
            println!("   {}", truncate(&response.content, 200));
        }
        Err(e) => {
            println!("❌ 发送消息失败: {}", e);
        }
    }
    println!();

    // 测试 4: 测试计算技能
    println!("📝 测试 4: 测试计算技能");
    let calc_message = Message::user("请帮我计算 (15 + 25) * 2 的结果");

    match agent.process(calc_message).await {
        Ok(response) => {
            println!("✅ 计算结果:");
            println!("   {}", truncate(&response.content, 250));
        }
        Err(e) => {
            println!("❌ 计算失败: {}", e);
        }
    }
    println!();

    // 测试 5: 测试数据处理技能
    println!("📝 测试 5: 测试 JSON 解析技能");
    let json_message = Message::user(
        r#"请解析这个 JSON: {"name": "Alice", "age": 30, "city": "Beijing"}"#
    );

    match agent.process(json_message).await {
        Ok(response) => {
            println!("✅ JSON 解析结果:");
            println!("   {}", truncate(&response.content, 250));
        }
        Err(e) => {
            println!("❌ JSON 解析失败: {}", e);
        }
    }
    println!();

    println!("🎉 所有测试完成！");
    println!("\n💡 提示: 如果看到 API 调用失败，请检查:");
    println!("   1. SILICONFLOW_API_KEY 环境变量是否设置");
    println!("   2. 网络连接是否正常");
    println!("   3. API 配额是否充足\n");

    Ok(())
}

fn create_test_agent(name: &str) -> Result<ClaudeAgent, Box<dyn std::error::Error>> {
    let api_key = std::env::var("SILICONFLOW_API_KEY")?;

    let config = AgentConfig::new(name, "你是一个测试助手，可以使用各种技能帮助用户")
        .with_model("deepseek-ai/DeepSeek-V3");

    let client = ClawClient::siliconflow(api_key);
    let mut agent = ClaudeAgent::with_client(config, client);

    // 注册计算技能
    for skill in create_compute_skills() {
        agent.register_skill(skill);
    }

    // 注册数据处理技能
    for skill in create_data_skills() {
        agent.register_skill(skill);
    }

    Ok(agent)
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
