use async_trait::async_trait;
use crate::skill::{Skill, SkillInput, SkillOutput};
use crate::error::SkillError;

pub struct HttpFetchSkill;

#[async_trait]
impl Skill for HttpFetchSkill {
    fn name(&self) -> &str { "http_fetch" }
    fn description(&self) -> &str { "Fetch the body of a URL via HTTP GET." }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to fetch." }
            },
            "required": ["url"]
        })
    }
    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let url = input.args.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("missing 'url'".to_string()))?;

        let body = reqwest::get(url).await
            .map_err(|e| SkillError::Execution(e.to_string()))?
            .text().await
            .map_err(|e| SkillError::Execution(e.to_string()))?;

        Ok(SkillOutput::ok(serde_json::json!({ "body": body })))
    }
}
