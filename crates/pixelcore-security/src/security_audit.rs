use crate::models::{SecurityAuditLog, SecurityEventType, SecuritySeverity};
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 安全审计日志管理器
#[derive(Debug, Clone)]
pub struct SecurityAuditor {
    logs: Arc<Mutex<VecDeque<SecurityAuditLog>>>,
    max_logs: usize,
    // 异常活动检测
    login_attempts: Arc<Mutex<HashMap<String, Vec<DateTime<Utc>>>>>,
    access_patterns: Arc<Mutex<HashMap<Uuid, Vec<AccessRecord>>>>,
}

#[derive(Debug, Clone)]
struct AccessRecord {
    timestamp: DateTime<Utc>,
    #[allow(dead_code)]
    resource: String,
    #[allow(dead_code)]
    ip_address: Option<String>,
}

impl SecurityAuditor {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(VecDeque::new())),
            max_logs,
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
            access_patterns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 记录安全审计日志
    pub fn log(&self, log: SecurityAuditLog) {
        let mut logs = self.logs.lock().unwrap();
        logs.push_back(log);

        // 保持日志数量在限制内
        while logs.len() > self.max_logs {
            logs.pop_front();
        }
    }

    /// 记录登录尝试
    pub fn log_login_attempt(&self, username: &str, success: bool, user_id: Option<Uuid>, ip: Option<String>) {
        // 记录登录尝试时间并检测暴力破解（在同一个锁内完成）
        let anomaly = {
            let mut attempts = self.login_attempts.lock().unwrap();
            attempts
                .entry(username.to_string())
                .or_insert_with(Vec::new)
                .push(Utc::now());

            // 检测暴力破解（在锁内完成检测）
            if !success {
                let user_attempts = attempts.get(username).unwrap();
                let recent_attempts: Vec<_> = user_attempts
                    .iter()
                    .filter(|&&t| Utc::now() - t < Duration::minutes(5))
                    .collect();

                if recent_attempts.len() >= 5 {
                    Some(SecurityAuditLog::new(
                        SecurityEventType::AnomalousActivity {
                            user_id: Uuid::nil(),
                            description: format!("Possible brute force attack on user: {}", username),
                            severity: SecuritySeverity::High,
                        },
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        }; // 释放 attempts 锁

        // 记录异常日志（如果有）
        if let Some(log) = anomaly {
            self.log(log);
        }

        // 记录审计日志
        let event = if success {
            SecurityEventType::LoginSuccess {
                user_id: user_id.unwrap(),
                method: crate::models::AuthMethod::Password,
            }
        } else {
            SecurityEventType::LoginFailure {
                username: username.to_string(),
                reason: "Invalid credentials".to_string(),
                method: crate::models::AuthMethod::Password,
            }
        };

        let mut log = SecurityAuditLog::new(event);
        if let Some(ip_addr) = ip {
            log = log.with_ip(ip_addr);
        }
        self.log(log);
    }

    /// 记录访问
    pub fn log_access(&self, user_id: Uuid, resource: &str, ip: Option<String>) {
        // 记录访问并检测异常（在同一个锁内完成）
        let anomaly = {
            let mut patterns = self.access_patterns.lock().unwrap();
            patterns
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(AccessRecord {
                    timestamp: Utc::now(),
                    resource: resource.to_string(),
                    ip_address: ip.clone(),
                });

            // 检测异常访问模式（在锁内完成检测）
            let user_accesses = patterns.get(&user_id).unwrap();
            let recent_accesses: Vec<_> = user_accesses
                .iter()
                .filter(|a| Utc::now() - a.timestamp < Duration::minutes(1))
                .collect();

            if recent_accesses.len() >= 100 {
                Some(SecurityAuditLog::new(
                    SecurityEventType::AnomalousActivity {
                        user_id,
                        description: "Unusually high access rate detected".to_string(),
                        severity: SecuritySeverity::Medium,
                    },
                ))
            } else {
                None
            }
        }; // 释放 patterns 锁

        // 记录异常日志（如果有）
        if let Some(log) = anomaly {
            self.log(log);
        }
    }

    /// 获取所有审计日志
    pub fn get_all_logs(&self) -> Vec<SecurityAuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter().cloned().collect()
    }

    /// 获取指定时间范围的审计日志
    pub fn get_logs_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<SecurityAuditLog> {
        let logs = self.logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.timestamp >= start && log.timestamp <= end)
            .cloned()
            .collect()
    }

    /// 搜索审计日志
    pub fn search_logs<F>(&self, predicate: F) -> Vec<SecurityAuditLog>
    where
        F: Fn(&SecurityAuditLog) -> bool,
    {
        let logs = self.logs.lock().unwrap();
        logs.iter().filter(|log| predicate(log)).cloned().collect()
    }

    /// 获取高危事件
    pub fn get_critical_events(&self) -> Vec<SecurityAuditLog> {
        self.search_logs(|log| {
            matches!(
                log.event_type,
                SecurityEventType::AnomalousActivity {
                    severity: SecuritySeverity::High | SecuritySeverity::Critical,
                    ..
                }
            )
        })
    }

    /// 获取失败的登录尝试
    pub fn get_failed_logins(&self) -> Vec<SecurityAuditLog> {
        self.search_logs(|log| {
            matches!(log.event_type, SecurityEventType::LoginFailure { .. })
        })
    }

    /// 获取访问被拒绝的事件
    pub fn get_access_denied(&self) -> Vec<SecurityAuditLog> {
        self.search_logs(|log| {
            matches!(log.event_type, SecurityEventType::AccessDenied { .. })
        })
    }

    /// 清理旧的登录尝试记录
    pub fn cleanup_old_attempts(&self) {
        let mut attempts = self.login_attempts.lock().unwrap();
        let cutoff = Utc::now() - Duration::hours(24);

        for user_attempts in attempts.values_mut() {
            user_attempts.retain(|&t| t > cutoff);
        }

        attempts.retain(|_, v| !v.is_empty());
    }

    /// 清理旧的访问记录
    pub fn cleanup_old_accesses(&self) {
        let mut patterns = self.access_patterns.lock().unwrap();
        let cutoff = Utc::now() - Duration::hours(24);

        for user_accesses in patterns.values_mut() {
            user_accesses.retain(|a| a.timestamp > cutoff);
        }

        patterns.retain(|_, v| !v.is_empty());
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

    /// 获取安全统计信息
    pub fn get_stats(&self) -> SecurityStats {
        let logs = self.logs.lock().unwrap();
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);

        let recent_logs: Vec<_> = logs
            .iter()
            .filter(|log| log.timestamp > last_24h)
            .collect();

        let failed_logins = recent_logs
            .iter()
            .filter(|log| matches!(log.event_type, SecurityEventType::LoginFailure { .. }))
            .count();

        let access_denied = recent_logs
            .iter()
            .filter(|log| matches!(log.event_type, SecurityEventType::AccessDenied { .. }))
            .count();

        let anomalous_activities = recent_logs
            .iter()
            .filter(|log| matches!(log.event_type, SecurityEventType::AnomalousActivity { .. }))
            .count();

        SecurityStats {
            total_logs: logs.len(),
            recent_logs_24h: recent_logs.len(),
            failed_logins_24h: failed_logins,
            access_denied_24h: access_denied,
            anomalous_activities_24h: anomalous_activities,
        }
    }
}

impl Default for SecurityAuditor {
    fn default() -> Self {
        Self::new(10000) // 默认保留 10000 条日志
    }
}

/// 安全统计信息
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub total_logs: usize,
    pub recent_logs_24h: usize,
    pub failed_logins_24h: usize,
    pub access_denied_24h: usize,
    pub anomalous_activities_24h: usize,
}
