//! Code execution skills demo
//!
//! This example demonstrates how to use Python, JavaScript, and Shell
//! code execution skills with proper sandboxing and security.

use pixelcore_skills::{PythonExecuteSkill, JavaScriptExecuteSkill, ShellExecuteSkill, Skill, SkillInput};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Code Execution Skills Demo ===\n");

    // Demo 1: Python code execution
    println!("Demo 1: Python Code Execution");
    println!("─────────────────────────────");

    let python_skill = PythonExecuteSkill;

    // Example 1: Simple calculation
    println!("\n1. Simple calculation:");
    let input = SkillInput {
        name: "python_execute".to_string(),
        args: json!({
            "code": "result = 2 + 2\nprint(f'2 + 2 = {result}')"
        }),
    };

    let result = python_skill.execute(input).await?;
    println!("   Success: {}", result.success);
    println!("   Output: {}", result.result["stdout"].as_str().unwrap().trim());

    // Example 2: List comprehension
    println!("\n2. List comprehension:");
    let input = SkillInput {
        name: "python_execute".to_string(),
        args: json!({
            "code": "squares = [x**2 for x in range(1, 6)]\nprint(f'Squares: {squares}')"
        }),
    };

    let result = python_skill.execute(input).await?;
    println!("   Output: {}", result.result["stdout"].as_str().unwrap().trim());

    // Example 3: JSON processing
    println!("\n3. JSON processing:");
    let input = SkillInput {
        name: "python_execute".to_string(),
        args: json!({
            "code": r#"
import json
data = {'name': 'Alice', 'age': 30, 'city': 'NYC'}
print(json.dumps(data, indent=2))
"#
        }),
    };

    let result = python_skill.execute(input).await?;
    println!("   Output:\n{}", result.result["stdout"].as_str().unwrap().trim());

    // Demo 2: JavaScript code execution
    println!("\n\nDemo 2: JavaScript Code Execution");
    println!("─────────────────────────────");

    let js_skill = JavaScriptExecuteSkill;

    // Example 1: Array operations
    println!("\n1. Array operations:");
    let input = SkillInput {
        name: "javascript_execute".to_string(),
        args: json!({
            "code": "const numbers = [1, 2, 3, 4, 5];\nconst doubled = numbers.map(x => x * 2);\nconsole.log('Doubled:', doubled);"
        }),
    };

    let result = js_skill.execute(input).await?;
    println!("   Output: {}", result.result["stdout"].as_str().unwrap().trim());

    // Example 2: Object manipulation
    println!("\n2. Object manipulation:");
    let input = SkillInput {
        name: "javascript_execute".to_string(),
        args: json!({
            "code": r#"
const user = { name: 'Bob', age: 25 };
const updated = { ...user, age: 26, city: 'LA' };
console.log(JSON.stringify(updated, null, 2));
"#
        }),
    };

    let result = js_skill.execute(input).await?;
    println!("   Output:\n{}", result.result["stdout"].as_str().unwrap().trim());

    // Example 3: Async/await
    println!("\n3. Async/await:");
    let input = SkillInput {
        name: "javascript_execute".to_string(),
        args: json!({
            "code": r#"
async function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
    console.log('Start');
    await delay(100);
    console.log('After 100ms');
}

main();
"#
        }),
    };

    let result = js_skill.execute(input).await?;
    println!("   Output: {}", result.result["stdout"].as_str().unwrap().trim());

    // Demo 3: Shell command execution
    println!("\n\nDemo 3: Shell Command Execution");
    println!("─────────────────────────────");

    let shell_skill = ShellExecuteSkill;

    // Example 1: List files
    println!("\n1. List files:");
    let input = SkillInput {
        name: "shell_execute".to_string(),
        args: json!({
            "command": "ls -la | head -5"
        }),
    };

    let result = shell_skill.execute(input).await?;
    println!("   Output:\n{}", result.result["stdout"].as_str().unwrap().trim());

    // Example 2: System information
    println!("\n2. System information:");
    let input = SkillInput {
        name: "shell_execute".to_string(),
        args: json!({
            "command": "uname -a"
        }),
    };

    let result = shell_skill.execute(input).await?;
    println!("   Output: {}", result.result["stdout"].as_str().unwrap().trim());

    // Example 3: Text processing
    println!("\n3. Text processing:");
    let input = SkillInput {
        name: "shell_execute".to_string(),
        args: json!({
            "command": "echo 'Hello World' | tr '[:lower:]' '[:upper:]'"
        }),
    };

    let result = shell_skill.execute(input).await?;
    println!("   Output: {}", result.result["stdout"].as_str().unwrap().trim());

    // Demo 4: Security - Dangerous command blocked
    println!("\n\nDemo 4: Security - Dangerous Command Blocked");
    println!("─────────────────────────────");

    println!("\nAttempting to run dangerous command: rm -rf /");
    let input = SkillInput {
        name: "shell_execute".to_string(),
        args: json!({
            "command": "rm -rf /"
        }),
    };

    match shell_skill.execute(input).await {
        Ok(_) => println!("   ❌ Command was allowed (should not happen!)"),
        Err(e) => println!("   ✓ Command blocked: {}", e),
    }

    // Demo 5: Timeout protection
    println!("\n\nDemo 5: Timeout Protection");
    println!("─────────────────────────────");

    println!("\nRunning infinite loop with 2 second timeout:");
    let input = SkillInput {
        name: "python_execute".to_string(),
        args: json!({
            "code": "import time\nwhile True:\n    time.sleep(1)",
            "timeout_seconds": 2
        }),
    };

    let result = python_skill.execute(input).await?;
    println!("   Timed out: {}", result.result["timed_out"]);
    println!("   Error: {}", result.result["stderr"].as_str().unwrap());

    println!("\n=== Demo completed successfully ===");
    Ok(())
}
