use pixelcore_registry::{AgentListing, AgentRegistry};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 服务需求描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequirement {
    /// 需要的技能列表
    pub required_skills: Vec<String>,
    /// 可选的技能列表
    pub optional_skills: Vec<String>,
    /// 最大预算 (每次调用)
    pub max_budget: Option<f64>,
    /// 最低信誉要求
    pub min_reputation: Option<f64>,
    /// 最大响应时间 (毫秒)
    pub max_response_time: Option<u64>,
    /// 最低可用性要求 (百分比)
    pub min_availability: Option<f64>,
}

/// 服务发现引擎
pub struct ServiceDiscovery {
    registry: AgentRegistry,
}

impl ServiceDiscovery {
    /// 创建新的服务发现引擎
    pub fn new(registry: AgentRegistry) -> Self {
        Self { registry }
    }

    /// 发现满足需求的服务
    pub fn discover(&self, requirement: &ServiceRequirement) -> Result<Vec<AgentListing>> {
        // 获取所有已发布的 Agent
        let all_agents = self.registry.list_published(0, 10000)?;

        // 过滤满足条件的 Agent
        let matched: Vec<AgentListing> = all_agents
            .into_iter()
            .filter(|agent| self.matches_requirement(agent, requirement))
            .collect();

        Ok(matched)
    }

    /// 检查 Agent 是否满足需求
    fn matches_requirement(&self, agent: &AgentListing, req: &ServiceRequirement) -> bool {
        // 检查必需技能
        for required_skill in &req.required_skills {
            if !self.has_skill(agent, required_skill) {
                return false;
            }
        }

        // 检查预算
        if let Some(max_budget) = req.max_budget {
            if !self.within_budget(agent, max_budget) {
                return false;
            }
        }

        // 检查信誉
        if let Some(min_reputation) = req.min_reputation {
            if agent.reputation_score < min_reputation {
                return false;
            }
        }

        // 检查响应时间
        if let Some(max_response_time) = req.max_response_time {
            if agent.sla.response_time_ms > max_response_time {
                return false;
            }
        }

        // 检查可用性
        if let Some(min_availability) = req.min_availability {
            if agent.sla.availability_percent < min_availability {
                return false;
            }
        }

        true
    }

    /// 检查 Agent 是否有指定技能
    fn has_skill(&self, agent: &AgentListing, skill_name: &str) -> bool {
        agent.capabilities.iter().any(|cap| {
            cap.skill_name.to_lowercase() == skill_name.to_lowercase()
        })
    }

    /// 检查 Agent 是否在预算内
    fn within_budget(&self, agent: &AgentListing, max_budget: f64) -> bool {
        use pixelcore_registry::PricingModel;

        match &agent.pricing {
            PricingModel::Free => true,
            PricingModel::PerCall { price } => *price <= max_budget,
            PricingModel::PerHour { price } => {
                // 假设一次调用平均需要 5 分钟
                let estimated_cost = price / 12.0;
                estimated_cost <= max_budget
            }
            PricingModel::Subscription { .. } => {
                // 订阅制不适用于单次预算
                false
            }
        }
    }

    /// 按技能发现服务
    pub fn discover_by_skill(&self, skill_name: &str) -> Result<Vec<AgentListing>> {
        let requirement = ServiceRequirement {
            required_skills: vec![skill_name.to_string()],
            optional_skills: vec![],
            max_budget: None,
            min_reputation: None,
            max_response_time: None,
            min_availability: None,
        };

        self.discover(&requirement)
    }

    /// 发现免费服务
    pub fn discover_free_services(&self) -> Result<Vec<AgentListing>> {
        let all_agents = self.registry.list_published(0, 10000)?;

        let free_agents: Vec<AgentListing> = all_agents
            .into_iter()
            .filter(|agent| {
                matches!(agent.pricing, pixelcore_registry::PricingModel::Free)
            })
            .collect();

        Ok(free_agents)
    }

    /// 发现高性能服务 (响应时间快)
    pub fn discover_fast_services(&self, max_response_ms: u64) -> Result<Vec<AgentListing>> {
        let all_agents = self.registry.list_published(0, 10000)?;

        let fast_agents: Vec<AgentListing> = all_agents
            .into_iter()
            .filter(|agent| agent.sla.response_time_ms <= max_response_ms)
            .collect();

        Ok(fast_agents)
    }

    /// 发现高可用服务
    pub fn discover_reliable_services(&self, min_availability: f64) -> Result<Vec<AgentListing>> {
        let all_agents = self.registry.list_published(0, 10000)?;

        let reliable_agents: Vec<AgentListing> = all_agents
            .into_iter()
            .filter(|agent| agent.sla.availability_percent >= min_availability)
            .collect();

        Ok(reliable_agents)
    }
}
