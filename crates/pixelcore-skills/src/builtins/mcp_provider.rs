use std::sync::Arc;
use anyhow::{Context, Result};
use pixelcore_claw::LocalMcpClient;
use crate::skill::Skill;
use super::mcp_skill::McpSkill;

/// MCP Skill 提供者
///
/// 自动从 MCP 服务器发现工具并创建对应的 Skills
pub struct McpSkillProvider {
    client: Arc<LocalMcpClient>,
    skills: Vec<Arc<dyn Skill>>,
}

impl McpSkillProvider {
    /// 创建新的 MCP Skill 提供者
    ///
    /// # Arguments
    /// * `command` - MCP 服务器命令
    /// * `args` - 命令参数
    ///
    /// # Example
    /// ```no_run
    /// use pixelcore_skills::McpSkillProvider;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let provider = McpSkillProvider::new(
    ///         "python3",
    ///         &["examples/mcp_server/server.py"]
    ///     ).await?;
    ///
    ///     let skills = provider.skills();
    ///     println!("Loaded {} skills", skills.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(command: &str, args: &[&str]) -> Result<Self> {
        // 启动 MCP 客户端
        let client = LocalMcpClient::new(command, args)
            .await
            .context("Failed to start MCP client")?;

        let client = Arc::new(client);

        // 获取所有工具
        let tools = client.list_tools()
            .await
            .context("Failed to list MCP tools")?;

        tracing::info!("Discovered {} MCP tools", tools.len());

        // 为每个工具创建 Skill
        let skills: Vec<Arc<dyn Skill>> = tools
            .into_iter()
            .map(|tool| {
                let skill = McpSkill::new(Arc::clone(&client), tool);
                Arc::new(skill) as Arc<dyn Skill>
            })
            .collect();

        Ok(Self { client, skills })
    }

    /// 获取所有 Skills
    pub fn skills(&self) -> &[Arc<dyn Skill>] {
        &self.skills
    }

    /// 获取 MCP 客户端
    pub fn client(&self) -> &Arc<LocalMcpClient> {
        &self.client
    }

    /// 检查 MCP 服务器是否还在运行
    pub async fn is_alive(&self) -> bool {
        self.client.is_alive().await
    }

    /// 关闭 MCP 服务器
    pub async fn shutdown(&self) -> Result<()> {
        self.client.shutdown().await
    }
}

impl Drop for McpSkillProvider {
    fn drop(&mut self) {
        tracing::debug!("McpSkillProvider dropped");
    }
}
