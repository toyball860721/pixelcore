use super::*;
use chrono::{Duration, Utc};
use uuid::Uuid;

#[test]
fn test_role_default_permissions() {
    // SuperAdmin 应该有所有权限
    let super_admin_perms = Role::SuperAdmin.default_permissions();
    assert!(super_admin_perms.len() > 0);
    assert!(super_admin_perms.contains(&Permission::new(Resource::All, Operation::All)));

    // TenantAdmin 应该有租户管理权限
    let tenant_admin_perms = Role::TenantAdmin.default_permissions();
    assert!(tenant_admin_perms.contains(&Permission::new(Resource::Tenant, Operation::All)));
    assert!(tenant_admin_perms.contains(&Permission::new(Resource::Agent, Operation::All)));

    // Developer 应该有 Agent 管理权限
    let developer_perms = Role::Developer.default_permissions();
    assert!(developer_perms.contains(&Permission::new(Resource::Agent, Operation::Create)));
    assert!(developer_perms.contains(&Permission::new(Resource::Agent, Operation::Read)));

    // User 应该只有基本权限
    let user_perms = Role::User.default_permissions();
    assert!(user_perms.contains(&Permission::new(Resource::Agent, Operation::Read)));
    assert!(!user_perms.contains(&Permission::new(Resource::Agent, Operation::Create)));
}

#[test]
fn test_role_priority() {
    assert!(Role::SuperAdmin.priority() > Role::TenantAdmin.priority());
    assert!(Role::TenantAdmin.priority() > Role::Developer.priority());
    assert!(Role::Developer.priority() > Role::User.priority());
}

#[test]
fn test_permission_matching() {
    // 测试精确匹配
    let perm = Permission::new(Resource::Agent, Operation::Read);
    assert!(perm.matches(Resource::Agent, Operation::Read));
    assert!(!perm.matches(Resource::Agent, Operation::Create));
    assert!(!perm.matches(Resource::User, Operation::Read));

    // 测试 All 资源匹配
    let perm_all_resource = Permission::new(Resource::All, Operation::Read);
    assert!(perm_all_resource.matches(Resource::Agent, Operation::Read));
    assert!(perm_all_resource.matches(Resource::User, Operation::Read));
    assert!(!perm_all_resource.matches(Resource::Agent, Operation::Create));

    // 测试 All 操作匹配
    let perm_all_operation = Permission::new(Resource::Agent, Operation::All);
    assert!(perm_all_operation.matches(Resource::Agent, Operation::Read));
    assert!(perm_all_operation.matches(Resource::Agent, Operation::Create));
    assert!(!perm_all_operation.matches(Resource::User, Operation::Read));

    // 测试 All 资源和操作匹配
    let perm_all = Permission::new(Resource::All, Operation::All);
    assert!(perm_all.matches(Resource::Agent, Operation::Read));
    assert!(perm_all.matches(Resource::User, Operation::Create));
}

#[test]
fn test_user_role_expiration() {
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();

    // 测试未过期的角色
    let role = UserRole::new(user_id, Role::Developer, granted_by)
        .with_expiry(Utc::now() + Duration::hours(1));
    assert!(!role.is_expired());
    assert!(role.is_valid());

    // 测试已过期的角色
    let expired_role = UserRole::new(user_id, Role::Developer, granted_by)
        .with_expiry(Utc::now() - Duration::hours(1));
    assert!(expired_role.is_expired());
    assert!(!expired_role.is_valid());

    // 测试永不过期的角色
    let permanent_role = UserRole::new(user_id, Role::Developer, granted_by);
    assert!(!permanent_role.is_expired());
    assert!(permanent_role.is_valid());
}

#[test]
fn test_rbac_assign_and_revoke_role() {
    let rbac = RbacManager::new();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();

    // 分配角色
    let user_role = UserRole::new(user_id, Role::Developer, granted_by);
    rbac.assign_role(user_role).unwrap();

    // 验证角色已分配
    let roles = rbac.get_user_roles(user_id);
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].role, Role::Developer);

    // 撤销角色
    rbac.revoke_role(user_id, Role::Developer, None).unwrap();

    // 验证角色已撤销
    let roles = rbac.get_user_roles(user_id);
    assert_eq!(roles.len(), 0);
}

#[test]
fn test_rbac_multiple_roles() {
    let rbac = RbacManager::new();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    // 分配多个角色
    rbac.assign_role(UserRole::new(user_id, Role::User, granted_by)).unwrap();
    rbac.assign_role(
        UserRole::new(user_id, Role::TenantAdmin, granted_by).with_tenant(tenant_id)
    ).unwrap();

    // 验证角色
    let roles = rbac.get_user_roles(user_id);
    assert_eq!(roles.len(), 2);

    // 获取最高角色
    let highest = rbac.get_highest_role(user_id);
    assert_eq!(highest, Some(Role::TenantAdmin));

    // 获取租户内的角色
    let tenant_role = rbac.get_user_role_in_tenant(user_id, tenant_id);
    assert_eq!(tenant_role, Some(Role::TenantAdmin));
}

#[test]
fn test_rbac_permission_check_with_role() {
    let rbac = RbacManager::new();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();

    // 分配 Developer 角色
    rbac.assign_role(UserRole::new(user_id, Role::Developer, granted_by)).unwrap();

    // Developer 应该可以创建 Agent
    assert!(rbac.check_permission(
        user_id,
        Resource::Agent,
        Operation::Create,
        None,
        None
    ).is_ok());

    // Developer 不应该可以删除用户
    assert!(rbac.check_permission(
        user_id,
        Resource::User,
        Operation::Delete,
        None,
        None
    ).is_err());
}

#[test]
fn test_rbac_super_admin_permissions() {
    let rbac = RbacManager::new();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();

    // 分配 SuperAdmin 角色
    rbac.assign_role(UserRole::new(user_id, Role::SuperAdmin, granted_by)).unwrap();

    // SuperAdmin 应该有所有权限
    assert!(rbac.check_permission(
        user_id,
        Resource::Agent,
        Operation::Create,
        None,
        None
    ).is_ok());

    assert!(rbac.check_permission(
        user_id,
        Resource::User,
        Operation::Delete,
        None,
        None
    ).is_ok());

    assert!(rbac.check_permission(
        user_id,
        Resource::Billing,
        Operation::Update,
        None,
        None
    ).is_ok());

    // 验证 is_super_admin
    assert!(rbac.is_super_admin(user_id));
}

#[test]
fn test_rbac_tenant_level_permissions() {
    let rbac = RbacManager::new();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    // 在 tenant1 中分配 TenantAdmin 角色
    rbac.assign_role(
        UserRole::new(user_id, Role::TenantAdmin, granted_by).with_tenant(tenant1)
    ).unwrap();

    // 在 tenant1 中应该有权限
    assert!(rbac.check_permission(
        user_id,
        Resource::Agent,
        Operation::Create,
        None,
        Some(tenant1)
    ).is_ok());

    // 在 tenant2 中不应该有权限
    assert!(rbac.check_permission(
        user_id,
        Resource::Agent,
        Operation::Create,
        None,
        Some(tenant2)
    ).is_err());

    // 验证 is_tenant_admin
    assert!(rbac.is_tenant_admin(user_id, tenant1));
    assert!(!rbac.is_tenant_admin(user_id, tenant2));
}

#[test]
fn test_rbac_custom_permissions() {
    let rbac = RbacManager::new();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();
    let resource_id = Uuid::new_v4();

    // 授予自定义权限
    let custom_perm = CustomPermission::new(
        user_id,
        Permission::new(Resource::Agent, Operation::Update),
        granted_by,
    ).with_resource(resource_id);

    rbac.grant_permission(custom_perm.clone()).unwrap();

    // 对特定资源应该有权限
    assert!(rbac.check_permission(
        user_id,
        Resource::Agent,
        Operation::Update,
        Some(resource_id),
        None
    ).is_ok());

    // 对其他资源不应该有权限
    let other_resource = Uuid::new_v4();
    assert!(rbac.check_permission(
        user_id,
        Resource::Agent,
        Operation::Update,
        Some(other_resource),
        None
    ).is_err());

    // 撤销权限
    rbac.revoke_permission(custom_perm.id).unwrap();

    // 权限应该被撤销
    assert!(rbac.check_permission(
        user_id,
        Resource::Agent,
        Operation::Update,
        Some(resource_id),
        None
    ).is_err());
}

#[test]
fn test_rbac_cleanup_expired() {
    let rbac = RbacManager::new();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();

    // 添加已过期的角色
    let expired_role = UserRole::new(user_id, Role::Developer, granted_by)
        .with_expiry(Utc::now() - Duration::hours(1));
    rbac.assign_role(expired_role).unwrap();

    // 添加有效的角色
    let valid_role = UserRole::new(user_id, Role::User, granted_by);
    rbac.assign_role(valid_role).unwrap();

    // 清理前应该有 1 个有效角色（过期的会被过滤）
    let roles_before = rbac.get_user_roles(user_id);
    assert_eq!(roles_before.len(), 1);

    // 执行清理
    rbac.cleanup_expired();

    // 清理后应该只有 1 个有效角色
    let roles_after = rbac.get_user_roles(user_id);
    assert_eq!(roles_after.len(), 1);
    assert_eq!(roles_after[0].role, Role::User);
}

#[test]
fn test_audit_logger_basic() {
    let logger = AuditLogger::new(100);
    let user_id = Uuid::new_v4();

    // 记录审计日志
    let log = AuditLog::new(
        user_id,
        AuditEventType::RoleAssigned {
            target_user_id: Uuid::new_v4(),
            role: Role::Developer,
            tenant_id: None,
        },
    );
    logger.log(log);

    // 验证日志已记录
    assert_eq!(logger.count(), 1);

    // 获取用户日志
    let user_logs = logger.get_user_logs(user_id);
    assert_eq!(user_logs.len(), 1);
}

#[test]
fn test_audit_logger_max_logs() {
    let logger = AuditLogger::new(5);
    let user_id = Uuid::new_v4();

    // 记录 10 条日志
    for _ in 0..10 {
        let log = AuditLog::new(
            user_id,
            AuditEventType::PermissionCheckSuccess {
                resource: Resource::Agent,
                operation: Operation::Read,
                resource_id: None,
            },
        );
        logger.log(log);
    }

    // 应该只保留最后 5 条
    assert_eq!(logger.count(), 5);
}

#[test]
fn test_audit_logger_time_range() {
    let logger = AuditLogger::new(100);
    let user_id = Uuid::new_v4();

    let now = Utc::now();

    // 记录日志
    let log = AuditLog::new(
        user_id,
        AuditEventType::PermissionCheckSuccess {
            resource: Resource::Agent,
            operation: Operation::Read,
            resource_id: None,
        },
    );
    logger.log(log);

    // 查询时间范围内的日志
    let logs = logger.get_logs_in_range(
        now - Duration::minutes(1),
        now + Duration::minutes(1),
    );
    assert_eq!(logs.len(), 1);

    // 查询时间范围外的日志
    let logs = logger.get_logs_in_range(
        now - Duration::hours(2),
        now - Duration::hours(1),
    );
    assert_eq!(logs.len(), 0);
}

#[test]
fn test_audit_logger_search() {
    let logger = AuditLogger::new(100);
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();

    // 记录不同用户的日志
    logger.log(AuditLog::new(
        user1,
        AuditEventType::RoleAssigned {
            target_user_id: Uuid::new_v4(),
            role: Role::Developer,
            tenant_id: None,
        },
    ));

    logger.log(AuditLog::new(
        user2,
        AuditEventType::PermissionCheckFailed {
            resource: Resource::Agent,
            operation: Operation::Delete,
            resource_id: None,
            reason: "No permission".to_string(),
        },
    ));

    // 搜索特定类型的日志
    let failed_checks = logger.search_logs(|log| {
        matches!(log.event_type, AuditEventType::PermissionCheckFailed { .. })
    });
    assert_eq!(failed_checks.len(), 1);
}

#[test]
fn test_audit_logger_clear() {
    let logger = AuditLogger::new(100);
    let user_id = Uuid::new_v4();

    // 记录日志
    logger.log(AuditLog::new(
        user_id,
        AuditEventType::PermissionCheckSuccess {
            resource: Resource::Agent,
            operation: Operation::Read,
            resource_id: None,
        },
    ));

    assert_eq!(logger.count(), 1);

    // 清空日志
    logger.clear();
    assert_eq!(logger.count(), 0);
}
