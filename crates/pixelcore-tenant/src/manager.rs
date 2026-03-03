use crate::models::{Tenant, TenantStatus, TenantConfig, TenantUsage, TenantMember, TenantIsolation, IsolationLevel};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 租户管理器
pub struct TenantManager {
    tenants: Arc<Mutex<Vec<Tenant>>>,
    usage: Arc<Mutex<Vec<TenantUsage>>>,
    members: Arc<Mutex<Vec<TenantMember>>>,
    isolation: Arc<Mutex<Vec<TenantIsolation>>>,
}

impl TenantManager {
    /// 创建新的租户管理器
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(Mutex::new(Vec::new())),
            usage: Arc::new(Mutex::new(Vec::new())),
            members: Arc::new(Mutex::new(Vec::new())),
            isolation: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建租户
    pub async fn create_tenant(
        &self,
        name: String,
        description: String,
        owner_id: Uuid,
        config: Option<TenantConfig>,
    ) -> Result<Tenant, String> {
        let mut tenant = Tenant::new(name, description, owner_id);

        if let Some(cfg) = config {
            tenant.update_config(cfg);
        }

        // 创建使用情况记录
        let usage = TenantUsage::new(tenant.id);

        // 创建隔离配置 (默认使用独立表)
        let isolation = TenantIsolation::new(tenant.id, IsolationLevel::SeparateTable);

        // 添加所有者为成员
        let member = TenantMember::new(tenant.id, owner_id, "owner".to_string());

        // 保存
        let mut tenants = self.tenants.lock().await;
        let mut usage_vec = self.usage.lock().await;
        let mut isolation_vec = self.isolation.lock().await;
        let mut members = self.members.lock().await;

        tenants.push(tenant.clone());
        usage_vec.push(usage);
        isolation_vec.push(isolation);
        members.push(member);

        Ok(tenant)
    }

    /// 获取租户
    pub async fn get_tenant(&self, tenant_id: Uuid) -> Option<Tenant> {
        let tenants = self.tenants.lock().await;
        tenants.iter().find(|t| t.id == tenant_id).cloned()
    }

    /// 获取用户的租户列表
    pub async fn get_user_tenants(&self, user_id: Uuid) -> Vec<Tenant> {
        let members = self.members.lock().await;
        let tenants = self.tenants.lock().await;

        let tenant_ids: Vec<Uuid> = members
            .iter()
            .filter(|m| m.user_id == user_id)
            .map(|m| m.tenant_id)
            .collect();

        tenants
            .iter()
            .filter(|t| tenant_ids.contains(&t.id))
            .cloned()
            .collect()
    }

    /// 更新租户配置
    pub async fn update_tenant_config(
        &self,
        tenant_id: Uuid,
        config: TenantConfig,
    ) -> Result<Tenant, String> {
        let mut tenants = self.tenants.lock().await;

        if let Some(tenant) = tenants.iter_mut().find(|t| t.id == tenant_id) {
            tenant.update_config(config);
            Ok(tenant.clone())
        } else {
            Err("Tenant not found".to_string())
        }
    }

    /// 暂停租户
    pub async fn suspend_tenant(&self, tenant_id: Uuid) -> Result<(), String> {
        let mut tenants = self.tenants.lock().await;

        if let Some(tenant) = tenants.iter_mut().find(|t| t.id == tenant_id) {
            tenant.suspend();
            Ok(())
        } else {
            Err("Tenant not found".to_string())
        }
    }

    /// 激活租户
    pub async fn activate_tenant(&self, tenant_id: Uuid) -> Result<(), String> {
        let mut tenants = self.tenants.lock().await;

        if let Some(tenant) = tenants.iter_mut().find(|t| t.id == tenant_id) {
            tenant.activate();
            Ok(())
        } else {
            Err("Tenant not found".to_string())
        }
    }

    /// 删除租户
    pub async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), String> {
        let mut tenants = self.tenants.lock().await;

        if let Some(tenant) = tenants.iter_mut().find(|t| t.id == tenant_id) {
            tenant.delete();
            Ok(())
        } else {
            Err("Tenant not found".to_string())
        }
    }

    /// 获取租户使用情况
    pub async fn get_tenant_usage(&self, tenant_id: Uuid) -> Option<TenantUsage> {
        let usage = self.usage.lock().await;
        usage.iter().find(|u| u.tenant_id == tenant_id).cloned()
    }

    /// 检查资源配额
    pub async fn check_quota(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
    ) -> Result<bool, String> {
        let tenant = self.get_tenant(tenant_id).await
            .ok_or("Tenant not found")?;

        if !tenant.is_active() {
            return Err("Tenant is not active".to_string());
        }

        let usage = self.get_tenant_usage(tenant_id).await
            .ok_or("Usage not found")?;

        let has_quota = match resource_type {
            "agent" => !usage.is_agents_exceeded(tenant.config.max_agents),
            "storage" => !usage.is_storage_exceeded(tenant.config.max_storage_gb),
            "api_call" => !usage.is_api_calls_exceeded(tenant.config.max_api_calls_per_month),
            _ => return Err("Unknown resource type".to_string()),
        };

        Ok(has_quota)
    }

    /// 增加 Agent
    pub async fn add_agent(&self, tenant_id: Uuid) -> Result<(), String> {
        // 检查配额
        if !self.check_quota(tenant_id, "agent").await? {
            return Err("Agent quota exceeded".to_string());
        }

        let mut usage = self.usage.lock().await;
        if let Some(u) = usage.iter_mut().find(|u| u.tenant_id == tenant_id) {
            u.add_agent()?;
            Ok(())
        } else {
            Err("Usage not found".to_string())
        }
    }

    /// 移除 Agent
    pub async fn remove_agent(&self, tenant_id: Uuid) -> Result<(), String> {
        let mut usage = self.usage.lock().await;
        if let Some(u) = usage.iter_mut().find(|u| u.tenant_id == tenant_id) {
            u.remove_agent()?;
            Ok(())
        } else {
            Err("Usage not found".to_string())
        }
    }

    /// 增加存储使用
    pub async fn add_storage(&self, tenant_id: Uuid, gb: f64) -> Result<(), String> {
        let mut usage = self.usage.lock().await;
        if let Some(u) = usage.iter_mut().find(|u| u.tenant_id == tenant_id) {
            u.add_storage(gb);
            Ok(())
        } else {
            Err("Usage not found".to_string())
        }
    }

    /// 记录 API 调用
    pub async fn record_api_call(&self, tenant_id: Uuid) -> Result<(), String> {
        // 检查配额
        if !self.check_quota(tenant_id, "api_call").await? {
            return Err("API call quota exceeded".to_string());
        }

        let mut usage = self.usage.lock().await;
        if let Some(u) = usage.iter_mut().find(|u| u.tenant_id == tenant_id) {
            u.add_api_call();
            Ok(())
        } else {
            Err("Usage not found".to_string())
        }
    }

    /// 添加成员
    pub async fn add_member(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        role: String,
    ) -> Result<TenantMember, String> {
        // 检查租户是否存在
        self.get_tenant(tenant_id).await
            .ok_or("Tenant not found")?;

        let member = TenantMember::new(tenant_id, user_id, role);

        let mut members = self.members.lock().await;
        members.push(member.clone());

        Ok(member)
    }

    /// 移除成员
    pub async fn remove_member(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), String> {
        let mut members = self.members.lock().await;

        let index = members
            .iter()
            .position(|m| m.tenant_id == tenant_id && m.user_id == user_id)
            .ok_or("Member not found")?;

        members.remove(index);
        Ok(())
    }

    /// 获取租户成员
    pub async fn get_tenant_members(&self, tenant_id: Uuid) -> Vec<TenantMember> {
        let members = self.members.lock().await;
        members
            .iter()
            .filter(|m| m.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    /// 检查用户是否是租户成员
    pub async fn is_member(&self, tenant_id: Uuid, user_id: Uuid) -> bool {
        let members = self.members.lock().await;
        members
            .iter()
            .any(|m| m.tenant_id == tenant_id && m.user_id == user_id)
    }

    /// 获取租户隔离配置
    pub async fn get_isolation(&self, tenant_id: Uuid) -> Option<TenantIsolation> {
        let isolation = self.isolation.lock().await;
        isolation.iter().find(|i| i.tenant_id == tenant_id).cloned()
    }

    /// 获取所有活跃租户
    pub async fn get_active_tenants(&self) -> Vec<Tenant> {
        let tenants = self.tenants.lock().await;
        tenants
            .iter()
            .filter(|t| t.status == TenantStatus::Active)
            .cloned()
            .collect()
    }

    /// 重置月度统计
    pub async fn reset_monthly_usage(&self, tenant_id: Uuid) -> Result<(), String> {
        let mut usage = self.usage.lock().await;
        if let Some(u) = usage.iter_mut().find(|u| u.tenant_id == tenant_id) {
            u.reset_monthly();
            Ok(())
        } else {
            Err("Usage not found".to_string())
        }
    }
}
