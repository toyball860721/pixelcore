use async_trait::async_trait;
use std::sync::Arc;
use pixelcore_claw::{LocalMcpClient, McpTool, ToolContent};
use crate::skill::{Skill, SkillInput, SkillOutput};
use crate::error::SkillError;

/// MCP 工具的 Skill 包装器
pub struct McpSkill {
    /// MCP 客户端（共享）
    client: Arc<LocalMcpClient>,
    /// 工具定义
    tool: McpTool,
}

impl McpSkill {
    /// 创建新的 MCP Skill
    pub fn new(client: Arc<LocalMcpClient>, tool: McpTool) -> Self {
        Self { client, tool }
    }

    /// 从工具内容提取文本
    fn extract_text(content: &[ToolContent]) -> String {
        content.iter()
            .filter_map(|c| match c {
                ToolContent::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl Skill for McpSkill {
    fn name(&self) -> &str {
        &self.tool.name
    }

    fn description(&self) -> &str {
        &self.tool.description
    }

    fn input_schema(&self) -> serde_json::Value {
        self.tool.input_schema.clone()
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        // 调用 MCP 工具
        let result = self.client
            .call_tool(&self.tool.name, Some(input.args))
            .await
            .map_err(|e| SkillError::Execution(e.to_string()))?;

        // 检查是否有错误
        if result.is_error.unwrap_or(false) {
            let error_msg = Self::extract_text(&result.content);
            return Ok(SkillOutput::err(error_msg));
        }

        // 提取结果文本
        let text = Self::extract_text(&result.content);

        // 尝试解析为 JSON，如果失败则返回纯文本
        let result_value = serde_json::from_str(&text)
            .unwrap_or_else(|_| serde_json::json!(text));

        Ok(SkillOutput::ok(result_value))
    }
}
