use crate::models::{Transaction, TransactionStatus, TransactionStats};
use crate::state_machine::TransactionStateMachine;
use crate::storage::TransactionStorage;
use anyhow::Result;
use std::path::Path;
use uuid::Uuid;

pub struct TransactionManager {
    storage: TransactionStorage,
}

impl TransactionManager {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        Ok(Self {
            storage: TransactionStorage::new(db_path)?,
        })
    }

    pub fn in_memory() -> Result<Self> {
        Ok(Self {
            storage: TransactionStorage::in_memory()?,
        })
    }

    pub fn create_transaction(&self, transaction: Transaction) -> Result<Uuid> {
        let id = transaction.id;
        self.storage.save(&transaction)?;
        Ok(id)
    }

    pub fn confirm_transaction(&self, id: &Uuid) -> Result<()> {
        // 简化实现
        Ok(())
    }

    pub fn execute_transaction(&self, id: &Uuid) -> Result<()> {
        Ok(())
    }

    pub fn complete_transaction(&self, id: &Uuid, result: serde_json::Value) -> Result<()> {
        Ok(())
    }

    pub fn get_transaction(&self, id: &Uuid) -> Result<Option<Transaction>> {
        self.storage.get(id)
    }

    pub fn get_stats(&self) -> Result<TransactionStats> {
        let transactions = self.storage.list(0, 10000)?;
        Ok(TransactionStats::from_transactions(&transactions))
    }
}
