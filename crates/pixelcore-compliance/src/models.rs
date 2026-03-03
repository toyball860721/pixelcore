use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GDPR 数据主体权利
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSubjectRight {
    /// 访问权（Right to Access）
    Access,
    /// 更正权（Right to Rectification）
    Rectification,
    /// 删除权（Right to Erasure / Right to be Forgotten）
    Erasure,
    /// 限制处理权（Right to Restriction of Processing）
    RestrictionOfProcessing,
    /// 数据可携带权（Right to Data Portability）
    DataPortability,
    /// 反对权（Right to Object）
    Object,
}

/// 数据主体请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub right: DataSubjectRight,
    pub status: RequestStatus,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl DataSubjectRequest {
    pub fn new(user_id: Uuid, right: DataSubjectRight) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            right,
            status: RequestStatus::Pending,
            requested_at: Utc::now(),
            completed_at: None,
            notes: None,
        }
    }

    pub fn complete(mut self) -> Self {
        self.status = RequestStatus::Completed;
        self.completed_at = Some(Utc::now());
        self
    }

    pub fn reject(mut self, reason: String) -> Self {
        self.status = RequestStatus::Rejected;
        self.notes = Some(reason);
        self.completed_at = Some(Utc::now());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    InProgress,
    Completed,
    Rejected,
}

/// 同意记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub purpose: String,
    pub consent_given: bool,
    pub consent_date: DateTime<Utc>,
    pub withdrawn_date: Option<DateTime<Utc>>,
    pub version: String,
}

impl ConsentRecord {
    pub fn new(user_id: Uuid, purpose: String, version: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            purpose,
            consent_given: true,
            consent_date: Utc::now(),
            withdrawn_date: None,
            version,
        }
    }

    pub fn withdraw(&mut self) {
        self.consent_given = false;
        self.withdrawn_date = Some(Utc::now());
    }

    pub fn is_active(&self) -> bool {
        self.consent_given && self.withdrawn_date.is_none()
    }
}

/// 数据保留策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub data_type: String,
    pub retention_period_days: i64,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl RetentionPolicy {
    pub fn new(data_type: String, retention_period_days: i64, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            data_type,
            retention_period_days,
            description,
            created_at: Utc::now(),
        }
    }
}

/// 用户数据导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// 数据导出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub format: ExportFormat,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
}

impl DataExportRequest {
    pub fn new(user_id: Uuid, format: ExportFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            format,
            requested_at: Utc::now(),
            completed_at: None,
            file_path: None,
        }
    }
}

/// 数据删除类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeletionType {
    /// 软删除（标记为已删除）
    Soft,
    /// 硬删除（物理删除）
    Hard,
}

/// 数据删除请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub deletion_type: DeletionType,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deleted_records: Vec<String>,
}

impl DataDeletionRequest {
    pub fn new(user_id: Uuid, deletion_type: DeletionType) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            deletion_type,
            requested_at: Utc::now(),
            completed_at: None,
            deleted_records: Vec::new(),
        }
    }
}

/// 不可篡改的审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmutableAuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    /// 前一条日志的哈希值（用于链式验证）
    pub previous_hash: String,
    /// 当前日志的哈希值
    pub hash: String,
}

impl ImmutableAuditLog {
    pub fn new(
        user_id: Option<Uuid>,
        action: String,
        resource_type: String,
        resource_id: Option<Uuid>,
        details: serde_json::Value,
        previous_hash: String,
    ) -> Self {
        let mut log = Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id,
            action,
            resource_type,
            resource_id,
            details,
            ip_address: None,
            previous_hash,
            hash: String::new(),
        };
        log.hash = log.calculate_hash();
        log
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self.hash = self.calculate_hash();
        self
    }

    /// 计算日志的哈希值
    fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        let data = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            self.id,
            self.timestamp.to_rfc3339(),
            self.user_id.map(|u| u.to_string()).unwrap_or_default(),
            self.action,
            self.resource_type,
            self.resource_id.map(|r| r.to_string()).unwrap_or_default(),
            self.details,
            self.previous_hash
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 验证日志的完整性
    pub fn verify(&self) -> bool {
        self.hash == self.calculate_hash()
    }
}

/// 合规报告类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceReportType {
    /// GDPR 合规报告
    Gdpr,
    /// 数据处理活动报告
    DataProcessingActivities,
    /// 数据主体请求报告
    DataSubjectRequests,
    /// 同意管理报告
    ConsentManagement,
    /// 数据保留报告
    DataRetention,
}

/// 合规报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: Uuid,
    pub report_type: ComplianceReportType,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub summary: serde_json::Value,
}

impl ComplianceReport {
    pub fn new(
        report_type: ComplianceReportType,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        summary: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            report_type,
            generated_at: Utc::now(),
            period_start,
            period_end,
            summary,
        }
    }
}
