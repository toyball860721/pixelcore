use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Agent 能力定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// 技能名称
    pub skill_name: String,
    /// 描述
    pub description: String,
    /// 输入参数 schema (JSON Schema)
    pub input_schema: serde_json::Value,
    /// 输出结果 schema (JSON Schema)
    pub output_schema: serde_json::Value,
}

/// 定价模型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PricingModel {
    /// 按次计费
    PerCall { price: f64 },
    /// 按小时计费
    PerHour { price: f64 },
    /// 订阅制
    Subscription { monthly_price: f64 },
    /// 免费
    Free,
}

/// 服务等级协议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevel {
    /// 响应时间 (毫秒)
    pub response_time_ms: u64,
    /// 可用性 (百分比, 0-100)
    pub availability_percent: f64,
    /// 并发限制
    pub max_concurrent_requests: u32,
}

/// Agent 状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    /// 草稿 (未发布)
    Draft,
    /// 已发布 (可用)
    Published,
    /// 已暂停
    Paused,
    /// 已下架
    Archived,
}

/// Agent 列表信息 (完整的注册信息)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentListing {
    /// Agent ID
    pub id: Uuid,
    /// Agent 名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 版本
    pub version: String,
    /// 所有者 ID
    pub owner_id: Uuid,
    /// 能力列表
    pub capabilities: Vec<Capability>,
    /// 定价模型
    pub pricing: PricingModel,
    /// 服务等级
    pub sla: ServiceLevel,
    /// 状态
    pub status: AgentStatus,
    /// 信誉分数 (0.0 - 5.0)
    pub reputation_score: f64,
    /// 总交易数
    pub total_transactions: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl AgentListing {
    /// 创建新的 Agent 列表
    pub fn new(
        name: String,
        description: String,
        version: String,
        owner_id: Uuid,
        capabilities: Vec<Capability>,
        pricing: PricingModel,
        sla: ServiceLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            version,
            owner_id,
            capabilities,
            pricing,
            sla,
            status: AgentStatus::Draft,
            reputation_score: 0.0,
            total_transactions: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// 发布 Agent
    pub fn publish(&mut self) {
        self.status = AgentStatus::Published;
        self.updated_at = Utc::now();
    }

    /// 暂停 Agent
    pub fn pause(&mut self) {
        self.status = AgentStatus::Paused;
        self.updated_at = Utc::now();
    }

    /// 下架 Agent
    pub fn archive(&mut self) {
        self.status = AgentStatus::Archived;
        self.updated_at = Utc::now();
    }
}

/// Agent 搜索过滤器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentFilter {
    /// 按名称搜索
    pub name: Option<String>,
    /// 按所有者 ID 过滤
    pub owner_id: Option<Uuid>,
    /// 按状态过滤
    pub status: Option<AgentStatus>,
    /// 按技能名称过滤
    pub skill_name: Option<String>,
    /// 最低信誉分数
    pub min_reputation: Option<f64>,
    /// 最大价格 (按次计费)
    pub max_price: Option<f64>,
}
