// 工作流错误处理和重试示例
// 演示如何使用错误处理策略
// 运行: cargo run --example workflow_error_handling

use pixelcore_runtime::{
    Workflow, WorkflowNode, WorkflowExecutor,
    ErrorHandlingStrategy, RetryPolicy
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 工作流错误处理示例\n");

    // 场景 1: 失败时停止（默认行为）
    println!("📝 场景 1: 失败时停止工作流");
    fail_strategy_demo().await?;
    println!();

    // 场景 2: 忽略错误继续执行
    println!("📝 场景 2: 忽略错误继续执行");
    ignore_strategy_demo().await?;
    println!();

    // 场景 3: 重试策略
    println!("📝 场景 3: 重试策略");
    retry_strategy_demo().await?;
    println!();

    // 场景 4: 指数退避重试
    println!("📝 场景 4: 指数退避重试");
    exponential_backoff_demo().await?;
    println!();

    println!("🎉 错误处理示例完成！\n");

    Ok(())
}

async fn fail_strategy_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(
        "失败停止工作流",
        "演示失败时停止整个工作流"
    );

    let start = WorkflowNode::start("开始");
    let task1 = WorkflowNode::task("任务1", "task1", serde_json::json!({}));
    // 任务2 使用默认的 Fail 策略
    let task2 = WorkflowNode::task("任务2", "task2", serde_json::json!({}));
    let end = WorkflowNode::end("结束");

    let start_id = workflow.add_node(start);
    let task1_id = workflow.add_node(task1);
    let task2_id = workflow.add_node(task2);
    let end_id = workflow.add_node(end);

    workflow.connect(start_id, task1_id);
    workflow.connect(task1_id, task2_id);
    workflow.connect(task2_id, end_id);

    println!("   工作流: 开始 → 任务1 → 任务2(Fail) → 结束");
    println!("   策略: 任务失败时停止整个工作流");

    workflow.validate()?;
    let executor = WorkflowExecutor::new(workflow);
    let context = executor.execute().await?;

    println!("   ✅ 执行完成");
    println!("   状态: {:?}", context.status);

    Ok(())
}

async fn ignore_strategy_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(
        "忽略错误工作流",
        "演示忽略错误继续执行"
    );

    let start = WorkflowNode::start("开始");
    let task1 = WorkflowNode::task("任务1", "task1", serde_json::json!({}));

    // 任务2 使用 Ignore 策略
    let task2 = WorkflowNode::task("任务2", "task2", serde_json::json!({}))
        .with_error_handling(ErrorHandlingStrategy::Ignore);

    let task3 = WorkflowNode::task("任务3", "task3", serde_json::json!({}));
    let end = WorkflowNode::end("结束");

    let start_id = workflow.add_node(start);
    let task1_id = workflow.add_node(task1);
    let task2_id = workflow.add_node(task2);
    let task3_id = workflow.add_node(task3);
    let end_id = workflow.add_node(end);

    workflow.connect(start_id, task1_id);
    workflow.connect(task1_id, task2_id);
    workflow.connect(task2_id, task3_id);
    workflow.connect(task3_id, end_id);

    println!("   工作流: 开始 → 任务1 → 任务2(Ignore) → 任务3 → 结束");
    println!("   策略: 任务2 失败时忽略错误，继续执行任务3");

    workflow.validate()?;
    let executor = WorkflowExecutor::new(workflow);
    let context = executor.execute().await?;

    println!("   ✅ 执行完成");
    println!("   状态: {:?}", context.status);
    println!("   执行的节点数: {}", context.node_results.len());

    Ok(())
}

async fn retry_strategy_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(
        "重试工作流",
        "演示任务失败时自动重试"
    );

    let start = WorkflowNode::start("开始");

    // 任务使用重试策略：最多重试 3 次
    let task = WorkflowNode::task("重试任务", "retry_task", serde_json::json!({}))
        .with_error_handling(ErrorHandlingStrategy::Retry {
            policy: RetryPolicy::new(3).with_delay(500),
        });

    let end = WorkflowNode::end("结束");

    let start_id = workflow.add_node(start);
    let task_id = workflow.add_node(task);
    let end_id = workflow.add_node(end);

    workflow.connect(start_id, task_id);
    workflow.connect(task_id, end_id);

    println!("   工作流: 开始 → 重试任务(Retry 3次) → 结束");
    println!("   策略: 任务失败时最多重试 3 次，每次延迟 500ms");

    workflow.validate()?;
    let executor = WorkflowExecutor::new(workflow);
    let context = executor.execute().await?;

    println!("   ✅ 执行完成");
    println!("   状态: {:?}", context.status);

    Ok(())
}

async fn exponential_backoff_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(
        "指数退避工作流",
        "演示指数退避重试策略"
    );

    let start = WorkflowNode::start("开始");

    // 使用指数退避重试策略
    let policy = RetryPolicy::new(4)
        .with_delay(100)
        .with_exponential_backoff(true);

    let task = WorkflowNode::task("指数退避任务", "backoff_task", serde_json::json!({}))
        .with_error_handling(ErrorHandlingStrategy::Retry { policy: policy.clone() });

    let end = WorkflowNode::end("结束");

    let start_id = workflow.add_node(start);
    let task_id = workflow.add_node(task);
    let end_id = workflow.add_node(end);

    workflow.connect(start_id, task_id);
    workflow.connect(task_id, end_id);

    println!("   工作流: 开始 → 指数退避任务 → 结束");
    println!("   策略: 指数退避重试");
    println!("   重试延迟:");
    for i in 0..4 {
        println!("     - 第 {} 次重试: {}ms", i + 1, policy.calculate_delay(i));
    }

    workflow.validate()?;
    let executor = WorkflowExecutor::new(workflow);
    let context = executor.execute().await?;

    println!("   ✅ 执行完成");
    println!("   状态: {:?}", context.status);

    Ok(())
}
