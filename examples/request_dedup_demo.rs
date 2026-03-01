use pixelcore_runtime::RequestDeduplicator;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// 模拟一个耗时的API调用
async fn expensive_api_call(id: &str, counter: Arc<AtomicUsize>) -> Result<String, String> {
    println!("  [API] Starting expensive call for: {}", id);
    counter.fetch_add(1, Ordering::SeqCst);

    // 模拟耗时操作
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let result = format!("Data for {}", id);
    println!("  [API] Completed call for: {}", id);
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Request Deduplicator Example ===\n");

    let dedup = Arc::new(RequestDeduplicator::new());
    let api_call_counter = Arc::new(AtomicUsize::new(0));

    // 场景1：单个请求
    println!("--- Scenario 1: Single Request ---");
    let result1 = dedup.execute("user:1".to_string(), || {
        let counter = Arc::clone(&api_call_counter);
        async move {
            expensive_api_call("user:1", counter).await
        }
    }).await;
    println!("Result: {:?}\n", result1);

    // 场景2：重复请求合并
    println!("--- Scenario 2: Duplicate Request Merging ---");
    println!("Launching 5 identical requests simultaneously...");

    let mut handles = vec![];
    for i in 1..=5 {
        let dedup_clone = Arc::clone(&dedup);
        let counter_clone = Arc::clone(&api_call_counter);

        let handle = tokio::spawn(async move {
            let result = dedup_clone.execute("user:2".to_string(), || {
                let counter = Arc::clone(&counter_clone);
                async move {
                    expensive_api_call("user:2", counter).await
                }
            }).await;
            println!("Request {} received: {:?}", i, result);
            result
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    for handle in handles {
        handle.await??;
    }

    let api_calls_after_dedup = api_call_counter.load(Ordering::SeqCst);
    println!("\nTotal API calls made: {} (should be 2: one for user:1, one for user:2)", api_calls_after_dedup);
    println!("Deduplication saved {} API calls!\n", 5 - 1);

    // 场景3：不同请求不会合并
    println!("--- Scenario 3: Different Requests (No Merging) ---");
    println!("Launching 3 different requests simultaneously...");

    let before_count = api_call_counter.load(Ordering::SeqCst);
    let mut handles = vec![];

    for i in 3..=5 {
        let dedup_clone = Arc::clone(&dedup);
        let counter_clone = Arc::clone(&api_call_counter);
        let user_id = format!("user:{}", i);

        let handle = tokio::spawn(async move {
            dedup_clone.execute(user_id.clone(), || {
                let counter = Arc::clone(&counter_clone);
                let id = user_id.clone();
                async move {
                    expensive_api_call(&id, counter).await
                }
            }).await
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    for handle in handles {
        handle.await??;
    }

    let after_count = api_call_counter.load(Ordering::SeqCst);
    println!("\nAPI calls for different keys: {}", after_count - before_count);
    println!("(Should be 3, as each key is different)\n");

    // 场景4：监控正在进行的请求
    println!("--- Scenario 4: Monitoring In-Flight Requests ---");

    // 启动一个长时间运行的请求
    let dedup_clone = Arc::clone(&dedup);
    let counter_clone = Arc::clone(&api_call_counter);
    let long_handle = tokio::spawn(async move {
        dedup_clone.execute("long_request".to_string(), || {
            let counter = Arc::clone(&counter_clone);
            async move {
                println!("  [API] Starting long request...");
                counter.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                println!("  [API] Long request completed");
                Ok("Long result".to_string())
            }
        }).await
    });

    // 等待一下，确保请求已经开始
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let in_flight = dedup.in_flight_count().await;
    println!("In-flight requests: {}", in_flight);

    // 等待请求完成
    long_handle.await??;

    let in_flight_after = dedup.in_flight_count().await;
    println!("In-flight requests after completion: {}\n", in_flight_after);

    // 总结
    println!("--- Summary ---");
    let total_api_calls = api_call_counter.load(Ordering::SeqCst);
    println!("Total API calls made: {}", total_api_calls);
    println!("Total requests processed: {}", 5 + 3 + 1 + 1);
    println!("Efficiency gain: {:.1}%", (1.0 - total_api_calls as f64 / 10.0) * 100.0);

    println!("\n=== Request Deduplicator Example Complete ===");
    Ok(())
}
