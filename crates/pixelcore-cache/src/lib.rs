use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Cache miss")]
    CacheMiss,
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Cache manager for Redis operations
pub struct CacheManager {
    connection: ConnectionManager,
    default_ttl: Duration,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(redis_url: &str, default_ttl: Duration) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            connection,
            default_ttl,
        })
    }

    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<T> {
        let value: String = self.connection.get(key).await?;
        let data = serde_json::from_str(&value)?;
        Ok(data)
    }

    /// Set a value in cache with default TTL
    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in cache with custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<()> {
        let serialized = serde_json::to_string(value)?;
        self.connection
            .set_ex(key, serialized, ttl.as_secs())
            .await?;
        Ok(())
    }

    /// Delete a key from cache
    pub async fn delete(&mut self, key: &str) -> Result<()> {
        self.connection.del(key).await?;
        Ok(())
    }

    /// Check if a key exists
    pub async fn exists(&mut self, key: &str) -> Result<bool> {
        let exists: bool = self.connection.exists(key).await?;
        Ok(exists)
    }

    /// Get or set a value (cache-aside pattern)
    pub async fn get_or_set<T, F, Fut>(
        &mut self,
        key: &str,
        fetch_fn: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try to get from cache
        match self.get::<T>(key).await {
            Ok(value) => Ok(value),
            Err(CacheError::CacheMiss) | Err(CacheError::Redis(_)) => {
                // Cache miss, fetch from source
                let value = fetch_fn().await?;
                // Store in cache
                self.set(key, &value).await?;
                Ok(value)
            }
            Err(e) => Err(e),
        }
    }

    /// Increment a counter
    pub async fn increment(&mut self, key: &str) -> Result<i64> {
        let value: i64 = self.connection.incr(key, 1).await?;
        Ok(value)
    }

    /// Decrement a counter
    pub async fn decrement(&mut self, key: &str) -> Result<i64> {
        let value: i64 = self.connection.decr(key, 1).await?;
        Ok(value)
    }

    /// Set expiration time for a key
    pub async fn expire(&mut self, key: &str, ttl: Duration) -> Result<()> {
        self.connection.expire(key, ttl.as_secs() as i64).await?;
        Ok(())
    }

    /// Get TTL for a key
    pub async fn ttl(&mut self, key: &str) -> Result<i64> {
        let ttl: i64 = self.connection.ttl(key).await?;
        Ok(ttl)
    }

    /// Clear all keys matching a pattern
    pub async fn clear_pattern(&mut self, pattern: &str) -> Result<u64> {
        let keys: Vec<String> = self.connection.keys(pattern).await?;
        if keys.is_empty() {
            return Ok(0);
        }
        let count = keys.len() as u64;
        self.connection.del(&keys).await?;
        Ok(count)
    }

    /// Get multiple values at once
    pub async fn mget<T: DeserializeOwned>(&mut self, keys: &[String]) -> Result<Vec<Option<T>>> {
        let values: Vec<Option<String>> = self.connection.get(keys).await?;
        let mut results = Vec::new();

        for value in values {
            match value {
                Some(v) => {
                    let data: T = serde_json::from_str(&v)?;
                    results.push(Some(data));
                }
                None => results.push(None),
            }
        }

        Ok(results)
    }

    /// Set multiple values at once
    pub async fn mset<T: Serialize>(&mut self, items: &[(String, T)]) -> Result<()> {
        let mut pairs = Vec::new();
        for (key, value) in items {
            let serialized = serde_json::to_string(value)?;
            pairs.push((key.clone(), serialized));
        }

        self.connection.set_multiple(&pairs).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u64,
        name: String,
    }

    async fn create_test_cache() -> CacheManager {
        CacheManager::new("redis://127.0.0.1:6379", Duration::from_secs(60))
            .await
            .expect("Failed to create cache manager")
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let mut cache = create_test_cache().await;
        let data = TestData {
            id: 1,
            name: "Test".to_string(),
        };

        cache.set("test:1", &data).await.unwrap();
        let retrieved: TestData = cache.get("test:1").await.unwrap();

        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut cache = create_test_cache().await;
        let data = TestData {
            id: 2,
            name: "Test2".to_string(),
        };

        cache.set("test:2", &data).await.unwrap();
        assert!(cache.exists("test:2").await.unwrap());

        cache.delete("test:2").await.unwrap();
        assert!(!cache.exists("test:2").await.unwrap());
    }

    #[tokio::test]
    async fn test_increment() {
        let mut cache = create_test_cache().await;

        let count1 = cache.increment("counter:1").await.unwrap();
        assert_eq!(count1, 1);

        let count2 = cache.increment("counter:1").await.unwrap();
        assert_eq!(count2, 2);
    }

    #[tokio::test]
    async fn test_ttl() {
        let mut cache = create_test_cache().await;
        let data = TestData {
            id: 3,
            name: "Test3".to_string(),
        };

        cache
            .set_with_ttl("test:3", &data, Duration::from_secs(10))
            .await
            .unwrap();

        let ttl = cache.ttl("test:3").await.unwrap();
        assert!(ttl > 0 && ttl <= 10);
    }
}
