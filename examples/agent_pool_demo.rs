use pixelcore_runtime::{AgentPool, AgentPoolConfig};
use std::sync::Arc;

#[derive(Clone)]
struct WorkerAgent {
    id: usize,
    name: String,
}

impl WorkerAgent {
    fn new(id: usize) -> Self {
        Self {
            id,
            name: format!("Worker-{}", id),
        }
    }

    async fn process_task(&self, task_id: usize) -> String {
        // 模拟任务处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        format!("{} processed task {}", self.name, task_id)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Agent Pool Example ===\n");

    // 配置Agent池
    let config = AgentPoolConfig {
        min_size: 3,
        max_size: 10,
        idle_timeout_secs: 60,
        acquire_timeout_ms: 5000,
    };

    println!("Agent Pool Configuration:");
    println!("  Min size: {}", config.min_size);
    println!("  Max size: {}", config.max_size);
    println!("  Idle timeout: {}s", config.idle_timeout_secs);
    println!("  Acquire timeout: {}ms\n", config.acquire_timeout_ms);

    // 创建Agent池
    let agent_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let pool = Arc::new(AgentPool::new(config, move || {
        let id = agent_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        WorkerAgent::new(id)
    }));

    // 初始化池（创建最小数量的Agent）
    pool.initialize().await;
    println!("✓ Agent pool initialized\n");

    // 显示初始统计信息
    let stats = pool.stats().await;
    println!("Initial pool stats:");
    println!("  Available agents: {}", stats.available_count);
    println!("  Max size: {}", stats.max_size);
    println!("  Min size: {}\n", stats.min_size);

    // 场景1：顺序处理任务
    println!("--- Scenario 1: Sequential Task Processing ---");
    for i in 1..=3 {
        let agent = pool.acquire().await?;
        println!("Acquired agent: {}", agent.agent.name);

        let result = agent.agent.process_task(i).await;
        println!("  Result: {}", result);

        pool.release(agent).await;
        println!("  Agent released back to pool");
    }
    println!();

    // 场景2：并发处理任务
    println!("--- Scenario 2: Concurrent Task Processing ---");
    let mut handles = vec![];

    for task_id in 1..=5 {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let agent = pool_clone.acquire().await.unwrap();
            println!("Task {} acquired agent: {}", task_id, agent.agent.name);

            let result = agent.agent.process_task(task_id).await;
            println!("  Task {} result: {}", task_id, result);

            pool_clone.release(agent).await;
            println!("  Task {} released agent", task_id);
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await?;
    }
    println!();

    // 显示最终统计信息
    let final_stats = pool.stats().await;
    println!("--- Final Pool Stats ---");
    println!("Available agents: {}", final_stats.available_count);
    println!("Max size: {}", final_stats.max_size);
    println!("Min size: {}", final_stats.min_size);

    println!("\n=== Agent Pool Example Complete ===");
    Ok(())
}
