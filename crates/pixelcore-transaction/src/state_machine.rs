use crate::models::{Transaction, TransactionStatus};
use anyhow::{anyhow, Result};

/// 交易状态机
pub struct TransactionStateMachine;

impl TransactionStateMachine {
    /// 验证状态转换是否合法
    pub fn can_transition(from: TransactionStatus, to: TransactionStatus) -> bool {
        use TransactionStatus::*;

        match (from, to) {
            // Pending 可以转到 Negotiating, Confirmed, Cancelled
            (Pending, Negotiating) => true,
            (Pending, Confirmed) => true,
            (Pending, Cancelled) => true,

            // Negotiating 可以转到 Confirmed, Cancelled
            (Negotiating, Confirmed) => true,
            (Negotiating, Cancelled) => true,

            // Confirmed 可以转到 Executing
            (Confirmed, Executing) => true,
            (Confirmed, Cancelled) => true,

            // Executing 可以转到 Completed, Failed, Disputed
            (Executing, Completed) => true,
            (Executing, Failed) => true,
            (Executing, Disputed) => true,

            // Disputed 可以转到 Completed, Failed, Cancelled
            (Disputed, Completed) => true,
            (Disputed, Failed) => true,
            (Disputed, Cancelled) => true,

            // 终态不能转换
            (Completed, _) => false,
            (Failed, _) => false,
            (Cancelled, _) => false,

            // 其他转换不合法
            _ => false,
        }
    }

    /// 执行状态转换
    pub fn transition(transaction: &mut Transaction, to: TransactionStatus) -> Result<()> {
        if !Self::can_transition(transaction.status, to) {
            return Err(anyhow!(
                "Invalid state transition: {:?} -> {:?}",
                transaction.status,
                to
            ));
        }

        transaction.status = to;
        Ok(())
    }

    /// 获取下一个可能的状态
    pub fn next_states(status: TransactionStatus) -> Vec<TransactionStatus> {
        use TransactionStatus::*;

        match status {
            Pending => vec![Negotiating, Confirmed, Cancelled],
            Negotiating => vec![Confirmed, Cancelled],
            Confirmed => vec![Executing, Cancelled],
            Executing => vec![Completed, Failed, Disputed],
            Disputed => vec![Completed, Failed, Cancelled],
            Completed | Failed | Cancelled => vec![],
        }
    }
}
