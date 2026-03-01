use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use std::collections::VecDeque;
use uuid::Uuid;

/// Agent池配置
#[derive(Debug, Clone)]
pub struct AgentPoolConfig {
    /// 最小Agent数量
    pub min_size: usize,
    /// 最大Agent数量
    pub max_size: usize,
    /// Agent空闲超时时间（秒）
    pub idle_timeout_secs: u64,
    /// 获取Agent的超时时间（毫秒）
    pub acquire_timeout_ms: u64,
}

impl Default for AgentPoolConfig {
    fn default() -> Self {
        Self {
            min_size: 2,
            max_size: 10,
            idle_timeout_secs: 300,  // 5分钟
            acquire_timeout_ms: 5000,  // 5秒
        }
    }
}

/// Agent池中的Agent包装器
pub struct PooledAgent<T> {
    pub id: Uuid,
    pub agent: T,
    pub created_at: std::time::Instant,
    pub last_used: std::time::Instant,
}

impl<T> PooledAgent<T> {
    pub fn new(agent: T) -> Self {
        let now = std::time::Instant::now();
        Self {
            id: Uuid::new_v4(),
            agent,
            created_at: now,
            last_used: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_used = std::time::Instant::now();
    }

    pub fn is_idle_timeout(&self, timeout_secs: u64) -> bool {
        self.last_used.elapsed().as_secs() > timeout_secs
    }
}

/// Agent池
pub struct AgentPool<T> {
    config: AgentPoolConfig,
    available: Arc<RwLock<VecDeque<PooledAgent<T>>>>,
    semaphore: Arc<Semaphore>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
}

impl<T: Send + 'static> AgentPool<T> {
    /// 创建新的Agent池
    pub fn new<F>(config: AgentPoolConfig, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(config.max_size));

        Self {
            config,
            available: Arc::new(RwLock::new(VecDeque::new())),
            semaphore,
            factory: Arc::new(factory),
        }
    }

    /// 初始化Agent池（创建最小数量的Agent）
    pub async fn initialize(&self) {
        let mut available = self.available.write().await;
        for _ in 0..self.config.min_size {
            let agent = (self.factory)();
            available.push_back(PooledAgent::new(agent));
        }
    }

    /// 从池中获取Agent
    pub async fn acquire(&self) -> Result<PooledAgent<T>, String> {
        // 尝试获取信号量许可
        let permit = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.acquire_timeout_ms),
            self.semaphore.acquire()
        )
        .await
        .map_err(|_| "Acquire timeout: no available agents".to_string())?
        .map_err(|e| format!("Semaphore error: {}", e))?;

        // 尝试从可用队列获取Agent
        let mut available = self.available.write().await;

        if let Some(mut agent) = available.pop_front() {
            agent.touch();
            permit.forget();  // 保持许可，直到Agent归还
            return Ok(agent);
        }

        // 如果没有可用Agent，创建新的
        drop(available);  // 释放锁
        let new_agent = (self.factory)();
        permit.forget();  // 保持许可
        Ok(PooledAgent::new(new_agent))
    }

    /// 归还Agent到池中
    pub async fn release(&self, mut agent: PooledAgent<T>) {
        agent.touch();

        // 检查是否超时，如果超时则丢弃
        if agent.is_idle_timeout(self.config.idle_timeout_secs) {
            self.semaphore.add_permits(1);
            return;
        }

        let mut available = self.available.write().await;

        // 如果池已满，丢弃Agent
        if available.len() >= self.config.max_size {
            self.semaphore.add_permits(1);
            return;
        }

        available.push_back(agent);
        self.semaphore.add_permits(1);
    }

    /// 获取池的统计信息
    pub async fn stats(&self) -> AgentPoolStats {
        let available = self.available.read().await;
        AgentPoolStats {
            available_count: available.len(),
            max_size: self.config.max_size,
            min_size: self.config.min_size,
        }
    }

    /// 清理空闲超时的Agent
    pub async fn cleanup_idle(&self) {
        let mut available = self.available.write().await;
        let timeout = self.config.idle_timeout_secs;

        // 保留未超时的Agent
        let mut new_queue = VecDeque::new();
        let mut removed_count = 0;

        while let Some(agent) = available.pop_front() {
            if agent.is_idle_timeout(timeout) && new_queue.len() >= self.config.min_size {
                removed_count += 1;
            } else {
                new_queue.push_back(agent);
            }
        }

        *available = new_queue;

        // 释放信号量许可
        if removed_count > 0 {
            self.semaphore.add_permits(removed_count);
        }
    }
}

/// Agent池统计信息
#[derive(Debug, Clone)]
pub struct AgentPoolStats {
    pub available_count: usize,
    pub max_size: usize,
    pub min_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestAgent {
        id: usize,
    }

    #[tokio::test]
    async fn test_agent_pool_basic() {
        let config = AgentPoolConfig {
            min_size: 2,
            max_size: 5,
            idle_timeout_secs: 60,
            acquire_timeout_ms: 1000,
        };

        let pool = AgentPool::new(config, || TestAgent { id: 1 });

        pool.initialize().await;

        // 获取Agent
        let agent1 = pool.acquire().await.unwrap();
        assert_eq!(agent1.agent.id, 1);

        // 归还Agent
        pool.release(agent1).await;

        // 再次获取应该得到同一个Agent
        let agent2 = pool.acquire().await.unwrap();
        assert_eq!(agent2.agent.id, 1);

        pool.release(agent2).await;
    }

    #[tokio::test]
    async fn test_agent_pool_stats() {
        let config = AgentPoolConfig::default();
        let pool = AgentPool::new(config, || TestAgent { id: 1 });

        pool.initialize().await;

        let stats = pool.stats().await;
        assert_eq!(stats.available_count, 2);
        assert_eq!(stats.min_size, 2);
        assert_eq!(stats.max_size, 10);
    }

    #[tokio::test]
    async fn test_agent_pool_concurrent_acquire() {
        let config = AgentPoolConfig {
            min_size: 2,
            max_size: 5,
            idle_timeout_secs: 60,
            acquire_timeout_ms: 1000,
        };

        let pool = Arc::new(AgentPool::new(config, || TestAgent { id: 1 }));
        pool.initialize().await;

        // 并发获取多个Agent
        let mut handles = vec![];
        for _ in 0..3 {
            let pool_clone = Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let agent = pool_clone.acquire().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                pool_clone.release(agent).await;
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }

        // 检查统计信息
        let stats = pool.stats().await;
        assert!(stats.available_count >= 2);
    }
}
