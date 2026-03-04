use pixelcore_cache::{CacheManager, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentListing {
    id: u64,
    name: String,
    price: f64,
    rating: f32,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== PixelCore Cache Demo ===\n");

    // 创建缓存管理器
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let mut cache = CacheManager::new(&redis_url, Duration::from_secs(300)).await?;

    println!("✓ Connected to Redis\n");

    // 示例 1: 基本的 set/get 操作
    println!("--- Example 1: Basic Set/Get ---");
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    cache.set("user:1", &user).await?;
    println!("✓ Cached user: {:?}", user);

    let cached_user: User = cache.get("user:1").await?;
    println!("✓ Retrieved user: {:?}\n", cached_user);

    // 示例 2: 带 TTL 的缓存
    println!("--- Example 2: Cache with TTL ---");
    let agent = AgentListing {
        id: 101,
        name: "Data Analyzer Pro".to_string(),
        price: 50.0,
        rating: 4.8,
    };

    cache
        .set_with_ttl("agent:101", &agent, Duration::from_secs(60))
        .await?;
    println!("✓ Cached agent with 60s TTL: {:?}", agent);

    let ttl = cache.ttl("agent:101").await?;
    println!("✓ TTL: {} seconds\n", ttl);

    // 示例 3: Cache-Aside 模式
    println!("--- Example 3: Cache-Aside Pattern ---");
    let user_id = 2;
    let key = format!("user:{}", user_id);

    let user = cache
        .get_or_set(&key, || async {
            println!("  Cache miss! Fetching from database...");
            // 模拟数据库查询
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(User {
                id: user_id,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            })
        })
        .await?;
    println!("✓ First call (cache miss): {:?}", user);

    let user = cache
        .get_or_set(&key, || async {
            println!("  This should not print!");
            Ok(User {
                id: user_id,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            })
        })
        .await?;
    println!("✓ Second call (cache hit): {:?}\n", user);

    // 示例 4: 计数器
    println!("--- Example 4: Counters ---");
    let counter_key = "api:calls:count";

    for i in 1..=5 {
        let count = cache.increment(counter_key).await?;
        println!("  API call #{}: count = {}", i, count);
    }
    println!();

    // 示例 5: 批量操作
    println!("--- Example 5: Batch Operations ---");
    let users = vec![
        (
            "user:3".to_string(),
            User {
                id: 3,
                name: "Charlie".to_string(),
                email: "charlie@example.com".to_string(),
            },
        ),
        (
            "user:4".to_string(),
            User {
                id: 4,
                name: "David".to_string(),
                email: "david@example.com".to_string(),
            },
        ),
    ];

    cache.mset(&users).await?;
    println!("✓ Cached {} users", users.len());

    let keys: Vec<String> = users.iter().map(|(k, _)| k.clone()).collect();
    let cached_users: Vec<Option<User>> = cache.mget(&keys).await?;
    println!("✓ Retrieved {} users:", cached_users.len());
    for user in cached_users.iter().flatten() {
        println!("  - {:?}", user);
    }
    println!();

    // 示例 6: 模式匹配删除
    println!("--- Example 6: Pattern Deletion ---");
    let count = cache.clear_pattern("user:*").await?;
    println!("✓ Deleted {} keys matching 'user:*'\n", count);

    // 示例 7: 缓存性能测试
    println!("--- Example 7: Performance Test ---");
    let iterations = 1000;
    let start = std::time::Instant::now();

    for i in 0..iterations {
        let key = format!("perf:test:{}", i);
        cache.set(&key, &i).await?;
    }

    let write_duration = start.elapsed();
    println!(
        "✓ Write performance: {} ops in {:?} ({:.2} ops/sec)",
        iterations,
        write_duration,
        iterations as f64 / write_duration.as_secs_f64()
    );

    let start = std::time::Instant::now();

    for i in 0..iterations {
        let key = format!("perf:test:{}", i);
        let _value: i32 = cache.get(&key).await?;
    }

    let read_duration = start.elapsed();
    println!(
        "✓ Read performance: {} ops in {:?} ({:.2} ops/sec)",
        iterations,
        read_duration,
        iterations as f64 / read_duration.as_secs_f64()
    );

    // 清理测试数据
    cache.clear_pattern("perf:test:*").await?;
    println!("✓ Cleaned up test data\n");

    println!("=== Demo Complete ===");

    Ok(())
}
