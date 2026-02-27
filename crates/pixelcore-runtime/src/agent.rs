use serde::{Deserialize, Serialize};
use uuid::Uuid;
use async_trait::async_trait;
use std::fmt;
use crate::error::RuntimeError;
use crate::message::Message;

pub type AgentId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Running,
    Paused,
    Stopped,
    Error(String),
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentState::Idle => write!(f, "idle"),
            AgentState::Running => write!(f, "running"),
            AgentState::Paused => write!(f, "paused"),
            AgentState::Stopped => write!(f, "stopped"),
            AgentState::Error(e) => write!(f, "error({e})"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: AgentId,
    pub name: String,
    pub system_prompt: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub metadata: serde_json::Value,
}

impl AgentConfig {
    pub fn new(name: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            system_prompt: system_prompt.into(),
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 8192,
            temperature: 0.7,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> AgentId;
    fn name(&self) -> &str;
    fn state(&self) -> &AgentState;
    fn config(&self) -> &AgentConfig;

    async fn start(&mut self) -> Result<(), RuntimeError>;
    async fn stop(&mut self) -> Result<(), RuntimeError>;
    async fn process(&mut self, message: Message) -> Result<Message, RuntimeError>;
}
