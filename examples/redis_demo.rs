//! Redis operations demo
//!
//! This example demonstrates how to use Redis skills to:
//! - Set key-value pairs
//! - Get values by key
//! - Set values with TTL (time-to-live)
//! - Check if keys exist
//! - Delete keys
//!
//! Note: This example requires a Redis server running at localhost:6379

use pixelcore_skills::{create_redis_skills, SkillInput};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Redis Operations Demo ===\n");
    println!("Note: This demo requires Redis running at localhost:6379\n");

    let redis_url = "redis://localhost:6379";

    // Get Redis skills
    let skills = create_redis_skills();
    let get_skill = &skills[0];
    let set_skill = &skills[1];
    let delete_skill = &skills[2];
    let exists_skill = &skills[3];

    // 1. Set a key-value pair
    println!("1. Setting key 'user:1:name' to 'Alice'...");
    let set_input = SkillInput {
        name: "redis_set".to_string(),
        args: json!({
            "url": redis_url,
            "key": "user:1:name",
            "value": "Alice"
        }),
    };
    let result = set_skill.execute(set_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result)?);

    // 2. Get the value
    println!("2. Getting value for key 'user:1:name'...");
    let get_input = SkillInput {
        name: "redis_get".to_string(),
        args: json!({
            "url": redis_url,
            "key": "user:1:name"
        }),
    };
    let result = get_skill.execute(get_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result.result)?);

    // 3. Set multiple key-value pairs
    println!("3. Setting multiple user keys...");
    let keys = vec![
        ("user:1:age", "30"),
        ("user:1:email", "alice@example.com"),
        ("user:2:name", "Bob"),
        ("user:2:age", "25"),
    ];

    for (key, value) in keys {
        let set_input = SkillInput {
            name: "redis_set".to_string(),
            args: json!({
                "url": redis_url,
                "key": key,
                "value": value
            }),
        };
        set_skill.execute(set_input).await?;
        println!("   Set {} = {}", key, value);
    }
    println!();

    // 4. Check if keys exist
    println!("4. Checking if keys exist...");
    let check_keys = vec!["user:1:name", "user:2:name", "user:3:name"];
    for key in check_keys {
        let exists_input = SkillInput {
            name: "redis_exists".to_string(),
            args: json!({
                "url": redis_url,
                "key": key
            }),
        };
        let result = exists_skill.execute(exists_input).await?;
        println!("   {} exists: {}", key, result.result["exists"]);
    }
    println!();

    // 5. Set a key with TTL (time-to-live)
    println!("5. Setting 'session:abc123' with 5 second TTL...");
    let set_input = SkillInput {
        name: "redis_set".to_string(),
        args: json!({
            "url": redis_url,
            "key": "session:abc123",
            "value": "session_data",
            "ttl": 5
        }),
    };
    let result = set_skill.execute(set_input).await?;
    println!("   Result: {}\n", serde_json::to_string_pretty(&result)?);

    // 6. Verify the session key exists
    println!("6. Checking if session key exists...");
    let exists_input = SkillInput {
        name: "redis_exists".to_string(),
        args: json!({
            "url": redis_url,
            "key": "session:abc123"
        }),
    };
    let result = exists_skill.execute(exists_input).await?;
    println!("   Session exists: {}\n", result.result["exists"]);

    // 7. Delete a key
    println!("7. Deleting 'user:2:name'...");
    let delete_input = SkillInput {
        name: "redis_delete".to_string(),
        args: json!({
            "url": redis_url,
            "key": "user:2:name"
        }),
    };
    let result = delete_skill.execute(delete_input).await?;
    println!("   Deleted: {}\n", result.result["deleted"]);

    // 8. Verify deletion
    println!("8. Verifying 'user:2:name' was deleted...");
    let exists_input = SkillInput {
        name: "redis_exists".to_string(),
        args: json!({
            "url": redis_url,
            "key": "user:2:name"
        }),
    };
    let result = exists_skill.execute(exists_input).await?;
    println!("   Exists: {}\n", result.result["exists"]);

    // 9. Wait for TTL to expire
    println!("9. Waiting 6 seconds for session key to expire...");
    tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

    let exists_input = SkillInput {
        name: "redis_exists".to_string(),
        args: json!({
            "url": redis_url,
            "key": "session:abc123"
        }),
    };
    let result = exists_skill.execute(exists_input).await?;
    println!("   Session exists after TTL: {}\n", result.result["exists"]);

    println!("=== Demo Complete ===");

    Ok(())
}

