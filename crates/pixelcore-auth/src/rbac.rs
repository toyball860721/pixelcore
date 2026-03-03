use crate::models::{CustomPermission, Operation, Resource, Role, UserRole};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RbacError {
    #[error("Permission denied: user {user_id} does not have {operation:?} permission on {resource:?}")]
    PermissionDenied {
        user_id: Uuid,
        resource: Resource,
        operation: Operation,
    },
    #[error("Role not found for user {0}")]
    RoleNotFound(Uuid),
    #[error("Invalid role assignment")]
    InvalidRoleAssignment,
}

pub type RbacResult<T> = Result<T, RbacError>;

/// RBAC 管理器
#[derive(Debug, Clone)]
pub struct RbacManager {
    user_roles: Arc<Mutex<HashMap<Uuid, Vec<UserRole>>>>,
    custom_permissions: Arc<Mutex<HashMap<Uuid, Vec<CustomPermission>>>>,
}

impl RbacManager {
    pub fn new() -> Self {
        Self {
            user_roles: Arc::new(Mutex::new(HashMap::new())),
            custom_permissions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 分配角色给用户
    pub fn assign_role(&self, user_role: UserRole) -> RbacResult<()> {
        let mut roles = self.user_roles.lock().unwrap();
        roles
            .entry(user_role.user_id)
            .or_insert_with(Vec::new)
            .push(user_role);
        Ok(())
    }

    /// 撤销用户的角色
    pub fn revoke_role(&self, user_id: Uuid, role: Role, tenant_id: Option<Uuid>) -> RbacResult<()> {
        let mut roles = self.user_roles.lock().unwrap();
        if let Some(user_roles) = roles.get_mut(&user_id) {
            user_roles.retain(|ur| {
                !(ur.role == role && ur.tenant_id == tenant_id)
            });
        }
        Ok(())
    }

    /// 获取用户的所有角色
    pub fn get_user_roles(&self, user_id: Uuid) -> Vec<UserRole> {
        let roles = self.user_roles.lock().unwrap();
        roles
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|ur| ur.is_valid())
            .collect()
    }

    /// 获取用户在特定租户的角色
    pub fn get_user_role_in_tenant(&self, user_id: Uuid, tenant_id: Uuid) -> Option<Role> {
        let roles = self.get_user_roles(user_id);
        roles
            .into_iter()
            .filter(|ur| ur.tenant_id == Some(tenant_id))
            .max_by_key(|ur| ur.role.priority())
            .map(|ur| ur.role)
    }

    /// 获取用户的最高角色
    pub fn get_highest_role(&self, user_id: Uuid) -> Option<Role> {
        let roles = self.get_user_roles(user_id);
        roles
            .into_iter()
            .max_by_key(|ur| ur.role.priority())
            .map(|ur| ur.role)
    }

    /// 授予自定义权限
    pub fn grant_permission(&self, permission: CustomPermission) -> RbacResult<()> {
        let mut perms = self.custom_permissions.lock().unwrap();
        perms
            .entry(permission.user_id)
            .or_insert_with(Vec::new)
            .push(permission);
        Ok(())
    }

    /// 撤销自定义权限
    pub fn revoke_permission(&self, permission_id: Uuid) -> RbacResult<()> {
        let mut perms = self.custom_permissions.lock().unwrap();
        for user_perms in perms.values_mut() {
            user_perms.retain(|p| p.id != permission_id);
        }
        Ok(())
    }

    /// 获取用户的所有自定义权限
    pub fn get_user_permissions(&self, user_id: Uuid) -> Vec<CustomPermission> {
        let perms = self.custom_permissions.lock().unwrap();
        perms
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.is_valid())
            .collect()
    }

    /// 检查用户是否有指定权限
    pub fn check_permission(
        &self,
        user_id: Uuid,
        resource: Resource,
        operation: Operation,
        resource_id: Option<Uuid>,
        tenant_id: Option<Uuid>,
    ) -> RbacResult<()> {
        // 1. 检查角色权限
        let roles = self.get_user_roles(user_id);

        // 如果指定了租户，优先检查租户内的角色
        if let Some(tid) = tenant_id {
            for user_role in &roles {
                if user_role.tenant_id == Some(tid) {
                    let permissions = user_role.role.default_permissions();
                    for perm in permissions {
                        if perm.matches(resource, operation) {
                            return Ok(());
                        }
                    }
                }
            }
        }

        // 检查全局角色
        for user_role in &roles {
            if user_role.tenant_id.is_none() {
                let permissions = user_role.role.default_permissions();
                for perm in permissions {
                    if perm.matches(resource, operation) {
                        return Ok(());
                    }
                }
            }
        }

        // 2. 检查自定义权限
        let custom_perms = self.get_user_permissions(user_id);
        for custom_perm in custom_perms {
            // 如果指定了资源 ID，必须匹配
            if let Some(rid) = resource_id {
                if custom_perm.resource_id.is_some() && custom_perm.resource_id != Some(rid) {
                    continue;
                }
            }

            if custom_perm.permission.matches(resource, operation) {
                return Ok(());
            }
        }

        // 3. 权限检查失败
        Err(RbacError::PermissionDenied {
            user_id,
            resource,
            operation,
        })
    }

    /// 检查用户是否有指定角色
    pub fn has_role(&self, user_id: Uuid, role: Role, tenant_id: Option<Uuid>) -> bool {
        let roles = self.get_user_roles(user_id);
        roles.iter().any(|ur| {
            ur.role == role && ur.tenant_id == tenant_id
        })
    }

    /// 检查用户是否是超级管理员
    pub fn is_super_admin(&self, user_id: Uuid) -> bool {
        self.has_role(user_id, Role::SuperAdmin, None)
    }

    /// 检查用户是否是租户管理员
    pub fn is_tenant_admin(&self, user_id: Uuid, tenant_id: Uuid) -> bool {
        self.has_role(user_id, Role::TenantAdmin, Some(tenant_id))
            || self.is_super_admin(user_id)
    }

    /// 获取所有用户角色（用于管理）
    pub fn list_all_roles(&self) -> Vec<(Uuid, Vec<UserRole>)> {
        let roles = self.user_roles.lock().unwrap();
        roles
            .iter()
            .map(|(user_id, user_roles)| {
                let valid_roles: Vec<UserRole> = user_roles
                    .iter()
                    .filter(|ur| ur.is_valid())
                    .cloned()
                    .collect();
                (*user_id, valid_roles)
            })
            .filter(|(_, roles)| !roles.is_empty())
            .collect()
    }

    /// 清理过期的角色和权限
    pub fn cleanup_expired(&self) {
        // 清理过期角色
        let mut roles = self.user_roles.lock().unwrap();
        for user_roles in roles.values_mut() {
            user_roles.retain(|ur| ur.is_valid());
        }

        // 清理过期权限
        let mut perms = self.custom_permissions.lock().unwrap();
        for user_perms in perms.values_mut() {
            user_perms.retain(|p| p.is_valid());
        }
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}
