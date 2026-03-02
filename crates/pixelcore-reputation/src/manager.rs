use crate::models::{ReputationRecord, Review, ReputationStats};
use crate::storage::ReputationStorage;
use crate::calculator::ReputationCalculator;
use anyhow::{Context, Result};
use std::path::Path;
use uuid::Uuid;

/// 信誉管理器
pub struct ReputationManager {
    storage: ReputationStorage,
    calculator: ReputationCalculator,
}

impl ReputationManager {
    /// 创建新的信誉管理器
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let storage = ReputationStorage::new(db_path)?;
        let calculator = ReputationCalculator::new();
        Ok(Self { storage, calculator })
    }

    /// 创建内存管理器
    pub fn in_memory() -> Result<Self> {
        let storage = ReputationStorage::in_memory()?;
        let calculator = ReputationCalculator::new();
        Ok(Self { storage, calculator })
    }

    /// 获取或创建信誉记录
    pub fn get_or_create_record(&self, agent_id: &Uuid) -> Result<ReputationRecord> {
        if let Some(record) = self.storage.get_record(agent_id)? {
            Ok(record)
        } else {
            let record = ReputationRecord::new(*agent_id);
            self.storage.save_record(&record)?;
            Ok(record)
        }
    }

    /// 添加评价
    pub fn add_review(
        &self,
        transaction_id: Uuid,
        agent_id: Uuid,
        reviewer_id: Uuid,
        rating: u8,
        comment: String,
    ) -> Result<Uuid> {
        // 获取或创建信誉记录 (确保记录存在)
        let mut record = self.get_or_create_record(&agent_id)?;

        // 创建评价
        let review = Review::new(transaction_id, agent_id, reviewer_id, rating, comment);
        let review_id = review.id;

        // 保存评价
        self.storage.save_review(&review)?;

        // 更新信誉记录
        record.add_review(review);
        self.calculator.update_record(&mut record);
        self.storage.save_record(&record)?;

        Ok(review_id)
    }

    /// 验证评价
    pub fn verify_review(&self, review_id: &Uuid, agent_id: &Uuid) -> Result<()> {
        let mut record = self.get_or_create_record(agent_id)?;

        // 找到并验证评价
        if let Some(review) = record.reviews.iter_mut().find(|r| r.id == *review_id) {
            review.verify();
            self.storage.save_review(review)?;
        }

        Ok(())
    }

    /// 记录交易
    pub fn record_transaction(
        &self,
        agent_id: &Uuid,
        success: bool,
        response_time_ms: u64,
    ) -> Result<()> {
        let mut record = self.get_or_create_record(agent_id)?;
        self.calculator.record_transaction(&mut record, success, response_time_ms);
        self.storage.save_record(&record)?;
        Ok(())
    }

    /// 获取信誉记录
    pub fn get_record(&self, agent_id: &Uuid) -> Result<Option<ReputationRecord>> {
        self.storage.get_record(agent_id)
    }

    /// 获取信誉统计
    pub fn get_stats(&self, agent_id: &Uuid) -> Result<Option<ReputationStats>> {
        if let Some(record) = self.storage.get_record(agent_id)? {
            Ok(Some(ReputationStats::from_record(&record)))
        } else {
            Ok(None)
        }
    }

    /// 检测异常
    pub fn detect_anomaly(&self, agent_id: &Uuid) -> Result<Vec<String>> {
        if let Some(record) = self.storage.get_record(agent_id)? {
            Ok(self.calculator.detect_anomaly(&record))
        } else {
            Ok(Vec::new())
        }
    }

    /// 计算趋势
    pub fn calculate_trend(&self, agent_id: &Uuid, recent_count: usize) -> Result<Option<f64>> {
        if let Some(record) = self.storage.get_record(agent_id)? {
            Ok(self.calculator.calculate_trend(&record, recent_count))
        } else {
            Ok(None)
        }
    }

    /// 重新计算所有信誉分数
    pub fn recalculate_all(&self) -> Result<usize> {
        // 这是一个简化实现,实际应该遍历所有记录
        // 由于我们没有实现 list_all 方法,这里返回 0
        Ok(0)
    }
}
