use crate::models::{UsageRecord, UsageType, UsageStats, Quota};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// 使用量追踪器
#[derive(Clone)]
pub struct UsageTracker {
    records: Arc<Mutex<Vec<UsageRecord>>>,
    quotas: Arc<Mutex<Vec<Quota>>>,
}

impl UsageTracker {
    /// 创建新的使用量追踪器
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
            quotas: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 记录使用量
    pub async fn record_usage(
        &self,
        user_id: Uuid,
        usage_type: UsageType,
        quantity: f64,
        unit: String,
    ) -> Result<UsageRecord, String> {
        // 检查配额
        self.check_and_update_quota(user_id, usage_type.clone(), quantity).await?;

        // 创建使用量记录
        let record = UsageRecord::new(user_id, usage_type, quantity, unit);

        // 保存记录
        let mut records = self.records.lock().await;
        records.push(record.clone());

        Ok(record)
    }

    /// 检查并更新配额
    async fn check_and_update_quota(
        &self,
        user_id: Uuid,
        usage_type: UsageType,
        quantity: f64,
    ) -> Result<(), String> {
        let mut quotas = self.quotas.lock().await;

        // 查找对应的配额
        if let Some(quota) = quotas.iter_mut().find(|q| {
            q.user_id == user_id && q.usage_type == usage_type
        }) {
            // 检查是否需要重置
            if quota.should_reset() {
                quota.reset();
            }

            // 检查配额
            quota.add_usage(quantity)?;
        }

        Ok(())
    }

    /// 获取用户的使用量统计
    pub async fn get_usage_stats(
        &self,
        user_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<UsageStats, String> {
        let records = self.records.lock().await;

        // 筛选时间范围内的记录
        let user_records: Vec<&UsageRecord> = records
            .iter()
            .filter(|r| {
                r.user_id == user_id
                    && r.recorded_at >= period_start
                    && r.recorded_at <= period_end
            })
            .collect();

        // 按类型统计使用量
        let mut usage_by_type: HashMap<UsageType, f64> = HashMap::new();
        let mut total_usage = 0.0;

        for record in user_records {
            *usage_by_type.entry(record.usage_type.clone()).or_insert(0.0) += record.quantity;
            total_usage += record.quantity;
        }

        Ok(UsageStats {
            user_id,
            period_start,
            period_end,
            usage_by_type,
            total_usage,
        })
    }

    /// 获取用户的使用量记录
    pub async fn get_usage_records(
        &self,
        user_id: Uuid,
        usage_type: Option<UsageType>,
        period_start: Option<DateTime<Utc>>,
        period_end: Option<DateTime<Utc>>,
    ) -> Vec<UsageRecord> {
        let records = self.records.lock().await;

        records
            .iter()
            .filter(|r| {
                r.user_id == user_id
                    && usage_type.as_ref().map_or(true, |t| &r.usage_type == t)
                    && period_start.map_or(true, |start| r.recorded_at >= start)
                    && period_end.map_or(true, |end| r.recorded_at <= end)
            })
            .cloned()
            .collect()
    }

    /// 设置配额
    pub async fn set_quota(
        &self,
        user_id: Uuid,
        usage_type: UsageType,
        limit: f64,
        reset_period_days: u32,
    ) -> Result<Quota, String> {
        let mut quotas = self.quotas.lock().await;

        // 检查是否已存在
        if let Some(existing) = quotas.iter_mut().find(|q| {
            q.user_id == user_id && q.usage_type == usage_type
        }) {
            // 更新现有配额
            existing.limit = limit;
            existing.reset_period_days = reset_period_days;
            return Ok(existing.clone());
        }

        // 创建新配额
        let quota = Quota::new(user_id, usage_type, limit, reset_period_days);
        quotas.push(quota.clone());

        Ok(quota)
    }

    /// 获取配额
    pub async fn get_quota(&self, user_id: Uuid, usage_type: UsageType) -> Option<Quota> {
        let quotas = self.quotas.lock().await;
        quotas
            .iter()
            .find(|q| q.user_id == user_id && q.usage_type == usage_type)
            .cloned()
    }

    /// 获取用户的所有配额
    pub async fn get_user_quotas(&self, user_id: Uuid) -> Vec<Quota> {
        let quotas = self.quotas.lock().await;
        quotas
            .iter()
            .filter(|q| q.user_id == user_id)
            .cloned()
            .collect()
    }

    /// 重置配额
    pub async fn reset_quota(&self, user_id: Uuid, usage_type: UsageType) -> Result<(), String> {
        let mut quotas = self.quotas.lock().await;

        if let Some(quota) = quotas.iter_mut().find(|q| {
            q.user_id == user_id && q.usage_type == usage_type
        }) {
            quota.reset();
            Ok(())
        } else {
            Err("Quota not found".to_string())
        }
    }

    /// 检查所有配额并自动重置
    pub async fn auto_reset_quotas(&self) {
        let mut quotas = self.quotas.lock().await;

        for quota in quotas.iter_mut() {
            if quota.should_reset() {
                quota.reset();
            }
        }
    }

    /// 获取使用量汇总 (按类型)
    pub async fn get_usage_summary(
        &self,
        user_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> HashMap<UsageType, f64> {
        let stats = self.get_usage_stats(user_id, period_start, period_end).await;
        stats.map(|s| s.usage_by_type).unwrap_or_default()
    }
}
