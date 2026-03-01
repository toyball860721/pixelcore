use pixelcore_runtime::workflow::{
    Workflow, WorkflowNode, WorkflowExecutor, ExecutionContext, WorkflowPersistence,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Workflow Persistence Example ===\n");

    // Create a simple workflow
    let mut workflow = Workflow::new("data_processing", "A simple data processing workflow");

    let start = WorkflowNode::start("Start");
    let fetch = WorkflowNode::task("Fetch Data", "fetch_task", serde_json::json!({"source": "database"}));
    let process = WorkflowNode::task("Process Data", "process_task", serde_json::json!({"algorithm": "transform"}));
    let save = WorkflowNode::task("Save Results", "save_task", serde_json::json!({"destination": "storage"}));
    let end = WorkflowNode::end("End");

    let start_id = workflow.add_node(start);
    let fetch_id = workflow.add_node(fetch);
    let process_id = workflow.add_node(process);
    let save_id = workflow.add_node(save);
    let end_id = workflow.add_node(end);

    workflow.connect(start_id, fetch_id);
    workflow.connect(fetch_id, process_id);
    workflow.connect(process_id, save_id);
    workflow.connect(save_id, end_id);

    println!("Created workflow: {}", workflow.name);
    println!("Description: {}", workflow.description);
    println!("Nodes: {}\n", workflow.nodes.len());

    // Create the directory for persistence
    std::fs::create_dir_all("./workflow_data")?;

    // Scenario 1: Save workflow definition
    println!("--- Scenario 1: Save Workflow Definition ---");
    WorkflowPersistence::save_workflow(&workflow, "./workflow_data/workflow.json").await?;
    println!("✓ Workflow saved to disk\n");

    // Scenario 2: Load workflow definition
    println!("--- Scenario 2: Load Workflow Definition ---");
    let loaded_workflow = WorkflowPersistence::load_workflow("./workflow_data/workflow.json").await?;
    println!("✓ Workflow loaded: {}", loaded_workflow.name);
    println!("  Description: {}", loaded_workflow.description);
    println!("  Nodes: {}\n", loaded_workflow.nodes.len());

    // Scenario 3: Execute workflow and save execution context
    println!("--- Scenario 3: Execute and Save Context ---");
    let mut context = ExecutionContext::new(workflow.id);

    // Simulate partial execution
    context.set_variable("data_source", serde_json::json!("database"));
    context.set_variable("records_fetched", serde_json::json!(1000));
    context.set_node_result(start_id, serde_json::json!({"status": "completed"}));
    context.set_node_result(fetch_id, serde_json::json!({"status": "completed", "records": 1000}));
    context.current_node = Some(process_id);

    println!("Execution state:");
    println!("  Current node: {:?}", context.current_node);
    println!("  Node results: {} nodes completed", context.node_results.len());
    println!("  Variables: {:?}", context.variables);

    // Save both workflow and context
    WorkflowPersistence::save_workflow_with_context(&workflow, &context, "./workflow_data").await?;
    println!("✓ Workflow and context saved\n");

    // Scenario 4: Load and resume execution
    println!("--- Scenario 4: Load and Resume Execution ---");
    let (restored_workflow, restored_context) = WorkflowPersistence::load_workflow_with_context("./workflow_data").await?;

    println!("Restored execution state:");
    println!("  Workflow: {}", restored_workflow.name);
    println!("  Current node: {:?}", restored_context.current_node);
    println!("  Node results: {} nodes completed", restored_context.node_results.len());
    println!("  Variables: {:?}", restored_context.variables);
    println!("✓ Execution can continue from where it left off\n");

    // Cleanup
    println!("--- Cleanup ---");
    std::fs::remove_dir_all("./workflow_data").ok();
    println!("✓ Cleaned up test data\n");

    println!("=== Persistence Example Complete ===");
    Ok(())
}
