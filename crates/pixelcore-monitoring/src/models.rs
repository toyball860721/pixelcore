use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// 计数器（只增不减）
    Counter,
    /// 仪表（可增可减）
    Gauge,
    /// 直方图
    Histogram,
    /// 摘要
    Summary,
}

/// 指标数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

impl MetricDataPoint {
    pub fn new(value: f64) -> Self {
        Self {
            timestamp: Utc::now(),
            value,
            labels: HashMap::new(),
        }
    }

    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
}

/// 指标定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub unit: Option<String>,
    pub data_points: Vec<MetricDataPoint>,
}

impl Metric {
    pub fn new(name: String, metric_type: MetricType, description: String) -> Self {
        Self {
            name,
            metric_type,
            description,
            unit: None,
            data_points: Vec::new(),
        }
    }

    pub fn with_unit(mut self, unit: String) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn record(&mut self, value: f64) {
        self.data_points.push(MetricDataPoint::new(value));
    }

    pub fn record_with_labels(&mut self, value: f64, labels: HashMap<String, String>) {
        let mut point = MetricDataPoint::new(value);
        point.labels = labels;
        self.data_points.push(point);
    }
}

/// 系统指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_usage_percent: f64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub disk_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// 业务指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub success_rate: f64,
    pub total_revenue: f64,
    pub active_users: u64,
    pub active_agents: u64,
}

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// 告警状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    Pending,
    Firing,
    Resolved,
    Acknowledged,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

impl AlertRule {
    pub fn new(
        name: String,
        description: String,
        metric_name: String,
        condition: AlertCondition,
        threshold: f64,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            metric_name,
            condition,
            threshold,
            duration_seconds: 60, // 默认 60 秒
            severity,
            enabled: true,
        }
    }

    pub fn evaluate(&self, value: f64) -> bool {
        match self.condition {
            AlertCondition::GreaterThan => value > self.threshold,
            AlertCondition::LessThan => value < self.threshold,
            AlertCondition::Equal => (value - self.threshold).abs() < f64::EPSILON,
            AlertCondition::GreaterThanOrEqual => value >= self.threshold,
            AlertCondition::LessThanOrEqual => value <= self.threshold,
        }
    }
}

/// 告警条件
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// 告警实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub message: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub fired_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
}

impl Alert {
    pub fn new(
        rule: &AlertRule,
        metric_value: f64,
        message: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            rule_name: rule.name.clone(),
            severity: rule.severity,
            status: AlertStatus::Firing,
            message,
            metric_value,
            threshold: rule.threshold,
            fired_at: Utc::now(),
            resolved_at: None,
            acknowledged_at: None,
            acknowledged_by: None,
        }
    }

    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(Utc::now());
    }

    pub fn acknowledge(&mut self, by: String) {
        self.status = AlertStatus::Acknowledged;
        self.acknowledged_at = Some(Utc::now());
        self.acknowledged_by = Some(by);
    }
}

/// 告警通知渠道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email { recipients: Vec<String> },
    Slack { webhook_url: String, channel: String },
    Webhook { url: String },
    Console,
}

/// 告警通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub channel: NotificationChannel,
    pub sent_at: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl AlertNotification {
    pub fn new(alert_id: Uuid, channel: NotificationChannel) -> Self {
        Self {
            id: Uuid::new_v4(),
            alert_id,
            channel,
            sent_at: Utc::now(),
            success: false,
            error_message: None,
        }
    }
}
