use crate::models::{Metric, SystemMetrics};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Metric not found: {0}")]
    MetricNotFound(String),
}

pub type MetricsResult<T> = Result<T, MetricsError>;

/// 指标收集器
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    metrics: Arc<Mutex<HashMap<String, Metric>>>,
    system: Arc<Mutex<System>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            system: Arc::new(Mutex::new(System::new_all())),
        }
    }

    /// 注册指标
    pub fn register_metric(&self, metric: Metric) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.insert(metric.name.clone(), metric);
    }

    /// 记录指标值
    pub fn record(&self, name: &str, value: f64) -> MetricsResult<()> {
        let mut metrics = self.metrics.lock().unwrap();
        let metric = metrics
            .get_mut(name)
            .ok_or_else(|| MetricsError::MetricNotFound(name.to_string()))?;
        metric.record(value);
        Ok(())
    }

    /// 记录带标签的指标值
    pub fn record_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> MetricsResult<()> {
        let mut metrics = self.metrics.lock().unwrap();
        let metric = metrics
            .get_mut(name)
            .ok_or_else(|| MetricsError::MetricNotFound(name.to_string()))?;
        metric.record_with_labels(value, labels);
        Ok(())
    }

    /// 获取指标
    pub fn get_metric(&self, name: &str) -> MetricsResult<Metric> {
        let metrics = self.metrics.lock().unwrap();
        metrics
            .get(name)
            .cloned()
            .ok_or_else(|| MetricsError::MetricNotFound(name.to_string()))
    }

    /// 获取所有指标
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.values().cloned().collect()
    }

    /// 收集系统指标
    pub fn collect_system_metrics(&self) -> SystemMetrics {
        let mut sys = self.system.lock().unwrap();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage() as f64;

        let memory_used = sys.used_memory();
        let memory_total = sys.total_memory();
        let memory_usage_percent = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64) * 100.0
        } else {
            0.0
        };

        // Note: In sysinfo 0.32, disk and network APIs have changed
        // For now, we'll use placeholder values
        // TODO: Update to use Disks and Networks types from sysinfo 0.32+
        let disk_used = 0u64;
        let disk_total = 0u64;
        let disk_usage_percent = 0.0;
        let network_rx = 0u64;
        let network_tx = 0u64;

        SystemMetrics {
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: cpu_usage,
            memory_used_bytes: memory_used,
            memory_total_bytes: memory_total,
            memory_usage_percent,
            disk_used_bytes: disk_used,
            disk_total_bytes: disk_total,
            disk_usage_percent,
            network_rx_bytes: network_rx,
            network_tx_bytes: network_tx,
        }
    }

    /// 清空指标数据
    pub fn clear_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        for metric in metrics.values_mut() {
            metric.data_points.clear();
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
