//! Excel file processing demo
//!
//! This example demonstrates how to use Excel skills to:
//! - Write JSON data to Excel files
//! - Read Excel files and convert to JSON
//! - Work with multiple sheets

use pixelcore_skills::{create_excel_skills, SkillInput};
use serde_json::json;
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Excel Processing Demo ===\n");

    // Get Excel skills
    let skills = create_excel_skills();
    let excel_read_skill = &skills[0];
    let excel_write_skill = &skills[1];

    // Create a temporary file for demo
    let temp_file = NamedTempFile::new()?;
    let file_path = temp_file.path().to_str().unwrap();

    // 1. Write data to Excel
    println!("1. Writing data to Excel file...");
    let data = vec![
        json!({"name": "Alice", "age": 30, "department": "Engineering", "salary": 75000}),
        json!({"name": "Bob", "age": 25, "department": "Marketing", "salary": 60000}),
        json!({"name": "Charlie", "age": 35, "department": "Sales", "salary": 80000}),
        json!({"name": "David", "age": 28, "department": "Engineering", "salary": 70000}),
        json!({"name": "Eve", "age": 32, "department": "HR", "salary": 65000}),
    ];

    let write_input = SkillInput {
        name: "excel_write".to_string(),
        args: json!({
            "file_path": file_path,
            "data": data,
            "sheet_name": "Employees"
        }),
    };

    let result = excel_write_skill.execute(write_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 2. Read data back from Excel
    println!("2. Reading data from Excel file...");
    let read_input = SkillInput {
        name: "excel_read".to_string(),
        args: json!({
            "file_path": file_path,
            "sheet_name": "Employees"
        }),
    };

    let result = excel_read_skill.execute(read_input).await?;
    println!("   Sheet: {}", result.result["sheet_name"]);
    println!("   Data: {}\n", serde_json::to_string_pretty(&result.result["rows"])?);

    // 3. Write another sheet with different data
    println!("3. Writing sales data to a new file...");
    let temp_file2 = NamedTempFile::new()?;
    let file_path2 = temp_file2.path().to_str().unwrap();

    let sales_data = vec![
        json!({"product": "Laptop", "quantity": 50, "price": 1200.00, "sold": true}),
        json!({"product": "Mouse", "quantity": 200, "price": 25.50, "sold": true}),
        json!({"product": "Keyboard", "quantity": 150, "price": 75.00, "sold": false}),
        json!({"product": "Monitor", "quantity": 80, "price": 350.00, "sold": true}),
    ];

    let write_input = SkillInput {
        name: "excel_write".to_string(),
        args: json!({
            "file_path": file_path2,
            "data": sales_data,
            "sheet_name": "Products"
        }),
    };

    let result = excel_write_skill.execute(write_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 4. Read the sales data
    println!("4. Reading sales data...");
    let read_input = SkillInput {
        name: "excel_read".to_string(),
        args: json!({
            "file_path": file_path2,
            "sheet_name": "Products"
        }),
    };

    let result = excel_read_skill.execute(read_input).await?;
    println!("   Sheet: {}", result.result["sheet_name"]);
    println!("   Data: {}\n", serde_json::to_string_pretty(&result.result["rows"])?);

    println!("=== Demo Complete ===");
    println!("\nNote: Temporary files will be automatically cleaned up.");

    Ok(())
}
