//! Cache strategies optimization demo
//!
//! This example demonstrates the enhanced caching capabilities
//! including different eviction strategies, batch operations,
//! and cache warmup.

use pixelcore_runtime::{SmartCache, CacheConfig, EvictionStrategy};

async fn demo_eviction_strategy(strategy: EvictionStrategy, name: &str) {
    println!("=== {} Eviction Strategy ===", name);

    let config = CacheConfig {
        max_entries: 3,
        default_ttl_secs: 3600,
        enable_stats: true,
        eviction_strategy: strategy,
    };
    let cache = SmartCache::new(config);

    // Add 3 entries
    cache.set("key1", "value1").await;
    cache.set("key2", "value2").await;
    cache.set("key3", "value3").await;

    println!("Added 3 entries: key1, key2, key3");

    // Access patterns differ based on strategy
    match strategy {
        EvictionStrategy::LRU => {
            // Access key1 and key2, making key3 least recently used
            cache.get(&"key1").await;
            cache.get(&"key2").await;
            println!("Accessed key1 and key2 (key3 is LRU)");
        }
        EvictionStrategy::LFU => {
            // Access key1 multiple times, making key2 and key3 least frequently used
            cache.get(&"key1").await;
            cache.get(&"key1").await;
            cache.get(&"key1").await;
            println!("Accessed key1 3 times (key2/key3 are LFU)");
        }
        EvictionStrategy::FIFO => {
            // FIFO doesn't care about access, key1 is oldest
            println!("No access needed (key1 is oldest)");
        }
    }

    // Add 4th entry, triggering eviction
    cache.set("key4", "value4").await;
    println!("Added key4 (triggered eviction)");

    // Check which key was evicted
    let key1 = cache.get(&"key1").await;
    let key2 = cache.get(&"key2").await;
    let key3 = cache.get(&"key3").await;
    let key4 = cache.get(&"key4").await;

    println!("After eviction:");
    println!("  key1: {}", if key1.is_some() { "present" } else { "evicted" });
    println!("  key2: {}", if key2.is_some() { "present" } else { "evicted" });
    println!("  key3: {}", if key3.is_some() { "present" } else { "evicted" });
    println!("  key4: {}", if key4.is_some() { "present" } else { "evicted" });

    let stats = cache.stats().await;
    println!("Stats: {} hits, {} misses, {} evictions\n", stats.hits, stats.misses, stats.evictions);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cache Strategies Optimization Demo ===\n");

    // Demo 1: Different eviction strategies
    println!("Demo 1: Eviction Strategies");
    println!("─────────────────────────────\n");

    demo_eviction_strategy(EvictionStrategy::LRU, "LRU").await;
    demo_eviction_strategy(EvictionStrategy::LFU, "LFU").await;
    demo_eviction_strategy(EvictionStrategy::FIFO, "FIFO").await;

    // Demo 2: Batch operations
    println!("Demo 2: Batch Operations");
    println!("─────────────────────────────");

    let config = CacheConfig::default();
    let cache = SmartCache::new(config);

    // Batch set
    let items = vec![
        ("user:1", "Alice"),
        ("user:2", "Bob"),
        ("user:3", "Charlie"),
        ("user:4", "David"),
        ("user:5", "Eve"),
    ];
    cache.set_many(items).await;
    println!("Batch set 5 users");

    // Batch get
    let keys = vec!["user:1", "user:3", "user:5"];
    let results = cache.get_many(&keys).await;
    println!("Batch get {} users:", results.len());
    for (key, value) in results {
        println!("  {}: {}", key, value);
    }

    let stats = cache.stats().await;
    println!("Cache size: {}/{}\n", stats.size, stats.max_size);

    // Demo 3: Cache warmup
    println!("Demo 3: Cache Warmup");
    println!("─────────────────────────────");

    let cache2 = SmartCache::new(CacheConfig::default());

    // Simulate data loader (e.g., from database)
    let loader = |key: String| async move {
        // Simulate database query
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Some(format!("data_for_{}", key))
    };

    let keys_to_warmup = vec![
        "product:1".to_string(),
        "product:2".to_string(),
        "product:3".to_string(),
    ];

    println!("Warming up cache with {} keys...", keys_to_warmup.len());
    let start = std::time::Instant::now();
    cache2.warmup(keys_to_warmup, loader).await;
    let duration = start.elapsed();

    println!("Warmup completed in {:.2}ms", duration.as_millis());

    // Verify data is cached
    let value = cache2.get(&"product:1".to_string()).await;
    println!("Retrieved from cache: {:?}", value);

    let stats = cache2.stats().await;
    println!("Cache stats: {} hits, {} misses, size: {}\n", stats.hits, stats.misses, stats.size);

    // Demo 4: Performance comparison
    println!("Demo 4: Hit Rate Comparison");
    println!("─────────────────────────────");

    let cache3 = SmartCache::new(CacheConfig::default());

    // Simulate workload
    for i in 0..100 {
        let key = format!("key{}", i % 10); // 10 unique keys, repeated
        if cache3.get(&key).await.is_none() {
            cache3.set(key, format!("value{}", i)).await;
        }
    }

    let stats = cache3.stats().await;
    println!("After 100 operations:");
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!("  Evictions: {}", stats.evictions);

    println!("\n=== Demo completed successfully ===");
    Ok(())
}