//! SQLite operations demo
//!
//! This example demonstrates how to use SQLite skills to:
//! - Create a database and table
//! - Insert data
//! - Query data
//! - Update and delete data

use pixelcore_skills::{create_sqlite_skills, Skill, SkillInput};
use serde_json::json;
use std::fs;
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SQLite Operations Demo ===\n");

    // Create a temporary database file
    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file.path().to_str().unwrap();
    println!("Created temporary database at: {}\n", db_path);

    // Get SQLite skills
    let skills = create_sqlite_skills();
    let query_skill = &skills[0];
    let execute_skill = &skills[1];

    // 1. Create table
    println!("1. Creating users table...");
    let create_table_input = SkillInput {
        name: "sqlite_execute".to_string(),
        args: json!({
            "db_path": db_path,
            "statement": "CREATE TABLE users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, age INTEGER, email TEXT)"
        }),
    };
    let result = execute_skill.execute(create_table_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result)?);

    // 2. Insert data
    println!("2. Inserting users...");
    let insert_input = SkillInput {
        name: "sqlite_execute".to_string(),
        args: json!({
            "db_path": db_path,
            "statement": "INSERT INTO users (name, age, email) VALUES ('Alice', 30, 'alice@example.com')"
        }),
    };
    let result = execute_skill.execute(insert_input).await?;
    println!("   Inserted Alice: {}", result.success);

    let insert_input = SkillInput {
        name: "sqlite_execute".to_string(),
        args: json!({
            "db_path": db_path,
            "statement": "INSERT INTO users (name, age, email) VALUES ('Bob', 25, 'bob@example.com')"
        }),
    };
    let result = execute_skill.execute(insert_input).await?;
    println!("   Inserted Bob: {}", result.success);

    let insert_input = SkillInput {
        name: "sqlite_execute".to_string(),
        args: json!({
            "db_path": db_path,
            "statement": "INSERT INTO users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')"
        }),
    };
    let result = execute_skill.execute(insert_input).await?;
    println!("   Inserted Charlie: {}\n", result.success);

    // 3. Query all users
    println!("3. Querying all users...");
    let query_input = SkillInput {
        name: "sqlite_query".to_string(),
        args: json!({
            "db_path": db_path,
            "query": "SELECT * FROM users ORDER BY age"
        }),
    };
    let result = query_skill.execute(query_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 4. Query with condition
    println!("4. Querying users older than 28...");
    let query_input = SkillInput {
        name: "sqlite_query".to_string(),
        args: json!({
            "db_path": db_path,
            "query": "SELECT name, age FROM users WHERE age > 28"
        }),
    };
    let result = query_skill.execute(query_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 5. Update data
    println!("5. Updating Bob's age...");
    let update_input = SkillInput {
        name: "sqlite_execute".to_string(),
        args: json!({
            "db_path": db_path,
            "statement": "UPDATE users SET age = 26 WHERE name = 'Bob'"
        }),
    };
    let result = execute_skill.execute(update_input).await?;
    println!("   Rows affected: {}\n", result.result["rows_affected"]);

    // 6. Delete data
    println!("6. Deleting Charlie...");
    let delete_input = SkillInput {
        name: "sqlite_execute".to_string(),
        args: json!({
            "db_path": db_path,
            "statement": "DELETE FROM users WHERE name = 'Charlie'"
        }),
    };
    let result = execute_skill.execute(delete_input).await?;
    println!("   Rows affected: {}\n", result.result["rows_affected"]);

    // 7. Final query
    println!("7. Final state of users table...");
    let query_input = SkillInput {
        name: "sqlite_query".to_string(),
        args: json!({
            "db_path": db_path,
            "query": "SELECT * FROM users"
        }),
    };
    let result = query_skill.execute(query_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    println!("=== Demo Complete ===");

    Ok(())
}

