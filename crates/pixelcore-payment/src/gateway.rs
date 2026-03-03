use crate::models::{PaymentTransaction, PaymentType, PaymentStatus};
use crate::account::AccountManager;
use uuid::Uuid;

/// 支付网关配置
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// 充值手续费率 (0.0 - 1.0)
    pub deposit_fee_rate: f64,
    /// 提现手续费率
    pub withdrawal_fee_rate: f64,
    /// 转账手续费率
    pub transfer_fee_rate: f64,
    /// 最小充值金额
    pub min_deposit: f64,
    /// 最小提现金额
    pub min_withdrawal: f64,
    /// 最大单笔交易金额
    pub max_transaction: f64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            deposit_fee_rate: 0.0,      // 充值免手续费
            withdrawal_fee_rate: 0.01,  // 提现 1% 手续费
            transfer_fee_rate: 0.005,   // 转账 0.5% 手续费
            min_deposit: 1.0,
            min_withdrawal: 10.0,
            max_transaction: 1000000.0,
        }
    }
}

/// 支付网关
pub struct PaymentGateway {
    account_manager: AccountManager,
    config: GatewayConfig,
}

impl PaymentGateway {
    /// 创建新的支付网关
    pub fn new(account_manager: AccountManager, config: GatewayConfig) -> Self {
        Self {
            account_manager,
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(account_manager: AccountManager) -> Self {
        Self::new(account_manager, GatewayConfig::default())
    }

    /// 计算手续费
    fn calculate_fee(&self, payment_type: &PaymentType, amount: f64) -> f64 {
        let rate = match payment_type {
            PaymentType::Deposit => self.config.deposit_fee_rate,
            PaymentType::Withdrawal => self.config.withdrawal_fee_rate,
            PaymentType::Transfer => self.config.transfer_fee_rate,
            _ => 0.0,
        };
        amount * rate
    }

    /// 验证交易金额
    fn validate_amount(&self, payment_type: &PaymentType, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        if amount > self.config.max_transaction {
            return Err(format!(
                "Amount exceeds maximum transaction limit of {}",
                self.config.max_transaction
            ));
        }

        match payment_type {
            PaymentType::Deposit => {
                if amount < self.config.min_deposit {
                    return Err(format!(
                        "Deposit amount must be at least {}",
                        self.config.min_deposit
                    ));
                }
            }
            PaymentType::Withdrawal => {
                if amount < self.config.min_withdrawal {
                    return Err(format!(
                        "Withdrawal amount must be at least {}",
                        self.config.min_withdrawal
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 充值
    pub async fn deposit(
        &self,
        account_id: Uuid,
        amount: f64,
    ) -> Result<PaymentTransaction, String> {
        self.validate_amount(&PaymentType::Deposit, amount)?;

        let fee = self.calculate_fee(&PaymentType::Deposit, amount);
        let net_amount = amount - fee;

        let mut transaction = self
            .account_manager
            .deposit(account_id, net_amount, format!("Deposit {} PixelCoin", amount))
            .await?;

        transaction.fee = fee;
        Ok(transaction)
    }

    /// 提现
    pub async fn withdraw(
        &self,
        account_id: Uuid,
        amount: f64,
    ) -> Result<PaymentTransaction, String> {
        self.validate_amount(&PaymentType::Withdrawal, amount)?;

        let fee = self.calculate_fee(&PaymentType::Withdrawal, amount);
        let total_amount = amount + fee;

        // 检查余额是否足够支付金额+手续费
        let balance = self.account_manager.get_available_balance(account_id).await?;
        if balance < total_amount {
            return Err("Insufficient balance to cover withdrawal and fee".to_string());
        }

        let mut transaction = self
            .account_manager
            .withdraw(account_id, total_amount, format!("Withdraw {} PixelCoin", amount))
            .await?;

        transaction.fee = fee;
        Ok(transaction)
    }

    /// 转账
    pub async fn transfer(
        &self,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: f64,
    ) -> Result<PaymentTransaction, String> {
        self.validate_amount(&PaymentType::Transfer, amount)?;

        let fee = self.calculate_fee(&PaymentType::Transfer, amount);
        let total_amount = amount + fee;

        // 检查余额
        let balance = self.account_manager.get_available_balance(from_account_id).await?;
        if balance < total_amount {
            return Err("Insufficient balance to cover transfer and fee".to_string());
        }

        // 先扣除手续费
        if fee > 0.0 {
            self.account_manager
                .withdraw(from_account_id, fee, "Transfer fee".to_string())
                .await?;
        }

        // 执行转账
        let mut transaction = self
            .account_manager
            .transfer(
                from_account_id,
                to_account_id,
                amount,
                format!("Transfer {} PixelCoin", amount),
            )
            .await?;

        transaction.fee = fee;
        Ok(transaction)
    }

    /// 支付 (用于服务购买等)
    pub async fn pay(
        &self,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: f64,
        description: String,
    ) -> Result<PaymentTransaction, String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        let transaction = self
            .account_manager
            .transfer(from_account_id, to_account_id, amount, description)
            .await?;

        Ok(transaction)
    }

    /// 退款
    pub async fn refund(
        &self,
        original_transaction_id: Uuid,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: f64,
    ) -> Result<PaymentTransaction, String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        let mut transaction = self
            .account_manager
            .transfer(
                from_account_id,
                to_account_id,
                amount,
                format!("Refund {} PixelCoin", amount),
            )
            .await?;

        transaction.payment_type = PaymentType::Refund;
        transaction.related_transaction = Some(original_transaction_id);

        Ok(transaction)
    }

    /// 获取网关配置
    pub fn get_config(&self) -> &GatewayConfig {
        &self.config
    }
}
