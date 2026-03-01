//! PostgreSQL database operations demo
//!
//! This example demonstrates how to use PostgreSQL skills to:
//! - Execute SELECT queries
//! - Execute INSERT/UPDATE/DELETE statements
//! - Work with PostgreSQL databases
//!
//! Note: This example requires a running PostgreSQL server.
//! You can set up PostgreSQL locally or use a cloud service.
//!
//! Example connection string:
//!   host=localhost user=postgres password=secret dbname=testdb

use pixelcore_skills::{create_postgres_skills, SkillInput};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PostgreSQL Operations Demo ===\n");

    // Get PostgreSQL skills
    let skills = create_postgres_skills();
    let query_skill = &skills[0];
    let execute_skill = &skills[1];

    // Check if connection string was provided
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example postgres_demo <connection_string>");
        println!("\nExample:");
        println!("  cargo run --example postgres_demo \"host=localhost user=postgres password=secret dbname=testdb\"");
        println!("\nConnection string format:");
        println!("  host=<host> port=<port> user=<user> password=<password> dbname=<database>");
        println!("\nThis demo will:");
        println!("  1. Create a test table");
        println!("  2. Insert sample data");
        println!("  3. Query the data");
        println!("  4. Update records");
        println!("  5. Delete records");
        println!("  6. Drop the test table");
        println!("\nNote: Requires a running PostgreSQL server.");
        return Ok(());
    }

    let connection_string = &args[1];
    println!("Connecting to PostgreSQL...\n");

    // 1. Create table
    println!("1. Creating test table 'users'...");
    let create_input = SkillInput {
        name: "postgres_execute".to_string(),
        args: json!({
            "connection_string": connection_string,
            "statement": "CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name VARCHAR(100), age INT, email VARCHAR(100))"
        }),
    };

    let result = execute_skill.execute(create_input).await?;
    if result.success {
        println!("   ✓ Table created successfully\n");
    } else {
        println!("   ✗ Failed to create table: {:?}\n", result.error);
        return Ok(());
    }

    // 2. Insert data
    println!("2. Inserting sample data...");
    let inserts = vec![
        "INSERT INTO users (name, age, email) VALUES ('Alice', 30, 'alice@example.com')",
        "INSERT INTO users (name, age, email) VALUES ('Bob', 25, 'bob@example.com')",
        "INSERT INTO users (name, age, email) VALUES ('Charlie', 35, 'charlie@example.com')",
    ];

    for insert_sql in inserts {
        let insert_input = SkillInput {
            name: "postgres_execute".to_string(),
            args: json!({
                "connection_string": connection_string,
                "statement": insert_sql
            }),
        };
        execute_skill.execute(insert_input).await?;
    }
    println!("   ✓ Inserted 3 users\n");

    // 3. Query all users
    println!("3. Querying all users...");
    let query_input = SkillInput {
        name: "postgres_query".to_string(),
        args: json!({
            "connection_string": connection_string,
            "query": "SELECT * FROM users ORDER BY age"
        }),
    };

    let result = query_skill.execute(query_input).await?;
    if result.success {
        println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);
    }

    // 4. Query with condition
    println!("4. Querying users older than 28...");
    let query_input = SkillInput {
        name: "postgres_query".to_string(),
        args: json!({
            "connection_string": connection_string,
            "query": "SELECT name, age FROM users WHERE age > 28"
        }),
    };

    let result = query_skill.execute(query_input).await?;
    if result.success {
        println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);
    }

    // 5. Update data
    println!("5. Updating Bob's age...");
    let update_input = SkillInput {
        name: "postgres_execute".to_string(),
        args: json!({
            "connection_string": connection_string,
            "statement": "UPDATE users SET age = 26 WHERE name = 'Bob'"
        }),
    };

    let result = execute_skill.execute(update_input).await?;
    if result.success {
        println!("   ✓ Rows affected: {}\n", result.result["rows_affected"]);
    }

    // 6. Delete data
    println!("6. Deleting Charlie...");
    let delete_input = SkillInput {
        name: "postgres_execute".to_string(),
        args: json!({
            "connection_string": connection_string,
            "statement": "DELETE FROM users WHERE name = 'Charlie'"
        }),
    };

    let result = execute_skill.execute(delete_input).await?;
    if result.success {
        println!("   ✓ Rows affected: {}\n", result.result["rows_affected"]);
    }

    // 7. Final query
    println!("7. Final state of users table...");
    let query_input = SkillInput {
        name: "postgres_query".to_string(),
        args: json!({
            "connection_string": connection_string,
            "query": "SELECT * FROM users"
        }),
    };

    let result = query_skill.execute(query_input).await?;
    if result.success {
        println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);
    }

    // 8. Clean up - drop table
    println!("8. Cleaning up - dropping test table...");
    let drop_input = SkillInput {
        name: "postgres_execute".to_string(),
        args: json!({
            "connection_string": connection_string,
            "statement": "DROP TABLE users"
        }),
    };

    let result = execute_skill.execute(drop_input).await?;
    if result.success {
        println!("   ✓ Table dropped successfully\n");
    }

    println!("=== Demo Complete ===");

    Ok(())
}
