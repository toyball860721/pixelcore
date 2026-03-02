use crate::models::{AgentListing, AgentFilter, AgentStatus};
use crate::storage::RegistryStorage;
use anyhow::{Context, Result};
use std::path::Path;
use uuid::Uuid;

/// Agent 注册表
pub struct AgentRegistry {
    storage: RegistryStorage,
}

impl AgentRegistry {
    /// 创建新的注册表
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let storage = RegistryStorage::new(db_path)?;
        Ok(Self { storage })
    }

    /// 创建内存注册表 (用于测试)
    pub fn in_memory() -> Result<Self> {
        let storage = RegistryStorage::in_memory()?;
        Ok(Self { storage })
    }

    /// 注册新的 Agent
    pub fn register(&self, listing: AgentListing) -> Result<Uuid> {
        let id = listing.id;
        self.storage.save(&listing)
            .context("Failed to register agent")?;
        Ok(id)
    }

    /// 发布 Agent (使其可被发现)
    pub fn publish(&self, id: &Uuid) -> Result<()> {
        let mut listing = self.storage.get(id)?
            .context("Agent not found")?;

        listing.publish();
        self.storage.save(&listing)?;
        Ok(())
    }

    /// 暂停 Agent
    pub fn pause(&self, id: &Uuid) -> Result<()> {
        let mut listing = self.storage.get(id)?
            .context("Agent not found")?;

        listing.pause();
        self.storage.save(&listing)?;
        Ok(())
    }

    /// 下架 Agent
    pub fn archive(&self, id: &Uuid) -> Result<()> {
        let mut listing = self.storage.get(id)?
            .context("Agent not found")?;

        listing.archive();
        self.storage.save(&listing)?;
        Ok(())
    }

    /// 更新 Agent 信息
    pub fn update(&self, listing: AgentListing) -> Result<()> {
        self.storage.save(&listing)
            .context("Failed to update agent")?;
        Ok(())
    }

    /// 删除 Agent
    pub fn delete(&self, id: &Uuid) -> Result<bool> {
        self.storage.delete(id)
    }

    /// 获取 Agent 详情
    pub fn get(&self, id: &Uuid) -> Result<Option<AgentListing>> {
        self.storage.get(id)
    }

    /// 列出所有 Agent
    pub fn list(&self, offset: usize, limit: usize) -> Result<Vec<AgentListing>> {
        self.storage.list(offset, limit)
    }

    /// 搜索 Agent
    pub fn search(&self, filter: &AgentFilter) -> Result<Vec<AgentListing>> {
        // 简单实现: 先获取所有,然后过滤
        // TODO: 在数据库层面实现更高效的搜索
        let all_listings = self.storage.list(0, 1000)?;

        let filtered: Vec<AgentListing> = all_listings
            .into_iter()
            .filter(|listing| {
                // 按名称过滤
                if let Some(ref name) = filter.name {
                    if !listing.name.to_lowercase().contains(&name.to_lowercase()) {
                        return false;
                    }
                }

                // 按所有者过滤
                if let Some(owner_id) = filter.owner_id {
                    if listing.owner_id != owner_id {
                        return false;
                    }
                }

                // 按状态过滤
                if let Some(status) = filter.status {
                    if listing.status != status {
                        return false;
                    }
                }

                // 按技能名称过滤
                if let Some(ref skill_name) = filter.skill_name {
                    let has_skill = listing.capabilities.iter().any(|cap| {
                        cap.skill_name.to_lowercase().contains(&skill_name.to_lowercase())
                    });
                    if !has_skill {
                        return false;
                    }
                }

                // 按最低信誉过滤
                if let Some(min_reputation) = filter.min_reputation {
                    if listing.reputation_score < min_reputation {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(filtered)
    }

    /// 获取已发布的 Agent 列表
    pub fn list_published(&self, _offset: usize, _limit: usize) -> Result<Vec<AgentListing>> {
        let filter = AgentFilter {
            status: Some(AgentStatus::Published),
            ..Default::default()
        };
        self.search(&filter)
    }

    /// 统计 Agent 数量
    pub fn count(&self) -> Result<usize> {
        self.storage.count()
    }
}
