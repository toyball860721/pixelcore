use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use crate::error::StorageError;
use crate::encrypted_store::EncryptedStore;

pub type StorageKey = String;
pub type StorageValue = serde_json::Value;

enum Backend {
    Memory(Arc<RwLock<HashMap<StorageKey, StorageValue>>>),
    Sled(sled::Db),
    Encrypted(EncryptedStore),
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

    /// 加密持久化模式，使用 SQLCipher 加密存储
    ///
    /// # Arguments
    /// * `path` - 数据库文件路径
    /// * `key` - 加密密钥（建议使用 32 字节的强密钥）
    pub fn open_encrypted(path: impl AsRef<Path>, key: &str) -> Result<Self, StorageError> {
        let store = EncryptedStore::open(path, key)?;
        Ok(Self {
            inner: Arc::new(Backend::Encrypted(store)),
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
            Backend::Encrypted(store) => store.get(key),
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
            Backend::Encrypted(store) => store.set(&key, &value),
        }
    }

    pub fn delete(&self, key: &str) -> Result<bool, StorageError> {
        match self.inner.as_ref() {
            Backend::Memory(map) => Ok(map.write().unwrap().remove(key).is_some()),
            Backend::Sled(db) => Ok(db.remove(key)?.is_some()),
            Backend::Encrypted(store) => store.delete(key),
        }
    }

    pub fn contains(&self, key: &str) -> Result<bool, StorageError> {
        match self.inner.as_ref() {
            Backend::Memory(map) => Ok(map.read().unwrap().contains_key(key)),
            Backend::Sled(db) => Ok(db.contains_key(key)?),
            Backend::Encrypted(store) => store.contains(key),
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
            Backend::Encrypted(store) => store.keys(),
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
