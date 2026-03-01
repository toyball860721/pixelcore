use pixelcore_runtime::{SmartCache, CacheConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Smart Cache Example ===\n");

    // 配置智能缓存
    let config = CacheConfig {
        max_entries: 5,
        default_ttl_secs: 10,  // 10秒TTL
        enable_stats: true,
    };

    println!("Cache Configuration:");
    println!("  Max entries: {}", config.max_entries);
    println!("  Default TTL: {}s", config.default_ttl_secs);
    println!("  Stats enabled: {}\n", config.enable_stats);

    // 创建缓存
    let cache: SmartCache<String, String> = SmartCache::new(config);

    // 场景1：基本的set和get操作
    println!("--- Scenario 1: Basic Set/Get ---");
    cache.set("user:1".to_string(), "Alice".to_string()).await;
    cache.set("user:2".to_string(), "Bob".to_string()).await;
    cache.set("user:3".to_string(), "Charlie".to_string()).await;

    let user1 = cache.get(&"user:1".to_string()).await;
    println!("Get user:1 = {:?}", user1);

    let user2 = cache.get(&"user:2".to_string()).await;
    println!("Get user:2 = {:?}", user2);

    let missing = cache.get(&"user:999".to_string()).await;
    println!("Get user:999 (missing) = {:?}\n", missing);

    // 场景2：缓存命中率统计
    println!("--- Scenario 2: Cache Hit Rate ---");
    // 多次访问相同的key（命中）
    for _ in 0..5 {
        cache.get(&"user:1".to_string()).await;
    }

    // 访问不存在的key（未命中）
    for i in 100..103 {
        cache.get(&format!("user:{}", i)).await;
    }

    let stats = cache.stats().await;
    println!("Cache Statistics:");
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("  Current size: {}/{}\n", stats.size, stats.max_size);

    // 场景3：LRU淘汰策略
    println!("--- Scenario 3: LRU Eviction ---");
    println!("Adding 2 more entries (will exceed max_entries=5)...");

    // 先访问user:1，使其成为最近使用
    cache.get(&"user:1".to_string()).await;
    println!("Accessed user:1 (mark as recently used)");

    // 添加新条目，触发LRU淘汰
    cache.set("user:4".to_string(), "David".to_string()).await;
    cache.set("user:5".to_string(), "Eve".to_string()).await;

    println!("Added user:4 and user:5");

    // 检查哪些条目被淘汰
    let size = cache.size().await;
    println!("Current cache size: {}", size);

    // user:2应该被淘汰（最久未访问）
    let user2_after = cache.get(&"user:2".to_string()).await;
    println!("Get user:2 after eviction = {:?} (should be None)", user2_after);

    // user:1应该还在（最近访问过）
    let user1_after = cache.get(&"user:1".to_string()).await;
    println!("Get user:1 after eviction = {:?} (should be Some)\n", user1_after);

    // 场景4：TTL过期
    println!("--- Scenario 4: TTL Expiration ---");
    cache.set_with_ttl("temp:key".to_string(), "temporary".to_string(), Some(2)).await;
    println!("Set temp:key with TTL=2s");

    let temp_before = cache.get(&"temp:key".to_string()).await;
    println!("Get temp:key immediately = {:?}", temp_before);

    println!("Waiting 3 seconds for expiration...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let temp_after = cache.get(&"temp:key".to_string()).await;
    println!("Get temp:key after 3s = {:?} (should be None)\n", temp_after);

    // 场景5：清理过期条目
    println!("--- Scenario 5: Cleanup Expired Entries ---");
    let removed = cache.cleanup_expired().await;
    println!("Cleaned up {} expired entries", removed);

    let final_stats = cache.stats().await;
    println!("Final cache size: {}\n", final_stats.size);

    // 场景6：清空缓存
    println!("--- Scenario 6: Clear Cache ---");
    cache.clear().await;
    let size_after_clear = cache.size().await;
    println!("Cache size after clear: {}", size_after_clear);

    println!("\n=== Smart Cache Example Complete ===");
    Ok(())
}
