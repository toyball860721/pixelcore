use serde::{Deserialize, Serialize};

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: usize,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
    /// 是否使用指数退避
    pub exponential_backoff: bool,
    /// 退避倍数
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            exponential_backoff: true,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    pub fn new(max_retries: usize) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }

    pub fn with_exponential_backoff(mut self, enabled: bool) -> Self {
        self.exponential_backoff = enabled;
        self
    }

    /// 计算第 n 次重试的延迟时间
    pub fn calculate_delay(&self, attempt: usize) -> u64 {
        if self.exponential_backoff {
            let multiplier = self.backoff_multiplier.powi(attempt as i32);
            (self.retry_delay_ms as f64 * multiplier) as u64
        } else {
            self.retry_delay_ms
        }
    }
}

/// 错误处理策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ErrorHandlingStrategy {
    /// 失败时停止整个工作流
    Fail,
    /// 忽略错误，继续执行
    Ignore,
    /// 重试
    Retry {
        policy: RetryPolicy,
    },
    /// 跳转到指定节点
    Fallback {
        /// 目标节点 ID
        fallback_node: String,
    },
}

impl Default for ErrorHandlingStrategy {
    fn default() -> Self {
        Self::Fail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.retry_delay_ms, 1000);
        assert!(policy.exponential_backoff);
    }

    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy::default();

        // 第 0 次重试：1000ms
        assert_eq!(policy.calculate_delay(0), 1000);

        // 第 1 次重试：2000ms
        assert_eq!(policy.calculate_delay(1), 2000);

        // 第 2 次重试：4000ms
        assert_eq!(policy.calculate_delay(2), 4000);
    }

    #[test]
    fn test_retry_policy_linear_backoff() {
        let policy = RetryPolicy::default()
            .with_exponential_backoff(false);

        assert_eq!(policy.calculate_delay(0), 1000);
        assert_eq!(policy.calculate_delay(1), 1000);
        assert_eq!(policy.calculate_delay(2), 1000);
    }

    #[test]
    fn test_error_handling_strategies() {
        let fail = ErrorHandlingStrategy::Fail;
        assert!(matches!(fail, ErrorHandlingStrategy::Fail));

        let ignore = ErrorHandlingStrategy::Ignore;
        assert!(matches!(ignore, ErrorHandlingStrategy::Ignore));

        let retry = ErrorHandlingStrategy::Retry {
            policy: RetryPolicy::new(5),
        };
        if let ErrorHandlingStrategy::Retry { policy } = retry {
            assert_eq!(policy.max_retries, 5);
        }
    }
}
