use async_trait::async_trait;
use std::sync::Arc;
use crate::skill::{Skill, SkillInput, SkillOutput};
use crate::error::SkillError;
use pixelcore_storage::Storage;

pub struct StorageGetSkill {
    pub store: Arc<Storage>,
}

pub struct StorageSetSkill {
    pub store: Arc<Storage>,
}

#[async_trait]
impl Skill for StorageGetSkill {
    fn name(&self) -> &str { "storage_get" }
    fn description(&self) -> &str { "Get a value from the key-value store." }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "key": { "type": "string", "description": "The key to retrieve." }
            },
            "required": ["key"]
        })
    }
    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let key = input.args.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("missing 'key'".to_string()))?;
        match self.store.get(key) {
            Ok(value) => Ok(SkillOutput::ok(serde_json::json!({ "value": value }))),
            Err(_) => Ok(SkillOutput::ok(serde_json::json!({ "value": null }))),
        }
    }
}

#[async_trait]
impl Skill for StorageSetSkill {
    fn name(&self) -> &str { "storage_set" }
    fn description(&self) -> &str { "Set a value in the key-value store." }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "key": { "type": "string", "description": "The key to set." },
                "value": { "description": "The value to store." }
            },
            "required": ["key", "value"]
        })
    }
    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let key = input.args.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::InvalidInput("missing 'key'".to_string()))?
            .to_string();
        let value = input.args.get("value")
            .cloned()
            .ok_or_else(|| SkillError::InvalidInput("missing 'value'".to_string()))?;
        self.store.set(key, value);
        Ok(SkillOutput::ok(serde_json::json!({ "ok": true })))
    }
}
