//! Comprehensive integration demo
//!
//! This example demonstrates how to combine multiple PixelCore features:
//! - Workflow engine
//! - Code execution skills
//! - Skill registry

use pixelcore_runtime::workflow::{Workflow, WorkflowNode};
use pixelcore_skills::{
    SkillRegistry, SkillInput,
    builtins::code_execution::{PythonExecuteSkill, JavaScriptExecuteSkill, ShellExecuteSkill},
};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PixelCore Comprehensive Integration Demo ===\n");

    // Demo 1: Workflow Definition
    println!("Demo 1: Creating a Data Processing Workflow");
    println!("─────────────────────────────────────────────");

    let mut workflow = Workflow::new(
        "data_processing",
        "A workflow that processes data using Python and JavaScript"
    );

    // Add nodes using helper methods
    let start = WorkflowNode::start("start");
    let python_task = WorkflowNode::task(
        "generate_data",
        "python_execute",
        json!({
            "code": "data = [i * 2 for i in range(1, 6)]\nprint(data)"
        })
    );
    let js_task = WorkflowNode::task(
        "process_data",
        "javascript_execute",
        json!({
            "code": "const data = [2, 4, 6, 8, 10]; console.log(data.reduce((a, b) => a + b, 0));"
        })
    );
    let end = WorkflowNode::end("end");

    // Add nodes to workflow
    let start_id = workflow.add_node(start);
    let python_id = workflow.add_node(python_task);
    let js_id = workflow.add_node(js_task);
    let end_id = workflow.add_node(end);

    // Connect nodes
    workflow.connect(start_id, python_id);
    workflow.connect(python_id, js_id);
    workflow.connect(js_id, end_id);

    println!("✓ Created workflow with {} nodes and {} edges",
        workflow.nodes.len(), workflow.edges.len());
    println!("✓ Workflow validation: {:?}\n", workflow.validate());

    // Demo 2: Skill Registry and Execution
    println!("Demo 2: Executing Code with Skills");
    println!("───────────────────────────────────");

    let mut registry = SkillRegistry::new();
    registry.register(Arc::new(PythonExecuteSkill));
    registry.register(Arc::new(JavaScriptExecuteSkill));
    registry.register(Arc::new(ShellExecuteSkill));

    println!("✓ Registered {} skills\n", registry.list().len());

    // Execute Python code
    println!("Executing Python: Calculate sum of squares");
    let python_input = SkillInput {
        name: "python_execute".to_string(),
        args: json!({
            "code": "result = sum([i**2 for i in range(1, 11)])\nprint(f'Sum of squares 1-10: {result}')"
        }),
    };

    match registry.get("python_execute") {
        Ok(skill) => {
            match skill.execute(python_input).await {
                Ok(output) => {
                    println!("✓ Python output:\n{}", output.result);
                }
                Err(e) => println!("✗ Python error: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to get skill: {}", e),
    }

    // Execute JavaScript code
    println!("\nExecuting JavaScript: Array operations");
    let js_input = SkillInput {
        name: "javascript_execute".to_string(),
        args: json!({
            "code": "const arr = [1, 2, 3, 4, 5]; const doubled = arr.map(x => x * 2); console.log('Doubled:', doubled.join(', '));"
        }),
    };

    match registry.get("javascript_execute") {
        Ok(skill) => {
            match skill.execute(js_input).await {
                Ok(output) => {
                    println!("✓ JavaScript output:\n{}", output.result);
                }
                Err(e) => println!("✗ JavaScript error: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to get skill: {}", e),
    }

    // Execute Shell command
    println!("\nExecuting Shell: System info");
    let shell_input = SkillInput {
        name: "shell_execute".to_string(),
        args: json!({
            "command": "echo 'System:' && uname -s && echo 'Date:' && date '+%Y-%m-%d'"
        }),
    };

    match registry.get("shell_execute") {
        Ok(skill) => {
            match skill.execute(shell_input).await {
                Ok(output) => {
                    println!("✓ Shell output:\n{}", output.result);
                }
                Err(e) => println!("✗ Shell error: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to get skill: {}", e),
    }

    println!("\n=== Demo Complete ===");
    println!("This demo showcased:");
    println!("  • Workflow creation with multiple nodes");
    println!("  • Skill registration and execution");
    println!("  • Python, JavaScript, and Shell code execution");
    println!("  • Error handling and output formatting");

    Ok(())
}
