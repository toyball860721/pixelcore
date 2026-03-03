use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// 备份类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    /// 全量备份
    Full,
    /// 增量备份
    Incremental,
    /// 差异备份
    Differential,
}

impl std::fmt::Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupType::Full => write!(f, "Full"),
            BackupType::Incremental => write!(f, "Incremental"),
            BackupType::Differential => write!(f, "Differential"),
        }
    }
}

/// 备份状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupStatus {
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已验证
    Verified,
}

/// 备份记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: Uuid,
    pub backup_type: BackupType,
    pub status: BackupStatus,
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
    pub size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    pub file_count: usize,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub metadata: BackupMetadata,
}

impl BackupRecord {
    pub fn new(
        backup_type: BackupType,
        source_path: PathBuf,
        backup_path: PathBuf,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            backup_type,
            status: BackupStatus::InProgress,
            source_path,
            backup_path,
            size_bytes: 0,
            compressed_size_bytes: None,
            file_count: 0,
            checksum: None,
            created_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            error_message: None,
            metadata: BackupMetadata::default(),
        }
    }

    pub fn complete(&mut self, size_bytes: u64, file_count: usize) {
        let completed_at = Utc::now();
        self.status = BackupStatus::Completed;
        self.size_bytes = size_bytes;
        self.file_count = file_count;
        self.completed_at = Some(completed_at);
        self.duration_ms = Some((completed_at - self.created_at).num_milliseconds() as u64);
    }

    pub fn fail(&mut self, error: String) {
        self.status = BackupStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }

    pub fn verify(&mut self, checksum: String) {
        self.status = BackupStatus::Verified;
        self.checksum = Some(checksum);
    }
}

/// 备份元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub hostname: Option<String>,
    pub version: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

impl Default for BackupMetadata {
    fn default() -> Self {
        Self {
            hostname: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            tags: Vec::new(),
            description: None,
        }
    }
}

/// 恢复记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRecord {
    pub id: Uuid,
    pub backup_id: Uuid,
    pub status: RestoreStatus,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub file_count: usize,
    pub restored_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
}

impl RestoreRecord {
    pub fn new(backup_id: Uuid, source_path: PathBuf, target_path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            backup_id,
            status: RestoreStatus::InProgress,
            source_path,
            target_path,
            file_count: 0,
            restored_bytes: 0,
            created_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            error_message: None,
        }
    }

    pub fn complete(&mut self, file_count: usize, restored_bytes: u64) {
        let completed_at = Utc::now();
        self.status = RestoreStatus::Completed;
        self.file_count = file_count;
        self.restored_bytes = restored_bytes;
        self.completed_at = Some(completed_at);
        self.duration_ms = Some((completed_at - self.created_at).num_milliseconds() as u64);
    }

    pub fn fail(&mut self, error: String) {
        self.status = RestoreStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }
}

/// 恢复状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestoreStatus {
    InProgress,
    Completed,
    Failed,
    Verified,
}

/// 备份策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPolicy {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub schedule: BackupSchedule,
    pub retention: RetentionPolicy,
    pub compression: bool,
    pub encryption: bool,
}

impl BackupPolicy {
    pub fn new(name: String, schedule: BackupSchedule) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            enabled: true,
            schedule,
            retention: RetentionPolicy::default(),
            compression: true,
            encryption: false,
        }
    }
}

/// 备份调度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupSchedule {
    /// 每小时
    Hourly,
    /// 每日 (小时)
    Daily { hour: u8 },
    /// 每周 (星期几, 小时)
    Weekly { day: u8, hour: u8 },
    /// 每月 (日期, 小时)
    Monthly { day: u8, hour: u8 },
    /// 自定义 cron 表达式
    Cron { expression: String },
}

/// 保留策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// 保留最近 N 个备份
    pub keep_last: usize,
    /// 保留 N 天内的备份
    pub keep_days: Option<u32>,
    /// 保留 N 周内的备份
    pub keep_weeks: Option<u32>,
    /// 保留 N 月内的备份
    pub keep_months: Option<u32>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_last: 7,
            keep_days: Some(30),
            keep_weeks: Some(4),
            keep_months: Some(12),
        }
    }
}

/// 备份统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStats {
    pub total_backups: usize,
    pub total_size_bytes: u64,
    pub total_compressed_size_bytes: u64,
    pub successful_backups: usize,
    pub failed_backups: usize,
    pub average_duration_ms: u64,
    pub oldest_backup: Option<DateTime<Utc>>,
    pub newest_backup: Option<DateTime<Utc>>,
}

impl BackupStats {
    pub fn new() -> Self {
        Self {
            total_backups: 0,
            total_size_bytes: 0,
            total_compressed_size_bytes: 0,
            successful_backups: 0,
            failed_backups: 0,
            average_duration_ms: 0,
            oldest_backup: None,
            newest_backup: None,
        }
    }
}

impl Default for BackupStats {
    fn default() -> Self {
        Self::new()
    }
}
