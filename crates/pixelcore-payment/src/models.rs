use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 账户类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    /// 个人账户
    Personal,
    /// 企业账户
    Business,
    /// 托管账户
    Escrow,
    /// 系统账户
    System,
}

/// 账户状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountStatus {
    /// 活跃
    Active,
    /// 冻结
    Frozen,
    /// 关闭
    Closed,
}

/// PixelCoin 账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// 账户 ID
    pub id: Uuid,
    /// 所有者 ID (Agent ID)
    pub owner_id: Uuid,
    /// 账户类型
    pub account_type: AccountType,
    /// 账户状态
    pub status: AccountStatus,
    /// 余额 (PixelCoin)
    pub balance: f64,
    /// 冻结金额
    pub frozen_balance: f64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl Account {
    /// 创建新账户
    pub fn new(owner_id: Uuid, account_type: AccountType) -> Self {
        Self {
            id: Uuid::new_v4(),
            owner_id,
            account_type,
            status: AccountStatus::Active,
            balance: 0.0,
            frozen_balance: 0.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    /// 获取可用余额
    pub fn available_balance(&self) -> f64 {
        self.balance - self.frozen_balance
    }

    /// 检查是否可以扣款
    pub fn can_debit(&self, amount: f64) -> bool {
        self.status == AccountStatus::Active && self.available_balance() >= amount
    }

    /// 冻结金额
    pub fn freeze(&mut self, amount: f64) -> Result<(), String> {
        if self.status != AccountStatus::Active {
            return Err("Account is not active".to_string());
        }
        if self.available_balance() < amount {
            return Err("Insufficient available balance".to_string());
        }
        self.frozen_balance += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 解冻金额
    pub fn unfreeze(&mut self, amount: f64) -> Result<(), String> {
        if self.frozen_balance < amount {
            return Err("Insufficient frozen balance".to_string());
        }
        self.frozen_balance -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 增加余额
    pub fn credit(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.balance += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 扣减余额
    pub fn debit(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        if !self.can_debit(amount) {
            return Err("Insufficient balance or account not active".to_string());
        }
        self.balance -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// 支付交易类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentType {
    /// 充值
    Deposit,
    /// 提现
    Withdrawal,
    /// 转账
    Transfer,
    /// 支付
    Payment,
    /// 退款
    Refund,
    /// 手续费
    Fee,
    /// 结算
    Settlement,
}

/// 支付交易状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    /// 待处理
    Pending,
    /// 处理中
    Processing,
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 支付交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    /// 交易 ID
    pub id: Uuid,
    /// 交易类型
    pub payment_type: PaymentType,
    /// 交易状态
    pub status: PaymentStatus,
    /// 源账户 ID
    pub from_account: Option<Uuid>,
    /// 目标账户 ID
    pub to_account: Option<Uuid>,
    /// 金额
    pub amount: f64,
    /// 手续费
    pub fee: f64,
    /// 关联交易 ID (如退款关联原支付)
    pub related_transaction: Option<Uuid>,
    /// 描述
    pub description: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl PaymentTransaction {
    /// 创建新交易
    pub fn new(
        payment_type: PaymentType,
        from_account: Option<Uuid>,
        to_account: Option<Uuid>,
        amount: f64,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            payment_type,
            status: PaymentStatus::Pending,
            from_account,
            to_account,
            amount,
            fee: 0.0,
            related_transaction: None,
            description,
            created_at: Utc::now(),
            completed_at: None,
            metadata: serde_json::json!({}),
        }
    }

    /// 标记为成功
    pub fn mark_success(&mut self) {
        self.status = PaymentStatus::Success;
        self.completed_at = Some(Utc::now());
    }

    /// 标记为失败
    pub fn mark_failed(&mut self) {
        self.status = PaymentStatus::Failed;
        self.completed_at = Some(Utc::now());
    }
}

/// 结算记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settlement {
    /// 结算 ID
    pub id: Uuid,
    /// 交易 ID
    pub transaction_id: Uuid,
    /// 卖方账户
    pub seller_account: Uuid,
    /// 买方账户
    pub buyer_account: Uuid,
    /// 结算金额
    pub amount: f64,
    /// 结算类型
    pub settlement_type: SettlementType,
    /// 结算状态
    pub status: SettlementStatus,
    /// 预定结算时间
    pub scheduled_at: Option<DateTime<Utc>>,
    /// 实际结算时间
    pub settled_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 结算类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettlementType {
    /// 即时结算
    Immediate,
    /// 延迟结算
    Delayed,
    /// 托管结算
    Escrow,
}

/// 结算状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettlementStatus {
    /// 待结算
    Pending,
    /// 已结算
    Settled,
    /// 已取消
    Cancelled,
}

impl Settlement {
    /// 创建新结算
    pub fn new(
        transaction_id: Uuid,
        seller_account: Uuid,
        buyer_account: Uuid,
        amount: f64,
        settlement_type: SettlementType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transaction_id,
            seller_account,
            buyer_account,
            amount,
            settlement_type,
            status: SettlementStatus::Pending,
            scheduled_at: None,
            settled_at: None,
            created_at: Utc::now(),
        }
    }

    /// 标记为已结算
    pub fn mark_settled(&mut self) {
        self.status = SettlementStatus::Settled;
        self.settled_at = Some(Utc::now());
    }
}

