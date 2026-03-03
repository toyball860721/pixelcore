use crate::models::{Account, AccountType, AccountStatus, PaymentTransaction, PaymentType, PaymentStatus};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 账户管理器
#[derive(Clone)]
pub struct AccountManager {
    accounts: Arc<Mutex<Vec<Account>>>,
    transactions: Arc<Mutex<Vec<PaymentTransaction>>>,
}

impl AccountManager {
    /// 创建新的账户管理器
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(Mutex::new(Vec::new())),
            transactions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建账户
    pub async fn create_account(
        &self,
        owner_id: Uuid,
        account_type: AccountType,
    ) -> Result<Account, String> {
        let account = Account::new(owner_id, account_type);
        let mut accounts = self.accounts.lock().await;
        accounts.push(account.clone());
        Ok(account)
    }

    /// 获取账户
    pub async fn get_account(&self, account_id: Uuid) -> Result<Account, String> {
        let accounts = self.accounts.lock().await;
        accounts
            .iter()
            .find(|a| a.id == account_id)
            .cloned()
            .ok_or_else(|| "Account not found".to_string())
    }

    /// 根据所有者获取账户
    pub async fn get_account_by_owner(&self, owner_id: Uuid) -> Result<Account, String> {
        let accounts = self.accounts.lock().await;
        accounts
            .iter()
            .find(|a| a.owner_id == owner_id)
            .cloned()
            .ok_or_else(|| "Account not found".to_string())
    }

    /// 查询余额
    pub async fn get_balance(&self, account_id: Uuid) -> Result<f64, String> {
        let account = self.get_account(account_id).await?;
        Ok(account.balance)
    }

    /// 查询可用余额
    pub async fn get_available_balance(&self, account_id: Uuid) -> Result<f64, String> {
        let account = self.get_account(account_id).await?;
        Ok(account.available_balance())
    }

    /// 转账
    pub async fn transfer(
        &self,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: f64,
        description: String,
    ) -> Result<PaymentTransaction, String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        let mut accounts = self.accounts.lock().await;

        // 查找源账户和目标账户
        let from_idx = accounts
            .iter()
            .position(|a| a.id == from_account_id)
            .ok_or_else(|| "Source account not found".to_string())?;

        let to_idx = accounts
            .iter()
            .position(|a| a.id == to_account_id)
            .ok_or_else(|| "Destination account not found".to_string())?;

        // 检查源账户余额
        if !accounts[from_idx].can_debit(amount) {
            return Err("Insufficient balance".to_string());
        }

        // 执行转账
        accounts[from_idx].debit(amount)?;
        accounts[to_idx].credit(amount)?;

        // 创建交易记录
        let mut transaction = PaymentTransaction::new(
            PaymentType::Transfer,
            Some(from_account_id),
            Some(to_account_id),
            amount,
            description,
        );
        transaction.mark_success();

        // 保存交易记录
        let mut transactions = self.transactions.lock().await;
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    /// 充值
    pub async fn deposit(
        &self,
        account_id: Uuid,
        amount: f64,
        description: String,
    ) -> Result<PaymentTransaction, String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        let mut accounts = self.accounts.lock().await;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| "Account not found".to_string())?;

        account.credit(amount)?;

        // 创建交易记录
        let mut transaction = PaymentTransaction::new(
            PaymentType::Deposit,
            None,
            Some(account_id),
            amount,
            description,
        );
        transaction.mark_success();

        // 保存交易记录
        let mut transactions = self.transactions.lock().await;
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    /// 提现
    pub async fn withdraw(
        &self,
        account_id: Uuid,
        amount: f64,
        description: String,
    ) -> Result<PaymentTransaction, String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        let mut accounts = self.accounts.lock().await;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| "Account not found".to_string())?;

        if !account.can_debit(amount) {
            return Err("Insufficient balance".to_string());
        }

        account.debit(amount)?;

        // 创建交易记录
        let mut transaction = PaymentTransaction::new(
            PaymentType::Withdrawal,
            Some(account_id),
            None,
            amount,
            description,
        );
        transaction.mark_success();

        // 保存交易记录
        let mut transactions = self.transactions.lock().await;
        transactions.push(transaction.clone());

        Ok(transaction)
    }

    /// 冻结账户
    pub async fn freeze_account(&self, account_id: Uuid) -> Result<(), String> {
        let mut accounts = self.accounts.lock().await;
        let account = accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| "Account not found".to_string())?;

        account.status = AccountStatus::Frozen;
        Ok(())
    }

    /// 解冻账户
    pub async fn unfreeze_account(&self, account_id: Uuid) -> Result<(), String> {
        let mut accounts = self.accounts.lock().await;
        let account = accounts
            .iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| "Account not found".to_string())?;

        account.status = AccountStatus::Active;
        Ok(())
    }

    /// 获取交易历史
    pub async fn get_transaction_history(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<PaymentTransaction>, String> {
        let transactions = self.transactions.lock().await;
        let history: Vec<PaymentTransaction> = transactions
            .iter()
            .filter(|t| {
                t.from_account == Some(account_id) || t.to_account == Some(account_id)
            })
            .cloned()
            .collect();
        Ok(history)
    }

    /// 获取所有账户
    pub async fn list_accounts(&self) -> Vec<Account> {
        let accounts = self.accounts.lock().await;
        accounts.clone()
    }
}
