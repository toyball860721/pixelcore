use pixelcore_heartbeat::{FlowMonitor, FlowStateMachineConfig};
use pixelcore_runtime::event::{Event, EventBus, EventKind};
use pixelcore_runtime::AgentId;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== 心流状态机演示 ===\n");

    // 创建事件总线
    let event_bus = EventBus::new();

    // 创建心流监控器（使用更宽松的配置以便演示）
    let config = FlowStateMachineConfig {
        working_min_rate: 0.5,
        deep_flow_min_rate: 2.0,
        hyperfocus_min_rate: 4.0,
        max_error_rate: 0.15,
        max_instability: 0.4,
        max_switch_frequency: 8.0,
        metrics_reset_interval: Duration::from_secs(60),
    };

    let monitor = FlowMonitor::new(event_bus.clone(), config);

    // 注册一个 Agent
    let agent_id = AgentId::new_v4();
    monitor.register_agent(agent_id).await;
    println!("已注册 Agent: {}\n", agent_id);

    // 启动监控
    monitor.run().await;

    // 订阅心流状态变化事件
    let mut receiver = event_bus.subscribe();
    tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    if let EventKind::Custom(ref kind) = event.kind {
                        if kind == "flow_state_changed" {
                            println!("🔄 心流状态变化: {}",
                                serde_json::to_string_pretty(&event.payload).unwrap());
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    println!("⚠️  跳过了 {} 个事件", n);
                }
                Err(_) => break,
            }
        }
    });

    // 模拟场景 1: 逐渐进入心流状态
    println!("📝 场景 1: 逐渐进入心流状态");
    println!("模拟快速完成多个任务...\n");

    for i in 1..=10 {
        // 发布任务开始事件
        event_bus.publish(Event::new(
            EventKind::TaskStarted,
            format!("agent:{}", agent_id),
            serde_json::json!({ "agent_id": agent_id.to_string(), "task_id": i }),
        )).unwrap();

        sleep(Duration::from_millis(100)).await;

        // 发布任务完成事件
        event_bus.publish(Event::new(
            EventKind::TaskCompleted,
            format!("agent:{}", agent_id),
            serde_json::json!({ "agent_id": agent_id.to_string(), "task_id": i }),
        )).unwrap();

        sleep(Duration::from_millis(200)).await;

        // 每 3 个任务后检查状态
        if i % 3 == 0 {
            // 等待事件处理
            sleep(Duration::from_millis(100)).await;

            if let Some(state) = monitor.get_flow_state(&agent_id).await {
                if let Some(metrics) = monitor.get_metrics_debug(&agent_id).await {
                    println!("  完成 {} 个任务后的状态: {:?}", i, state);
                    println!("    指标: {}", metrics);
                }
            }
        }
    }

    sleep(Duration::from_secs(1)).await;

    // 模拟场景 2: 任务失败导致心流下降
    println!("\n📝 场景 2: 任务失败影响心流");
    println!("模拟一些任务失败...\n");

    for i in 11..=15 {
        event_bus.publish(Event::new(
            EventKind::TaskStarted,
            format!("agent:{}", agent_id),
            serde_json::json!({ "agent_id": agent_id.to_string(), "task_id": i }),
        )).unwrap();

        sleep(Duration::from_millis(100)).await;

        // 一半的任务失败
        if i % 2 == 0 {
            event_bus.publish(Event::new(
                EventKind::TaskFailed,
                format!("agent:{}", agent_id),
                serde_json::json!({ "agent_id": agent_id.to_string(), "task_id": i }),
            )).unwrap();
            println!("  ❌ 任务 {} 失败", i);
        } else {
            event_bus.publish(Event::new(
                EventKind::TaskCompleted,
                format!("agent:{}", agent_id),
                serde_json::json!({ "agent_id": agent_id.to_string(), "task_id": i }),
            )).unwrap();
            println!("  ✅ 任务 {} 完成", i);
        }

        sleep(Duration::from_millis(200)).await;
    }

    if let Some(state) = monitor.get_flow_state(&agent_id).await {
        println!("\n  失败后的状态: {:?}", state);
    }

    sleep(Duration::from_secs(1)).await;

    // 模拟场景 3: Agent 停止
    println!("\n📝 场景 3: Agent 停止");
    event_bus.publish(Event::new(
        EventKind::AgentStopped,
        format!("agent:{}", agent_id),
        serde_json::json!({ "agent_id": agent_id.to_string() }),
    )).unwrap();

    sleep(Duration::from_millis(500)).await;

    if let Some(state) = monitor.get_flow_state(&agent_id).await {
        println!("  停止后的状态: {:?}", state);
    }

    println!("\n=== 演示结束 ===");
}
