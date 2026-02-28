// 多 Agent 协作 + 任务调度综合示例
// 演示如何结合 MessageBus 和 TaskScheduler 实现完整的多 Agent 协作系统
// 运行: cargo run --example agent_task_system

use pixelcore_runtime::{
    Agent, AgentConfig, Message, MessageBus, BusMessage,
    TaskScheduler, Task, TaskPriority, TaskStatus, SchedulerConfig
};
use pixelcore_agents::ClaudeAgent;
use pixelcore_claw::ClawClient;
use pixelcore_skills::builtins::{create_compute_skills, create_data_skills};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 多 Agent 协作任务系统示例\n");

    // 创建消息总线和任务调度器
    let bus = Arc::new(MessageBus::new());
    let scheduler = Arc::new(TaskScheduler::new(SchedulerConfig {
        max_concurrent_tasks: 3,
        max_queue_size: 100,
    }));

    println!("✅ 系统初始化完成");
    println!("   - 消息总线: 已创建");
    println!("   - 任务调度器: 已创建 (最大并发: 3)\n");

    // 创建 Agent 团队
    println!("📝 创建 Agent 团队...");

    let coordinator = create_agent("协调者", "你是任务协调者，负责分配和管理任务").await?;
    let worker1 = create_agent("计算工作者", "你是计算专家，负责数学计算任务").await?;
    let worker2 = create_agent("数据工作者", "你是数据处理专家，负责数据处理任务").await?;

    let coordinator_id = coordinator.id();
    let worker1_id = worker1.id();
    let worker2_id = worker2.id();

    println!("✅ Agent 团队创建完成");
    println!("   - 协调者: {}", coordinator_id);
    println!("   - 计算工作者: {}", worker1_id);
    println!("   - 数据工作者: {}", worker2_id);
    println!();

    // 订阅消息
    let mut coordinator_rx = bus.subscribe_agent(coordinator_id).await;
    let mut worker1_rx = bus.subscribe_agent(worker1_id).await;
    let mut worker2_rx = bus.subscribe_agent(worker2_id).await;
    let mut task_rx = bus.subscribe_topic("task.new").await;

    // 场景: 协调者分配任务给工作者
    println!("📋 场景: 协调者分配任务流程\n");

    // 1. 协调者创建任务
    println!("1️⃣ 协调者创建任务");

    let task1 = Task::new(
        "计算任务: 100 + 200",
        TaskPriority::High,
        serde_json::json!({
            "type": "calculation",
            "expression": "100 + 200"
        })
    );

    let task2 = Task::new(
        "数据处理: 解析 JSON",
        TaskPriority::Normal,
        serde_json::json!({
            "type": "data_processing",
            "data": r#"{"name": "Alice", "age": 30}"#
        })
    );

    let task1_id = scheduler.submit(task1.clone()).await?;
    let task2_id = scheduler.submit(task2.clone()).await?;

    println!("   ✅ 创建了 2 个任务");
    println!("   - 任务 1: {} (优先级: {:?})", task1.name, task1.priority);
    println!("   - 任务 2: {} (优先级: {:?})", task2.name, task2.priority);
    println!();

    // 2. 协调者广播任务通知
    println!("2️⃣ 协调者广播任务通知");

    let broadcast = BusMessage::broadcast(
        coordinator_id,
        "task.new",
        serde_json::json!({
            "message": "有新任务可用",
            "count": 2
        })
    );

    bus.publish(broadcast).await;
    println!("   ✅ 广播消息已发送\n");

    // 3. 工作者接收通知
    println!("3️⃣ 工作者接收任务通知");

    tokio::select! {
        msg = task_rx.recv() => {
            if let Some(msg) = msg {
                println!("   ✅ 工作者收到通知: {:?}", msg.payload);
            }
        }
        _ = sleep(Duration::from_millis(100)) => {
            println!("   ⏱️ 超时");
        }
    }
    println!();

    // 4. 协调者分配任务给特定工作者
    println!("4️⃣ 协调者分配任务");

    // 获取高优先级任务
    if let Some(task) = scheduler.next_task().await {
        println!("   📤 分配任务给计算工作者: {}", task.name);

        scheduler.assign_task(task.id, worker1_id).await;

        let assign_msg = BusMessage::direct(
            coordinator_id,
            worker1_id,
            "task.assign",
            serde_json::json!({
                "task_id": task.id.to_string(),
                "task_name": task.name,
                "payload": task.payload
            })
        );

        bus.publish(assign_msg).await;
    }

    // 获取普通任务
    if let Some(task) = scheduler.next_task().await {
        println!("   📤 分配任务给数据工作者: {}", task.name);

        scheduler.assign_task(task.id, worker2_id).await;

        let assign_msg = BusMessage::direct(
            coordinator_id,
            worker2_id,
            "task.assign",
            serde_json::json!({
                "task_id": task.id.to_string(),
                "task_name": task.name,
                "payload": task.payload
            })
        );

        bus.publish(assign_msg).await;
    }
    println!();

    // 5. 工作者接收任务
    println!("5️⃣ 工作者接收并执行任务");

    // 工作者 1 接收任务
    tokio::select! {
        msg = worker1_rx.recv() => {
            if let Some(msg) = msg {
                println!("   ✅ 计算工作者收到任务");
                println!("      任务: {:?}", msg.payload.get("task_name"));

                // 模拟执行
                scheduler.update_task_status(task1_id, TaskStatus::Running).await;
                sleep(Duration::from_millis(50)).await;
                scheduler.set_task_result(task1_id, serde_json::json!({"result": 300})).await;

                println!("      ✅ 任务完成，结果: 300");

                // 回复协调者
                let result_msg = BusMessage::direct(
                    worker1_id,
                    coordinator_id,
                    "task.result",
                    serde_json::json!({
                        "task_id": task1_id.to_string(),
                        "result": 300,
                        "status": "completed"
                    })
                );
                bus.publish(result_msg).await;
            }
        }
        _ = sleep(Duration::from_millis(200)) => {}
    }

    // 工作者 2 接收任务
    tokio::select! {
        msg = worker2_rx.recv() => {
            if let Some(msg) = msg {
                println!("   ✅ 数据工作者收到任务");
                println!("      任务: {:?}", msg.payload.get("task_name"));

                // 模拟执行
                scheduler.update_task_status(task2_id, TaskStatus::Running).await;
                sleep(Duration::from_millis(50)).await;
                scheduler.set_task_result(
                    task2_id,
                    serde_json::json!({"name": "Alice", "age": 30})
                ).await;

                println!("      ✅ 任务完成");

                // 回复协调者
                let result_msg = BusMessage::direct(
                    worker2_id,
                    coordinator_id,
                    "task.result",
                    serde_json::json!({
                        "task_id": task2_id.to_string(),
                        "result": {"name": "Alice", "age": 30},
                        "status": "completed"
                    })
                );
                bus.publish(result_msg).await;
            }
        }
        _ = sleep(Duration::from_millis(200)) => {}
    }
    println!();

    // 6. 协调者接收结果
    println!("6️⃣ 协调者接收任务结果");

    let mut results_received = 0;
    while results_received < 2 {
        tokio::select! {
            msg = coordinator_rx.recv() => {
                if let Some(msg) = msg {
                    if msg.topic == "task.result" {
                        println!("   ✅ 收到结果: {:?}", msg.payload.get("result"));
                        results_received += 1;
                    }
                }
            }
            _ = sleep(Duration::from_millis(200)) => {
                break;
            }
        }
    }
    println!();

    // 最终统计
    println!("📊 系统统计:");
    println!("   消息总线:");
    println!("     - Agent 订阅者: {}", bus.agent_subscriber_count().await);
    println!("     - 主题订阅者: {}", bus.topic_subscriber_count("task.new").await);
    println!();
    println!("   任务调度器:");
    println!("     - 总任务数: {}", scheduler.get_all_tasks().await.len());
    println!("     - 已完成: {}", scheduler.get_tasks_by_status(TaskStatus::Completed).await.len());
    println!("     - 运行中: {}", scheduler.running_tasks_count().await);
    println!("     - 队列长度: {}", scheduler.queue_length().await);
    println!();

    println!("🎉 多 Agent 协作任务系统示例完成！\n");

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
