use serde::{Deserialize, Serialize};
use pixelcore_runtime::message::{Message, MessageRole};

// ── Unified request/response types ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub id: String,
    pub model: String,
    pub stop_reason: Option<String>,
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: ApiContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
}

// ── Conversions ──────────────────────────────────────────────────────────────

impl From<&Message> for ApiMessage {
    fn from(msg: &Message) -> Self {
        let role = match msg.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System | MessageRole::Tool => "user",
        };
        Self {
            role: role.to_string(),
            content: ApiContent::Text(msg.content.clone()),
        }
    }
}

// ── OpenAI-compatible types (used by SiliconFlow etc.) ───────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiTool {
    #[serde(rename = "type")]
    pub kind: String,
    pub function: OpenAiFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl OpenAiMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: Some(content.into()), tool_calls: None, tool_call_id: None, name: None }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: Some(content.into()), tool_calls: None, tool_call_id: None, name: None }
    }
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: "tool".to_string(), content: Some(content.into()), tool_calls: None, tool_call_id: Some(tool_call_id.into()), name: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: OpenAiToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiChoice {
    pub message: OpenAiMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

impl LlmRequest {
    pub fn to_openai(&self) -> OpenAiRequest {
        let mut messages: Vec<OpenAiMessage> = Vec::new();
        if let Some(sys) = &self.system {
            messages.push(OpenAiMessage::system(sys));
        }
        for m in &self.messages {
            match &m.content {
                ApiContent::Text(s) => {
                    messages.push(OpenAiMessage { role: m.role.clone(), content: Some(s.clone()), tool_calls: None, tool_call_id: None, name: None });
                }
                ApiContent::Blocks(blocks) => {
                    // assistant message with tool_use blocks
                    let tool_calls: Vec<OpenAiToolCall> = blocks.iter().filter_map(|b| match b {
                        ContentBlock::ToolUse { id, name, input } => Some(OpenAiToolCall {
                            id: id.clone(),
                            kind: "function".to_string(),
                            function: OpenAiToolCallFunction {
                                name: name.clone(),
                                arguments: input.to_string(),
                            },
                        }),
                        _ => None,
                    }).collect();

                    let text: String = blocks.iter().filter_map(|b| match b {
                        ContentBlock::Text { text } => Some(text.as_str()),
                        _ => None,
                    }).collect::<Vec<_>>().join("");

                    if !tool_calls.is_empty() {
                        messages.push(OpenAiMessage {
                            role: m.role.clone(),
                            content: if text.is_empty() { None } else { Some(text) },
                            tool_calls: Some(tool_calls),
                            tool_call_id: None,
                            name: None,
                        });
                    } else {
                        // tool result blocks
                        for b in blocks {
                            if let ContentBlock::ToolResult { tool_use_id, content } = b {
                                messages.push(OpenAiMessage::tool_result(tool_use_id, content));
                            }
                        }
                        if !text.is_empty() {
                            messages.push(OpenAiMessage { role: m.role.clone(), content: Some(text), tool_calls: None, tool_call_id: None, name: None });
                        }
                    }
                }
            }
        }

        let tools = self.tools.as_ref().map(|ts| ts.iter().map(|t| OpenAiTool {
            kind: "function".to_string(),
            function: OpenAiFunction {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.input_schema.clone(),
            },
        }).collect());

        OpenAiRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            tools,
            tool_choice: None,
        }
    }
}

impl OpenAiResponse {
    pub fn into_llm_response(self) -> LlmResponse {
        let choice = self.choices.into_iter().next().unwrap_or_else(|| OpenAiChoice {
            message: OpenAiMessage::user(""),
            finish_reason: None,
        });
        let stop_reason = choice.finish_reason.clone();
        let msg = choice.message;

        let mut blocks: Vec<ContentBlock> = Vec::new();
        if let Some(text) = &msg.content {
            if !text.is_empty() {
                blocks.push(ContentBlock::Text { text: text.clone() });
            }
        }
        if let Some(tool_calls) = msg.tool_calls {
            for tc in tool_calls {
                let input: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(serde_json::Value::Null);
                blocks.push(ContentBlock::ToolUse { id: tc.id, name: tc.function.name, input });
            }
        }

        LlmResponse {
            id: self.id,
            model: self.model,
            stop_reason,
            content: blocks,
            usage: Usage {
                input_tokens: self.usage.prompt_tokens,
                output_tokens: self.usage.completion_tokens,
            },
        }
    }
}

// ── Back-compat aliases (drop after full migration) ──────────────────────────
pub type McpRequest = LlmRequest;
pub type McpResponse = LlmResponse;
