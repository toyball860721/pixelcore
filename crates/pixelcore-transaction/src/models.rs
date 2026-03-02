use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 交易状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// 待处理
    Pending,
    /// 协商中
    Negotiating,
    /// 已确认
    Confirmed,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 争议中
    Disputed,
    /// 已取消
    Cancelled,
}

impl TransactionStatus {
    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// 是否可以取消
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Pending | Self::Negotiating)
    }
}

/// 交易类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// 服务调用
    ServiceCall {
        /// Agent ID
        agent_id: Uuid,
        /// 技能名称
        skill_name: String,
        /// 输入参数
        input: serde_json::Value,
    },
    /// 数据购买
    DataPurchase {
        /// 数据 ID
        data_id: Uuid,
        /// 数据类型
        data_type: String,
    },
    /// 订阅
    Subscription {
        /// Agent ID
        agent_id: Uuid,
        /// 订阅周期 (天)
        period_days: u32,
    },
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// 交易 ID
    pub id: Uuid,
    /// 买方 ID
    pub buyer_id: Uuid,
    /// 卖方 ID
    pub seller_id: Uuid,
    /// 交易类型
    pub transaction_type: TransactionType,
    /// 交易状态
    pub status: TransactionStatus,
    /// 金额
    pub amount: f64,
    /// 货币单位
    pub currency: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 确认时间
    pub confirmed_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 执行结果
    pub result: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl Transaction {
    /// 创建新交易
    pub fn new(
        buyer_id: Uuid,
        seller_id: Uuid,
        transaction_type: TransactionType,
        amount: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            buyer_id,
            seller_id,
            transaction_type,
            status: TransactionStatus::Pending,
            amount,
            currency: "PixelCoin".to_string(),
            created_at: Utc::now(),
            confirmed_at: None,
            completed_at: None,
            result: None,
            error: None,
            metadata: serde_json::json!({}),
        }
    }

    /// 确认交易
    pub fn confirm(&mut self) {
        self.status = TransactionStatus::Confirmed;
        self.confirmed_at = Some(Utc::now());
    }

    /// 开始执行
    pub fn start_execution(&mut self) {
        self.status = TransactionStatus::Executing;
    }

    /// 完成交易
    pub fn complete(&mut self, result: serde_json::Value) {
        self.status = TransactionStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    /// 失败
    pub fn fail(&mut self, error: String) {
        self.status = TransactionStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }

    /// 取消
    pub fn cancel(&mut self) {
        if self.status.can_cancel() {
            self.status = TransactionStatus::Cancelled;
            self.completed_at = Some(Utc::now());
        }
    }

    /// 发起争议
    pub fn dispute(&mut self) {
        self.status = TransactionStatus::Disputed;
    }

    /// 获取执行时长 (毫秒)
    pub fn execution_duration_ms(&self) -> Option<i64> {
        if let (Some(confirmed), Some(completed)) = (self.confirmed_at, self.completed_at) {
            Some((completed - confirmed).num_milliseconds())
        } else {
            None
        }
    }
}

/// 交易历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistory {
    /// 交易 ID
    pub transaction_id: Uuid,
    /// 状态
    pub status: TransactionStatus,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 备注
    pub note: String,
}

impl TransactionHistory {
    /// 创建新的历史记录
    pub fn new(transaction_id: Uuid, status: TransactionStatus, note: String) -> Self {
        Self {
            transaction_id,
            status,
            timestamp: Utc::now(),
            note,
        }
    }
}

/// 交易统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStats {
    /// 总交易数
    pub total_count: u64,
    /// 成功交易数
    pub successful_count: u64,
    /// 失败交易数
    pub failed_count: u64,
    /// 总金额
    pub total_amount: f64,
    /// 平均金额
    pub average_amount: f64,
    /// 平均执行时长 (毫秒)
    pub average_duration_ms: i64,
}

impl TransactionStats {
    /// 从交易列表计算统计信息
    pub fn from_transactions(transactions: &[Transaction]) -> Self {
        let total_count = transactions.len() as u64;
        let successful_count = transactions.iter()
            .filter(|t| t.status == TransactionStatus::Completed)
            .count() as u64;
        let failed_count = transactions.iter()
            .filter(|t| t.status == TransactionStatus::Failed)
            .count() as u64;

        let total_amount: f64 = transactions.iter()
            .map(|t| t.amount)
            .sum();

        let average_amount = if total_count > 0 {
            total_amount / total_count as f64
        } else {
            0.0
        };

        let durations: Vec<i64> = transactions.iter()
            .filter_map(|t| t.execution_duration_ms())
            .collect();

        let average_duration_ms = if !durations.is_empty() {
            durations.iter().sum::<i64>() / durations.len() as i64
        } else {
            0
        };

        Self {
            total_count,
            successful_count,
            failed_count,
            total_amount,
            average_amount,
            average_duration_ms,
        }
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.successful_count as f64 / self.total_count as f64
        }
    }
}
