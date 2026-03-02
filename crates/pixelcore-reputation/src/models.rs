use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 评价/评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// 评价 ID
    pub id: Uuid,
    /// 交易 ID
    pub transaction_id: Uuid,
    /// Agent ID (被评价的 Agent)
    pub agent_id: Uuid,
    /// 评价者 ID
    pub reviewer_id: Uuid,
    /// 评分 (1-5)
    pub rating: u8,
    /// 评价内容
    pub comment: String,
    /// 是否已验证 (防刷)
    pub verified: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl Review {
    /// 创建新的评价
    pub fn new(
        transaction_id: Uuid,
        agent_id: Uuid,
        reviewer_id: Uuid,
        rating: u8,
        comment: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transaction_id,
            agent_id,
            reviewer_id,
            rating: rating.clamp(1, 5), // 确保评分在 1-5 之间
            comment,
            verified: false,
            created_at: Utc::now(),
        }
    }

    /// 验证评价
    pub fn verify(&mut self) {
        self.verified = true;
    }
}

/// 信誉等级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReputationLevel {
    /// 新手 (0-10 单)
    Newcomer,
    /// 普通 (11-50 单)
    Regular,
    /// 优秀 (51-200 单)
    Excellent,
    /// 专家 (201+ 单)
    Expert,
}

impl ReputationLevel {
    /// 根据交易数量计算等级
    pub fn from_transaction_count(count: u64) -> Self {
        match count {
            0..=10 => Self::Newcomer,
            11..=50 => Self::Regular,
            51..=200 => Self::Excellent,
            _ => Self::Expert,
        }
    }

    /// 获取等级名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Newcomer => "新手",
            Self::Regular => "普通",
            Self::Excellent => "优秀",
            Self::Expert => "专家",
        }
    }

    /// 获取等级描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Newcomer => "刚开始提供服务",
            Self::Regular => "有一定服务经验",
            Self::Excellent => "经验丰富的服务提供者",
            Self::Expert => "顶级专家级服务提供者",
        }
    }
}

/// 信誉记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationRecord {
    /// Agent ID
    pub agent_id: Uuid,
    /// 信誉分数 (0.0 - 5.0)
    pub score: f64,
    /// 总交易数
    pub total_transactions: u64,
    /// 成功交易数
    pub successful_transactions: u64,
    /// 平均响应时间 (毫秒)
    pub average_response_time_ms: u64,
    /// 信誉等级
    pub level: ReputationLevel,
    /// 评价列表
    pub reviews: Vec<Review>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl ReputationRecord {
    /// 创建新的信誉记录
    pub fn new(agent_id: Uuid) -> Self {
        Self {
            agent_id,
            score: 0.0,
            total_transactions: 0,
            successful_transactions: 0,
            average_response_time_ms: 0,
            level: ReputationLevel::Newcomer,
            reviews: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_transactions == 0 {
            0.0
        } else {
            self.successful_transactions as f64 / self.total_transactions as f64
        }
    }

    /// 添加评价
    pub fn add_review(&mut self, review: Review) {
        self.reviews.push(review);
        self.updated_at = Utc::now();
    }

    /// 获取平均评分
    pub fn average_rating(&self) -> f64 {
        if self.reviews.is_empty() {
            0.0
        } else {
            let sum: u32 = self.reviews.iter().map(|r| r.rating as u32).sum();
            sum as f64 / self.reviews.len() as f64
        }
    }

    /// 获取已验证的评价数量
    pub fn verified_review_count(&self) -> usize {
        self.reviews.iter().filter(|r| r.verified).count()
    }
}

/// 信誉统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationStats {
    /// 5 星评价数
    pub five_star_count: usize,
    /// 4 星评价数
    pub four_star_count: usize,
    /// 3 星评价数
    pub three_star_count: usize,
    /// 2 星评价数
    pub two_star_count: usize,
    /// 1 星评价数
    pub one_star_count: usize,
    /// 总评价数
    pub total_reviews: usize,
    /// 平均评分
    pub average_rating: f64,
    /// 成功率
    pub success_rate: f64,
}

impl ReputationStats {
    /// 从信誉记录计算统计信息
    pub fn from_record(record: &ReputationRecord) -> Self {
        let mut stats = Self {
            five_star_count: 0,
            four_star_count: 0,
            three_star_count: 0,
            two_star_count: 0,
            one_star_count: 0,
            total_reviews: record.reviews.len(),
            average_rating: record.average_rating(),
            success_rate: record.success_rate(),
        };

        for review in &record.reviews {
            match review.rating {
                5 => stats.five_star_count += 1,
                4 => stats.four_star_count += 1,
                3 => stats.three_star_count += 1,
                2 => stats.two_star_count += 1,
                1 => stats.one_star_count += 1,
                _ => {}
            }
        }

        stats
    }
}
