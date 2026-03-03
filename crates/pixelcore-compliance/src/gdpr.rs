use crate::models::{ConsentRecord, DataSubjectRequest, DataSubjectRight, RequestStatus, RetentionPolicy};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GdprError {
    #[error("Request not found")]
    RequestNotFound,
    #[error("Consent not found")]
    ConsentNotFound,
    #[error("Policy not found")]
    PolicyNotFound,
    #[error("Invalid request")]
    InvalidRequest,
}

pub type GdprResult<T> = Result<T, GdprError>;

/// GDPR 合规管理器
#[derive(Debug, Clone)]
pub struct GdprManager {
    requests: Arc<Mutex<HashMap<Uuid, DataSubjectRequest>>>,
    consents: Arc<Mutex<HashMap<Uuid, Vec<ConsentRecord>>>>,
    policies: Arc<Mutex<HashMap<String, RetentionPolicy>>>,
}

impl GdprManager {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            consents: Arc::new(Mutex::new(HashMap::new())),
            policies: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建数据主体请求
    pub fn create_request(
        &self,
        user_id: Uuid,
        right: DataSubjectRight,
    ) -> GdprResult<DataSubjectRequest> {
        let request = DataSubjectRequest::new(user_id, right);
        let mut requests = self.requests.lock().unwrap();
        requests.insert(request.id, request.clone());
        Ok(request)
    }

    /// 获取请求
    pub fn get_request(&self, request_id: Uuid) -> GdprResult<DataSubjectRequest> {
        let requests = self.requests.lock().unwrap();
        requests
            .get(&request_id)
            .cloned()
            .ok_or(GdprError::RequestNotFound)
    }

    /// 更新请求状态
    pub fn update_request_status(
        &self,
        request_id: Uuid,
        status: RequestStatus,
    ) -> GdprResult<()> {
        let mut requests = self.requests.lock().unwrap();
        let request = requests
            .get_mut(&request_id)
            .ok_or(GdprError::RequestNotFound)?;
        request.status = status;
        if status == RequestStatus::Completed || status == RequestStatus::Rejected {
            request.completed_at = Some(Utc::now());
        }
        Ok(())
    }

    /// 完成请求
    pub fn complete_request(&self, request_id: Uuid) -> GdprResult<()> {
        self.update_request_status(request_id, RequestStatus::Completed)
    }

    /// 拒绝请求
    pub fn reject_request(&self, request_id: Uuid, reason: String) -> GdprResult<()> {
        let mut requests = self.requests.lock().unwrap();
        let request = requests
            .get_mut(&request_id)
            .ok_or(GdprError::RequestNotFound)?;
        *request = request.clone().reject(reason);
        Ok(())
    }

    /// 获取用户的所有请求
    pub fn get_user_requests(&self, user_id: Uuid) -> Vec<DataSubjectRequest> {
        let requests = self.requests.lock().unwrap();
        requests
            .values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect()
    }

    /// 获取待处理的请求
    pub fn get_pending_requests(&self) -> Vec<DataSubjectRequest> {
        let requests = self.requests.lock().unwrap();
        requests
            .values()
            .filter(|r| r.status == RequestStatus::Pending)
            .cloned()
            .collect()
    }

    /// 记录同意
    pub fn record_consent(
        &self,
        user_id: Uuid,
        purpose: String,
        version: String,
    ) -> GdprResult<ConsentRecord> {
        let consent = ConsentRecord::new(user_id, purpose, version);
        let mut consents = self.consents.lock().unwrap();
        consents
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(consent.clone());
        Ok(consent)
    }

    /// 撤回同意
    pub fn withdraw_consent(&self, consent_id: Uuid) -> GdprResult<()> {
        let mut consents = self.consents.lock().unwrap();
        for user_consents in consents.values_mut() {
            if let Some(consent) = user_consents.iter_mut().find(|c| c.id == consent_id) {
                consent.withdraw();
                return Ok(());
            }
        }
        Err(GdprError::ConsentNotFound)
    }

    /// 获取用户的所有同意记录
    pub fn get_user_consents(&self, user_id: Uuid) -> Vec<ConsentRecord> {
        let consents = self.consents.lock().unwrap();
        consents.get(&user_id).cloned().unwrap_or_default()
    }

    /// 获取用户的活跃同意
    pub fn get_active_consents(&self, user_id: Uuid) -> Vec<ConsentRecord> {
        self.get_user_consents(user_id)
            .into_iter()
            .filter(|c| c.is_active())
            .collect()
    }

    /// 添加数据保留策略
    pub fn add_retention_policy(&self, policy: RetentionPolicy) -> GdprResult<()> {
        let mut policies = self.policies.lock().unwrap();
        policies.insert(policy.data_type.clone(), policy);
        Ok(())
    }

    /// 获取数据保留策略
    pub fn get_retention_policy(&self, data_type: &str) -> GdprResult<RetentionPolicy> {
        let policies = self.policies.lock().unwrap();
        policies
            .get(data_type)
            .cloned()
            .ok_or(GdprError::PolicyNotFound)
    }

    /// 列出所有保留策略
    pub fn list_retention_policies(&self) -> Vec<RetentionPolicy> {
        let policies = self.policies.lock().unwrap();
        policies.values().cloned().collect()
    }

    /// 检查数据是否应该被删除（根据保留策略）
    pub fn should_delete_data(&self, data_type: &str, data_age_days: i64) -> bool {
        if let Ok(policy) = self.get_retention_policy(data_type) {
            data_age_days > policy.retention_period_days
        } else {
            false
        }
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> GdprStatistics {
        let requests = self.requests.lock().unwrap();
        let consents = self.consents.lock().unwrap();

        let total_requests = requests.len();
        let pending_requests = requests
            .values()
            .filter(|r| r.status == RequestStatus::Pending)
            .count();
        let completed_requests = requests
            .values()
            .filter(|r| r.status == RequestStatus::Completed)
            .count();

        let total_consents: usize = consents.values().map(|v| v.len()).sum();
        let active_consents: usize = consents
            .values()
            .map(|v| v.iter().filter(|c| c.is_active()).count())
            .sum();

        GdprStatistics {
            total_requests,
            pending_requests,
            completed_requests,
            total_consents,
            active_consents,
        }
    }
}

impl Default for GdprManager {
    fn default() -> Self {
        Self::new()
    }
}

/// GDPR 统计信息
#[derive(Debug, Clone)]
pub struct GdprStatistics {
    pub total_requests: usize,
    pub pending_requests: usize,
    pub completed_requests: usize,
    pub total_consents: usize,
    pub active_consents: usize,
}
