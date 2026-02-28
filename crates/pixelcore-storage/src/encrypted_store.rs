use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::error::StorageError;
use crate::store::{StorageKey, StorageValue};

#[derive(Clone)]
pub struct EncryptedStore {
    conn: Arc<Mutex<Connection>>,
}

impl EncryptedStore {
    /// 创建或打开加密的 SQLite 数据库
    ///
    /// # Arguments
    /// * `path` - 数据库文件路径
    /// * `key` - 加密密钥（建议使用 32 字节的强密钥）
    pub fn open(path: impl AsRef<Path>, key: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;

        // 设置 SQLCipher 加密密钥
        conn.pragma_update(None, "key", key)?;

        // 创建键值对表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // 创建索引以提高查询性能
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_updated_at ON kv_store(updated_at)",
            [],
        )?;

        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    pub fn get(&self, key: &str) -> Result<StorageValue, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM kv_store WHERE key = ?")?;
        let value: String = stmt.query_row(params![key], |row| row.get(0))
            .map_err(|_| StorageError::NotFound(key.to_string()))?;

        Ok(serde_json::from_str(&value)?)
    }

    pub fn set(&self, key: &str, value: &StorageValue) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let value_str = serde_json::to_string(value)?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO kv_store (key, value, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at",
            params![key, value_str, now],
        )?;

        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM kv_store WHERE key = ?", params![key])?;
        Ok(affected > 0)
    }

    pub fn contains(&self, key: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT 1 FROM kv_store WHERE key = ?")?;
        let exists = stmt.exists(params![key])?;
        Ok(exists)
    }

    pub fn keys(&self) -> Result<Vec<StorageKey>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key FROM kv_store")?;
        let keys = stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(keys)
    }

    pub fn clear(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM kv_store", [])?;
        Ok(())
    }
}

