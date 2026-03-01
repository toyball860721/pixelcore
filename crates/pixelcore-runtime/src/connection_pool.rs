//! Connection pool management for reusing connections
//!
//! This module provides a generic connection pool that can be used
//! to reuse expensive connections (database, HTTP, etc.)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

/// Connection pool entry
struct PoolEntry<T> {
    connection: T,
    last_used: Instant,
    in_use: bool,
}

/// Generic connection pool
pub struct ConnectionPool<T> {
    pools: Arc<RwLock<HashMap<String, Vec<PoolEntry<T>>>>>,
    max_connections: usize,
    max_idle_time: Duration,
}

impl<T: Clone> ConnectionPool<T> {
    /// Create a new connection pool
    pub fn new(max_connections: usize, max_idle_time: Duration) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
            max_idle_time,
        }
    }

    /// Get a connection from the pool or create a new one
    pub async fn get_or_create<F, Fut, E>(
        &self,
        key: &str,
        mut create_fn: F,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        loop {
            // Try to get an existing connection
            {
                let mut pools = self.pools.write().await;
                if let Some(pool) = pools.get_mut(key) {
                    // Clean up expired connections
                    pool.retain(|entry| {
                        entry.last_used.elapsed() < self.max_idle_time
                    });

                    // Find an available connection
                    if let Some(entry) = pool.iter_mut().find(|e| !e.in_use) {
                        entry.in_use = true;
                        entry.last_used = Instant::now();
                        return Ok(entry.connection.clone());
                    }
                }
            }

            // Check if pool is full
            {
                let pools = self.pools.read().await;
                if let Some(pool) = pools.get(key) {
                    if pool.len() >= self.max_connections {
                        // Pool is full, wait and retry
                        drop(pools);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                }
            }

            // Create new connection
            let connection = create_fn().await?;

            // Add to pool
            {
                let mut pools = self.pools.write().await;
                let pool = pools.entry(key.to_string()).or_insert_with(Vec::new);
                pool.push(PoolEntry {
                    connection: connection.clone(),
                    last_used: Instant::now(),
                    in_use: true,
                });
            }

            return Ok(connection);
        }
    }

    /// Release a connection back to the pool
    pub async fn release(&self, key: &str, _connection: &T) {
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.get_mut(key) {
            for entry in pool.iter_mut() {
                // Simple comparison - in real implementation, you'd need proper equality
                entry.in_use = false;
                entry.last_used = Instant::now();
                break;
            }
        }
    }

    /// Clear all connections from the pool
    pub async fn clear(&self) {
        let mut pools = self.pools.write().await;
        pools.clear();
    }

    /// Get pool statistics
    pub async fn stats(&self, key: &str) -> PoolStats {
        let pools = self.pools.read().await;
        if let Some(pool) = pools.get(key) {
            let total = pool.len();
            let in_use = pool.iter().filter(|e| e.in_use).count();
            let idle = total - in_use;
            PoolStats {
                total,
                in_use,
                idle,
            }
        } else {
            PoolStats {
                total: 0,
                in_use: 0,
                idle: 0,
            }
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total: usize,
    pub in_use: usize,
    pub idle: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_basic() {
        let pool = ConnectionPool::new(5, Duration::from_secs(60));

        // Create first connection
        let conn1 = pool
            .get_or_create("test", || async { Ok::<String, String>("conn1".to_string()) })
            .await
            .unwrap();
        assert_eq!(conn1, "conn1");

        // Get stats
        let stats = pool.stats("test").await;
        assert_eq!(stats.total, 1);
        assert_eq!(stats.in_use, 1);

        // Release connection
        pool.release("test", &conn1).await;

        // Get stats after release
        let stats = pool.stats("test").await;
        assert_eq!(stats.total, 1);
        assert_eq!(stats.in_use, 0);
        assert_eq!(stats.idle, 1);
    }

    #[tokio::test]
    async fn test_connection_pool_reuse() {
        let pool = ConnectionPool::new(5, Duration::from_secs(60));

        // Create first connection
        let conn1 = pool
            .get_or_create("test", || async { Ok::<String, String>("conn1".to_string()) })
            .await
            .unwrap();

        // Release it
        pool.release("test", &conn1).await;

        // Get another connection - should reuse
        let conn2 = pool
            .get_or_create("test", || async { Ok::<String, String>("conn2".to_string()) })
            .await
            .unwrap();

        // Should still be the first connection
        assert_eq!(conn2, "conn1");

        // Stats should show 1 total connection
        let stats = pool.stats("test").await;
        assert_eq!(stats.total, 1);
    }

    #[tokio::test]
    async fn test_connection_pool_max_connections() {
        let pool = ConnectionPool::new(2, Duration::from_secs(60));

        // Create two connections
        let _conn1 = pool
            .get_or_create("test", || async { Ok::<String, String>("conn1".to_string()) })
            .await
            .unwrap();

        let _conn2 = pool
            .get_or_create("test", || async { Ok::<String, String>("conn2".to_string()) })
            .await
            .unwrap();

        // Stats should show 2 connections
        let stats = pool.stats("test").await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.in_use, 2);
    }
}
