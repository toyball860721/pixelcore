use pixelcore_heartbeat::{FlowStateMachine, FlowStateMachineConfig, FlowState, FlowLevel};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== 心流状态机直接测试 ===\n");

    // 创建状态机（使用更宽松的配置）
    let config = FlowStateMachineConfig {
        working_min_rate: 0.5,
        deep_flow_min_rate: 2.0,
        hyperfocus_min_rate: 4.0,
        max_error_rate: 0.15,
        max_instability: 0.4,
        max_switch_frequency: 8.0,
        metrics_reset_interval: Duration::from_secs(60),
    };

    let mut machine = FlowStateMachine::new(config);

    println!("初始状态: {:?}\n", machine.state());

    // 场景 1: 快速完成多个任务
    println!("📝 场景 1: 快速完成多个任务");

    for i in 1..=10 {
        machine.task_started();
        sleep(Duration::from_millis(50)).await;
        machine.task_completed();
        sleep(Duration::from_millis(50)).await;

        if i % 3 == 0 {
            let metrics = machine.metrics();
            println!(
                "  完成 {} 个任务: 状态={:?}, 完成速率={:.2}/min, 错误率={:.2}",
                i,
                machine.state(),
                metrics.completion_rate(),
                metrics.error_rate()
            );
        }
    }

    println!();

    // 场景 2: 任务失败
    println!("📝 场景 2: 一些任务失败");

    for i in 11..=15 {
        machine.task_started();
        sleep(Duration::from_millis(50)).await;

        if i % 2 == 0 {
            machine.task_failed();
            println!("  ❌ 任务 {} 失败", i);
        } else {
            machine.task_completed();
            println!("  ✅ 任务 {} 完成", i);
        }

        sleep(Duration::from_millis(50)).await;
    }

    let metrics = machine.metrics();
    println!(
        "\n  失败后: 状态={:?}, 完成速率={:.2}/min, 错误率={:.2}",
        machine.state(),
        metrics.completion_rate(),
        metrics.error_rate()
    );

    println!();

    // 场景 3: 设置为 Idle
    println!("📝 场景 3: 设置为 Idle");
    machine.set_idle();
    println!("  状态: {:?}", machine.state());

    println!("\n=== 测试结束 ===");
}
