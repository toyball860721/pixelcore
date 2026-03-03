use crate::models::EncryptionKey;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum KeyManagerError {
    #[error("Key not found")]
    KeyNotFound,
    #[error("No active key available")]
    NoActiveKey,
}

pub type KeyManagerResult<T> = Result<T, KeyManagerError>;

/// 密钥管理器
#[derive(Debug, Clone)]
pub struct KeyManager {
    keys: Arc<Mutex<HashMap<Uuid, EncryptionKey>>>,
    active_key_id: Arc<Mutex<Option<Uuid>>>,
    rotation_interval_days: i64,
}

impl KeyManager {
    pub fn new(rotation_interval_days: i64) -> Self {
        Self {
            keys: Arc::new(Mutex::new(HashMap::new())),
            active_key_id: Arc::new(Mutex::new(None)),
            rotation_interval_days,
        }
    }

    /// 生成新的加密密钥
    pub fn generate_key(&self) -> KeyManagerResult<Uuid> {
        let key = EncryptionKey::new_aes256();
        let key_id = key.id;

        let mut keys = self.keys.lock().unwrap();
        keys.insert(key_id, key);

        // 如果没有活跃密钥，设置为活跃密钥
        let mut active_key_id = self.active_key_id.lock().unwrap();
        if active_key_id.is_none() {
            *active_key_id = Some(key_id);
        }

        Ok(key_id)
    }

    /// 获取活跃的加密密钥
    pub fn get_active_key(&self) -> KeyManagerResult<EncryptionKey> {
        let active_key_id = self.active_key_id.lock().unwrap();
        let key_id = active_key_id.ok_or(KeyManagerError::NoActiveKey)?;

        let keys = self.keys.lock().unwrap();
        keys.get(&key_id)
            .cloned()
            .ok_or(KeyManagerError::KeyNotFound)
    }

    /// 获取指定的加密密钥
    pub fn get_key(&self, key_id: Uuid) -> KeyManagerResult<EncryptionKey> {
        let keys = self.keys.lock().unwrap();
        keys.get(&key_id)
            .cloned()
            .ok_or(KeyManagerError::KeyNotFound)
    }

    /// 轮换密钥
    pub fn rotate_key(&self) -> KeyManagerResult<Uuid> {
        // 生成新密钥
        let new_key_id = self.generate_key()?;

        // 更新旧密钥的轮换时间
        let mut keys = self.keys.lock().unwrap();
        let mut active_key_id = self.active_key_id.lock().unwrap();

        if let Some(old_key_id) = *active_key_id {
            if let Some(old_key) = keys.get_mut(&old_key_id) {
                old_key.rotated_at = Some(Utc::now());
            }
        }

        // 设置新密钥为活跃密钥
        *active_key_id = Some(new_key_id);

        Ok(new_key_id)
    }

    /// 检查是否需要轮换密钥
    pub fn should_rotate(&self) -> bool {
        let active_key = match self.get_active_key() {
            Ok(key) => key,
            Err(_) => return true, // 没有活跃密钥，需要生成
        };

        let age = Utc::now() - active_key.created_at;
        age > Duration::days(self.rotation_interval_days)
    }

    /// 自动轮换密钥（如果需要）
    pub fn auto_rotate(&self) -> KeyManagerResult<Option<Uuid>> {
        if self.should_rotate() {
            Ok(Some(self.rotate_key()?))
        } else {
            Ok(None)
        }
    }

    /// 列出所有密钥
    pub fn list_keys(&self) -> Vec<EncryptionKey> {
        let keys = self.keys.lock().unwrap();
        keys.values().cloned().collect()
    }

    /// 删除旧密钥（保留最近的 N 个）
    pub fn cleanup_old_keys(&self, keep_count: usize) {
        let mut keys = self.keys.lock().unwrap();
        let active_key_id = self.active_key_id.lock().unwrap();

        // 按创建时间排序
        let mut key_list: Vec<_> = keys.iter().collect();
        key_list.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        // 保留活跃密钥和最近的 N 个密钥（使用 HashSet 去重）
        let mut keys_to_keep = std::collections::HashSet::new();

        // 添加最近的 N 个密钥
        for (id, _) in key_list.iter().take(keep_count) {
            keys_to_keep.insert(**id);
        }

        // 添加活跃密钥
        if let Some(active_id) = *active_key_id {
            keys_to_keep.insert(active_id);
        }

        keys.retain(|id, _| keys_to_keep.contains(id));
    }

    /// 获取密钥统计信息
    pub fn get_stats(&self) -> KeyStats {
        let keys = self.keys.lock().unwrap();
        let active_key_id = self.active_key_id.lock().unwrap();

        let active_key = active_key_id
            .and_then(|id| keys.get(&id))
            .cloned();

        KeyStats {
            total_keys: keys.len(),
            active_key_id: *active_key_id,
            active_key_age_days: active_key.map(|key| {
                (Utc::now() - key.created_at).num_days()
            }),
            should_rotate: self.should_rotate(),
        }
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new(90) // 默认 90 天轮换一次
    }
}

/// 密钥统计信息
#[derive(Debug, Clone)]
pub struct KeyStats {
    pub total_keys: usize,
    pub active_key_id: Option<Uuid>,
    pub active_key_age_days: Option<i64>,
    pub should_rotate: bool,
}
