//! XML processing demo
//!
//! This example demonstrates how to use XML skills to:
//! - Parse XML strings into JSON
//! - Serialize JSON data to XML strings
//! - Work with complex XML structures

use pixelcore_skills::{create_data_skills, SkillInput};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== XML Processing Demo ===\n");

    // Get data skills (includes XML skills)
    let skills = create_data_skills();
    let xml_parse_skill = &skills[5];  // XmlParseSkill
    let xml_serialize_skill = &skills[6];  // XmlSerializeSkill

    // 1. Parse a simple XML string
    println!("1. Parsing simple XML...");
    let simple_xml = r#"
<user>
    <name>Alice</name>
    <age>30</age>
    <email>alice@example.com</email>
</user>
"#;
    let parse_input = SkillInput {
        name: "xml_parse".to_string(),
        args: json!({
            "xml_string": simple_xml
        }),
    };
    let result = xml_parse_skill.execute(parse_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 2. Parse XML with nested structures
    println!("2. Parsing nested XML...");
    let nested_xml = r#"
<company>
    <name>TechCorp</name>
    <employees>
        <employee>
            <name>Bob</name>
            <role>Developer</role>
        </employee>
        <employee>
            <name>Charlie</name>
            <role>Designer</role>
        </employee>
    </employees>
</company>
"#;
    let parse_input = SkillInput {
        name: "xml_parse".to_string(),
        args: json!({
            "xml_string": nested_xml
        }),
    };
    let result = xml_parse_skill.execute(parse_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 3. Serialize JSON to XML
    println!("3. Serializing JSON to XML...");
    let json_data = json!({
        "name": "David",
        "age": 28,
        "skills": {
            "programming": "Rust",
            "database": "PostgreSQL"
        }
    });
    let serialize_input = SkillInput {
        name: "xml_serialize".to_string(),
        args: json!({
            "data": json_data,
            "root_name": "developer"
        }),
    };
    let result = xml_serialize_skill.execute(serialize_input).await?;
    println!("   XML output:\n{}\n", result.result["xml"].as_str().unwrap());

    // 4. Serialize complex JSON structure
    println!("4. Serializing complex JSON to XML...");
    let complex_data = json!({
        "title": "Project Plan",
        "version": "1.0",
        "tasks": {
            "task": "Implement feature",
            "priority": "high",
            "status": "in-progress"
        }
    });
    let serialize_input = SkillInput {
        name: "xml_serialize".to_string(),
        args: json!({
            "data": complex_data,
            "root_name": "project"
        }),
    };
    let result = xml_serialize_skill.execute(serialize_input).await?;
    println!("   XML output:\n{}\n", result.result["xml"].as_str().unwrap());

    // 5. Round-trip: XML -> JSON -> XML
    println!("5. Round-trip conversion (XML -> JSON -> XML)...");
    let original_xml = r#"
<config>
    <database>
        <host>localhost</host>
        <port>5432</port>
    </database>
</config>
"#;

    // Parse XML to JSON
    let parse_input = SkillInput {
        name: "xml_parse".to_string(),
        args: json!({
            "xml_string": original_xml
        }),
    };
    let parsed = xml_parse_skill.execute(parse_input).await?;
    println!("   Parsed JSON: {}", serde_json::to_string_pretty(&parsed.result)?);

    // Serialize back to XML
    let serialize_input = SkillInput {
        name: "xml_serialize".to_string(),
        args: json!({
            "data": parsed.result,
            "root_name": "root"
        }),
    };
    let serialized = xml_serialize_skill.execute(serialize_input).await?;
    println!("   Serialized XML:\n{}", serialized.result["xml"].as_str().unwrap());

    println!("=== Demo Complete ===");

    Ok(())
}
