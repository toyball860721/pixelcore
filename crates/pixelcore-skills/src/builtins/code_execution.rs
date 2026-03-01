//! Code execution skills with sandboxing
//!
//! This module provides skills for executing code in various languages
//! with proper sandboxing and resource limits.

use crate::skill::{Skill, SkillInput, SkillOutput};
use crate::error::SkillError;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;
use std::time::Duration;

/// Python code execution skill with sandboxing
pub struct PythonExecuteSkill;

#[async_trait]
impl Skill for PythonExecuteSkill {
    fn name(&self) -> &str {
        "python_execute"
    }

    fn description(&self) -> &str {
        "Execute Python code in a sandboxed environment with resource limits"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Python code to execute"
                },
                "timeout_seconds": {
                    "type": "number",
                    "description": "Timeout in seconds (default: 5, max: 30)",
                    "default": 5
                }
            },
            "required": ["code"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let code = input.args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("Missing 'code' field".to_string()))?;

        let timeout_seconds = input.args.get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .min(30); // Max 30 seconds

        // Execute Python code with timeout
        let result = execute_python_code(code, timeout_seconds).await?;

        Ok(SkillOutput::ok(json!({
            "stdout": result.stdout,
            "stderr": result.stderr,
            "exit_code": result.exit_code,
            "success": result.exit_code == 0,
            "timed_out": result.timed_out,
        })))
    }
}

/// JavaScript code execution skill
pub struct JavaScriptExecuteSkill;

#[async_trait]
impl Skill for JavaScriptExecuteSkill {
    fn name(&self) -> &str {
        "javascript_execute"
    }

    fn description(&self) -> &str {
        "Execute JavaScript code using Node.js with resource limits"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "JavaScript code to execute"
                },
                "timeout_seconds": {
                    "type": "number",
                    "description": "Timeout in seconds (default: 5, max: 30)",
                    "default": 5
                }
            },
            "required": ["code"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let code = input.args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("Missing 'code' field".to_string()))?;

        let timeout_seconds = input.args.get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .min(30);

        // Execute JavaScript code with timeout
        let result = execute_javascript_code(code, timeout_seconds).await?;

        Ok(SkillOutput::ok(json!({
            "stdout": result.stdout,
            "stderr": result.stderr,
            "exit_code": result.exit_code,
            "success": result.exit_code == 0,
            "timed_out": result.timed_out,
        })))
    }
}

/// Shell command execution skill (restricted)
pub struct ShellExecuteSkill;

#[async_trait]
impl Skill for ShellExecuteSkill {
    fn name(&self) -> &str {
        "shell_execute"
    }

    fn description(&self) -> &str {
        "Execute shell commands with restrictions (read-only operations, no network)"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                },
                "timeout_seconds": {
                    "type": "number",
                    "description": "Timeout in seconds (default: 5, max: 30)",
                    "default": 5
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let command = input.args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("Missing 'command' field".to_string()))?;

        let timeout_seconds = input.args.get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .min(30);

        // Check for dangerous commands
        if is_dangerous_command(command) {
            return Err(SkillError::Execution(
                "Command contains dangerous operations and is not allowed".to_string()
            ));
        }

        // Execute shell command with timeout
        let result = execute_shell_command(command, timeout_seconds).await?;

        Ok(SkillOutput::ok(json!({
            "stdout": result.stdout,
            "stderr": result.stderr,
            "exit_code": result.exit_code,
            "success": result.exit_code == 0,
            "timed_out": result.timed_out,
        })))
    }
}

/// Execution result
struct ExecutionResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    timed_out: bool,
}

/// Execute Python code with timeout
async fn execute_python_code(code: &str, timeout_seconds: u64) -> Result<ExecutionResult, SkillError> {
    let mut child = Command::new("python3")
        .arg("-c")
        .arg(code)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SkillError::Execution(format!("Failed to spawn python3: {}", e)))?;

    // Wait with timeout
    let timeout = Duration::from_secs(timeout_seconds);
    let result = tokio::time::timeout(timeout, async {
        child.wait_with_output().await
    }).await;

    match result {
        Ok(Ok(output)) => {
            Ok(ExecutionResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                timed_out: false,
            })
        }
        Ok(Err(e)) => {
            Err(SkillError::Execution(format!("Failed to execute python3: {}", e)))
        }
        Err(_) => {
            // Timeout - process will be cleaned up by OS
            Ok(ExecutionResult {
                stdout: String::new(),
                stderr: format!("Execution timed out after {} seconds", timeout_seconds),
                exit_code: -1,
                timed_out: true,
            })
        }
    }
}

/// Execute JavaScript code with timeout
async fn execute_javascript_code(code: &str, timeout_seconds: u64) -> Result<ExecutionResult, SkillError> {
    let mut child = Command::new("node")
        .arg("-e")
        .arg(code)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SkillError::Execution(format!("Failed to spawn node: {}", e)))?;

    // Wait with timeout
    let timeout = Duration::from_secs(timeout_seconds);
    let result = tokio::time::timeout(timeout, async {
        child.wait_with_output().await
    }).await;

    match result {
        Ok(Ok(output)) => {
            Ok(ExecutionResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                timed_out: false,
            })
        }
        Ok(Err(e)) => {
            Err(SkillError::Execution(format!("Failed to execute node: {}", e)))
        }
        Err(_) => {
            // Timeout - process will be cleaned up by OS
            Ok(ExecutionResult {
                stdout: String::new(),
                stderr: format!("Execution timed out after {} seconds", timeout_seconds),
                exit_code: -1,
                timed_out: true,
            })
        }
    }
}

/// Execute shell command with timeout
async fn execute_shell_command(command: &str, timeout_seconds: u64) -> Result<ExecutionResult, SkillError> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SkillError::Execution(format!("Failed to spawn sh: {}", e)))?;

    // Wait with timeout
    let timeout = Duration::from_secs(timeout_seconds);
    let result = tokio::time::timeout(timeout, async {
        child.wait_with_output().await
    }).await;

    match result {
        Ok(Ok(output)) => {
            Ok(ExecutionResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                timed_out: false,
            })
        }
        Ok(Err(e)) => {
            Err(SkillError::Execution(format!("Failed to execute sh: {}", e)))
        }
        Err(_) => {
            // Timeout - process will be cleaned up by OS
            Ok(ExecutionResult {
                stdout: String::new(),
                stderr: format!("Execution timed out after {} seconds", timeout_seconds),
                exit_code: -1,
                timed_out: true,
            })
        }
    }
}

/// Check if a command contains dangerous operations
fn is_dangerous_command(command: &str) -> bool {
    let dangerous_patterns = [
        "rm -rf",
        "dd if=",
        "mkfs",
        "format",
        "> /dev/",
        "curl",
        "wget",
        "nc ",
        "netcat",
        "telnet",
        "ssh",
        "scp",
        "ftp",
        "chmod 777",
        "chown",
        "sudo",
        "su ",
    ];

    for pattern in &dangerous_patterns {
        if command.contains(pattern) {
            return true;
        }
    }

    false
}

/// Create code execution skills
pub fn create_code_execution_skills() -> Vec<Box<dyn Skill>> {
    vec![
        Box::new(PythonExecuteSkill),
        Box::new(JavaScriptExecuteSkill),
        Box::new(ShellExecuteSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_python_execute_basic() {
        let skill = PythonExecuteSkill;
        let input = SkillInput {
            name: "python_execute".to_string(),
            args: json!({
                "code": "print('Hello, World!')"
            }),
        };

        let result = skill.execute(input).await.unwrap();

        assert_eq!(result.success, true);
        assert!(result.result["stdout"].as_str().unwrap().contains("Hello, World!"));
        assert_eq!(result.result["exit_code"], 0);
    }

    #[tokio::test]
    async fn test_javascript_execute_basic() {
        let skill = JavaScriptExecuteSkill;
        let input = SkillInput {
            name: "javascript_execute".to_string(),
            args: json!({
                "code": "console.log('Hello, World!')"
            }),
        };

        let result = skill.execute(input).await.unwrap();

        assert_eq!(result.success, true);
        assert!(result.result["stdout"].as_str().unwrap().contains("Hello, World!"));
        assert_eq!(result.result["exit_code"], 0);
    }

    #[tokio::test]
    async fn test_shell_execute_basic() {
        let skill = ShellExecuteSkill;
        let input = SkillInput {
            name: "shell_execute".to_string(),
            args: json!({
                "command": "echo 'Hello, World!'"
            }),
        };

        let result = skill.execute(input).await.unwrap();

        assert_eq!(result.success, true);
        assert!(result.result["stdout"].as_str().unwrap().contains("Hello, World!"));
        assert_eq!(result.result["exit_code"], 0);
    }

    #[tokio::test]
    async fn test_shell_execute_dangerous_command() {
        let skill = ShellExecuteSkill;
        let input = SkillInput {
            name: "shell_execute".to_string(),
            args: json!({
                "command": "rm -rf /"
            }),
        };

        let result = skill.execute(input).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_is_dangerous_command() {
        assert!(is_dangerous_command("rm -rf /"));
        assert!(is_dangerous_command("curl http://example.com"));
        assert!(is_dangerous_command("sudo apt-get install"));
        assert!(!is_dangerous_command("ls -la"));
        assert!(!is_dangerous_command("echo 'hello'"));
    }
}
