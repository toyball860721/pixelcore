use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::models::{Operation, Resource, Role};

/// 审计事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// 角色分配
    RoleAssigned {
        target_user_id: Uuid,
        role: Role,
        tenant_id: Option<Uuid>,
    },
    /// 角色撤销
    RoleRevoked {
        target_user_id: Uuid,
        role: Role,
        tenant_id: Option<Uuid>,
    },
    /// 权限授予
    PermissionGranted {
        target_user_id: Uuid,
        resource: Resource,
        operation: Operation,
        resource_id: Option<Uuid>,
    },
    /// 权限撤销
    PermissionRevoked {
        permission_id: Uuid,
    },
    /// 权限检查成功
    PermissionCheckSuccess {
        resource: Resource,
        operation: Operation,
        resource_id: Option<Uuid>,
    },
    /// 权限检查失败
    PermissionCheckFailed {
        resource: Resource,
        operation: Operation,
        resource_id: Option<Uuid>,
        reason: String,
    },
}

/// 审计日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: AuditEventType,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl AuditLog {
    pub fn new(user_id: Uuid, event_type: AuditEventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            event_type,
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            metadata: None,
        }
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// 审计日志管理器
#[derive(Debug, Clone)]
pub struct AuditLogger {
    logs: Arc<Mutex<VecDeque<AuditLog>>>,
    max_logs: usize,
}

impl AuditLogger {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(VecDeque::new())),
            max_logs,
        }
    }

    /// 记录审计日志
    pub fn log(&self, audit_log: AuditLog) {
        let mut logs = self.logs.lock().unwrap();
        logs.push_back(audit_log);

        // 保持日志数量在限制内
        while logs.len() > self.max_logs {
            logs.pop_front();
        }
    }

    /// 获取所有审计日志
    pub fn get_all_logs(&self) -> Vec<AuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter().cloned().collect()
    }

    /// 获取指定用户的审计日志
    pub fn get_user_logs(&self, user_id: Uuid) -> Vec<AuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.user_id == user_id)
            .cloned()
            .collect()
    }

    /// 获取指定时间范围的审计日志
    pub fn get_logs_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.timestamp >= start && log.timestamp <= end)
            .cloned()
            .collect()
    }

    /// 搜索审计日志
    pub fn search_logs<F>(&self, predicate: F) -> Vec<AuditLog>
    where
        F: Fn(&AuditLog) -> bool,
    {
        let logs = self.logs.lock().unwrap();
        logs.iter().filter(|log| predicate(log)).cloned().collect()
    }

    /// 清空所有日志
    pub fn clear(&self) {
        let mut logs = self.logs.lock().unwrap();
        logs.clear();
    }

    /// 获取日志数量
    pub fn count(&self) -> usize {
        let logs = self.logs.lock().unwrap();
        logs.len()
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000) // 默认保留 10000 条日志
    }
}
