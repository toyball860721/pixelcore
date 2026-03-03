use crate::models::{BillingRule, UsageType, PricingModel, Invoice, InvoiceItem, InvoiceStatus};
use crate::usage_tracker::UsageTracker;
use uuid::Uuid;
use chrono::{DateTime, Utc, Datelike};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 计费引擎
pub struct BillingEngine {
    usage_tracker: UsageTracker,
    rules: Arc<Mutex<Vec<BillingRule>>>,
    invoices: Arc<Mutex<Vec<Invoice>>>,
}

impl BillingEngine {
    /// 创建新的计费引擎
    pub fn new(usage_tracker: UsageTracker) -> Self {
        Self {
            usage_tracker,
            rules: Arc::new(Mutex::new(Vec::new())),
            invoices: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 添加计费规则
    pub async fn add_rule(&self, rule: BillingRule) -> Result<(), String> {
        let mut rules = self.rules.lock().await;

        // 检查是否已存在相同类型的规则
        if rules.iter().any(|r| r.usage_type == rule.usage_type && r.enabled) {
            return Err("A rule for this usage type already exists".to_string());
        }

        rules.push(rule);
        Ok(())
    }

    /// 获取计费规则
    pub async fn get_rule(&self, usage_type: &UsageType) -> Option<BillingRule> {
        let rules = self.rules.lock().await;
        rules
            .iter()
            .find(|r| &r.usage_type == usage_type && r.enabled)
            .cloned()
    }

    /// 获取所有计费规则
    pub async fn get_all_rules(&self) -> Vec<BillingRule> {
        let rules = self.rules.lock().await;
        rules.clone()
    }

    /// 更新计费规则
    pub async fn update_rule(&self, rule_id: Uuid, new_rule: BillingRule) -> Result<(), String> {
        let mut rules = self.rules.lock().await;

        if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            *rule = new_rule;
            Ok(())
        } else {
            Err("Rule not found".to_string())
        }
    }

    /// 禁用计费规则
    pub async fn disable_rule(&self, rule_id: Uuid) -> Result<(), String> {
        let mut rules = self.rules.lock().await;

        if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = false;
            Ok(())
        } else {
            Err("Rule not found".to_string())
        }
    }

    /// 生成账单
    pub async fn generate_invoice(
        &self,
        user_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Invoice, String> {
        // 获取使用量统计
        let stats = self
            .usage_tracker
            .get_usage_stats(user_id, period_start, period_end)
            .await?;

        // 获取计费规则
        let rules = self.rules.lock().await;

        // 计算每个使用量类型的费用
        let mut items = Vec::new();

        for (usage_type, quantity) in stats.usage_by_type {
            if let Some(rule) = rules.iter().find(|r| r.usage_type == usage_type && r.enabled) {
                let cost = rule.calculate_cost(quantity);

                let (unit_price, unit) = match &rule.pricing_model {
                    PricingModel::PayAsYouGo { unit_price, unit } => (*unit_price, unit.clone()),
                    PricingModel::Subscription { monthly_fee, .. } => (*monthly_fee, "month".to_string()),
                    PricingModel::Tiered { tiers } => {
                        if let Some((_, price)) = tiers.first() {
                            (*price, "unit".to_string())
                        } else {
                            (0.0, "unit".to_string())
                        }
                    }
                    PricingModel::Custom { base_fee, .. } => (*base_fee, "custom".to_string()),
                };

                items.push(InvoiceItem {
                    usage_type: usage_type.clone(),
                    description: format!("{:?} usage", usage_type),
                    quantity,
                    unit,
                    unit_price,
                    subtotal: cost,
                });
            }
        }

        // 生成账单编号
        let invoice_number = self.generate_invoice_number(user_id, period_start).await;

        // 创建账单
        let mut invoice = Invoice::new(user_id, invoice_number, period_start, period_end, items);
        invoice.mark_pending();

        // 保存账单
        let mut invoices = self.invoices.lock().await;
        invoices.push(invoice.clone());

        Ok(invoice)
    }

    /// 生成账单编号
    async fn generate_invoice_number(&self, user_id: Uuid, period_start: DateTime<Utc>) -> String {
        let year = period_start.year();
        let month = period_start.month();
        let user_short = &user_id.to_string()[..8];

        format!("INV-{}-{:02}-{}", year, month, user_short)
    }

    /// 获取账单
    pub async fn get_invoice(&self, invoice_id: Uuid) -> Option<Invoice> {
        let invoices = self.invoices.lock().await;
        invoices.iter().find(|i| i.id == invoice_id).cloned()
    }

    /// 获取用户的所有账单
    pub async fn get_user_invoices(&self, user_id: Uuid) -> Vec<Invoice> {
        let invoices = self.invoices.lock().await;
        invoices
            .iter()
            .filter(|i| i.user_id == user_id)
            .cloned()
            .collect()
    }

    /// 获取待支付账单
    pub async fn get_pending_invoices(&self, user_id: Uuid) -> Vec<Invoice> {
        let invoices = self.invoices.lock().await;
        invoices
            .iter()
            .filter(|i| i.user_id == user_id && i.status == InvoiceStatus::Pending)
            .cloned()
            .collect()
    }

    /// 标记账单为已支付
    pub async fn mark_invoice_paid(&self, invoice_id: Uuid) -> Result<Invoice, String> {
        let mut invoices = self.invoices.lock().await;

        if let Some(invoice) = invoices.iter_mut().find(|i| i.id == invoice_id) {
            invoice.mark_paid();
            Ok(invoice.clone())
        } else {
            Err("Invoice not found".to_string())
        }
    }

    /// 取消账单
    pub async fn cancel_invoice(&self, invoice_id: Uuid) -> Result<Invoice, String> {
        let mut invoices = self.invoices.lock().await;

        if let Some(invoice) = invoices.iter_mut().find(|i| i.id == invoice_id) {
            if invoice.status == InvoiceStatus::Paid {
                return Err("Cannot cancel paid invoice".to_string());
            }
            invoice.status = InvoiceStatus::Cancelled;
            Ok(invoice.clone())
        } else {
            Err("Invoice not found".to_string())
        }
    }

    /// 更新逾期账单状态
    pub async fn update_overdue_invoices(&self) {
        let mut invoices = self.invoices.lock().await;

        for invoice in invoices.iter_mut() {
            invoice.update_overdue_status();
        }
    }

    /// 获取逾期账单
    pub async fn get_overdue_invoices(&self, user_id: Uuid) -> Vec<Invoice> {
        let invoices = self.invoices.lock().await;
        invoices
            .iter()
            .filter(|i| i.user_id == user_id && i.status == InvoiceStatus::Overdue)
            .cloned()
            .collect()
    }

    /// 计算预估费用
    pub async fn estimate_cost(
        &self,
        usage_type: UsageType,
        quantity: f64,
    ) -> Result<f64, String> {
        let rules = self.rules.lock().await;

        if let Some(rule) = rules.iter().find(|r| r.usage_type == usage_type && r.enabled) {
            Ok(rule.calculate_cost(quantity))
        } else {
            Err("No billing rule found for this usage type".to_string())
        }
    }

    /// 生成月度账单
    pub async fn generate_monthly_invoice(&self, user_id: Uuid, year: i32, month: u32) -> Result<Invoice, String> {
        let period_start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or("Invalid date")?
            .and_hms_opt(0, 0, 0)
            .ok_or("Invalid time")?
            .and_utc();

        let period_end = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or("Invalid date")?
        .and_hms_opt(0, 0, 0)
        .ok_or("Invalid time")?
        .and_utc();

        self.generate_invoice(user_id, period_start, period_end).await
    }
}
