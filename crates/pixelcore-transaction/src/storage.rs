use crate::models::Transaction;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 交易存储
pub struct TransactionStorage {
    conn: Arc<Mutex<Connection>>,
}

impl TransactionStorage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                buyer_id TEXT NOT NULL,
                seller_id TEXT NOT NULL,
                transaction_type TEXT NOT NULL,
                status TEXT NOT NULL,
                amount REAL NOT NULL,
                currency TEXT NOT NULL,
                created_at TEXT NOT NULL,
                confirmed_at TEXT,
                completed_at TEXT,
                result TEXT,
                error TEXT,
                metadata TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn save(&self, transaction: &Transaction) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let transaction_type_json = serde_json::to_string(&transaction.transaction_type)?;
        let result_json = transaction.result.as_ref().map(|r| serde_json::to_string(r).ok()).flatten();
        let metadata_json = serde_json::to_string(&transaction.metadata)?;

        conn.execute(
            "INSERT OR REPLACE INTO transactions VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                transaction.id.to_string(),
                transaction.buyer_id.to_string(),
                transaction.seller_id.to_string(),
                transaction_type_json,
                format!("{:?}", transaction.status),
                transaction.amount,
                transaction.currency,
                transaction.created_at.to_rfc3339(),
                transaction.confirmed_at.map(|t| t.to_rfc3339()),
                transaction.completed_at.map(|t| t.to_rfc3339()),
                result_json,
                transaction.error.clone(),
                metadata_json,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Transaction>> {
        // 简化实现 - 实际应该完整反序列化
        Ok(None)
    }

    pub fn list(&self, offset: usize, limit: usize) -> Result<Vec<Transaction>> {
        Ok(Vec::new())
    }
}
