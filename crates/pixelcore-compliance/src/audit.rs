use crate::models::{ComplianceReport, ComplianceReportType, ImmutableAuditLog};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Audit log verification failed")]
    VerificationFailed,
    #[error("Log chain broken at index {0}")]
    ChainBroken(usize),
}

pub type AuditResult<T> = Result<T, AuditError>;

/// 不可篡改审计日志管理器
#[derive(Debug, Clone)]
pub struct ImmutableAuditLogger {
    logs: Arc<Mutex<VecDeque<ImmutableAuditLog>>>,
    max_logs: usize,
}

impl ImmutableAuditLogger {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(VecDeque::new())),
            max_logs,
        }
    }

    /// 记录审计日志
    pub fn log(
        &self,
        user_id: Option<Uuid>,
        action: String,
        resource_type: String,
        resource_id: Option<Uuid>,
        details: serde_json::Value,
    ) -> ImmutableAuditLog {
        let mut logs = self.logs.lock().unwrap();

        // 获取前一条日志的哈希值
        let previous_hash = logs
            .back()
            .map(|log| log.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));

        // 创建新日志
        let log = ImmutableAuditLog::new(
            user_id,
            action,
            resource_type,
            resource_id,
            details,
            previous_hash,
        );

        logs.push_back(log.clone());

        // 保持日志数量在限制内
        while logs.len() > self.max_logs {
            logs.pop_front();
        }

        log
    }

    /// 验证单条日志的完整性
    pub fn verify_log(&self, log: &ImmutableAuditLog) -> bool {
        log.verify()
    }

    /// 验证整个日志链的完整性
    pub fn verify_chain(&self) -> AuditResult<()> {
        let logs = self.logs.lock().unwrap();

        for (i, log) in logs.iter().enumerate() {
            // 验证日志本身的哈希
            if !log.verify() {
                return Err(AuditError::VerificationFailed);
            }

            // 验证链式关系
            if i > 0 {
                let previous_log = &logs[i - 1];
                if log.previous_hash != previous_log.hash {
                    return Err(AuditError::ChainBroken(i));
                }
            }
        }

        Ok(())
    }

    /// 获取所有审计日志
    pub fn get_all_logs(&self) -> Vec<ImmutableAuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter().cloned().collect()
    }

    /// 获取指定用户的审计日志
    pub fn get_user_logs(&self, user_id: Uuid) -> Vec<ImmutableAuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.user_id == Some(user_id))
            .cloned()
            .collect()
    }

    /// 获取指定时间范围的审计日志
    pub fn get_logs_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<ImmutableAuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.timestamp >= start && log.timestamp <= end)
            .cloned()
            .collect()
    }

    /// 搜索审计日志
    pub fn search_logs<F>(&self, predicate: F) -> Vec<ImmutableAuditLog>
    where
        F: Fn(&ImmutableAuditLog) -> bool,
    {
        let logs = self.logs.lock().unwrap();
        logs.iter().filter(|log| predicate(log)).cloned().collect()
    }

    /// 获取日志数量
    pub fn count(&self) -> usize {
        let logs = self.logs.lock().unwrap();
        logs.len()
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> AuditStatistics {
        let logs = self.logs.lock().unwrap();

        let total_logs = logs.len();
        let unique_users = logs
            .iter()
            .filter_map(|log| log.user_id)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let actions: std::collections::HashMap<String, usize> = logs.iter().fold(
            std::collections::HashMap::new(),
            |mut acc, log| {
                *acc.entry(log.action.clone()).or_insert(0) += 1;
                acc
            },
        );

        AuditStatistics {
            total_logs,
            unique_users,
            top_actions: actions
                .into_iter()
                .collect::<Vec<_>>()
                .into_iter()
                .take(10)
                .collect(),
        }
    }
}

impl Default for ImmutableAuditLogger {
    fn default() -> Self {
        Self::new(100000) // 默认保留 100000 条日志
    }
}

/// 审计统计信息
#[derive(Debug, Clone)]
pub struct AuditStatistics {
    pub total_logs: usize,
    pub unique_users: usize,
    pub top_actions: Vec<(String, usize)>,
}

/// 合规报告生成器
#[derive(Debug, Clone)]
pub struct ComplianceReporter {
    reports: Arc<Mutex<Vec<ComplianceReport>>>,
}

impl ComplianceReporter {
    pub fn new() -> Self {
        Self {
            reports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 生成 GDPR 合规报告
    pub fn generate_gdpr_report(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        summary: serde_json::Value,
    ) -> ComplianceReport {
        let report = ComplianceReport::new(
            ComplianceReportType::Gdpr,
            period_start,
            period_end,
            summary,
        );

        let mut reports = self.reports.lock().unwrap();
        reports.push(report.clone());

        report
    }

    /// 生成数据处理活动报告
    pub fn generate_data_processing_report(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        summary: serde_json::Value,
    ) -> ComplianceReport {
        let report = ComplianceReport::new(
            ComplianceReportType::DataProcessingActivities,
            period_start,
            period_end,
            summary,
        );

        let mut reports = self.reports.lock().unwrap();
        reports.push(report.clone());

        report
    }

    /// 生成数据主体请求报告
    pub fn generate_data_subject_requests_report(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        summary: serde_json::Value,
    ) -> ComplianceReport {
        let report = ComplianceReport::new(
            ComplianceReportType::DataSubjectRequests,
            period_start,
            period_end,
            summary,
        );

        let mut reports = self.reports.lock().unwrap();
        reports.push(report.clone());

        report
    }

    /// 获取所有报告
    pub fn get_all_reports(&self) -> Vec<ComplianceReport> {
        let reports = self.reports.lock().unwrap();
        reports.clone()
    }

    /// 获取指定类型的报告
    pub fn get_reports_by_type(
        &self,
        report_type: ComplianceReportType,
    ) -> Vec<ComplianceReport> {
        let reports = self.reports.lock().unwrap();
        reports
            .iter()
            .filter(|r| r.report_type == report_type)
            .cloned()
            .collect()
    }
}

impl Default for ComplianceReporter {
    fn default() -> Self {
        Self::new()
    }
}
