//! Data processing skills for JSON, CSV, and YAML manipulation

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use crate::{Skill, SkillInput, SkillOutput, SkillError};

/// JSON parse skill - parses JSON strings
pub struct JsonParseSkill;

#[async_trait]
impl Skill for JsonParseSkill {
    fn name(&self) -> &str {
        "json_parse"
    }

    fn description(&self) -> &str {
        "Parse a JSON string into a JSON object"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "json_string": {
                    "type": "string",
                    "description": "JSON string to parse"
                }
            },
            "required": ["json_string"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let json_string = input.args.get("json_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'json_string' parameter".to_string()))?;

        match serde_json::from_str::<serde_json::Value>(json_string) {
            Ok(parsed) => Ok(SkillOutput {
                success: true,
                result: parsed,
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(format!("Failed to parse JSON: {}", e)),
            }),
        }
    }
}

/// JSON query skill - queries JSON data using JSONPath-like syntax
pub struct JsonQuerySkill;

#[async_trait]
impl Skill for JsonQuerySkill {
    fn name(&self) -> &str {
        "json_query"
    }

    fn description(&self) -> &str {
        "Query JSON data using dot notation (e.g., 'users.0.name', 'data.items')"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "object",
                    "description": "JSON data to query"
                },
                "path": {
                    "type": "string",
                    "description": "Dot-notation path (e.g., 'users.0.name')"
                }
            },
            "required": ["data", "path"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let data = input.args.get("data")
            .ok_or_else(|| SkillError::Execution("Missing 'data' parameter".to_string()))?;

        let path = input.args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'path' parameter".to_string()))?;

        match query_json(data, path) {
            Some(result) => Ok(SkillOutput {
                success: true,
                result: result.clone(),
                error: None,
            }),
            None => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(format!("Path '{}' not found in data", path)),
            }),
        }
    }
}

/// CSV parse skill - parses CSV strings into JSON
pub struct CsvParseSkill;

#[async_trait]
impl Skill for CsvParseSkill {
    fn name(&self) -> &str {
        "csv_parse"
    }

    fn description(&self) -> &str {
        "Parse CSV string into JSON array of objects"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "csv_string": {
                    "type": "string",
                    "description": "CSV string to parse"
                },
                "delimiter": {
                    "type": "string",
                    "description": "Field delimiter (default: ',')",
                    "default": ","
                }
            },
            "required": ["csv_string"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let csv_string = input.args.get("csv_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'csv_string' parameter".to_string()))?;

        let delimiter = input.args.get("delimiter")
            .and_then(|v| v.as_str())
            .unwrap_or(",");

        match parse_csv(csv_string, delimiter) {
            Ok(result) => Ok(SkillOutput {
                success: true,
                result: json!(result),
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

/// YAML parse skill - parses YAML strings into JSON
pub struct YamlParseSkill;

#[async_trait]
impl Skill for YamlParseSkill {
    fn name(&self) -> &str {
        "yaml_parse"
    }

    fn description(&self) -> &str {
        "Parse a YAML string into a JSON object"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "yaml_string": {
                    "type": "string",
                    "description": "YAML string to parse"
                }
            },
            "required": ["yaml_string"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let yaml_string = input.args.get("yaml_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'yaml_string' parameter".to_string()))?;

        match serde_yaml::from_str::<serde_json::Value>(yaml_string) {
            Ok(parsed) => Ok(SkillOutput {
                success: true,
                result: parsed,
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(format!("Failed to parse YAML: {}", e)),
            }),
        }
    }
}

/// YAML serialize skill - serializes JSON to YAML string
pub struct YamlSerializeSkill;

#[async_trait]
impl Skill for YamlSerializeSkill {
    fn name(&self) -> &str {
        "yaml_serialize"
    }

    fn description(&self) -> &str {
        "Serialize a JSON object into a YAML string"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "object",
                    "description": "JSON data to serialize"
                }
            },
            "required": ["data"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let data = input.args.get("data")
            .ok_or_else(|| SkillError::Execution("Missing 'data' parameter".to_string()))?;

        match serde_yaml::to_string(data) {
            Ok(yaml_string) => Ok(SkillOutput {
                success: true,
                result: json!({"yaml": yaml_string}),
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(format!("Failed to serialize to YAML: {}", e)),
            }),
        }
    }
}

/// Query JSON data using dot notation
fn query_json<'a>(data: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;

    for part in parts {
        if part.is_empty() {
            continue;
        }

        // Try to parse as array index
        if let Ok(index) = part.parse::<usize>() {
            current = current.get(index)?;
        } else {
            // Try as object key
            current = current.get(part)?;
        }
    }

    Some(current)
}

/// Parse CSV string into JSON
fn parse_csv(csv_string: &str, delimiter: &str) -> Result<Vec<serde_json::Value>, String> {
    let lines: Vec<&str> = csv_string.trim().lines().collect();
    if lines.is_empty() {
        return Ok(vec![]);
    }

    // First line is header
    let headers: Vec<&str> = lines[0].split(delimiter).map(|s| s.trim()).collect();

    // Parse remaining lines
    let mut result = Vec::new();
    for line in &lines[1..] {
        let values: Vec<&str> = line.split(delimiter).map(|s| s.trim()).collect();

        if values.len() != headers.len() {
            return Err(format!("Row has {} columns, expected {}", values.len(), headers.len()));
        }

        let mut obj = serde_json::Map::new();
        for (header, value) in headers.iter().zip(values.iter()) {
            obj.insert(header.to_string(), json!(value));
        }
        result.push(json!(obj));
    }

    Ok(result)
}

/// Create data processing skills
pub fn create_data_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(JsonParseSkill),
        Arc::new(JsonQuerySkill),
        Arc::new(CsvParseSkill),
        Arc::new(YamlParseSkill),
        Arc::new(YamlSerializeSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_parse() {
        let skill = JsonParseSkill;
        let input = SkillInput {
            name: "json_parse".to_string(),
            args: json!({"json_string": r#"{"name": "Alice", "age": 30}"#}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result["name"], "Alice");
        assert_eq!(output.result["age"], 30);
    }

    #[tokio::test]
    async fn test_json_query() {
        let skill = JsonQuerySkill;
        let data = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });
        let input = SkillInput {
            name: "json_query".to_string(),
            args: json!({"data": data, "path": "users.0.name"}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result, "Alice");
    }

    #[tokio::test]
    async fn test_csv_parse() {
        let skill = CsvParseSkill;
        let input = SkillInput {
            name: "csv_parse".to_string(),
            args: json!({"csv_string": "name,age\nAlice,30\nBob,25"}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        let result = output.result.as_array().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["name"], "Alice");
        assert_eq!(result[0]["age"], "30");
    }

    #[tokio::test]
    async fn test_yaml_parse() {
        let skill = YamlParseSkill;
        let yaml_str = r#"
name: Alice
age: 30
hobbies:
  - reading
  - coding
"#;
        let input = SkillInput {
            name: "yaml_parse".to_string(),
            args: json!({"yaml_string": yaml_str}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result["name"], "Alice");
        assert_eq!(output.result["age"], 30);
        assert_eq!(output.result["hobbies"][0], "reading");
        assert_eq!(output.result["hobbies"][1], "coding");
    }

    #[tokio::test]
    async fn test_yaml_serialize() {
        let skill = YamlSerializeSkill;
        let data = json!({
            "name": "Bob",
            "age": 25,
            "active": true
        });
        let input = SkillInput {
            name: "yaml_serialize".to_string(),
            args: json!({"data": data}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        let yaml_string = output.result["yaml"].as_str().unwrap();
        assert!(yaml_string.contains("name: Bob"));
        assert!(yaml_string.contains("age: 25"));
        assert!(yaml_string.contains("active: true"));
    }
}
