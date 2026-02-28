// 任务调度器示例
// 演示如何使用 TaskScheduler 进行任务管理和调度
// 运行: cargo run --example task_scheduler_demo

use pixelcore_runtime::{TaskScheduler, Task, TaskPriority, TaskStatus, SchedulerConfig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 任务调度器示例\n");

    // 创建调度器
    let config = SchedulerConfig {
        max_concurrent_tasks: 3,
        max_queue_size: 100,
    };
    let scheduler = TaskScheduler::new(config);

    println!("✅ 调度器创建完成");
    println!("   - 最大并发: 3");
    println!("   - 队列容量: 100\n");

    // 场景 1: 提交不同优先级的任务
    println!("📝 场景 1: 提交不同优先级的任务");

    let task1 = Task::new("低优先级任务", TaskPriority::Low, serde_json::json!({
        "action": "process_data",
        "data": [1, 2, 3]
    }));

    let task2 = Task::new("高优先级任务", TaskPriority::High, serde_json::json!({
        "action": "urgent_calculation",
        "value": 100
    }));

    let task3 = Task::new("普通任务", TaskPriority::Normal, serde_json::json!({
        "action": "send_email",
        "to": "user@example.com"
    }));

    let task4 = Task::new("关键任务", TaskPriority::Critical, serde_json::json!({
        "action": "system_backup"
    }));

    let _id1 = scheduler.submit(task1).await?;
    let _id2 = scheduler.submit(task2).await?;
    let _id3 = scheduler.submit(task3).await?;
    let _id4 = scheduler.submit(task4).await?;

    println!("   ✅ 提交了 4 个任务");
    println!("   - 队列长度: {}", scheduler.queue_length().await);
    println!();

    // 场景 2: 按优先级获取任务
    println!("📊 场景 2: 按优先级获取任务");

    let mut execution_order = Vec::new();

    while let Some(task) = scheduler.next_task().await {
        println!("   🔹 获取任务: {} (优先级: {:?})", task.name, task.priority);
        execution_order.push((task.name.clone(), task.priority));

        // 模拟任务执行
        scheduler.update_task_status(task.id, TaskStatus::Running).await;
    }

    println!();
    println!("   执行顺序:");
    for (i, (name, priority)) in execution_order.iter().enumerate() {
        println!("   {}. {} ({:?})", i + 1, name, priority);
    }
    println!();

    // 场景 3: 任务状态管理
    println!("📝 场景 3: 任务状态管理");

    let task5 = Task::new("测试任务", TaskPriority::Normal, serde_json::json!({}));
    let id5 = scheduler.submit(task5).await?;

    println!("   初始状态: {:?}", scheduler.get_task(&id5).await.unwrap().status);

    scheduler.update_task_status(id5, TaskStatus::Running).await;
    println!("   更新为运行中: {:?}", scheduler.get_task(&id5).await.unwrap().status);

    scheduler.set_task_result(id5, serde_json::json!({"result": "success"})).await;
    println!("   设置结果: {:?}", scheduler.get_task(&id5).await.unwrap().status);

    let task_info = scheduler.get_task(&id5).await.unwrap();
    println!("   任务信息:");
    println!("     - 创建时间: {}", task_info.created_at);
    println!("     - 开始时间: {:?}", task_info.started_at);
    println!("     - 完成时间: {:?}", task_info.completed_at);
    println!("     - 结果: {:?}", task_info.result);
    println!();

    // 场景 4: 并发控制
    println!("📊 场景 4: 并发控制");

    println!("   可用槽位: {}", scheduler.available_slots());

    // 提交多个任务
    for i in 0..5 {
        let task = Task::new(
            format!("并发任务 {}", i + 1),
            TaskPriority::Normal,
            serde_json::json!({})
        );
        scheduler.submit(task).await?;
    }

    println!("   提交了 5 个任务");
    println!("   队列长度: {}", scheduler.queue_length().await);

    // 模拟并发执行
    let mut handles = Vec::new();

    for _ in 0..3 {
        if let Some(task) = scheduler.next_task().await {
            let scheduler_clone = scheduler.clone();
            let task_id = task.id;

            let handle = tokio::spawn(async move {
                // 获取信号量许可
                let sem = scheduler_clone.semaphore();
                let _permit = sem.acquire().await.unwrap();

                println!("   🔹 开始执行: {}", task.name);
                scheduler_clone.update_task_status(task_id, TaskStatus::Running).await;

                // 模拟任务执行
                sleep(Duration::from_millis(100)).await;

                scheduler_clone.set_task_result(task_id, serde_json::json!({"status": "ok"})).await;
                println!("   ✅ 完成: {}", task.name);
            });

            handles.push(handle);
        }
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await?;
    }

    println!();
    println!("   剩余队列: {}", scheduler.queue_length().await);
    println!("   运行中任务: {}", scheduler.running_tasks_count().await);
    println!();

    // 场景 5: 任务取消
    println!("📝 场景 5: 任务取消");

    let task6 = Task::new("可取消任务", TaskPriority::Normal, serde_json::json!({}));
    let id6 = scheduler.submit(task6).await?;

    println!("   任务状态: {:?}", scheduler.get_task(&id6).await.unwrap().status);

    scheduler.cancel_task(id6).await?;
    println!("   取消后状态: {:?}", scheduler.get_task(&id6).await.unwrap().status);
    println!();

    // 统计信息
    println!("📊 最终统计:");
    let all_tasks = scheduler.get_all_tasks().await;
    println!("   - 总任务数: {}", all_tasks.len());

    let completed = scheduler.get_tasks_by_status(TaskStatus::Completed).await;
    println!("   - 已完成: {}", completed.len());

    let cancelled = scheduler.get_tasks_by_status(TaskStatus::Cancelled).await;
    println!("   - 已取消: {}", cancelled.len());

    let pending = scheduler.get_tasks_by_status(TaskStatus::Pending).await;
    println!("   - 待处理: {}", pending.len());
    println!();

    println!("🎉 任务调度器示例完成！\n");

    Ok(())
}
