use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 默认TTL（秒）
    pub default_ttl_secs: i64,
    /// 是否启用统计
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl_secs: 3600,  // 1小时
            enable_stats: true,
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    access_count: usize,
    last_accessed: DateTime<Utc>,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl_secs: Option<i64>) -> Self {
        let now = Utc::now();
        let expires_at = ttl_secs.map(|secs| now + Duration::seconds(secs));

        Self {
            value,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// 智能缓存
pub struct SmartCache<K, V> {
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl<K: Eq + Hash + Clone, V: Clone> SmartCache<K, V> {
    /// 创建新的智能缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 获取缓存值
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            // 检查是否过期
            if entry.is_expired() {
                entries.remove(key);
                self.record_miss().await;
                return None;
            }

            // 更新访问信息
            entry.touch();
            self.record_hit().await;
            Some(entry.value.clone())
        } else {
            self.record_miss().await;
            None
        }
    }

    /// 设置缓存值（使用默认TTL）
    pub async fn set(&self, key: K, value: V) {
        self.set_with_ttl(key, value, None).await;
    }

    /// 设置缓存值（指定TTL）
    pub async fn set_with_ttl(&self, key: K, value: V, ttl_secs: Option<i64>) {
        let mut entries = self.entries.write().await;

        let ttl = ttl_secs.or(Some(self.config.default_ttl_secs));
        let entry = CacheEntry::new(value, ttl);

        entries.insert(key, entry);

        // 如果超过最大条目数，执行LRU淘汰
        if entries.len() > self.config.max_entries {
            self.evict_lru(&mut entries);
        }
    }

    /// 删除缓存值
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;
        entries.remove(key).map(|entry| entry.value)
    }

    /// 清空缓存
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// 获取缓存大小
    pub async fn size(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    /// 清理过期条目
    pub async fn cleanup_expired(&self) -> usize {
        let mut entries = self.entries.write().await;
        let original_len = entries.len();

        entries.retain(|_, entry| !entry.is_expired());

        original_len - entries.len()
    }

    /// LRU淘汰策略
    fn evict_lru(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        if entries.is_empty() {
            return;
        }

        // 找到最久未访问的条目
        let lru_key = entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, _)| k.clone());

        if let Some(key) = lru_key {
            entries.remove(&key);
        }
    }

    /// 记录缓存命中
    async fn record_hit(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.hits += 1;
        }
    }

    /// 记录缓存未命中
    async fn record_miss(&self) {
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
        }
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        let entries = self.entries.read().await;

        CacheStats {
            hits: stats.hits,
            misses: stats.misses,
            size: entries.len(),
            max_size: self.config.max_entries,
        }
    }

    /// 获取缓存命中率
    pub async fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        let total = stats.hits + stats.misses;
        if total == 0 {
            0.0
        } else {
            stats.hits as f64 / total as f64
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub size: usize,
    pub max_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic() {
        let config = CacheConfig::default();
        let cache = SmartCache::new(config);

        // 设置和获取
        cache.set("key1".to_string(), "value1".to_string()).await;
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));

        // 不存在的key
        let missing = cache.get(&"key2".to_string()).await;
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let config = CacheConfig {
            max_entries: 100,
            default_ttl_secs: 1,  // 1秒TTL
            enable_stats: true,
        };
        let cache = SmartCache::new(config);

        cache.set("key1".to_string(), "value1".to_string()).await;

        // 立即获取应该成功
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));

        // 等待过期
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 过期后应该返回None
        let expired = cache.get(&"key1".to_string()).await;
        assert_eq!(expired, None);
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let config = CacheConfig {
            max_entries: 3,
            default_ttl_secs: 3600,
            enable_stats: false,
        };
        let cache = SmartCache::new(config);

        // 添加3个条目
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;
        cache.set("key3".to_string(), "value3".to_string()).await;

        // 访问key1，使其成为最近使用
        cache.get(&"key1".to_string()).await;

        // 添加第4个条目，应该淘汰key2（最久未访问）
        cache.set("key4".to_string(), "value4".to_string()).await;

        let size = cache.size().await;
        assert_eq!(size, 3);

        // key2应该被淘汰
        let key2 = cache.get(&"key2".to_string()).await;
        assert_eq!(key2, None);

        // key1应该还在
        let key1 = cache.get(&"key1".to_string()).await;
        assert_eq!(key1, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let config = CacheConfig::default();
        let cache = SmartCache::new(config);

        cache.set("key1".to_string(), "value1".to_string()).await;

        // 命中
        cache.get(&"key1".to_string()).await;
        cache.get(&"key1".to_string()).await;

        // 未命中
        cache.get(&"key2".to_string()).await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);

        let hit_rate = cache.hit_rate().await;
        assert!((hit_rate - 0.666).abs() < 0.01);
    }
}
