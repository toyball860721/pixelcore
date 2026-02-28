// Tauri 命令测试脚本
// 这个脚本可以独立运行，测试 PixelCore Runtime 的核心功能

use pixelcore_runtime::{Agent, AgentConfig, Message, RuntimeError};
use pixelcore_agents::ClaudeAgent;
use pixelcore_skills::{SkillRegistry, builtins};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 开始测试 PixelCore 应用功能...\n");

    // 测试 1: 创建 Agent
    println!("📝 测试 1: 创建 Agent");
    let mut agent = create_test_agent("test-agent-1", "你是一个测试助手")?;
    println!("✅ Agent 创建成功: test-agent-1\n");

    // 测试 2: 获取可用技能
    println!("📝 测试 2: 获取可用技能");
    let skills = get_available_skills();
    println!("✅ 找到 {} 个技能:", skills.len());
    for skill in skills.iter().take(5) {
        println!("   - {}: {}", skill.name, skill.description);
    }
    if skills.len() > 5 {
        println!("   ... 还有 {} 个技能", skills.len() - 5);
    }
    println!();

    // 测试 3: 发送简单消息
    println!("📝 测试 3: 发送简单消息");
    let message = Message {
        role: "user".to_string(),
        content: "你好，请介绍一下你自己".to_string(),
    };

    match agent.send_message(message).await {
        Ok(response) => {
            println!("✅ 收到回复: {}",
                if response.len() > 100 {
                    format!("{}...", &response[..100])
                } else {
                    response
                }
            );
        }
        Err(e) => {
            println!("❌ 发送消息失败: {}", e);
        }
    }
    println!();

    // 测试 4: 测试计算技能
    println!("📝 测试 4: 测试计算技能");
    let calc_message = Message {
        role: "user".to_string(),
        content: "请帮我计算 (15 + 25) * 2 的结果".to_string(),
    };

    match agent.send_message(calc_message).await {
        Ok(response) => {
            println!("✅ 计算结果: {}",
                if response.len() > 150 {
                    format!("{}...", &response[..150])
                } else {
                    response
                }
            );
        }
        Err(e) => {
            println!("❌ 计算失败: {}", e);
        }
    }
    println!();

    // 测试 5: 测试数据处理技能
    println!("📝 测试 5: 测试 JSON 解析技能");
    let json_message = Message {
        role: "user".to_string(),
        content: r#"请解析这个 JSON: {"name": "Alice", "age": 30, "city": "Beijing"}"#.to_string(),
    };

    match agent.send_message(json_message).await {
        Ok(response) => {
            println!("✅ JSON 解析结果: {}",
                if response.len() > 150 {
                    format!("{}...", &response[..150])
                } else {
                    response
                }
            );
        }
        Err(e) => {
            println!("❌ JSON 解析失败: {}", e);
        }
    }
    println!();

    println!("🎉 所有测试完成！\n");

    Ok(())
}

fn create_test_agent(id: &str, system_prompt: &str) -> Result<ClaudeAgent, RuntimeError> {
    let api_key = std::env::var("SILICONFLOW_API_KEY")
        .map_err(|_| RuntimeError::ConfigError("SILICONFLOW_API_KEY not set".to_string()))?;

    let config = AgentConfig {
        id: id.to_string(),
        name: format!("Test Agent {}", id),
        system_prompt: system_prompt.to_string(),
        model: "deepseek-ai/DeepSeek-V3".to_string(),
        api_key,
        api_base: "https://api.siliconflow.cn/v1".to_string(),
        max_tokens: 2000,
        temperature: 0.7,
    };

    let mut registry = SkillRegistry::new();
    builtins::register_all(&mut registry);

    ClaudeAgent::new(config, registry)
}

#[derive(Debug)]
struct SkillInfo {
    name: String,
    description: String,
}

fn get_available_skills() -> Vec<SkillInfo> {
    let mut registry = SkillRegistry::new();
    builtins::register_all(&mut registry);

    registry.list_skills()
        .into_iter()
        .map(|skill| SkillInfo {
            name: skill.name,
            description: skill.description,
        })
        .collect()
}
