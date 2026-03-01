use pixelcore_runtime::workflow::{
    Workflow, WorkflowNode, WorkflowExecutor, NodeType, WorkflowEdge, EdgeCondition,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Workflow Parallel Example ===\n");

    // 创建一个包含并行节点的工作流
    let mut workflow = Workflow::new("parallel_example", "Workflow with parallel execution");

    // 创建节点
    let start = WorkflowNode::start("Start");

    // 并行节点：同时执行3个分支
    let parallel_node = WorkflowNode::new(
        "Parallel Tasks",
        NodeType::Parallel {
            branches: 3,
        }
    );

    // 3个并行任务
    let task1 = WorkflowNode::task(
        "Task 1",
        "parallel_task_1",
        serde_json::json!({"id": 1, "message": "Processing branch 1"})
    );

    let task2 = WorkflowNode::task(
        "Task 2",
        "parallel_task_2",
        serde_json::json!({"id": 2, "message": "Processing branch 2"})
    );

    let task3 = WorkflowNode::task(
        "Task 3",
        "parallel_task_3",
        serde_json::json!({"id": 3, "message": "Processing branch 3"})
    );

    let end = WorkflowNode::end("End");

    let start_id = workflow.add_node(start);
    let parallel_id = workflow.add_node(parallel_node);
    let task1_id = workflow.add_node(task1);
    let task2_id = workflow.add_node(task2);
    let task3_id = workflow.add_node(task3);
    let end_id = workflow.add_node(end);

    // 连接节点
    workflow.connect(start_id, parallel_id);

    // 并行分支：使用ParallelBranch边连接
    workflow.add_edge(WorkflowEdge::parallel_branch(parallel_id, task1_id, 0));
    workflow.add_edge(WorkflowEdge::parallel_branch(parallel_id, task2_id, 1));
    workflow.add_edge(WorkflowEdge::parallel_branch(parallel_id, task3_id, 2));

    // 汇聚节点：并行执行完成后继续
    workflow.connect_when(parallel_id, end_id, "all_branches_complete");

    println!("Created workflow: {}", workflow.name);
    println!("Description: {}", workflow.description);
    println!("Parallel branches: 3\n");

    // 执行工作流
    println!("--- Executing Workflow ---");
    println!("Starting parallel execution of 3 tasks...\n");

    let executor = WorkflowExecutor::new(workflow);
    let result = executor.execute().await;

    match result {
        Ok(context) => {
            println!("\n✓ Workflow completed successfully");
            println!("Status: {:?}", context.status);
            println!("Execution ID: {}", context.execution_id);

            println!("\nNode results:");
            for (node_id, result) in &context.node_results {
                println!("  Node {}: {:?}", node_id, result);
            }

            println!("\nAll {} parallel branches completed successfully!", 3);
        }
        Err(e) => {
            println!("\n✗ Workflow failed: {}", e);
        }
    }

    println!("\n=== Parallel Example Complete ===");
    Ok(())
}
