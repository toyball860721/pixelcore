use crate::models::{ReputationRecord, ReputationLevel};

/// 信誉计算器
pub struct ReputationCalculator;

impl ReputationCalculator {
    /// 创建新的信誉计算器
    pub fn new() -> Self {
        Self
    }

    /// 计算信誉分数
    ///
    /// 综合考虑多个因素:
    /// - 平均评分 (40%)
    /// - 成功率 (30%)
    /// - 交易数量 (20%)
    /// - 响应时间 (10%)
    pub fn calculate_score(&self, record: &ReputationRecord) -> f64 {
        let mut score = 0.0;

        // 1. 平均评分 (权重 40%)
        let avg_rating = record.average_rating();
        score += avg_rating * 0.4;

        // 2. 成功率 (权重 30%)
        let success_rate = record.success_rate();
        score += success_rate * 5.0 * 0.3; // 归一化到 0-5

        // 3. 交易数量 (权重 20%)
        let transaction_score = self.calculate_transaction_score(record.total_transactions);
        score += transaction_score * 0.2;

        // 4. 响应时间 (权重 10%)
        let response_score = self.calculate_response_score(record.average_response_time_ms);
        score += response_score * 0.1;

        // 确保分数在 0-5 之间
        score.clamp(0.0, 5.0)
    }

    /// 计算交易数量得分 (0-5)
    fn calculate_transaction_score(&self, count: u64) -> f64 {
        match count {
            0 => 0.0,
            1..=10 => 2.0,
            11..=50 => 3.5,
            51..=200 => 4.5,
            _ => 5.0,
        }
    }

    /// 计算响应时间得分 (0-5)
    fn calculate_response_score(&self, response_time_ms: u64) -> f64 {
        match response_time_ms {
            0..=500 => 5.0,      // 极快
            501..=1000 => 4.5,   // 很快
            1001..=2000 => 4.0,  // 快
            2001..=5000 => 3.0,  // 一般
            5001..=10000 => 2.0, // 慢
            _ => 1.0,            // 很慢
        }
    }

    /// 更新信誉记录
    pub fn update_record(&self, record: &mut ReputationRecord) {
        // 重新计算分数
        record.score = self.calculate_score(record);

        // 更新等级
        record.level = ReputationLevel::from_transaction_count(record.total_transactions);

        // 更新时间
        record.updated_at = chrono::Utc::now();
    }

    /// 记录新交易
    pub fn record_transaction(
        &self,
        record: &mut ReputationRecord,
        success: bool,
        response_time_ms: u64,
    ) {
        record.total_transactions += 1;
        if success {
            record.successful_transactions += 1;
        }

        // 更新平均响应时间 (移动平均)
        if record.total_transactions == 1 {
            record.average_response_time_ms = response_time_ms;
        } else {
            let total_time = record.average_response_time_ms * (record.total_transactions - 1);
            record.average_response_time_ms = (total_time + response_time_ms) / record.total_transactions;
        }

        self.update_record(record);
    }

    /// 检测异常评分 (可能的刷单行为)
    pub fn detect_anomaly(&self, record: &ReputationRecord) -> Vec<String> {
        let mut anomalies = Vec::new();

        // 1. 检查评分分布是否异常 (全是 5 星或全是 1 星)
        if record.reviews.len() >= 10 {
            let five_star_ratio = record.reviews.iter()
                .filter(|r| r.rating == 5)
                .count() as f64 / record.reviews.len() as f64;

            let one_star_ratio = record.reviews.iter()
                .filter(|r| r.rating == 1)
                .count() as f64 / record.reviews.len() as f64;

            if five_star_ratio > 0.95 {
                anomalies.push("评分分布异常: 95%+ 为 5 星".to_string());
            }

            if one_star_ratio > 0.95 {
                anomalies.push("评分分布异常: 95%+ 为 1 星".to_string());
            }
        }

        // 2. 检查未验证评价比例
        let verified_ratio = record.verified_review_count() as f64 / record.reviews.len().max(1) as f64;
        if verified_ratio < 0.5 && record.reviews.len() >= 10 {
            anomalies.push("未验证评价过多".to_string());
        }

        // 3. 检查成功率异常
        if record.total_transactions >= 10 {
            let success_rate = record.success_rate();
            if success_rate == 1.0 {
                anomalies.push("成功率异常: 100% 成功".to_string());
            } else if success_rate < 0.5 {
                anomalies.push("成功率过低: < 50%".to_string());
            }
        }

        anomalies
    }

    /// 计算信誉趋势 (最近 N 个评价的平均分)
    pub fn calculate_trend(&self, record: &ReputationRecord, recent_count: usize) -> Option<f64> {
        if record.reviews.is_empty() {
            return None;
        }

        let recent_reviews: Vec<_> = record.reviews
            .iter()
            .rev()
            .take(recent_count)
            .collect();

        if recent_reviews.is_empty() {
            return None;
        }

        let sum: u32 = recent_reviews.iter().map(|r| r.rating as u32).sum();
        Some(sum as f64 / recent_reviews.len() as f64)
    }
}

impl Default for ReputationCalculator {
    fn default() -> Self {
        Self::new()
    }
}
