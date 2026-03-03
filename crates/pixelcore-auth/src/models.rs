use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// 角色定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// 超级管理员 - 拥有所有权限
    SuperAdmin,
    /// 租户管理员 - 管理租户内的所有资源
    TenantAdmin,
    /// 开发者 - 可以创建和管理 Agent
    Developer,
    /// 普通用户 - 只能使用 Agent
    User,
}

impl Role {
    /// 获取角色的默认权限
    pub fn default_permissions(&self) -> HashSet<Permission> {
        match self {
            Role::SuperAdmin => {
                // 超级管理员拥有所有权限
                vec![
                    Permission::new(Resource::All, Operation::All),
                ]
                .into_iter()
                .collect()
            }
            Role::TenantAdmin => {
                // 租户管理员可以管理租户内的所有资源
                vec![
                    Permission::new(Resource::Tenant, Operation::All),
                    Permission::new(Resource::Agent, Operation::All),
                    Permission::new(Resource::User, Operation::Read),
                    Permission::new(Resource::Billing, Operation::Read),
                    Permission::new(Resource::Audit, Operation::Read),
                ]
                .into_iter()
                .collect()
            }
            Role::Developer => {
                // 开发者可以创建和管理自己的 Agent
                vec![
                    Permission::new(Resource::Agent, Operation::Create),
                    Permission::new(Resource::Agent, Operation::Read),
                    Permission::new(Resource::Agent, Operation::Update),
                    Permission::new(Resource::Agent, Operation::Delete),
                    Permission::new(Resource::Marketplace, Operation::Read),
                    Permission::new(Resource::Transaction, Operation::Read),
                ]
                .into_iter()
                .collect()
            }
            Role::User => {
                // 普通用户只能使用 Agent
                vec![
                    Permission::new(Resource::Agent, Operation::Read),
                    Permission::new(Resource::Marketplace, Operation::Read),
                    Permission::new(Resource::Transaction, Operation::Create),
                    Permission::new(Resource::Transaction, Operation::Read),
                ]
                .into_iter()
                .collect()
            }
        }
    }

    /// 获取角色的优先级（数字越大优先级越高）
    pub fn priority(&self) -> u8 {
        match self {
            Role::SuperAdmin => 100,
            Role::TenantAdmin => 50,
            Role::Developer => 20,
            Role::User => 10,
        }
    }
}

/// 资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    /// 所有资源（仅超级管理员）
    All,
    /// 租户
    Tenant,
    /// Agent
    Agent,
    /// 用户
    User,
    /// 市场
    Marketplace,
    /// 交易
    Transaction,
    /// 计费
    Billing,
    /// 审计日志
    Audit,
}

/// 操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// 所有操作（仅超级管理员）
    All,
    /// 创建
    Create,
    /// 读取
    Read,
    /// 更新
    Update,
    /// 删除
    Delete,
    /// 执行
    Execute,
}

/// 权限定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: Resource,
    pub operation: Operation,
}

impl Permission {
    pub fn new(resource: Resource, operation: Operation) -> Self {
        Self { resource, operation }
    }

    /// 检查是否匹配指定的权限
    pub fn matches(&self, resource: Resource, operation: Operation) -> bool {
        // 检查资源匹配
        let resource_match = self.resource == Resource::All || self.resource == resource;

        // 检查操作匹配
        let operation_match = self.operation == Operation::All || self.operation == operation;

        resource_match && operation_match
    }
}

/// 用户角色分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: Uuid,
    pub role: Role,
    pub tenant_id: Option<Uuid>, // 租户级别的角色需要指定租户
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl UserRole {
    pub fn new(user_id: Uuid, role: Role, granted_by: Uuid) -> Self {
        Self {
            user_id,
            role,
            tenant_id: None,
            granted_by,
            granted_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// 检查角色是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 检查角色是否有效
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// 自定义权限（用于细粒度控制）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission: Permission,
    pub resource_id: Option<Uuid>, // 特定资源的权限
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl CustomPermission {
    pub fn new(user_id: Uuid, permission: Permission, granted_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            permission,
            resource_id: None,
            granted_by,
            granted_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn with_resource(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// 检查权限是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 检查权限是否有效
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}
