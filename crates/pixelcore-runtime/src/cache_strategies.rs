//! Cache eviction strategies
//!
//! This module provides different cache eviction strategies
//! for optimizing cache performance.

use std::collections::HashMap;
use std::hash::Hash;
use chrono::{DateTime, Utc};

/// Cache eviction strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionStrategy {
    /// Least Recently Used - evict the least recently accessed entry
    LRU,
    /// Least Frequently Used - evict the least frequently accessed entry
    LFU,
    /// First In First Out - evict the oldest entry
    FIFO,
}

impl Default for EvictionStrategy {
    fn default() -> Self {
        Self::LRU
    }
}

/// Cache entry metadata for eviction decisions
#[derive(Debug, Clone)]
pub struct EntryMetadata {
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: usize,
}

impl EntryMetadata {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            last_accessed: now,
            access_count: 0,
        }
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }
}

/// Eviction policy trait
pub trait EvictionPolicy<K> {
    /// Select a key to evict from the cache
    fn select_victim<V>(&self, entries: &HashMap<K, (V, EntryMetadata)>) -> Option<K>;
}

/// LRU eviction policy
pub struct LRUPolicy;

impl<K: Clone + Eq + Hash> EvictionPolicy<K> for LRUPolicy {
    fn select_victim<V>(&self, entries: &HashMap<K, (V, EntryMetadata)>) -> Option<K> {
        entries
            .iter()
            .min_by_key(|(_, (_, meta))| meta.last_accessed)
            .map(|(k, _)| k.clone())
    }
}

/// LFU eviction policy
pub struct LFUPolicy;

impl<K: Clone + Eq + Hash> EvictionPolicy<K> for LFUPolicy {
    fn select_victim<V>(&self, entries: &HashMap<K, (V, EntryMetadata)>) -> Option<K> {
        entries
            .iter()
            .min_by_key(|(_, (_, meta))| meta.access_count)
            .map(|(k, _)| k.clone())
    }
}

/// FIFO eviction policy
pub struct FIFOPolicy;

impl<K: Clone + Eq + Hash> EvictionPolicy<K> for FIFOPolicy {
    fn select_victim<V>(&self, entries: &HashMap<K, (V, EntryMetadata)>) -> Option<K> {
        entries
            .iter()
            .min_by_key(|(_, (_, meta))| meta.created_at)
            .map(|(k, _)| k.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_policy() {
        let mut entries = HashMap::new();

        let mut meta1 = EntryMetadata::new();
        meta1.last_accessed = Utc::now() - chrono::Duration::seconds(10);
        entries.insert("key1", ("value1", meta1));

        let mut meta2 = EntryMetadata::new();
        meta2.last_accessed = Utc::now() - chrono::Duration::seconds(5);
        entries.insert("key2", ("value2", meta2));

        let policy = LRUPolicy;
        let victim = policy.select_victim(&entries);
        assert_eq!(victim, Some("key1")); // key1 is least recently used
    }

    #[test]
    fn test_lfu_policy() {
        let mut entries = HashMap::new();

        let mut meta1 = EntryMetadata::new();
        meta1.access_count = 5;
        entries.insert("key1", ("value1", meta1));

        let mut meta2 = EntryMetadata::new();
        meta2.access_count = 10;
        entries.insert("key2", ("value2", meta2));

        let policy = LFUPolicy;
        let victim = policy.select_victim(&entries);
        assert_eq!(victim, Some("key1")); // key1 is least frequently used
    }

    #[test]
    fn test_fifo_policy() {
        let mut entries = HashMap::new();

        let mut meta1 = EntryMetadata::new();
        meta1.created_at = Utc::now() - chrono::Duration::seconds(10);
        entries.insert("key1", ("value1", meta1));

        let mut meta2 = EntryMetadata::new();
        meta2.created_at = Utc::now() - chrono::Duration::seconds(5);
        entries.insert("key2", ("value2", meta2));

        let policy = FIFOPolicy;
        let victim = policy.select_victim(&entries);
        assert_eq!(victim, Some("key1")); // key1 is oldest
    }
}
