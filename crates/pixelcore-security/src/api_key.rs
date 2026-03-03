use crate::models::ApiKey;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ApiKeyError {
    #[error("API key not found")]
    NotFound,
    #[error("API key expired")]
    Expired,
    #[error("API key inactive")]
    Inactive,
    #[error("Invalid API key")]
    Invalid,
    #[error("Insufficient scopes")]
    InsufficientScopes,
}

pub type ApiKeyResult<T> = Result<T, ApiKeyError>;

/// API Key 管理器
#[derive(Debug, Clone)]
pub struct ApiKeyManager {
    keys: Arc<Mutex<HashMap<String, ApiKey>>>,
    keys_by_user: Arc<Mutex<HashMap<Uuid, Vec<String>>>>,
}

impl ApiKeyManager {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(Mutex::new(HashMap::new())),
            keys_by_user: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建新的 API Key
    pub fn create_key(
        &self,
        user_id: Uuid,
        name: String,
        scopes: Vec<String>,
    ) -> ApiKeyResult<ApiKey> {
        let api_key = ApiKey::new(user_id, name, scopes);

        let mut keys = self.keys.lock().unwrap();
        let mut keys_by_user = self.keys_by_user.lock().unwrap();

        keys.insert(api_key.key.clone(), api_key.clone());
        keys_by_user
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(api_key.key.clone());

        Ok(api_key)
    }

    /// 验证 API Key
    pub fn verify_key(&self, key: &str) -> ApiKeyResult<ApiKey> {
        let mut keys = self.keys.lock().unwrap();

        let api_key = keys
            .get_mut(key)
            .ok_or(ApiKeyError::NotFound)?;

        if !api_key.is_active {
            return Err(ApiKeyError::Inactive);
        }

        if api_key.is_expired() {
            return Err(ApiKeyError::Expired);
        }

        // 更新最后使用时间
        api_key.last_used_at = Some(Utc::now());

        Ok(api_key.clone())
    }

    /// 检查 API Key 是否有指定的 scope
    pub fn check_scope(&self, key: &str, required_scope: &str) -> ApiKeyResult<()> {
        let api_key = self.verify_key(key)?;

        if api_key.scopes.contains(&required_scope.to_string())
            || api_key.scopes.contains(&"*".to_string())
        {
            Ok(())
        } else {
            Err(ApiKeyError::InsufficientScopes)
        }
    }

    /// 撤销 API Key
    pub fn revoke_key(&self, key: &str) -> ApiKeyResult<()> {
        let mut keys = self.keys.lock().unwrap();

        let api_key = keys
            .get_mut(key)
            .ok_or(ApiKeyError::NotFound)?;

        api_key.is_active = false;

        Ok(())
    }

    /// 删除 API Key
    pub fn delete_key(&self, key: &str) -> ApiKeyResult<()> {
        let mut keys = self.keys.lock().unwrap();
        let mut keys_by_user = self.keys_by_user.lock().unwrap();

        let api_key = keys.remove(key).ok_or(ApiKeyError::NotFound)?;

        if let Some(user_keys) = keys_by_user.get_mut(&api_key.user_id) {
            user_keys.retain(|k| k != key);
        }

        Ok(())
    }

    /// 获取用户的所有 API Keys
    pub fn get_user_keys(&self, user_id: Uuid) -> Vec<ApiKey> {
        let keys = self.keys.lock().unwrap();
        let keys_by_user = self.keys_by_user.lock().unwrap();

        keys_by_user
            .get(&user_id)
            .map(|user_keys| {
                user_keys
                    .iter()
                    .filter_map(|key| keys.get(key).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取 API Key 详情
    pub fn get_key(&self, key: &str) -> ApiKeyResult<ApiKey> {
        let keys = self.keys.lock().unwrap();
        keys.get(key).cloned().ok_or(ApiKeyError::NotFound)
    }

    /// 更新 API Key 的 scopes
    pub fn update_scopes(&self, key: &str, scopes: Vec<String>) -> ApiKeyResult<()> {
        let mut keys = self.keys.lock().unwrap();

        let api_key = keys
            .get_mut(key)
            .ok_or(ApiKeyError::NotFound)?;

        api_key.scopes = scopes;

        Ok(())
    }

    /// 清理过期的 API Keys
    pub fn cleanup_expired(&self) {
        let mut keys = self.keys.lock().unwrap();
        let mut keys_by_user = self.keys_by_user.lock().unwrap();

        let expired_keys: Vec<String> = keys
            .iter()
            .filter(|(_, api_key)| api_key.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            if let Some(api_key) = keys.remove(&key) {
                if let Some(user_keys) = keys_by_user.get_mut(&api_key.user_id) {
                    user_keys.retain(|k| k != &key);
                }
            }
        }
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}
