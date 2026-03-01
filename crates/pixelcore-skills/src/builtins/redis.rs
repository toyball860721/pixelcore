//! Redis operations skills

use async_trait::async_trait;
use redis::{AsyncCommands, Client};
use serde_json::json;
use std::sync::Arc;

use crate::{Skill, SkillInput, SkillOutput, SkillError};

/// Redis get skill - retrieves a value by key
pub struct RedisGetSkill;

#[async_trait]
impl Skill for RedisGetSkill {
    fn name(&self) -> &str {
        "redis_get"
    }

    fn description(&self) -> &str {
        "Get a value from Redis by key"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Redis connection URL (e.g., redis://localhost:6379)"
                },
                "key": {
                    "type": "string",
                    "description": "Key to retrieve"
                }
            },
            "required": ["url", "key"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let url = input.args.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'url' parameter".to_string()))?;

        let key = input.args.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'key' parameter".to_string()))?;

        match redis_get(url, key).await {
            Ok(value) => Ok(SkillOutput {
                success: true,
                result: json!({"value": value}),
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(e),
            }),
        }
    }
}

/// Redis set skill - sets a key-value pair
pub struct RedisSetSkill;

#[async_trait]
impl Skill for RedisSetSkill {
    fn name(&self) -> &str {
        "redis_set"
    }

    fn description(&self) -> &str {
        "Set a key-value pair in Redis with optional TTL (time-to-live in seconds)"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Redis connection URL (e.g., redis://localhost:6379)"
                },
                "key": {
                    "type": "string",
                    "description": "Key to set"
                },
                "value": {
                    "type": "string",
                    "description": "Value to store"
                },
                "ttl": {
                    "type": "integer",
                    "description": "Time-to-live in seconds (optional)"
                }
            },
            "required": ["url", "key", "value"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let url = input.args.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'url' parameter".to_string()))?;

        let key = input.args.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'key' parameter".to_string()))?;

        let value = input.args.get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'value' parameter".to_string()))?;

        let ttl = input.args.get("ttl")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        match redis_set(url, key, value, ttl).await {
            Ok(_) => Ok(SkillOutput {
                success: true,
                result: json!({"status": "OK"}),
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(e),
            }),
        }
    }
}

/// Redis delete skill - deletes a key
pub struct RedisDeleteSkill;

#[async_trait]
impl Skill for RedisDeleteSkill {
    fn name(&self) -> &str {
        "redis_delete"
    }

    fn description(&self) -> &str {
        "Delete a key from Redis"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Redis connection URL (e.g., redis://localhost:6379)"
                },
                "key": {
                    "type": "string",
                    "description": "Key to delete"
                }
            },
            "required": ["url", "key"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let url = input.args.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'url' parameter".to_string()))?;

        let key = input.args.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'key' parameter".to_string()))?;

        match redis_delete(url, key).await {
            Ok(deleted) => Ok(SkillOutput {
                success: true,
                result: json!({"deleted": deleted}),
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(e),
            }),
        }
    }
}

/// Redis exists skill - checks if a key exists
pub struct RedisExistsSkill;

#[async_trait]
impl Skill for RedisExistsSkill {
    fn name(&self) -> &str {
        "redis_exists"
    }

    fn description(&self) -> &str {
        "Check if a key exists in Redis"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Redis connection URL (e.g., redis://localhost:6379)"
                },
                "key": {
                    "type": "string",
                    "description": "Key to check"
                }
            },
            "required": ["url", "key"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let url = input.args.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'url' parameter".to_string()))?;

        let key = input.args.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'key' parameter".to_string()))?;

        match redis_exists(url, key).await {
            Ok(exists) => Ok(SkillOutput {
                success: true,
                result: json!({"exists": exists}),
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(e),
            }),
        }
    }
}

/// Helper function to get a value from Redis
async fn redis_get(url: &str, key: &str) -> Result<Option<String>, String> {
    let client = Client::open(url)
        .map_err(|e| format!("Failed to create Redis client: {}", e))?;

    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

    let value: Option<String> = conn.get(key).await
        .map_err(|e| format!("Failed to get value: {}", e))?;

    Ok(value)
}

/// Helper function to set a value in Redis
async fn redis_set(url: &str, key: &str, value: &str, ttl: Option<usize>) -> Result<(), String> {
    let client = Client::open(url)
        .map_err(|e| format!("Failed to create Redis client: {}", e))?;

    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

    if let Some(ttl_seconds) = ttl {
        conn.set_ex::<_, _, ()>(key, value, ttl_seconds as u64).await
            .map_err(|e| format!("Failed to set value with TTL: {}", e))?;
    } else {
        conn.set::<_, _, ()>(key, value).await
            .map_err(|e| format!("Failed to set value: {}", e))?;
    }

    Ok(())
}

/// Helper function to delete a key from Redis
async fn redis_delete(url: &str, key: &str) -> Result<bool, String> {
    let client = Client::open(url)
        .map_err(|e| format!("Failed to create Redis client: {}", e))?;

    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

    let deleted: i32 = conn.del(key).await
        .map_err(|e| format!("Failed to delete key: {}", e))?;

    Ok(deleted > 0)
}

/// Helper function to check if a key exists in Redis
async fn redis_exists(url: &str, key: &str) -> Result<bool, String> {
    let client = Client::open(url)
        .map_err(|e| format!("Failed to create Redis client: {}", e))?;

    let mut conn = client.get_multiplexed_async_connection().await
        .map_err(|e| format!("Failed to connect to Redis: {}", e))?;

    let exists: bool = conn.exists(key).await
        .map_err(|e| format!("Failed to check existence: {}", e))?;

    Ok(exists)
}

/// Create Redis skills
pub fn create_redis_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(RedisGetSkill),
        Arc::new(RedisSetSkill),
        Arc::new(RedisDeleteSkill),
        Arc::new(RedisExistsSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIS_URL: &str = "redis://localhost:6379";

    async fn is_redis_available() -> bool {
        match Client::open(REDIS_URL) {
            Ok(client) => {
                client.get_multiplexed_async_connection().await.is_ok()
            }
            Err(_) => false,
        }
    }

    #[tokio::test]
    async fn test_redis_set_and_get() {
        if !is_redis_available().await {
            println!("Skipping test: Redis not available at {}", REDIS_URL);
            return;
        }

        let set_skill = RedisSetSkill;
        let get_skill = RedisGetSkill;

        // Set a value
        let set_input = SkillInput {
            name: "redis_set".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "test_key",
                "value": "test_value"
            }),
        };
        let set_output = set_skill.execute(set_input).await.unwrap();
        assert!(set_output.success);

        // Get the value
        let get_input = SkillInput {
            name: "redis_get".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "test_key"
            }),
        };
        let get_output = get_skill.execute(get_input).await.unwrap();
        assert!(get_output.success);
        assert_eq!(get_output.result["value"], "test_value");
    }

    #[tokio::test]
    async fn test_redis_set_with_ttl() {
        if !is_redis_available().await {
            println!("Skipping test: Redis not available at {}", REDIS_URL);
            return;
        }

        let set_skill = RedisSetSkill;
        let exists_skill = RedisExistsSkill;

        // Set a value with TTL
        let set_input = SkillInput {
            name: "redis_set".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "test_ttl_key",
                "value": "test_value",
                "ttl": 1
            }),
        };
        let set_output = set_skill.execute(set_input).await.unwrap();
        assert!(set_output.success);

        // Check it exists
        let exists_input = SkillInput {
            name: "redis_exists".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "test_ttl_key"
            }),
        };
        let exists_output = exists_skill.execute(exists_input.clone()).await.unwrap();
        assert!(exists_output.success);
        assert_eq!(exists_output.result["exists"], true);

        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Check it no longer exists
        let exists_output = exists_skill.execute(exists_input).await.unwrap();
        assert!(exists_output.success);
        assert_eq!(exists_output.result["exists"], false);
    }

    #[tokio::test]
    async fn test_redis_delete() {
        if !is_redis_available().await {
            println!("Skipping test: Redis not available at {}", REDIS_URL);
            return;
        }

        let set_skill = RedisSetSkill;
        let delete_skill = RedisDeleteSkill;
        let exists_skill = RedisExistsSkill;

        // Set a value
        let set_input = SkillInput {
            name: "redis_set".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "test_delete_key",
                "value": "test_value"
            }),
        };
        set_skill.execute(set_input).await.unwrap();

        // Delete the key
        let delete_input = SkillInput {
            name: "redis_delete".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "test_delete_key"
            }),
        };
        let delete_output = delete_skill.execute(delete_input).await.unwrap();
        assert!(delete_output.success);
        assert_eq!(delete_output.result["deleted"], true);

        // Check it no longer exists
        let exists_input = SkillInput {
            name: "redis_exists".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "test_delete_key"
            }),
        };
        let exists_output = exists_skill.execute(exists_input).await.unwrap();
        assert!(exists_output.success);
        assert_eq!(exists_output.result["exists"], false);
    }

    #[tokio::test]
    async fn test_redis_exists() {
        if !is_redis_available().await {
            println!("Skipping test: Redis not available at {}", REDIS_URL);
            return;
        }

        let exists_skill = RedisExistsSkill;

        // Check non-existent key
        let exists_input = SkillInput {
            name: "redis_exists".to_string(),
            args: json!({
                "url": REDIS_URL,
                "key": "non_existent_key"
            }),
        };
        let exists_output = exists_skill.execute(exists_input).await.unwrap();
        assert!(exists_output.success);
        assert_eq!(exists_output.result["exists"], false);
    }
}

