use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex, broadcast};
use std::future::Future;

/// 请求去重器
pub struct RequestDeduplicator<K, V> {
    /// 正在进行的请求
    in_flight: Arc<RwLock<HashMap<K, Arc<Mutex<Option<broadcast::Sender<Result<V, String>>>>>>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> RequestDeduplicator<K, V> {
    /// 创建新的请求去重器
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 执行请求（如果相同请求正在进行，则等待其结果）
    pub async fn execute<F, Fut>(&self, key: K, f: F) -> Result<V, String>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<V, String>>,
    {
        // 检查是否有相同的请求正在进行
        {
            let in_flight = self.in_flight.read().await;
            if let Some(sender_mutex) = in_flight.get(&key) {
                // 有相同请求正在进行，等待其结果
                let sender_mutex = Arc::clone(sender_mutex);
                drop(in_flight);

                return self.wait_for_result(sender_mutex).await;
            }
        }

        // 没有相同请求，创建新的广播通道
        let (tx, _rx) = broadcast::channel(100);
        let tx = Arc::new(Mutex::new(Some(tx)));

        // 注册请求
        {
            let mut in_flight = self.in_flight.write().await;
            in_flight.insert(key.clone(), Arc::clone(&tx));
        }

        // 执行请求
        let result = f().await;

        // 广播结果给所有等待者
        {
            let mut tx_guard = tx.lock().await;
            if let Some(sender) = tx_guard.take() {
                let _ = sender.send(result.clone());
            }
        }

        // 清理
        {
            let mut in_flight = self.in_flight.write().await;
            in_flight.remove(&key);
        }

        result
    }

    /// 等待正在进行的请求结果
    async fn wait_for_result(
        &self,
        sender_mutex: Arc<Mutex<Option<broadcast::Sender<Result<V, String>>>>>,
    ) -> Result<V, String> {
        let mut rx = {
            let sender_guard = sender_mutex.lock().await;
            if let Some(sender) = sender_guard.as_ref() {
                sender.subscribe()
            } else {
                return Err("Request already completed".to_string());
            }
        };

        match rx.recv().await {
            Ok(result) => result,
            Err(_) => Err("Failed to receive result".to_string()),
        }
    }

    /// 获取当前正在进行的请求数量
    pub async fn in_flight_count(&self) -> usize {
        let in_flight = self.in_flight.read().await;
        in_flight.len()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for RequestDeduplicator<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_deduplicator_basic() {
        let dedup = RequestDeduplicator::new();

        let result = dedup.execute("key1".to_string(), || async {
            Ok("value1".to_string())
        }).await;

        assert_eq!(result, Ok("value1".to_string()));
    }

    #[tokio::test]
    async fn test_deduplicator_merges_requests() {
        let dedup = Arc::new(RequestDeduplicator::new());
        let counter = Arc::new(AtomicUsize::new(0));

        // 启动3个相同的请求
        let mut handles = vec![];
        for _ in 0..3 {
            let dedup_clone = Arc::clone(&dedup);
            let counter_clone = Arc::clone(&counter);

            let handle = tokio::spawn(async move {
                dedup_clone.execute("same_key".to_string(), || {
                    let counter = Arc::clone(&counter_clone);
                    async move {
                        // 增加计数器（应该只执行一次）
                        counter.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        Ok("result".to_string())
                    }
                }).await
            });
            handles.push(handle);
        }

        // 等待所有请求完成
        let mut results = vec![];
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }

        // 所有请求都应该得到相同的结果
        assert_eq!(results.len(), 3);
        for result in &results {
            assert_eq!(result, &Ok("result".to_string()));
        }

        // 计数器应该只增加了1次（请求被合并）
        let count = counter.load(Ordering::SeqCst);
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_deduplicator_different_keys() {
        let dedup = Arc::new(RequestDeduplicator::new());
        let counter = Arc::new(AtomicUsize::new(0));

        // 启动2个不同的请求
        let mut handles = vec![];
        for i in 0..2 {
            let dedup_clone = Arc::clone(&dedup);
            let counter_clone = Arc::clone(&counter);
            let key = format!("key{}", i);

            let handle = tokio::spawn(async move {
                dedup_clone.execute(key, || {
                    let counter = Arc::clone(&counter_clone);
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        Ok(format!("result{}", counter.load(Ordering::SeqCst)))
                    }
                }).await
            });
            handles.push(handle);
        }

        // 等待所有请求完成
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // 计数器应该增加了2次（不同的key不会合并）
        let count = counter.load(Ordering::SeqCst);
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_deduplicator_in_flight_count() {
        let dedup = Arc::new(RequestDeduplicator::new());

        // 启动一个长时间运行的请求
        let dedup_clone = Arc::clone(&dedup);
        let handle = tokio::spawn(async move {
            dedup_clone.execute("long_key".to_string(), || async {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                Ok("result".to_string())
            }).await
        });

        // 等待一下，确保请求已经开始
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // 检查正在进行的请求数量
        let count = dedup.in_flight_count().await;
        assert_eq!(count, 1);

        // 等待请求完成
        handle.await.unwrap().unwrap();

        // 请求完成后，正在进行的请求数量应该为0
        let count_after = dedup.in_flight_count().await;
        assert_eq!(count_after, 0);
    }
}
