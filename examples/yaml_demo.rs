//! YAML processing demo
//!
//! This example demonstrates how to use YAML skills to:
//! - Parse YAML strings into JSON
//! - Serialize JSON data to YAML strings
//! - Work with complex YAML structures

use pixelcore_skills::{create_data_skills, SkillInput};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== YAML Processing Demo ===\n");

    // Get data skills (includes YAML skills)
    let skills = create_data_skills();
    let yaml_parse_skill = &skills[3];  // YamlParseSkill
    let yaml_serialize_skill = &skills[4];  // YamlSerializeSkill

    // 1. Parse a simple YAML string
    println!("1. Parsing simple YAML...");
    let simple_yaml = r#"
name: Alice
age: 30
email: alice@example.com
"#;
    let parse_input = SkillInput {
        name: "yaml_parse".to_string(),
        args: json!({
            "yaml_string": simple_yaml
        }),
    };
    let result = yaml_parse_skill.execute(parse_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 2. Parse YAML with nested structures
    println!("2. Parsing nested YAML...");
    let nested_yaml = r#"
user:
  name: Bob
  age: 25
  address:
    street: 123 Main St
    city: San Francisco
    zip: 94102
  hobbies:
    - reading
    - coding
    - hiking
"#;
    let parse_input = SkillInput {
        name: "yaml_parse".to_string(),
        args: json!({
            "yaml_string": nested_yaml
        }),
    };
    let result = yaml_parse_skill.execute(parse_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 3. Parse YAML with arrays
    println!("3. Parsing YAML with arrays...");
    let array_yaml = r#"
users:
  - name: Alice
    age: 30
  - name: Bob
    age: 25
  - name: Charlie
    age: 35
"#;
    let parse_input = SkillInput {
        name: "yaml_parse".to_string(),
        args: json!({
            "yaml_string": array_yaml
        }),
    };
    let result = yaml_parse_skill.execute(parse_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 4. Serialize JSON to YAML
    println!("4. Serializing JSON to YAML...");
    let json_data = json!({
        "server": {
            "host": "localhost",
            "port": 8080,
            "ssl": true
        },
        "database": {
            "host": "db.example.com",
            "port": 5432,
            "name": "myapp"
        },
        "features": ["auth", "api", "websockets"]
    });
    let serialize_input = SkillInput {
        name: "yaml_serialize".to_string(),
        args: json!({
            "data": json_data
        }),
    };
    let result = yaml_serialize_skill.execute(serialize_input).await?;
    println!("   YAML output:\n{}\n", result.result["yaml"].as_str().unwrap());

    // 5. Round-trip: YAML -> JSON -> YAML
    println!("5. Round-trip conversion (YAML -> JSON -> YAML)...");
    let original_yaml = r#"
config:
  version: "1.0"
  debug: true
  timeout: 30
"#;

    // Parse YAML to JSON
    let parse_input = SkillInput {
        name: "yaml_parse".to_string(),
        args: json!({
            "yaml_string": original_yaml
        }),
    };
    let parsed = yaml_parse_skill.execute(parse_input).await?;
    println!("   Parsed JSON: {}", serde_json::to_string_pretty(&parsed.result)?);

    // Serialize back to YAML
    let serialize_input = SkillInput {
        name: "yaml_serialize".to_string(),
        args: json!({
            "data": parsed.result
        }),
    };
    let serialized = yaml_serialize_skill.execute(serialize_input).await?;
    println!("   Serialized YAML:\n{}", serialized.result["yaml"].as_str().unwrap());

    println!("=== Demo Complete ===");

    Ok(())
}

