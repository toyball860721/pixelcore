use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 使用量类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UsageType {
    /// API 调用
    ApiCall,
    /// 计算资源 (CPU 小时)
    ComputeHours,
    /// 存储空间 (GB)
    Storage,
    /// 网络流量 (GB)
    Bandwidth,
    /// 自定义类型
    Custom(String),
}

/// 使用量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// 记录 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 使用量类型
    pub usage_type: UsageType,
    /// 使用量
    pub quantity: f64,
    /// 单位
    pub unit: String,
    /// 记录时间
    pub recorded_at: DateTime<Utc>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl UsageRecord {
    /// 创建新的使用量记录
    pub fn new(user_id: Uuid, usage_type: UsageType, quantity: f64, unit: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            usage_type,
            quantity,
            unit,
            recorded_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}

/// 使用量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// 用户 ID
    pub user_id: Uuid,
    /// 统计周期开始时间
    pub period_start: DateTime<Utc>,
    /// 统计周期结束时间
    pub period_end: DateTime<Utc>,
    /// 各类型使用量
    pub usage_by_type: std::collections::HashMap<UsageType, f64>,
    /// 总使用量
    pub total_usage: f64,
}

/// 定价模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PricingModel {
    /// 按量计费
    PayAsYouGo {
        /// 单价
        unit_price: f64,
        /// 单位
        unit: String,
    },
    /// 包月套餐
    Subscription {
        /// 月费
        monthly_fee: f64,
        /// 包含的使用量
        included_quota: f64,
        /// 超出部分单价
        overage_price: f64,
    },
    /// 阶梯定价
    Tiered {
        /// 阶梯价格 (使用量, 单价)
        tiers: Vec<(f64, f64)>,
    },
    /// 企业定制
    Custom {
        /// 基础费用
        base_fee: f64,
        /// 自定义规则
        rules: serde_json::Value,
    },
}

/// 计费规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRule {
    /// 规则 ID
    pub id: Uuid,
    /// 规则名称
    pub name: String,
    /// 使用量类型
    pub usage_type: UsageType,
    /// 定价模型
    pub pricing_model: PricingModel,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl BillingRule {
    /// 创建新的计费规则
    pub fn new(name: String, usage_type: UsageType, pricing_model: PricingModel) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            usage_type,
            pricing_model,
            enabled: true,
            created_at: Utc::now(),
        }
    }

    /// 计算费用
    pub fn calculate_cost(&self, quantity: f64) -> f64 {
        if !self.enabled {
            return 0.0;
        }

        match &self.pricing_model {
            PricingModel::PayAsYouGo { unit_price, .. } => quantity * unit_price,
            PricingModel::Subscription {
                monthly_fee,
                included_quota,
                overage_price,
            } => {
                if quantity <= *included_quota {
                    *monthly_fee
                } else {
                    monthly_fee + (quantity - included_quota) * overage_price
                }
            }
            PricingModel::Tiered { tiers } => {
                let mut cost = 0.0;
                let mut remaining = quantity;

                for (i, (tier_limit, tier_price)) in tiers.iter().enumerate() {
                    if remaining <= 0.0 {
                        break;
                    }

                    let tier_quantity = if i == tiers.len() - 1 {
                        // 最后一个阶梯，使用所有剩余量
                        remaining
                    } else {
                        // 计算当前阶梯的使用量
                        remaining.min(*tier_limit)
                    };

                    cost += tier_quantity * tier_price;
                    remaining -= tier_quantity;
                }

                cost
            }
            PricingModel::Custom { base_fee, .. } => {
                // 简化实现，实际应该根据 rules 计算
                *base_fee
            }
        }
    }
}

/// 账单状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvoiceStatus {
    /// 草稿
    Draft,
    /// 待支付
    Pending,
    /// 已支付
    Paid,
    /// 逾期
    Overdue,
    /// 已取消
    Cancelled,
}

/// 账单明细项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    /// 使用量类型
    pub usage_type: UsageType,
    /// 描述
    pub description: String,
    /// 使用量
    pub quantity: f64,
    /// 单位
    pub unit: String,
    /// 单价
    pub unit_price: f64,
    /// 小计
    pub subtotal: f64,
}

/// 账单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// 账单 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 账单编号
    pub invoice_number: String,
    /// 账单状态
    pub status: InvoiceStatus,
    /// 账单周期开始
    pub period_start: DateTime<Utc>,
    /// 账单周期结束
    pub period_end: DateTime<Utc>,
    /// 账单明细
    pub items: Vec<InvoiceItem>,
    /// 小计
    pub subtotal: f64,
    /// 税费
    pub tax: f64,
    /// 总计
    pub total: f64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 到期时间
    pub due_date: DateTime<Utc>,
    /// 支付时间
    pub paid_at: Option<DateTime<Utc>>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl Invoice {
    /// 创建新账单
    pub fn new(
        user_id: Uuid,
        invoice_number: String,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        items: Vec<InvoiceItem>,
    ) -> Self {
        let subtotal: f64 = items.iter().map(|item| item.subtotal).sum();
        let tax = subtotal * 0.0; // 暂时不收税
        let total = subtotal + tax;

        Self {
            id: Uuid::new_v4(),
            user_id,
            invoice_number,
            status: InvoiceStatus::Draft,
            period_start,
            period_end,
            items,
            subtotal,
            tax,
            total,
            created_at: Utc::now(),
            due_date: Utc::now() + chrono::Duration::days(30),
            paid_at: None,
            metadata: serde_json::json!({}),
        }
    }

    /// 标记为待支付
    pub fn mark_pending(&mut self) {
        self.status = InvoiceStatus::Pending;
    }

    /// 标记为已支付
    pub fn mark_paid(&mut self) {
        self.status = InvoiceStatus::Paid;
        self.paid_at = Some(Utc::now());
    }

    /// 检查是否逾期
    pub fn is_overdue(&self) -> bool {
        self.status == InvoiceStatus::Pending && Utc::now() > self.due_date
    }

    /// 更新逾期状态
    pub fn update_overdue_status(&mut self) {
        if self.is_overdue() {
            self.status = InvoiceStatus::Overdue;
        }
    }
}

/// 配额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quota {
    /// 配额 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 使用量类型
    pub usage_type: UsageType,
    /// 配额限制
    pub limit: f64,
    /// 已使用量
    pub used: f64,
    /// 重置周期 (天)
    pub reset_period_days: u32,
    /// 上次重置时间
    pub last_reset: DateTime<Utc>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Quota {
    /// 创建新配额
    pub fn new(user_id: Uuid, usage_type: UsageType, limit: f64, reset_period_days: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            usage_type,
            limit,
            used: 0.0,
            reset_period_days,
            last_reset: Utc::now(),
            created_at: Utc::now(),
        }
    }

    /// 获取剩余配额
    pub fn remaining(&self) -> f64 {
        (self.limit - self.used).max(0.0)
    }

    /// 检查是否超出配额
    pub fn is_exceeded(&self) -> bool {
        self.used >= self.limit
    }

    /// 增加使用量
    pub fn add_usage(&mut self, quantity: f64) -> Result<(), String> {
        if self.used + quantity > self.limit {
            return Err("Quota exceeded".to_string());
        }
        self.used += quantity;
        Ok(())
    }

    /// 检查是否需要重置
    pub fn should_reset(&self) -> bool {
        let elapsed = Utc::now() - self.last_reset;
        elapsed.num_days() >= self.reset_period_days as i64
    }

    /// 重置配额
    pub fn reset(&mut self) {
        self.used = 0.0;
        self.last_reset = Utc::now();
    }
}
