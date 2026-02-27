use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn};

use pixelcore_claw::{
    ClawClient,
    types::{LlmRequest, ApiMessage, ApiContent, ContentBlock},
};
use pixelcore_runtime::{Agent, AgentId, AgentConfig, AgentState, RuntimeError, Message};
use pixelcore_skills::{Skill, SkillInput, SkillRegistry};

const MAX_TOOL_ROUNDS: usize = 10;

pub struct ClaudeAgent {
    config: AgentConfig,
    state: AgentState,
    client: ClawClient,
    /// Full conversation history with blocks preserved for tool-use round-trips.
    history: Vec<ApiMessage>,
    skills: SkillRegistry,
}

impl ClaudeAgent {
    pub fn new(config: AgentConfig) -> Result<Self, RuntimeError> {
        let client = ClawClient::from_env()
            .map_err(|e| RuntimeError::Other(anyhow!(e.to_string())))?;
        Ok(Self::with_client(config, client))
    }

    pub fn with_api_key(config: AgentConfig, api_key: impl Into<String>) -> Self {
        Self::with_client(config, ClawClient::new(api_key))
    }

    pub fn with_client(config: AgentConfig, client: ClawClient) -> Self {
        Self {
            config,
            state: AgentState::Idle,
            client,
            history: Vec::new(),
            skills: SkillRegistry::new(),
        }
    }

    /// Register a skill so the agent can call it as a tool.
    pub fn register_skill(&mut self, skill: Arc<dyn Skill>) {
        self.skills.register(skill);
    }

    pub fn reset(&mut self) {
        self.history.clear();
    }

    fn build_request(&self) -> LlmRequest {
        let tools = self.skills.as_tools();
        LlmRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            messages: self.history.clone(),
            system: Some(self.config.system_prompt.clone()),
            tools: if tools.is_empty() { None } else { Some(tools) },
            temperature: Some(self.config.temperature),
        }
    }

    fn extract_text(blocks: &[ContentBlock]) -> String {
        blocks.iter().filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        }).collect::<Vec<_>>().join("")
    }

    fn extract_tool_uses(blocks: &[ContentBlock]) -> Vec<(String, String, serde_json::Value)> {
        blocks.iter().filter_map(|b| match b {
            ContentBlock::ToolUse { id, name, input } => Some((id.clone(), name.clone(), input.clone())),
            _ => None,
        }).collect()
    }
}

#[async_trait]
impl Agent for ClaudeAgent {
    fn id(&self) -> AgentId { self.config.id }
    fn name(&self) -> &str { &self.config.name }
    fn state(&self) -> &AgentState { &self.state }
    fn config(&self) -> &AgentConfig { &self.config }

    async fn start(&mut self) -> Result<(), RuntimeError> {
        info!(agent = %self.config.name, "starting");
        self.state = AgentState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), RuntimeError> {
        info!(agent = %self.config.name, "stopping");
        self.state = AgentState::Stopped;
        Ok(())
    }

    async fn process(&mut self, message: Message) -> Result<Message, RuntimeError> {
        if self.state != AgentState::Running {
            return Err(RuntimeError::Other(anyhow!(
                "agent '{}' is not running (state: {})",
                self.config.name, self.state
            )));
        }

        // Append user message.
        self.history.push(ApiMessage {
            role: "user".to_string(),
            content: ApiContent::Text(message.content),
        });

        for _ in 0..MAX_TOOL_ROUNDS {
            let request = self.build_request();
            let response = self.client.complete(request).await.map_err(|e| {
                warn!(agent = %self.config.name, error = %e, "API call failed");
                self.state = AgentState::Error(e.to_string());
                RuntimeError::Other(anyhow!(e.to_string()))
            })?;

            info!(
                agent = %self.config.name,
                input_tokens = response.usage.input_tokens,
                output_tokens = response.usage.output_tokens,
                stop_reason = ?response.stop_reason,
                "turn complete"
            );

            let tool_uses = Self::extract_tool_uses(&response.content);
            let stop_reason = response.stop_reason.as_deref().unwrap_or("");

            // Append assistant turn with full blocks.
            self.history.push(ApiMessage {
                role: "assistant".to_string(),
                content: ApiContent::Blocks(response.content.clone()),
            });

            if tool_uses.is_empty() || stop_reason == "end_turn" || stop_reason == "stop" {
                let text = Self::extract_text(&response.content);
                return Ok(Message::assistant(text));
            }

            // Execute each tool call and collect results.
            let mut result_blocks: Vec<ContentBlock> = Vec::new();
            for (tool_use_id, skill_name, input) in &tool_uses {
                let result = match self.skills.get(skill_name) {
                    Ok(skill) => {
                        let skill_input = SkillInput { name: skill_name.clone(), args: input.clone() };
                        match skill.execute(skill_input).await {
                            Ok(out) => out.result.to_string(),
                            Err(e) => format!("error: {e}"),
                        }
                    }
                    Err(_) => format!("unknown skill: {skill_name}"),
                };
                info!(agent = %self.config.name, skill = %skill_name, "skill executed");
                result_blocks.push(ContentBlock::ToolResult {
                    tool_use_id: tool_use_id.clone(),
                    content: result,
                });
            }

            // Append tool results as a user turn.
            self.history.push(ApiMessage {
                role: "user".to_string(),
                content: ApiContent::Blocks(result_blocks),
            });
        }

        Err(RuntimeError::Other(anyhow!("exceeded max tool rounds ({})", MAX_TOOL_ROUNDS)))
    }
}
