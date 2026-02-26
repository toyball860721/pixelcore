use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::SkillError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

impl SkillOutput {
    pub fn ok(result: serde_json::Value) -> Self {
        Self { success: true, result, error: None }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, result: serde_json::Value::Null, error: Some(msg.into()) }
    }
}

#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError>;
}
