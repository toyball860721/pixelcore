use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex, oneshot};
use tokio::time::{Duration, sleep};

/// 批量处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 最大批量大小
    pub max_batch_size: usize,
    /// 批量等待时间（毫秒）
    pub batch_window_ms: u64,
    /// 是否启用统计
    pub enable_stats: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_window_ms: 50,  // 50ms窗口
            enable_stats: true,
        }
    }
}

/// 批量请求项
struct BatchItem<K, V> {
    key: K,
    sender: oneshot::Sender<Result<V, String>>,
}

/// 批量请求处理器
pub struct BatchProcessor<K, V> {
    config: BatchConfig,
    pending: Arc<Mutex<Vec<BatchItem<K, V>>>>,
    stats: Arc<RwLock<BatchStats>>,
    processing: Arc<Mutex<bool>>,
}

impl<K: Eq + Hash + Clone + Send + 'static, V: Clone + Send + 'static> BatchProcessor<K, V> {
    /// 创建新的批量处理器
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            pending: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(RwLock::new(BatchStats::default())),
            processing: Arc::new(Mutex::new(false)),
        }
    }

    /// 提交请求到批量处理器
    pub async fn submit<F, Fut>(&self, key: K, batch_fn: F) -> Result<V, String>
    where
        F: FnOnce(Vec<K>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<HashMap<K, V>, String>> + Send,
    {
        let (tx, rx) = oneshot::channel();

        // 添加到待处理队列
        {
            let mut pending = self.pending.lock().await;
            pending.push(BatchItem { key: key.clone(), sender: tx });

            // 如果达到最大批量大小，立即触发处理
            if pending.len() >= self.config.max_batch_size {
                drop(pending);
                self.trigger_batch_processing(batch_fn).await;
            } else {
                drop(pending);
                // 启动定时器（如果还没有在处理）
                self.start_batch_timer(batch_fn).await;
            }
        }

        // 等待结果
        rx.await.map_err(|_| "Failed to receive result".to_string())?
    }

    /// 启动批量处理定时器
    async fn start_batch_timer<F, Fut>(&self, batch_fn: F)
    where
        F: FnOnce(Vec<K>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<HashMap<K, V>, String>> + Send,
    {
        let mut processing = self.processing.lock().await;
        if *processing {
            return;  // 已经有定时器在运行
        }
        *processing = true;
        drop(processing);

        let pending = Arc::clone(&self.pending);
        let processing_flag = Arc::clone(&self.processing);
        let window_ms = self.config.batch_window_ms;
        let stats = Arc::clone(&self.stats);
        let enable_stats = self.config.enable_stats;

        tokio::spawn(async move {
            sleep(Duration::from_millis(window_ms)).await;

            // 获取待处理的请求
            let items = {
                let mut pending = pending.lock().await;
                std::mem::take(&mut *pending)
            };

            if !items.is_empty() {
                // 执行批量处理
                let keys: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
                let batch_size = keys.len();

                let result = batch_fn(keys).await;

                // 分发结果
                match result {
                    Ok(results) => {
                        for item in items {
                            if let Some(value) = results.get(&item.key) {
                                let _ = item.sender.send(Ok(value.clone()));
                            } else {
                                let _ = item.sender.send(Err("Key not found in batch result".to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        for item in items {
                            let _ = item.sender.send(Err(e.clone()));
                        }
                    }
                }

                // 更新统计
                if enable_stats {
                    let mut stats = stats.write().await;
                    stats.total_batches += 1;
                    stats.total_requests += batch_size;
                    stats.avg_batch_size = stats.total_requests as f64 / stats.total_batches as f64;
                }
            }

            // 重置处理标志
            let mut processing = processing_flag.lock().await;
            *processing = false;
        });
    }

    /// 触发批量处理（立即执行）
    async fn trigger_batch_processing<F, Fut>(&self, batch_fn: F)
    where
        F: FnOnce(Vec<K>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<HashMap<K, V>, String>> + Send,
    {
        // 获取待处理的请求
        let items = {
            let mut pending = self.pending.lock().await;
            std::mem::take(&mut *pending)
        };

        if items.is_empty() {
            return;
        }

        let keys: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
        let batch_size = keys.len();

        let result = batch_fn(keys).await;

        // 分发结果
        match result {
            Ok(results) => {
                for item in items {
                    if let Some(value) = results.get(&item.key) {
                        let _ = item.sender.send(Ok(value.clone()));
                    } else {
                        let _ = item.sender.send(Err("Key not found in batch result".to_string()));
                    }
                }
            }
            Err(e) => {
                for item in items {
                    let _ = item.sender.send(Err(e.clone()));
                }
            }
        }

        // 更新统计
        if self.config.enable_stats {
            let mut stats = self.stats.write().await;
            stats.total_batches += 1;
            stats.total_requests += batch_size;
            stats.avg_batch_size = stats.total_requests as f64 / stats.total_batches as f64;
        }
    }

    /// 获取统计信息
    pub async fn stats(&self) -> BatchStats {
        self.stats.read().await.clone()
    }
}

/// 批量处理统计信息
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub total_batches: usize,
    pub total_requests: usize,
    pub avg_batch_size: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_processor_basic() {
        let config = BatchConfig {
            max_batch_size: 10,
            batch_window_ms: 100,
            enable_stats: true,
        };

        let processor = Arc::new(BatchProcessor::new(config));

        let result = processor.submit(1, |keys| async move {
            let mut results = HashMap::new();
            for key in keys {
                results.insert(key, format!("value_{}", key));
            }
            Ok(results)
        }).await;

        assert_eq!(result, Ok("value_1".to_string()));
    }

    #[tokio::test]
    async fn test_batch_processor_batching() {
        let config = BatchConfig {
            max_batch_size: 10,
            batch_window_ms: 100,
            enable_stats: true,
        };

        let processor = Arc::new(BatchProcessor::new(config));

        // 提交多个请求
        let mut handles = vec![];
        for i in 1..=5 {
            let proc = Arc::clone(&processor);
            let handle = tokio::spawn(async move {
                proc.submit(i, |keys| async move {
                    let mut results = HashMap::new();
                    for key in keys {
                        results.insert(key, format!("value_{}", key));
                    }
                    Ok(results)
                }).await
            });
            handles.push(handle);
        }

        // 等待所有请求完成
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // 检查统计信息
        let stats = processor.stats().await;
        assert!(stats.total_batches > 0);
        assert_eq!(stats.total_requests, 5);
    }

    #[tokio::test]
    async fn test_batch_processor_max_size() {
        let config = BatchConfig {
            max_batch_size: 3,
            batch_window_ms: 1000,  // 长窗口，确保是批量大小触发
            enable_stats: true,
        };

        let processor = Arc::new(BatchProcessor::new(config));

        // 提交3个请求，应该立即触发批处理
        let mut handles = vec![];
        for i in 1..=3 {
            let proc = Arc::clone(&processor);
            let handle = tokio::spawn(async move {
                proc.submit(i, |keys| async move {
                    let mut results = HashMap::new();
                    for key in keys {
                        results.insert(key, format!("value_{}", key));
                    }
                    Ok(results)
                }).await
            });
            handles.push(handle);
        }

        // 等待所有请求完成
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        let stats = processor.stats().await;
        assert_eq!(stats.total_batches, 1);
        assert_eq!(stats.total_requests, 3);
    }
}
