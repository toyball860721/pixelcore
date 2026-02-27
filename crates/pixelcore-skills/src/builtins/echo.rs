use async_trait::async_trait;
use crate::skill::{Skill, SkillInput, SkillOutput};
use crate::error::SkillError;

pub struct EchoSkill;

#[async_trait]
impl Skill for EchoSkill {
    fn name(&self) -> &str { "echo" }
    fn description(&self) -> &str { "Echo back the input message unchanged." }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": { "type": "string", "description": "The message to echo back." }
            },
            "required": ["message"]
        })
    }
    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let msg = input.args.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("missing 'message'".to_string()))?;
        Ok(SkillOutput::ok(serde_json::json!({ "message": msg })))
    }
}
