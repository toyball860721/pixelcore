use pixelcore_runtime::{BatchProcessor, BatchConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// 模拟批量数据库查询
async fn batch_db_query(ids: Vec<i32>, counter: Arc<AtomicUsize>) -> Result<HashMap<i32, String>, String> {
    println!("  [DB] Executing batch query for {} IDs: {:?}", ids.len(), ids);
    counter.fetch_add(1, Ordering::SeqCst);

    // 模拟数据库查询延迟
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut results = HashMap::new();
    for id in ids {
        results.insert(id, format!("User data for ID {}", id));
    }

    println!("  [DB] Batch query completed");
    Ok(results)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Batch Processor Example ===\n");

    // 配置批量处理器
    let config = BatchConfig {
        max_batch_size: 10,
        batch_window_ms: 50,  // 50ms窗口
        enable_stats: true,
    };

    println!("Batch Processor Configuration:");
    println!("  Max batch size: {}", config.max_batch_size);
    println!("  Batch window: {}ms", config.batch_window_ms);
    println!("  Stats enabled: {}\n", config.enable_stats);

    let processor = Arc::new(BatchProcessor::new(config));
    let query_counter = Arc::new(AtomicUsize::new(0));

    // 场景1：单个请求
    println!("--- Scenario 1: Single Request ---");
    let counter_clone = Arc::clone(&query_counter);
    let result = processor.submit(1, move |ids| {
        let counter = Arc::clone(&counter_clone);
        async move {
            batch_db_query(ids, counter).await
        }
    }).await;
    println!("Result: {:?}\n", result);

    // 场景2：批量请求（时间窗口内）
    println!("--- Scenario 2: Batched Requests (within time window) ---");
    println!("Submitting 5 requests within 50ms window...");

    let mut handles = vec![];
    for id in 2..=6 {
        let proc = Arc::clone(&processor);
        let counter_clone = Arc::clone(&query_counter);

        let handle = tokio::spawn(async move {
            let result = proc.submit(id, move |ids| {
                let counter = Arc::clone(&counter_clone);
                async move {
                    batch_db_query(ids, counter).await
                }
            }).await;
            println!("Request {} received: {:?}", id, result);
            result
        });
        handles.push(handle);

        // 小延迟，确保在窗口内
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // 等待所有请求完成
    for handle in handles {
        handle.await??;
    }

    let queries_after_batch = query_counter.load(Ordering::SeqCst);
    println!("\nTotal DB queries: {} (should be 2: one for ID 1, one batch for IDs 2-6)", queries_after_batch);
    println!("Batching saved {} queries!\n", 5 - 1);

    // 场景3：达到最大批量大小
    println!("--- Scenario 3: Max Batch Size Trigger ---");
    println!("Submitting 10 requests (will trigger immediate batch)...");

    let before_count = query_counter.load(Ordering::SeqCst);
    let mut handles = vec![];

    for id in 10..20 {
        let proc = Arc::clone(&processor);
        let counter_clone = Arc::clone(&query_counter);

        let handle = tokio::spawn(async move {
            proc.submit(id, move |ids| {
                let counter = Arc::clone(&counter_clone);
                async move {
                    batch_db_query(ids, counter).await
                }
            }).await
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    for handle in handles {
        handle.await??;
    }

    let after_count = query_counter.load(Ordering::SeqCst);
    println!("\nDB queries for max batch: {}", after_count - before_count);
    println!("(Should be 1, as all 10 requests fit in one batch)\n");

    // 场景4：查看统计信息
    println!("--- Scenario 4: Batch Statistics ---");
    let stats = processor.stats().await;
    println!("Batch Statistics:");
    println!("  Total batches: {}", stats.total_batches);
    println!("  Total requests: {}", stats.total_requests);
    println!("  Average batch size: {:.2}", stats.avg_batch_size);

    let total_queries = query_counter.load(Ordering::SeqCst);
    let efficiency = (1.0 - total_queries as f64 / stats.total_requests as f64) * 100.0;
    println!("\nEfficiency:");
    println!("  Total DB queries: {}", total_queries);
    println!("  Total requests: {}", stats.total_requests);
    println!("  Query reduction: {:.1}%", efficiency);

    println!("\n=== Batch Processor Example Complete ===");
    Ok(())
}
