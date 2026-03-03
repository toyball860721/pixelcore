use crate::models::{DataDeletionRequest, DeletionType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DeletionError {
    #[error("Deletion request not found")]
    RequestNotFound,
    #[error("Deletion failed: {0}")]
    DeletionFailed(String),
}

pub type DeletionResult<T> = Result<T, DeletionError>;

/// 数据删除管理器
#[derive(Debug, Clone)]
pub struct DataDeleter {
    requests: Arc<Mutex<HashMap<Uuid, DataDeletionRequest>>>,
    soft_deleted_users: Arc<Mutex<Vec<Uuid>>>,
}

impl DataDeleter {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            soft_deleted_users: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建删除请求
    pub fn create_deletion_request(
        &self,
        user_id: Uuid,
        deletion_type: DeletionType,
    ) -> DeletionResult<DataDeletionRequest> {
        let request = DataDeletionRequest::new(user_id, deletion_type);
        let mut requests = self.requests.lock().unwrap();
        requests.insert(request.id, request.clone());
        Ok(request)
    }

    /// 获取删除请求
    pub fn get_deletion_request(&self, request_id: Uuid) -> DeletionResult<DataDeletionRequest> {
        let requests = self.requests.lock().unwrap();
        requests
            .get(&request_id)
            .cloned()
            .ok_or(DeletionError::RequestNotFound)
    }

    /// 执行软删除
    pub fn execute_soft_deletion(&self, request_id: Uuid) -> DeletionResult<Vec<String>> {
        let mut requests = self.requests.lock().unwrap();
        let request = requests
            .get_mut(&request_id)
            .ok_or(DeletionError::RequestNotFound)?;

        // 标记用户为已删除
        let mut soft_deleted = self.soft_deleted_users.lock().unwrap();
        soft_deleted.push(request.user_id);

        // 记录删除的记录
        let deleted_records = vec![
            format!("user:{}", request.user_id),
            format!("user_profile:{}", request.user_id),
            format!("user_settings:{}", request.user_id),
        ];

        request.deleted_records = deleted_records.clone();
        request.completed_at = Some(Utc::now());

        Ok(deleted_records)
    }

    /// 执行硬删除
    pub fn execute_hard_deletion(&self, request_id: Uuid) -> DeletionResult<Vec<String>> {
        let mut requests = self.requests.lock().unwrap();
        let request = requests
            .get_mut(&request_id)
            .ok_or(DeletionError::RequestNotFound)?;

        // 在实际应用中，这里会删除数据库中的所有相关记录
        // 这里我们只是模拟删除过程
        let deleted_records = vec![
            format!("user:{}", request.user_id),
            format!("user_profile:{}", request.user_id),
            format!("user_settings:{}", request.user_id),
            format!("user_activities:{}", request.user_id),
            format!("user_consents:{}", request.user_id),
            format!("user_api_keys:{}", request.user_id),
        ];

        request.deleted_records = deleted_records.clone();
        request.completed_at = Some(Utc::now());

        Ok(deleted_records)
    }

    /// 检查用户是否被软删除
    pub fn is_soft_deleted(&self, user_id: Uuid) -> bool {
        let soft_deleted = self.soft_deleted_users.lock().unwrap();
        soft_deleted.contains(&user_id)
    }

    /// 恢复软删除的用户
    pub fn restore_soft_deleted_user(&self, user_id: Uuid) -> DeletionResult<()> {
        let mut soft_deleted = self.soft_deleted_users.lock().unwrap();
        soft_deleted.retain(|&id| id != user_id);
        Ok(())
    }

    /// 获取用户的所有删除请求
    pub fn get_user_deletion_requests(&self, user_id: Uuid) -> Vec<DataDeletionRequest> {
        let requests = self.requests.lock().unwrap();
        requests
            .values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect()
    }

    /// 获取所有软删除的用户
    pub fn get_soft_deleted_users(&self) -> Vec<Uuid> {
        let soft_deleted = self.soft_deleted_users.lock().unwrap();
        soft_deleted.clone()
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> DeletionStatistics {
        let requests = self.requests.lock().unwrap();
        let soft_deleted = self.soft_deleted_users.lock().unwrap();

        let total_requests = requests.len();
        let completed_requests = requests
            .values()
            .filter(|r| r.completed_at.is_some())
            .count();
        let soft_deletions = requests
            .values()
            .filter(|r| r.deletion_type == DeletionType::Soft && r.completed_at.is_some())
            .count();
        let hard_deletions = requests
            .values()
            .filter(|r| r.deletion_type == DeletionType::Hard && r.completed_at.is_some())
            .count();

        DeletionStatistics {
            total_requests,
            completed_requests,
            soft_deletions,
            hard_deletions,
            soft_deleted_users: soft_deleted.len(),
        }
    }
}

impl Default for DataDeleter {
    fn default() -> Self {
        Self::new()
    }
}

/// 删除统计信息
#[derive(Debug, Clone)]
pub struct DeletionStatistics {
    pub total_requests: usize,
    pub completed_requests: usize,
    pub soft_deletions: usize,
    pub hard_deletions: usize,
    pub soft_deleted_users: usize,
}
