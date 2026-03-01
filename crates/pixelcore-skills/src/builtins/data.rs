//! Data processing skills for JSON, CSV, YAML, and XML manipulation

use async_trait::async_trait;
use quick_xml::events::{Event, BytesStart};
use quick_xml::{Reader, Writer};
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

/// XML parse skill - parses XML strings into JSON
pub struct XmlParseSkill;

#[async_trait]
impl Skill for XmlParseSkill {
    fn name(&self) -> &str {
        "xml_parse"
    }

    fn description(&self) -> &str {
        "Parse an XML string into a JSON object"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "xml_string": {
                    "type": "string",
                    "description": "XML string to parse"
                }
            },
            "required": ["xml_string"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let xml_string = input.args.get("xml_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::Execution("Missing 'xml_string' parameter".to_string()))?;

        match parse_xml_to_json(xml_string) {
            Ok(parsed) => Ok(SkillOutput {
                success: true,
                result: parsed,
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(format!("Failed to parse XML: {}", e)),
            }),
        }
    }
}

/// XML serialize skill - serializes JSON to XML string
pub struct XmlSerializeSkill;

#[async_trait]
impl Skill for XmlSerializeSkill {
    fn name(&self) -> &str {
        "xml_serialize"
    }

    fn description(&self) -> &str {
        "Serialize a JSON object into an XML string"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "object",
                    "description": "JSON data to serialize"
                },
                "root_name": {
                    "type": "string",
                    "description": "Root element name (default: 'root')",
                    "default": "root"
                }
            },
            "required": ["data"]
        })
    }

    async fn execute(&self, input: SkillInput) -> Result<SkillOutput, SkillError> {
        let data = input.args.get("data")
            .ok_or_else(|| SkillError::Execution("Missing 'data' parameter".to_string()))?;

        let root_name = input.args.get("root_name")
            .and_then(|v| v.as_str())
            .unwrap_or("root");

        match json_to_xml(data, root_name) {
            Ok(xml_string) => Ok(SkillOutput {
                success: true,
                result: json!({"xml": xml_string}),
                error: None,
            }),
            Err(e) => Ok(SkillOutput {
                success: false,
                result: json!(null),
                error: Some(format!("Failed to serialize to XML: {}", e)),
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

/// Parse XML string to JSON
fn parse_xml_to_json(xml_string: &str) -> Result<serde_json::Value, String> {
    let mut reader = Reader::from_str(xml_string);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<(String, serde_json::Map<String, serde_json::Value>)> = Vec::new();
    let mut current_text = String::new();
    let mut root: Option<serde_json::Value> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                stack.push((name, serde_json::Map::new()));
                current_text.clear();
            }
            Ok(Event::End(_)) => {
                if let Some((name, mut obj)) = stack.pop() {
                    // If we have text content, add it
                    if !current_text.trim().is_empty() {
                        obj.insert("_text".to_string(), json!(current_text.trim()));
                    }

                    let value = json!(obj);

                    if let Some((_, parent_obj)) = stack.last_mut() {
                        // Add to parent
                        if let Some(existing) = parent_obj.get_mut(&name) {
                            // Convert to array if not already
                            if let Some(arr) = existing.as_array_mut() {
                                arr.push(value);
                            } else {
                                let old_value = existing.clone();
                                *existing = json!([old_value, value]);
                            }
                        } else {
                            parent_obj.insert(name, value);
                        }
                    } else {
                        // This is the root element
                        root = Some(json!({ name: value }));
                    }
                    current_text.clear();
                }
            }
            Ok(Event::Text(e)) => {
                current_text.push_str(&e.unescape().map_err(|e| e.to_string())?);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    root.ok_or_else(|| "No root element found".to_string())
}

/// Convert JSON to XML string
fn json_to_xml(data: &serde_json::Value, root_name: &str) -> Result<String, String> {
    let mut writer = Writer::new(Vec::new());

    // Write XML declaration
    writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| format!("Failed to write XML declaration: {}", e))?;

    // Write root element
    write_json_as_xml(&mut writer, root_name, data)?;

    String::from_utf8(writer.into_inner())
        .map_err(|e| format!("Failed to convert XML to string: {}", e))
}

/// Helper function to recursively write JSON as XML
fn write_json_as_xml(writer: &mut Writer<Vec<u8>>, name: &str, value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::Object(map) => {
            let elem = BytesStart::new(name);
            writer.write_event(Event::Start(elem.borrow()))
                .map_err(|e| format!("Failed to write start tag: {}", e))?;

            for (key, val) in map {
                write_json_as_xml(writer, key, val)?;
            }

            writer.write_event(Event::End(BytesStart::new(name).to_end()))
                .map_err(|e| format!("Failed to write end tag: {}", e))?;
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                write_json_as_xml(writer, name, item)?;
            }
        }
        serde_json::Value::String(s) => {
            let elem = BytesStart::new(name);
            writer.write_event(Event::Start(elem.borrow()))
                .map_err(|e| format!("Failed to write start tag: {}", e))?;
            writer.write_event(Event::Text(quick_xml::events::BytesText::new(s)))
                .map_err(|e| format!("Failed to write text: {}", e))?;
            writer.write_event(Event::End(BytesStart::new(name).to_end()))
                .map_err(|e| format!("Failed to write end tag: {}", e))?;
        }
        serde_json::Value::Number(n) => {
            let elem = BytesStart::new(name);
            writer.write_event(Event::Start(elem.borrow()))
                .map_err(|e| format!("Failed to write start tag: {}", e))?;
            writer.write_event(Event::Text(quick_xml::events::BytesText::new(&n.to_string())))
                .map_err(|e| format!("Failed to write text: {}", e))?;
            writer.write_event(Event::End(BytesStart::new(name).to_end()))
                .map_err(|e| format!("Failed to write end tag: {}", e))?;
        }
        serde_json::Value::Bool(b) => {
            let elem = BytesStart::new(name);
            writer.write_event(Event::Start(elem.borrow()))
                .map_err(|e| format!("Failed to write start tag: {}", e))?;
            writer.write_event(Event::Text(quick_xml::events::BytesText::new(&b.to_string())))
                .map_err(|e| format!("Failed to write text: {}", e))?;
            writer.write_event(Event::End(BytesStart::new(name).to_end()))
                .map_err(|e| format!("Failed to write end tag: {}", e))?;
        }
        serde_json::Value::Null => {
            let elem = BytesStart::new(name);
            writer.write_event(Event::Empty(elem))
                .map_err(|e| format!("Failed to write empty tag: {}", e))?;
        }
    }
    Ok(())
}

/// Create data processing skills
pub fn create_data_skills() -> Vec<Arc<dyn Skill>> {
    vec![
        Arc::new(JsonParseSkill),
        Arc::new(JsonQuerySkill),
        Arc::new(CsvParseSkill),
        Arc::new(YamlParseSkill),
        Arc::new(YamlSerializeSkill),
        Arc::new(XmlParseSkill),
        Arc::new(XmlSerializeSkill),
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

    #[tokio::test]
    async fn test_xml_parse() {
        let skill = XmlParseSkill;
        let xml_str = r#"
<user>
    <name>Alice</name>
    <age>30</age>
    <email>alice@example.com</email>
</user>
"#;
        let input = SkillInput {
            name: "xml_parse".to_string(),
            args: json!({"xml_string": xml_str}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        assert_eq!(output.result["user"]["name"]["_text"], "Alice");
        assert_eq!(output.result["user"]["age"]["_text"], "30");
        assert_eq!(output.result["user"]["email"]["_text"], "alice@example.com");
    }

    #[tokio::test]
    async fn test_xml_serialize() {
        let skill = XmlSerializeSkill;
        let data = json!({
            "name": "Bob",
            "age": 25,
            "active": true
        });
        let input = SkillInput {
            name: "xml_serialize".to_string(),
            args: json!({"data": data, "root_name": "user"}),
        };
        let output = skill.execute(input).await.unwrap();
        assert!(output.success);
        let xml_string = output.result["xml"].as_str().unwrap();
        assert!(xml_string.contains("<user>"));
        assert!(xml_string.contains("<name>Bob</name>"));
        assert!(xml_string.contains("<age>25</age>"));
        assert!(xml_string.contains("<active>true</active>"));
        assert!(xml_string.contains("</user>"));
    }
}
