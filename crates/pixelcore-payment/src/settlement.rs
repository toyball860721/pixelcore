use crate::models::{Settlement, SettlementType, SettlementStatus};
use crate::account::AccountManager;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 结算管理器
pub struct SettlementManager {
    account_manager: AccountManager,
    settlements: Arc<Mutex<Vec<Settlement>>>,
}

impl SettlementManager {
    /// 创建新的结算管理器
    pub fn new(account_manager: AccountManager) -> Self {
        Self {
            account_manager,
            settlements: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建即时结算
    pub async fn create_immediate_settlement(
        &self,
        transaction_id: Uuid,
        seller_account: Uuid,
        buyer_account: Uuid,
        amount: f64,
    ) -> Result<Settlement, String> {
        // 立即执行转账
        self.account_manager
            .transfer(
                buyer_account,
                seller_account,
                amount,
                format!("Settlement for transaction {}", transaction_id),
            )
            .await?;

        // 创建结算记录
        let mut settlement = Settlement::new(
            transaction_id,
            seller_account,
            buyer_account,
            amount,
            SettlementType::Immediate,
        );
        settlement.mark_settled();

        // 保存结算记录
        let mut settlements = self.settlements.lock().await;
        settlements.push(settlement.clone());

        Ok(settlement)
    }

    /// 创建延迟结算
    pub async fn create_delayed_settlement(
        &self,
        transaction_id: Uuid,
        seller_account: Uuid,
        buyer_account: Uuid,
        amount: f64,
        delay_days: i64,
    ) -> Result<Settlement, String> {
        // 冻结买方资金
        let mut accounts = vec![];
        {
            let account = self.account_manager.get_account(buyer_account).await?;
            accounts.push(account);
        }

        // 创建延迟结算记录
        let mut settlement = Settlement::new(
            transaction_id,
            seller_account,
            buyer_account,
            amount,
            SettlementType::Delayed,
        );
        settlement.scheduled_at = Some(Utc::now() + Duration::days(delay_days));

        // 保存结算记录
        let mut settlements = self.settlements.lock().await;
        settlements.push(settlement.clone());

        Ok(settlement)
    }

    /// 创建托管结算
    pub async fn create_escrow_settlement(
        &self,
        transaction_id: Uuid,
        seller_account: Uuid,
        buyer_account: Uuid,
        escrow_account: Uuid,
        amount: f64,
    ) -> Result<Settlement, String> {
        // 将资金转入托管账户
        self.account_manager
            .transfer(
                buyer_account,
                escrow_account,
                amount,
                format!("Escrow for transaction {}", transaction_id),
            )
            .await?;

        // 创建托管结算记录
        let settlement = Settlement::new(
            transaction_id,
            seller_account,
            buyer_account,
            amount,
            SettlementType::Escrow,
        );

        // 保存结算记录
        let mut settlements = self.settlements.lock().await;
        settlements.push(settlement.clone());

        Ok(settlement)
    }

    /// 执行托管结算 (释放资金给卖方)
    pub async fn release_escrow(
        &self,
        settlement_id: Uuid,
        escrow_account: Uuid,
    ) -> Result<Settlement, String> {
        let mut settlements = self.settlements.lock().await;

        let settlement = settlements
            .iter_mut()
            .find(|s| s.id == settlement_id)
            .ok_or_else(|| "Settlement not found".to_string())?;

        if settlement.settlement_type != SettlementType::Escrow {
            return Err("Not an escrow settlement".to_string());
        }

        if settlement.status != SettlementStatus::Pending {
            return Err("Settlement already processed".to_string());
        }

        // 从托管账户转给卖方
        self.account_manager
            .transfer(
                escrow_account,
                settlement.seller_account,
                settlement.amount,
                format!("Escrow release for transaction {}", settlement.transaction_id),
            )
            .await?;

        settlement.mark_settled();

        Ok(settlement.clone())
    }

    /// 取消托管结算 (退款给买方)
    pub async fn cancel_escrow(
        &self,
        settlement_id: Uuid,
        escrow_account: Uuid,
    ) -> Result<Settlement, String> {
        let mut settlements = self.settlements.lock().await;

        let settlement = settlements
            .iter_mut()
            .find(|s| s.id == settlement_id)
            .ok_or_else(|| "Settlement not found".to_string())?;

        if settlement.settlement_type != SettlementType::Escrow {
            return Err("Not an escrow settlement".to_string());
        }

        if settlement.status != SettlementStatus::Pending {
            return Err("Settlement already processed".to_string());
        }

        // 从托管账户退款给买方
        self.account_manager
            .transfer(
                escrow_account,
                settlement.buyer_account,
                settlement.amount,
                format!("Escrow refund for transaction {}", settlement.transaction_id),
            )
            .await?;

        settlement.status = SettlementStatus::Cancelled;

        Ok(settlement.clone())
    }

    /// 执行延迟结算
    pub async fn execute_delayed_settlement(
        &self,
        settlement_id: Uuid,
    ) -> Result<Settlement, String> {
        let mut settlements = self.settlements.lock().await;

        let settlement = settlements
            .iter_mut()
            .find(|s| s.id == settlement_id)
            .ok_or_else(|| "Settlement not found".to_string())?;

        if settlement.settlement_type != SettlementType::Delayed {
            return Err("Not a delayed settlement".to_string());
        }

        if settlement.status != SettlementStatus::Pending {
            return Err("Settlement already processed".to_string());
        }

        // 检查是否到达预定时间
        if let Some(scheduled_at) = settlement.scheduled_at {
            if Utc::now() < scheduled_at {
                return Err("Settlement not yet due".to_string());
            }
        }

        // 执行转账
        self.account_manager
            .transfer(
                settlement.buyer_account,
                settlement.seller_account,
                settlement.amount,
                format!("Delayed settlement for transaction {}", settlement.transaction_id),
            )
            .await?;

        settlement.mark_settled();

        Ok(settlement.clone())
    }

    /// 获取待处理的结算
    pub async fn get_pending_settlements(&self) -> Vec<Settlement> {
        let settlements = self.settlements.lock().await;
        settlements
            .iter()
            .filter(|s| s.status == SettlementStatus::Pending)
            .cloned()
            .collect()
    }

    /// 获取到期的延迟结算
    pub async fn get_due_settlements(&self) -> Vec<Settlement> {
        let settlements = self.settlements.lock().await;
        let now = Utc::now();

        settlements
            .iter()
            .filter(|s| {
                s.status == SettlementStatus::Pending
                    && s.settlement_type == SettlementType::Delayed
                    && s.scheduled_at.map_or(false, |t| t <= now)
            })
            .cloned()
            .collect()
    }

    /// 获取结算记录
    pub async fn get_settlement(&self, settlement_id: Uuid) -> Result<Settlement, String> {
        let settlements = self.settlements.lock().await;
        settlements
            .iter()
            .find(|s| s.id == settlement_id)
            .cloned()
            .ok_or_else(|| "Settlement not found".to_string())
    }

    /// 分账 (将金额按比例分配给多个账户)
    pub async fn split_payment(
        &self,
        transaction_id: Uuid,
        from_account: Uuid,
        splits: Vec<(Uuid, f64)>, // (账户ID, 比例 0.0-1.0)
        total_amount: f64,
    ) -> Result<Vec<Settlement>, String> {
        // 验证比例总和为 1.0
        let total_ratio: f64 = splits.iter().map(|(_, ratio)| ratio).sum();
        if (total_ratio - 1.0).abs() > 0.001 {
            return Err("Split ratios must sum to 1.0".to_string());
        }

        let mut settlements = Vec::new();

        for (to_account, ratio) in splits {
            let amount = total_amount * ratio;

            // 执行转账
            self.account_manager
                .transfer(
                    from_account,
                    to_account,
                    amount,
                    format!("Split payment for transaction {}", transaction_id),
                )
                .await?;

            // 创建结算记录
            let mut settlement = Settlement::new(
                transaction_id,
                to_account,
                from_account,
                amount,
                SettlementType::Immediate,
            );
            settlement.mark_settled();

            settlements.push(settlement);
        }

        // 保存所有结算记录
        let mut all_settlements = self.settlements.lock().await;
        all_settlements.extend(settlements.clone());

        Ok(settlements)
    }
}
