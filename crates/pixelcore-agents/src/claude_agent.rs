use anyhow::anyhow;
use async_trait::async_trait;
use tracing::{info, warn};

use pixelcore_claw::{
    ClawClient,
    types::{LlmRequest, ApiMessage, ContentBlock},
};

use pixelcore_runtime::{Agent, AgentId, AgentConfig, AgentState, RuntimeError, Message};

/// A concrete Agent that calls the Claude API via ClawClient.
pub struct ClaudeAgent {
    config: AgentConfig,
    state: AgentState,
    client: ClawClient,
    /// Conversation history (excludes system prompt).
    history: Vec<Message>,
}

impl ClaudeAgent {
    /// Create from an existing config. Reads ANTHROPIC_API_KEY from env.
    pub fn new(config: AgentConfig) -> Result<Self, RuntimeError> {
        let client = ClawClient::from_env()
            .map_err(|e| RuntimeError::Other(anyhow!(e.to_string())))?;
        Ok(Self {
            config,
            state: AgentState::Idle,
            client,
            history: Vec::new(),
        })
    }

    /// Create with an explicit API key.
    pub fn with_api_key(config: AgentConfig, api_key: impl Into<String>) -> Self {
        Self {
            config,
            state: AgentState::Idle,
            client: ClawClient::new(api_key),
            history: Vec::new(),
        }
    }

    /// Clear conversation history.
    pub fn reset(&mut self) {
        self.history.clear();
    }

    /// Read-only view of the conversation history.
    pub fn history(&self) -> &[Message] {
        &self.history
    }

    fn build_request(&self) -> LlmRequest {
        let messages: Vec<ApiMessage> = self.history.iter().map(ApiMessage::from).collect();
        LlmRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            messages,
            system: Some(self.config.system_prompt.clone()),
            tools: None,
            temperature: Some(self.config.temperature),
        }
    }

    fn extract_text(blocks: &[ContentBlock]) -> String {
        blocks
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

#[async_trait]
impl Agent for ClaudeAgent {
    fn id(&self) -> AgentId {
        self.config.id
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn state(&self) -> &AgentState {
        &self.state
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }

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

        // Append user message to history.
        self.history.push(message);

        let request = self.build_request();

        let response = self.client.complete(request).await.map_err(|e| {
            warn!(agent = %self.config.name, error = %e, "API call failed");
            self.state = AgentState::Error(e.to_string());
            RuntimeError::Other(anyhow!(e.to_string()))
        })?;

        let text = Self::extract_text(&response.content);
        let reply = Message::assistant(text);

        // Append assistant reply to history.
        self.history.push(reply.clone());

        info!(
            agent = %self.config.name,
            input_tokens = response.usage.input_tokens,
            output_tokens = response.usage.output_tokens,
            "turn complete"
        );

        Ok(reply)
    }
}
