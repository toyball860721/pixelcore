// 多 Agent 协作示例
// 演示如何使用 MessageBus 实现 Agent 间通信
// 运行: cargo run --example multi_agent_collaboration

use pixelcore_runtime::{Agent, AgentConfig, Message, MessageBus, BusMessage};
use pixelcore_agents::ClaudeAgent;
use pixelcore_claw::ClawClient;
use pixelcore_skills::builtins::{create_compute_skills, create_data_skills};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤝 多 Agent 协作示例\n");

    // 创建消息总线
    let bus = Arc::new(MessageBus::new());

    // 创建 3 个 Agent
    println!("📝 创建 3 个 Agent...");
    let coordinator = create_agent("协调者", "你是协调者，负责分配任务").await?;
    let worker1 = create_agent("工作者1", "你是工作者，负责执行计算任务").await?;
    let worker2 = create_agent("工作者2", "你是工作者，负责执行数据处理任务").await?;

    let coordinator_id = coordinator.id();
    let worker1_id = worker1.id();
    let worker2_id = worker2.id();

    println!("✅ Agent 创建完成");
    println!("   - 协调者 ID: {}", coordinator_id);
    println!("   - 工作者1 ID: {}", worker1_id);
    println!("   - 工作者2 ID: {}", worker2_id);
    println!();

    // 订阅消息
    println!("📡 设置消息订阅...");
    let mut coordinator_rx = bus.subscribe_agent(coordinator_id).await;
    let mut worker1_rx = bus.subscribe_agent(worker1_id).await;
    let mut worker2_rx = bus.subscribe_agent(worker2_id).await;
    println!("✅ 订阅完成\n");

    // 场景 1: 协调者广播任务
    println!("📢 场景 1: 协调者广播任务");
    let mut task_rx = bus.subscribe_topic("task.broadcast").await;

    let broadcast_msg = BusMessage::broadcast(
        coordinator_id,
        "task.broadcast",
        serde_json::json!({
            "task": "计算 100 + 200",
            "priority": "high"
        })
    );

    bus.publish(broadcast_msg).await;
    println!("   协调者发送广播任务");

    // 模拟 worker 接收任务
    if let Ok(msg) = tokio::time::timeout(Duration::from_millis(100), task_rx.recv()).await {
        if let Some(msg) = msg {
            println!("   ✅ 工作者接收到任务: {:?}", msg.payload);
        }
    }
    println!();

    // 场景 2: 点对点消息
    println!("📨 场景 2: 协调者向工作者1发送直接消息");
    let direct_msg = BusMessage::direct(
        coordinator_id,
        worker1_id,
        "task.assign",
        serde_json::json!({
            "task": "计算 50 * 2",
            "deadline": "5s"
        })
    );

    bus.publish(direct_msg).await;
    println!("   协调者 -> 工作者1: 分配任务");

    if let Ok(msg) = tokio::time::timeout(Duration::from_millis(100), worker1_rx.recv()).await {
        if let Some(msg) = msg {
            println!("   ✅ 工作者1 收到: {:?}", msg.payload);
        }
    }
    println!();

    // 场景 3: 工作者回复结果
    println!("📬 场景 3: 工作者1回复结果给协调者");
    let result_msg = BusMessage::direct(
        worker1_id,
        coordinator_id,
        "task.result",
        serde_json::json!({
            "task_id": "task-001",
            "result": 100,
            "status": "completed"
        })
    );

    bus.publish(result_msg).await;
    println!("   工作者1 -> 协调者: 任务完成");

    if let Ok(msg) = tokio::time::timeout(Duration::from_millis(100), coordinator_rx.recv()).await {
        if let Some(msg) = msg {
            println!("   ✅ 协调者收到结果: {:?}", msg.payload);
        }
    }
    println!();

    // 统计信息
    println!("📊 消息总线统计:");
    println!("   - task.broadcast 订阅者: {}", bus.topic_subscriber_count("task.broadcast").await);
    println!("   - Agent 订阅者总数: {}", bus.agent_subscriber_count().await);
    println!();

    println!("🎉 多 Agent 协作示例完成！\n");

    Ok(())
}

async fn create_agent(name: &str, system_prompt: &str) -> Result<ClaudeAgent, Box<dyn std::error::Error>> {
    let api_key = std::env::var("SILICONFLOW_API_KEY")?;

    let config = AgentConfig::new(name, system_prompt)
        .with_model("deepseek-ai/DeepSeek-V3");

    let client = ClawClient::siliconflow(api_key);
    let mut agent = ClaudeAgent::with_client(config, client);

    for skill in create_compute_skills() {
        agent.register_skill(skill);
    }

    for skill in create_data_skills() {
        agent.register_skill(skill);
    }

    agent.start().await?;

    Ok(agent)
}
