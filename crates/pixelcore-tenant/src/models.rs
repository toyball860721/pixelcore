use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 租户状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantStatus {
    /// 活跃
    Active,
    /// 暂停
    Suspended,
    /// 已删除
    Deleted,
}

/// 租户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// 最大 Agent 数量
    pub max_agents: u32,
    /// 最大存储空间 (GB)
    pub max_storage_gb: f64,
    /// 最大 API 调用次数 (每月)
    pub max_api_calls_per_month: u64,
    /// 是否启用高级功能
    pub enable_advanced_features: bool,
    /// 自定义配置
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            max_agents: 10,
            max_storage_gb: 10.0,
            max_api_calls_per_month: 10000,
            enable_advanced_features: false,
            custom_settings: HashMap::new(),
        }
    }
}

/// 租户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// 租户 ID
    pub id: Uuid,
    /// 租户名称
    pub name: String,
    /// 租户描述
    pub description: String,
    /// 租户状态
    pub status: TenantStatus,
    /// 租户配置
    pub config: TenantConfig,
    /// 所有者 ID
    pub owner_id: Uuid,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl Tenant {
    /// 创建新租户
    pub fn new(name: String, description: String, owner_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: TenantStatus::Active,
            config: TenantConfig::default(),
            owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: TenantConfig) {
        self.config = config;
        self.updated_at = Utc::now();
    }

    /// 暂停租户
    pub fn suspend(&mut self) {
        self.status = TenantStatus::Suspended;
        self.updated_at = Utc::now();
    }

    /// 激活租户
    pub fn activate(&mut self) {
        self.status = TenantStatus::Active;
        self.updated_at = Utc::now();
    }

    /// 删除租户
    pub fn delete(&mut self) {
        self.status = TenantStatus::Deleted;
        self.updated_at = Utc::now();
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active
    }
}

/// 租户资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsage {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 当前 Agent 数量
    pub current_agents: u32,
    /// 当前存储使用 (GB)
    pub current_storage_gb: f64,
    /// 当前月 API 调用次数
    pub current_api_calls: u64,
    /// 统计周期开始时间
    pub period_start: DateTime<Utc>,
    /// 统计周期结束时间
    pub period_end: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl TenantUsage {
    /// 创建新的使用情况记录
    pub fn new(tenant_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            tenant_id,
            current_agents: 0,
            current_storage_gb: 0.0,
            current_api_calls: 0,
            period_start: now,
            period_end: now,
            updated_at: now,
        }
    }

    /// 检查是否超出 Agent 配额
    pub fn is_agents_exceeded(&self, max_agents: u32) -> bool {
        self.current_agents >= max_agents
    }

    /// 检查是否超出存储配额
    pub fn is_storage_exceeded(&self, max_storage_gb: f64) -> bool {
        self.current_storage_gb >= max_storage_gb
    }

    /// 检查是否超出 API 调用配额
    pub fn is_api_calls_exceeded(&self, max_api_calls: u64) -> bool {
        self.current_api_calls >= max_api_calls
    }

    /// 增加 Agent 数量
    pub fn add_agent(&mut self) -> Result<(), String> {
        self.current_agents += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 减少 Agent 数量
    pub fn remove_agent(&mut self) -> Result<(), String> {
        if self.current_agents == 0 {
            return Err("No agents to remove".to_string());
        }
        self.current_agents -= 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 增加存储使用
    pub fn add_storage(&mut self, gb: f64) {
        self.current_storage_gb += gb;
        self.updated_at = Utc::now();
    }

    /// 减少存储使用
    pub fn remove_storage(&mut self, gb: f64) {
        self.current_storage_gb = (self.current_storage_gb - gb).max(0.0);
        self.updated_at = Utc::now();
    }

    /// 增加 API 调用次数
    pub fn add_api_call(&mut self) {
        self.current_api_calls += 1;
        self.updated_at = Utc::now();
    }

    /// 重置月度统计
    pub fn reset_monthly(&mut self) {
        self.current_api_calls = 0;
        self.period_start = Utc::now();
        self.period_end = Utc::now();
        self.updated_at = Utc::now();
    }
}

/// 租户成员
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMember {
    /// 成员 ID
    pub id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// 角色
    pub role: String,
    /// 加入时间
    pub joined_at: DateTime<Utc>,
    /// 元数据
    pub metadata: serde_json::Value,
}

impl TenantMember {
    /// 创建新成员
    pub fn new(tenant_id: Uuid, user_id: Uuid, role: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            role,
            joined_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}

/// 数据隔离策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationLevel {
    /// 共享数据库，共享表
    Shared,
    /// 共享数据库，独立表
    SeparateTable,
    /// 独立数据库
    SeparateDatabase,
}

/// 租户隔离配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantIsolation {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 隔离级别
    pub isolation_level: IsolationLevel,
    /// 数据库名称 (如果是独立数据库)
    pub database_name: Option<String>,
    /// 表前缀 (如果是独立表)
    pub table_prefix: Option<String>,
}

impl TenantIsolation {
    /// 创建新的隔离配置
    pub fn new(tenant_id: Uuid, isolation_level: IsolationLevel) -> Self {
        let (database_name, table_prefix) = match isolation_level {
            IsolationLevel::Shared => (None, None),
            IsolationLevel::SeparateTable => {
                (None, Some(format!("tenant_{}", tenant_id.to_string().replace("-", ""))))
            }
            IsolationLevel::SeparateDatabase => {
                (Some(format!("tenant_{}", tenant_id.to_string().replace("-", ""))), None)
            }
        };

        Self {
            tenant_id,
            isolation_level,
            database_name,
            table_prefix,
        }
    }

    /// 获取表名
    pub fn get_table_name(&self, base_table: &str) -> String {
        match &self.table_prefix {
            Some(prefix) => format!("{}_{}", prefix, base_table),
            None => base_table.to_string(),
        }
    }
}
