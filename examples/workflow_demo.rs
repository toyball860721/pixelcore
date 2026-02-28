// 工作流引擎示例
// 演示如何使用 Workflow 和 WorkflowExecutor
// 运行: cargo run --example workflow_demo

use pixelcore_runtime::{
    Workflow, WorkflowNode, WorkflowExecutor, ExecutionStatus
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 工作流引擎示例\n");

    // 场景 1: 简单的线性工作流
    println!("📝 场景 1: 简单的线性工作流");
    simple_workflow().await?;
    println!();

    // 场景 2: 带决策分支的工作流
    println!("📝 场景 2: 带决策分支的工作流");
    decision_workflow().await?;
    println!();

    // 场景 3: 多任务工作流
    println!("📝 场景 3: 多任务工作流");
    multi_task_workflow().await?;
    println!();

    println!("🎉 工作流引擎示例完成！\n");

    Ok(())
}

async fn simple_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // 创建工作流
    let mut workflow = Workflow::new(
        "简单工作流",
        "一个包含开始、任务和结束节点的简单工作流"
    );

    // 添加节点
    let start = WorkflowNode::start("开始");
    let task1 = WorkflowNode::task(
        "任务1",
        "process_data",
        serde_json::json!({"input": "hello"})
    );
    let task2 = WorkflowNode::task(
        "任务2",
        "transform_data",
        serde_json::json!({"operation": "uppercase"})
    );
    let end = WorkflowNode::end("结束");

    let start_id = workflow.add_node(start);
    let task1_id = workflow.add_node(task1);
    let task2_id = workflow.add_node(task2);
    let end_id = workflow.add_node(end);

    // 连接节点
    workflow.connect(start_id, task1_id);
    workflow.connect(task1_id, task2_id);
    workflow.connect(task2_id, end_id);

    println!("   工作流结构:");
    println!("   开始 → 任务1 → 任务2 → 结束");
    println!();

    // 验证工作流
    workflow.validate()?;
    println!("   ✅ 工作流验证通过");

    // 执行工作流
    let executor = WorkflowExecutor::new(workflow);
    let context = executor.execute().await?;

    println!("   ✅ 工作流执行完成");
    println!("   执行状态: {:?}", context.status);
    println!("   执行时间: {:?}", context.completed_at.unwrap() - context.started_at);
    println!("   节点结果数: {}", context.node_results.len());

    Ok(())
}

async fn decision_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // 创建带决策的工作流
    let mut workflow = Workflow::new(
        "决策工作流",
        "包含条件分支的工作流"
    );

    // 设置变量
    workflow.set_variable("value", serde_json::json!(100));

    // 添加节点
    let start = WorkflowNode::start("开始");
    let decision = WorkflowNode::decision("检查值", "value > 50");
    let task_high = WorkflowNode::task(
        "高值处理",
        "process_high_value",
        serde_json::json!({})
    );
    let task_low = WorkflowNode::task(
        "低值处理",
        "process_low_value",
        serde_json::json!({})
    );
    let end = WorkflowNode::end("结束");

    let start_id = workflow.add_node(start);
    let decision_id = workflow.add_node(decision);
    let task_high_id = workflow.add_node(task_high);
    let task_low_id = workflow.add_node(task_low);
    let end_id = workflow.add_node(end);

    // 连接节点
    workflow.connect(start_id, decision_id);

    // 决策分支
    use pixelcore_runtime::WorkflowEdge;
    workflow.add_edge(WorkflowEdge::branch(decision_id, task_high_id, true));
    workflow.add_edge(WorkflowEdge::branch(decision_id, task_low_id, false));

    workflow.connect(task_high_id, end_id);
    workflow.connect(task_low_id, end_id);

    println!("   工作流结构:");
    println!("   开始 → 决策");
    println!("            ├─ true → 高值处理 → 结束");
    println!("            └─ false → 低值处理 → 结束");
    println!();

    // 验证并执行
    workflow.validate()?;
    println!("   ✅ 工作流验证通过");

    let executor = WorkflowExecutor::new(workflow);
    let context = executor.execute().await?;

    println!("   ✅ 工作流执行完成");
    println!("   执行状态: {:?}", context.status);

    Ok(())
}

async fn multi_task_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // 创建多任务工作流
    let mut workflow = Workflow::new(
        "数据处理流水线",
        "包含多个数据处理任务的工作流"
    );

    // 添加节点
    let start = WorkflowNode::start("开始");

    let fetch = WorkflowNode::task(
        "获取数据",
        "fetch_data",
        serde_json::json!({"source": "database"})
    );

    let validate = WorkflowNode::task(
        "验证数据",
        "validate_data",
        serde_json::json!({"rules": ["not_null", "valid_format"]})
    );

    let transform = WorkflowNode::task(
        "转换数据",
        "transform_data",
        serde_json::json!({"format": "json"})
    );

    let save = WorkflowNode::task(
        "保存数据",
        "save_data",
        serde_json::json!({"destination": "storage"})
    );

    let end = WorkflowNode::end("结束");

    let start_id = workflow.add_node(start);
    let fetch_id = workflow.add_node(fetch);
    let validate_id = workflow.add_node(validate);
    let transform_id = workflow.add_node(transform);
    let save_id = workflow.add_node(save);
    let end_id = workflow.add_node(end);

    // 连接节点
    workflow.connect(start_id, fetch_id);
    workflow.connect(fetch_id, validate_id);
    workflow.connect(validate_id, transform_id);
    workflow.connect(transform_id, save_id);
    workflow.connect(save_id, end_id);

    println!("   工作流结构:");
    println!("   开始 → 获取数据 → 验证数据 → 转换数据 → 保存数据 → 结束");
    println!();

    // 验证并执行
    workflow.validate()?;
    println!("   ✅ 工作流验证通过");
    println!("   节点数: {}", workflow.nodes.len());
    println!("   边数: {}", workflow.edges.len());

    let executor = WorkflowExecutor::new(workflow);
    let context = executor.execute().await?;

    println!("   ✅ 工作流执行完成");
    println!("   执行状态: {:?}", context.status);
    println!("   执行的节点数: {}", context.node_results.len());

    // 显示每个节点的结果
    println!("\n   节点执行结果:");
    for (node_id, result) in &context.node_results {
        println!("     - {}: {:?}", node_id, result);
    }

    Ok(())
}
