use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use crate::error::StorageError;

pub type StorageKey = String;
pub type StorageValue = serde_json::Value;

enum Backend {
    Memory(Arc<RwLock<HashMap<StorageKey, StorageValue>>>),
    Sled(sled::Db),
}

#[derive(Clone)]
pub struct Storage {
    inner: Arc<Backend>,
}

impl Storage {
    /// 内存模式，进程退出后数据丢失（适合测试）
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Backend::Memory(Arc::new(RwLock::new(HashMap::new())))),
        }
    }

    /// 持久化模式，数据写入 sled 数据库文件
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let db = sled::open(path)?;
        Ok(Self {
            inner: Arc::new(Backend::Sled(db)),
        })
    }

    pub fn get(&self, key: &str) -> Result<StorageValue, StorageError> {
        match self.inner.as_ref() {
            Backend::Memory(map) => map
                .read()
                .unwrap()
                .get(key)
                .cloned()
                .ok_or_else(|| StorageError::NotFound(key.to_string())),
            Backend::Sled(db) => {
                let bytes = db
                    .get(key)?
                    .ok_or_else(|| StorageError::NotFound(key.to_string()))?;
                Ok(serde_json::from_slice(&bytes)?)
            }
        }
    }

    pub fn set(&self, key: impl Into<StorageKey>, value: StorageValue) -> Result<(), StorageError> {
        let key = key.into();
        match self.inner.as_ref() {
            Backend::Memory(map) => {
                map.write().unwrap().insert(key, value);
                Ok(())
            }
            Backend::Sled(db) => {
                let bytes = serde_json::to_vec(&value)?;
                db.insert(key, bytes)?;
                Ok(())
            }
        }
    }

    pub fn delete(&self, key: &str) -> Result<bool, StorageError> {
        match self.inner.as_ref() {
            Backend::Memory(map) => Ok(map.write().unwrap().remove(key).is_some()),
            Backend::Sled(db) => Ok(db.remove(key)?.is_some()),
        }
    }

    pub fn contains(&self, key: &str) -> Result<bool, StorageError> {
        match self.inner.as_ref() {
            Backend::Memory(map) => Ok(map.read().unwrap().contains_key(key)),
            Backend::Sled(db) => Ok(db.contains_key(key)?),
        }
    }

    pub fn keys(&self) -> Result<Vec<StorageKey>, StorageError> {
        match self.inner.as_ref() {
            Backend::Memory(map) => Ok(map.read().unwrap().keys().cloned().collect()),
            Backend::Sled(db) => {
                let keys = db
                    .iter()
                    .keys()
                    .filter_map(|r| r.ok())
                    .filter_map(|k| String::from_utf8(k.to_vec()).ok())
                    .collect();
                Ok(keys)
            }
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
