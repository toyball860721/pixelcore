use async_trait::async_trait;
use std::sync::Arc;
use crate::skill::{Skill, SkillInput, SkillOutput};
use crate::error::SkillError;
use pixelcore_runtime::agent::AgentId;
use pixelcore_runtime::message::Message;
use pixelcore_swarm::coordinator::Coordinator;

pub struct DelegateSkill {
    pub coordinator: Arc<Coordinator>,
}

#[async_trait]
impl Skill for DelegateSkill {
    fn name(&self) -> &str { "delegate" }

    fn description(&self) -> &str {
        "Delegate a task to another agent in the swarm and return its response."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "The ID of the target agent."
                },
                "message": {
                    "type": "string",
                    "description": "The message to send to the target agent."
                }
            },
            "required": ["agent_id", "message"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let agent_id = input.args.get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("missing 'agent_id'".to_string()))?;

        let content = input.args.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("missing 'message'".to_string()))?;

        let id: AgentId = agent_id.parse()
            .map_err(|_| SkillError::InvalidInput(format!("invalid agent_id UUID: {agent_id}")))?;
        let msg = Message::user(content);

        let reply = self.coordinator.route(&id, msg).await
            .map_err(|e| SkillError::Execution(e.to_string()))?;

        Ok(SkillOutput::ok(serde_json::json!({ "response": reply.content })))
    }
}
