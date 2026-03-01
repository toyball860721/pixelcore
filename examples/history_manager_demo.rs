use pixelcore_runtime::{HistoryManager, HistoryConfig};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

impl ChatMessage {
    fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== History Manager Example ===\n");

    // 配置历史记录管理器
    let config = HistoryConfig {
        max_entries: 10,  // 最多保留10条记录
        time_window_secs: 3600,  // 1小时时间窗口
        auto_cleanup: true,
    };

    println!("History Manager Configuration:");
    println!("  Max entries: {}", config.max_entries);
    println!("  Time window: {}s", config.time_window_secs);
    println!("  Auto cleanup: {}\n", config.auto_cleanup);

    // 创建历史记录管理器
    let manager = HistoryManager::new(config);

    // 场景1：添加聊天消息
    println!("--- Scenario 1: Adding Chat Messages ---");
    manager.add(ChatMessage::new("user", "Hello!")).await;
    manager.add(ChatMessage::new("assistant", "Hi! How can I help you?")).await;
    manager.add(ChatMessage::new("user", "What's the weather like?")).await;
    manager.add(ChatMessage::new("assistant", "I don't have access to real-time weather data.")).await;

    let all_messages = manager.get_all().await;
    println!("Total messages: {}", all_messages.len());
    for (i, entry) in all_messages.iter().enumerate() {
        println!("  {}. [{}] {}", i + 1, entry.data.role, entry.data.content);
    }
    println!();

    // 场景2：获取最近的消息
    println!("--- Scenario 2: Getting Recent Messages ---");
    let recent = manager.get_recent(2).await;
    println!("Last 2 messages:");
    for entry in &recent {
        println!("  [{}] {}", entry.data.role, entry.data.content);
    }
    println!();

    // 场景3：测试最大记录数限制
    println!("--- Scenario 3: Testing Max Entries Limit ---");
    println!("Adding 8 more messages (total will exceed max_entries=10)...");
    for i in 1..=8 {
        manager.add(ChatMessage::new("user", &format!("Message {}", i))).await;
    }

    let all_after = manager.get_all().await;
    println!("Total messages after adding more: {}", all_after.len());
    println!("(Should be limited to max_entries=10)");
    println!();

    // 场景4：查看统计信息
    println!("--- Scenario 4: History Statistics ---");
    let stats = manager.stats().await;
    println!("Statistics:");
    println!("  Total count: {}", stats.total_count);
    println!("  Max entries: {}", stats.max_entries);
    println!("  Estimated memory: {} bytes", stats.estimated_memory_bytes);
    if let Some(oldest) = stats.oldest_timestamp {
        println!("  Oldest entry: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(newest) = stats.newest_timestamp {
        println!("  Newest entry: {}", newest.format("%Y-%m-%d %H:%M:%S"));
    }
    println!();

    // 场景5：带元数据的记录
    println!("--- Scenario 5: Adding Entry with Metadata ---");
    let metadata = serde_json::json!({
        "source": "api",
        "model": "claude-4",
        "tokens": 150
    });
    manager.add_with_metadata(
        ChatMessage::new("assistant", "This message has metadata"),
        metadata
    ).await;

    let last_entry = manager.get_recent(1).await;
    if let Some(entry) = last_entry.first() {
        println!("Last entry with metadata:");
        println!("  Content: {}", entry.data.content);
        println!("  Metadata: {}", entry.metadata);
    }
    println!();

    // 场景6：清空历史
    println!("--- Scenario 6: Clearing History ---");
    manager.clear().await;
    let after_clear = manager.get_all().await;
    println!("Messages after clear: {}", after_clear.len());

    println!("\n=== History Manager Example Complete ===");
    Ok(())
}
