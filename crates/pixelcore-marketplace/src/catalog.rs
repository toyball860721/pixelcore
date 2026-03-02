use pixelcore_registry::{AgentListing, AgentRegistry, AgentFilter, AgentStatus};
use anyhow::Result;
use uuid::Uuid;

/// 服务目录 - 管理和展示可用的 Agent 服务
pub struct ServiceCatalog {
    registry: AgentRegistry,
}

impl ServiceCatalog {
    /// 创建新的服务目录
    pub fn new(registry: AgentRegistry) -> Self {
        Self { registry }
    }

    /// 浏览所有已发布的服务
    pub fn browse_all(&self, page: usize, page_size: usize) -> Result<Vec<AgentListing>> {
        let offset = page * page_size;
        self.registry.list_published(offset, page_size)
    }

    /// 按类别浏览 (通过技能名称分类)
    pub fn browse_by_category(&self, category: &str) -> Result<Vec<AgentListing>> {
        let filter = AgentFilter {
            skill_name: Some(category.to_string()),
            status: Some(AgentStatus::Published),
            ..Default::default()
        };
        self.registry.search(&filter)
    }

    /// 按价格范围筛选
    pub fn filter_by_price(&self, max_price: f64) -> Result<Vec<AgentListing>> {
        let filter = AgentFilter {
            max_price: Some(max_price),
            status: Some(AgentStatus::Published),
            ..Default::default()
        };
        self.registry.search(&filter)
    }

    /// 按信誉筛选
    pub fn filter_by_reputation(&self, min_reputation: f64) -> Result<Vec<AgentListing>> {
        let filter = AgentFilter {
            min_reputation: Some(min_reputation),
            status: Some(AgentStatus::Published),
            ..Default::default()
        };
        self.registry.search(&filter)
    }

    /// 获取热门服务 (按交易量排序)
    pub fn get_popular(&self, limit: usize) -> Result<Vec<AgentListing>> {
        let mut agents = self.registry.list_published(0, 1000)?;

        // 按交易量排序
        agents.sort_by(|a, b| b.total_transactions.cmp(&a.total_transactions));

        // 取前 N 个
        agents.truncate(limit);

        Ok(agents)
    }

    /// 获取高评分服务 (按信誉排序)
    pub fn get_top_rated(&self, limit: usize) -> Result<Vec<AgentListing>> {
        let mut agents = self.registry.list_published(0, 1000)?;

        // 按信誉分数排序
        agents.sort_by(|a, b| {
            b.reputation_score.partial_cmp(&a.reputation_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 取前 N 个
        agents.truncate(limit);

        Ok(agents)
    }

    /// 获取新上架服务 (按创建时间排序)
    pub fn get_newest(&self, limit: usize) -> Result<Vec<AgentListing>> {
        let mut agents = self.registry.list_published(0, 1000)?;

        // 按创建时间排序 (已经是降序)
        agents.truncate(limit);

        Ok(agents)
    }

    /// 获取服务详情
    pub fn get_details(&self, agent_id: &Uuid) -> Result<Option<AgentListing>> {
        self.registry.get(agent_id)
    }

    /// 搜索服务
    pub fn search(&self, query: &str) -> Result<Vec<AgentListing>> {
        let filter = AgentFilter {
            name: Some(query.to_string()),
            status: Some(AgentStatus::Published),
            ..Default::default()
        };
        self.registry.search(&filter)
    }

    /// 获取服务统计信息
    pub fn get_statistics(&self) -> Result<CatalogStatistics> {
        let all_agents = self.registry.list_published(0, 10000)?;

        let total_services = all_agents.len();
        let total_transactions: u64 = all_agents.iter()
            .map(|a| a.total_transactions)
            .sum();

        let avg_reputation = if total_services > 0 {
            all_agents.iter()
                .map(|a| a.reputation_score)
                .sum::<f64>() / total_services as f64
        } else {
            0.0
        };

        // 统计各类别的服务数量
        let mut categories = std::collections::HashMap::new();
        for agent in &all_agents {
            for cap in &agent.capabilities {
                *categories.entry(cap.skill_name.clone()).or_insert(0) += 1;
            }
        }

        Ok(CatalogStatistics {
            total_services,
            total_transactions,
            average_reputation: avg_reputation,
            categories,
        })
    }
}

/// 目录统计信息
#[derive(Debug, Clone)]
pub struct CatalogStatistics {
    /// 总服务数
    pub total_services: usize,
    /// 总交易数
    pub total_transactions: u64,
    /// 平均信誉分数
    pub average_reputation: f64,
    /// 各类别的服务数量
    pub categories: std::collections::HashMap<String, usize>,
}
