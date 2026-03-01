use pixelcore_runtime::workflow::{
    Workflow, WorkflowNode, WorkflowExecutor, NodeType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Workflow Loop Example ===\n");

    // 创建一个包含循环的工作流
    let mut workflow = Workflow::new("loop_example", "Workflow with loop node");

    // 创建节点
    let start = WorkflowNode::start("Start");

    // 循环节点：使用"true"作为条件，依赖max_iterations限制循环次数
    let loop_node = WorkflowNode::new(
        "Loop Counter",
        NodeType::Loop {
            condition: "true".to_string(),  // 简单条件，总是为真
            max_iterations: 5,  // 最多循环5次
        }
    );

    let task_in_loop = WorkflowNode::task(
        "Task in Loop",
        "loop_task",
        serde_json::json!({"message": "Executing loop iteration"})
    );

    let end = WorkflowNode::end("End");

    let start_id = workflow.add_node(start);
    let loop_id = workflow.add_node(loop_node);
    let task_id = workflow.add_node(task_in_loop);
    let end_id = workflow.add_node(end);

    // 连接节点
    workflow.connect(start_id, loop_id);
    workflow.connect(loop_id, task_id);  // 循环体：loop -> task
    workflow.connect_when(loop_id, end_id, "exit");  // 循环退出：loop -> end

    println!("Created workflow: {}", workflow.name);
    println!("Description: {}", workflow.description);
    println!("Loop max iterations: 5\n");

    // 执行工作流
    println!("--- Executing Workflow ---");
    let executor = WorkflowExecutor::new(workflow);
    let result = executor.execute().await;

    match result {
        Ok(context) => {
            println!("\n✓ Workflow completed successfully");
            println!("Status: {:?}", context.status);
            println!("Execution ID: {}", context.execution_id);

            // 显示循环迭代信息
            let loop_var_key = format!("loop_{}_iteration", loop_id);
            if let Some(iterations) = context.get_variable(&loop_var_key) {
                println!("Final iteration count: {}", iterations);
            }

            println!("\nNode results:");
            for (node_id, result) in &context.node_results {
                println!("  Node {}: {:?}", node_id, result);
            }
        }
        Err(e) => {
            println!("\n✗ Workflow failed: {}", e);
        }
    }

    println!("\n=== Loop Example Complete ===");
    Ok(())
}
