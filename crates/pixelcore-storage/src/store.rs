use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::error::StorageError;

pub type StorageKey = String;
pub type StorageValue = serde_json::Value;

#[derive(Clone, Default)]
pub struct Storage {
    inner: Arc<RwLock<HashMap<StorageKey, StorageValue>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Result<StorageValue, StorageError> {
        self.inner
            .read()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(key.to_string()))
    }

    pub fn set(&self, key: impl Into<StorageKey>, value: StorageValue) {
        self.inner.write().unwrap().insert(key.into(), value);
    }

    pub fn delete(&self, key: &str) -> bool {
        self.inner.write().unwrap().remove(key).is_some()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.inner.read().unwrap().contains_key(key)
    }

    pub fn keys(&self) -> Vec<StorageKey> {
        self.inner.read().unwrap().keys().cloned().collect()
    }
}
