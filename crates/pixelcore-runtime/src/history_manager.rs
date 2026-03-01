use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

/// 历史记录配置
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    /// 最大记录数
    pub max_entries: usize,
    /// 时间窗口（秒），超过此时间的记录将被清理
    pub time_window_secs: i64,
    /// 是否启用自动清理
    pub auto_cleanup: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            time_window_secs: 3600,  // 1小时
            auto_cleanup: true,
        }
    }
}

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry<T> {
    pub id: uuid::Uuid,
    pub data: T,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl<T> HistoryEntry<T> {
    pub fn new(data: T) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            data,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_metadata(data: T, metadata: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            data,
            timestamp: Utc::now(),
            metadata,
        }
    }

    pub fn is_expired(&self, window_secs: i64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        age.num_seconds() > window_secs
    }
}

/// 历史记录管理器
pub struct HistoryManager<T> {
    config: HistoryConfig,
    entries: Arc<RwLock<VecDeque<HistoryEntry<T>>>>,
}

impl<T: Clone> HistoryManager<T> {
    /// 创建新的历史记录管理器
    pub fn new(config: HistoryConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// 添加记录
    pub async fn add(&self, data: T) -> uuid::Uuid {
        let entry = HistoryEntry::new(data);
        let id = entry.id;

        let mut entries = self.entries.write().await;

        // 添加新记录
        entries.push_back(entry);

        // 如果超过最大数量，移除最旧的记录
        while entries.len() > self.config.max_entries {
            entries.pop_front();
        }

        // 如果启用自动清理，清理过期记录
        if self.config.auto_cleanup {
            self.cleanup_expired_internal(&mut entries);
        }

        id
    }

    /// 添加带元数据的记录
    pub async fn add_with_metadata(&self, data: T, metadata: serde_json::Value) -> uuid::Uuid {
        let entry = HistoryEntry::with_metadata(data, metadata);
        let id = entry.id;

        let mut entries = self.entries.write().await;
        entries.push_back(entry);

        // 限制数量
        while entries.len() > self.config.max_entries {
            entries.pop_front();
        }

        // 自动清理
        if self.config.auto_cleanup {
            self.cleanup_expired_internal(&mut entries);
        }

        id
    }

    /// 获取所有记录
    pub async fn get_all(&self) -> Vec<HistoryEntry<T>> {
        let entries = self.entries.read().await;
        entries.iter().cloned().collect()
    }

    /// 获取最近N条记录
    pub async fn get_recent(&self, count: usize) -> Vec<HistoryEntry<T>> {
        let entries = self.entries.read().await;
        entries.iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// 根据ID获取记录
    pub async fn get_by_id(&self, id: &uuid::Uuid) -> Option<HistoryEntry<T>> {
        let entries = self.entries.read().await;
        entries.iter()
            .find(|e| &e.id == id)
            .cloned()
    }

    /// 清理过期记录
    pub async fn cleanup_expired(&self) -> usize {
        let mut entries = self.entries.write().await;
        self.cleanup_expired_internal(&mut entries)
    }

    /// 内部清理方法
    fn cleanup_expired_internal(&self, entries: &mut VecDeque<HistoryEntry<T>>) -> usize {
        let window = self.config.time_window_secs;
        let original_len = entries.len();

        entries.retain(|entry| !entry.is_expired(window));

        original_len - entries.len()
    }

    /// 清空所有记录
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// 获取统计信息
    pub async fn stats(&self) -> HistoryStats {
        let entries = self.entries.read().await;

        let total_count = entries.len();
        let oldest_timestamp = entries.front().map(|e| e.timestamp);
        let newest_timestamp = entries.back().map(|e| e.timestamp);

        // 估算内存使用（粗略估计）
        let estimated_memory_bytes = total_count * std::mem::size_of::<HistoryEntry<T>>();

        HistoryStats {
            total_count,
            max_entries: self.config.max_entries,
            oldest_timestamp,
            newest_timestamp,
            estimated_memory_bytes,
        }
    }
}

/// 历史记录统计信息
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_count: usize,
    pub max_entries: usize,
    pub oldest_timestamp: Option<DateTime<Utc>>,
    pub newest_timestamp: Option<DateTime<Utc>>,
    pub estimated_memory_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_history_manager_basic() {
        let config = HistoryConfig {
            max_entries: 5,
            time_window_secs: 3600,
            auto_cleanup: false,
        };

        let manager = HistoryManager::new(config);

        // 添加记录
        manager.add("entry1".to_string()).await;
        manager.add("entry2".to_string()).await;
        manager.add("entry3".to_string()).await;

        let all = manager.get_all().await;
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].data, "entry1");
        assert_eq!(all[2].data, "entry3");
    }

    #[tokio::test]
    async fn test_history_manager_max_entries() {
        let config = HistoryConfig {
            max_entries: 3,
            time_window_secs: 3600,
            auto_cleanup: false,
        };

        let manager = HistoryManager::new(config);

        // 添加超过最大数量的记录
        for i in 1..=5 {
            manager.add(format!("entry{}", i)).await;
        }

        let all = manager.get_all().await;
        assert_eq!(all.len(), 3);
        // 应该保留最新的3条
        assert_eq!(all[0].data, "entry3");
        assert_eq!(all[2].data, "entry5");
    }

    #[tokio::test]
    async fn test_history_manager_get_recent() {
        let config = HistoryConfig::default();
        let manager = HistoryManager::new(config);

        for i in 1..=10 {
            manager.add(format!("entry{}", i)).await;
        }

        let recent = manager.get_recent(3).await;
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].data, "entry8");
        assert_eq!(recent[2].data, "entry10");
    }

    #[tokio::test]
    async fn test_history_manager_stats() {
        let config = HistoryConfig::default();
        let manager = HistoryManager::new(config);

        manager.add("entry1".to_string()).await;
        manager.add("entry2".to_string()).await;

        let stats = manager.stats().await;
        assert_eq!(stats.total_count, 2);
        assert!(stats.oldest_timestamp.is_some());
        assert!(stats.newest_timestamp.is_some());
    }
}
