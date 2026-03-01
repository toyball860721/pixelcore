use pixelcore_runtime::workflow::{
    Workflow, WorkflowNode, WorkflowExecutor, WorkflowPersistence, NodeType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Workflow Checkpoint Resume Example ===\n");

    // 创建一个多步骤工作流
    let mut workflow = Workflow::new("checkpoint_demo", "Workflow with checkpoint resume");

    let start = WorkflowNode::start("Start");
    let step1 = WorkflowNode::task("Step 1", "task1", serde_json::json!({"step": 1}));
    let step2 = WorkflowNode::task("Step 2", "task2", serde_json::json!({"step": 2}));
    let step3 = WorkflowNode::task("Step 3", "task3", serde_json::json!({"step": 3}));
    let end = WorkflowNode::end("End");

    let start_id = workflow.add_node(start);
    let step1_id = workflow.add_node(step1);
    let step2_id = workflow.add_node(step2);
    let step3_id = workflow.add_node(step3);
    let end_id = workflow.add_node(end);

    workflow.connect(start_id, step1_id);
    workflow.connect(step1_id, step2_id);
    workflow.connect(step2_id, step3_id);
    workflow.connect(step3_id, end_id);

    println!("Created workflow: {}", workflow.name);
    println!("Steps: Start -> Step1 -> Step2 -> Step3 -> End\n");

    // 场景1：正常执行并保存检查点
    println!("--- Scenario 1: Execute and Save Checkpoint ---");
    let executor = WorkflowExecutor::new(workflow.clone());

    // 执行工作流
    println!("Executing workflow...");
    let result = executor.execute().await;

    match result {
        Ok(context) => {
            println!("✓ Workflow completed successfully");
            println!("Status: {:?}", context.status);
            println!("Execution ID: {}", context.execution_id);

            // 保存检查点
            println!("\nSaving checkpoint...");
            WorkflowPersistence::save_workflow_with_context(
                &workflow,
                &context,
                "./checkpoint_data"
            ).await?;
            println!("✓ Checkpoint saved to ./checkpoint_data\n");
        }
        Err(e) => {
            println!("✗ Workflow failed: {}", e);
        }
    }

    // 场景2：从检查点恢复执行
    println!("--- Scenario 2: Resume from Checkpoint ---");
    println!("Loading checkpoint...");

    let (restored_workflow, restored_context) =
        WorkflowPersistence::load_workflow_with_context("./checkpoint_data").await?;

    println!("✓ Checkpoint loaded");
    println!("Workflow: {}", restored_workflow.name);
    println!("Execution ID: {}", restored_context.execution_id);
    println!("Status: {:?}", restored_context.status);
    println!("Current node: {:?}", restored_context.current_node);

    // 创建新的executor并从检查点恢复
    let resumed_executor = WorkflowExecutor::from_checkpoint(
        restored_workflow.clone(),
        restored_context.clone()
    );

    println!("\nResuming execution from checkpoint...");
    let resume_result = resumed_executor.resume().await;

    match resume_result {
        Ok(final_context) => {
            println!("✓ Workflow resumed and completed");
            println!("Final status: {:?}", final_context.status);
            println!("Execution ID: {}", final_context.execution_id);
        }
        Err(e) => {
            println!("✗ Resume failed: {}", e);
        }
    }

    // 场景3：模拟中断和恢复
    println!("\n--- Scenario 3: Simulated Interruption and Recovery ---");
    println!("Creating a new workflow execution...");

    let mut workflow2 = Workflow::new("interrupted_workflow", "Workflow that will be interrupted");

    let start2 = WorkflowNode::start("Start");
    let task1 = WorkflowNode::task("Task 1", "task1", serde_json::json!({"data": "A"}));
    let task2 = WorkflowNode::task("Task 2", "task2", serde_json::json!({"data": "B"}));
    let end2 = WorkflowNode::end("End");

    let start2_id = workflow2.add_node(start2);
    let task1_id = workflow2.add_node(task1);
    let task2_id = workflow2.add_node(task2);
    let end2_id = workflow2.add_node(end2);

    workflow2.connect(start2_id, task1_id);
    workflow2.connect(task1_id, task2_id);
    workflow2.connect(task2_id, end2_id);

    let executor2 = WorkflowExecutor::new(workflow2.clone());

    // 执行一部分
    println!("Starting execution...");
    let partial_result = executor2.execute().await;

    if let Ok(partial_context) = partial_result {
        println!("Execution completed (simulating interruption point)");

        // 保存检查点
        println!("Saving checkpoint before interruption...");
        WorkflowPersistence::save_workflow_with_context(
            &workflow2,
            &partial_context,
            "./checkpoint_interrupted"
        ).await?;
        println!("✓ Checkpoint saved");

        // 模拟程序重启，从检查点恢复
        println!("\n[Simulating program restart...]");
        println!("Loading checkpoint after restart...");

        let (recovered_workflow, recovered_context) =
            WorkflowPersistence::load_workflow_with_context("./checkpoint_interrupted").await?;

        println!("✓ Checkpoint loaded after restart");
        println!("Resuming from where we left off...");

        let recovered_executor = WorkflowExecutor::from_checkpoint(
            recovered_workflow,
            recovered_context
        );

        let final_result = recovered_executor.resume().await;

        match final_result {
            Ok(ctx) => {
                println!("✓ Workflow successfully recovered and completed");
                println!("Final status: {:?}", ctx.status);
            }
            Err(e) => {
                println!("✗ Recovery failed: {}", e);
            }
        }
    }

    // 清理
    println!("\n--- Cleanup ---");
    std::fs::remove_dir_all("./checkpoint_data").ok();
    std::fs::remove_dir_all("./checkpoint_interrupted").ok();
    println!("✓ Cleaned up checkpoint data");

    println!("\n=== Checkpoint Resume Example Complete ===");
    Ok(())
}
