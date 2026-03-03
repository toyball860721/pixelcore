use pixelcore_auth::{
    AuditEventType, AuditLog, AuditLogger, Operation, Permission, RbacManager, Resource, Role,
    UserRole, CustomPermission,
};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    println!("=== PixelCore RBAC 系统演示 ===\n");

    // 创建 RBAC 管理器和审计日志器
    let rbac = RbacManager::new();
    let audit = AuditLogger::new(1000);

    // 创建用户
    let super_admin_id = Uuid::new_v4();
    let tenant_admin_id = Uuid::new_v4();
    let developer_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let granted_by = Uuid::new_v4();

    // 创建租户
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();

    println!("1. 角色分配演示");
    println!("================");

    // 分配超级管理员角色
    let super_admin_role = UserRole::new(super_admin_id, Role::SuperAdmin, granted_by);
    rbac.assign_role(super_admin_role.clone()).unwrap();
    audit.log(AuditLog::new(
        granted_by,
        AuditEventType::RoleAssigned {
            target_user_id: super_admin_id,
            role: Role::SuperAdmin,
            tenant_id: None,
        },
    ));
    println!("✓ 分配超级管理员角色给用户 {}", super_admin_id);

    // 分配租户管理员角色
    let tenant_admin_role = UserRole::new(tenant_admin_id, Role::TenantAdmin, granted_by)
        .with_tenant(tenant1);
    rbac.assign_role(tenant_admin_role.clone()).unwrap();
    audit.log(AuditLog::new(
        granted_by,
        AuditEventType::RoleAssigned {
            target_user_id: tenant_admin_id,
            role: Role::TenantAdmin,
            tenant_id: Some(tenant1),
        },
    ));
    println!("✓ 分配租户管理员角色给用户 {} (租户: {})", tenant_admin_id, tenant1);

    // 分配开发者角色
    let developer_role = UserRole::new(developer_id, Role::Developer, granted_by);
    rbac.assign_role(developer_role.clone()).unwrap();
    audit.log(AuditLog::new(
        granted_by,
        AuditEventType::RoleAssigned {
            target_user_id: developer_id,
            role: Role::Developer,
            tenant_id: None,
        },
    ));
    println!("✓ 分配开发者角色给用户 {}", developer_id);

    // 分配普通用户角色
    let user_role = UserRole::new(user_id, Role::User, granted_by);
    rbac.assign_role(user_role.clone()).unwrap();
    audit.log(AuditLog::new(
        granted_by,
        AuditEventType::RoleAssigned {
            target_user_id: user_id,
            role: Role::User,
            tenant_id: None,
        },
    ));
    println!("✓ 分配普通用户角色给用户 {}\n", user_id);

    println!("2. 权限检查演示");
    println!("================");

    // 超级管理员权限检查
    println!("\n超级管理员权限:");
    check_and_log(
        &rbac,
        &audit,
        super_admin_id,
        Resource::Agent,
        Operation::Create,
        None,
        None,
        "创建 Agent",
    );
    check_and_log(
        &rbac,
        &audit,
        super_admin_id,
        Resource::User,
        Operation::Delete,
        None,
        None,
        "删除用户",
    );
    check_and_log(
        &rbac,
        &audit,
        super_admin_id,
        Resource::Billing,
        Operation::Update,
        None,
        None,
        "更新计费",
    );

    // 租户管理员权限检查
    println!("\n租户管理员权限 (租户 {}):", tenant1);
    check_and_log(
        &rbac,
        &audit,
        tenant_admin_id,
        Resource::Agent,
        Operation::Create,
        None,
        Some(tenant1),
        "在租户内创建 Agent",
    );
    check_and_log(
        &rbac,
        &audit,
        tenant_admin_id,
        Resource::Agent,
        Operation::Delete,
        None,
        Some(tenant1),
        "在租户内删除 Agent",
    );
    check_and_log(
        &rbac,
        &audit,
        tenant_admin_id,
        Resource::Agent,
        Operation::Create,
        None,
        Some(tenant2),
        "在其他租户创建 Agent (应该失败)",
    );

    // 开发者权限检查
    println!("\n开发者权限:");
    check_and_log(
        &rbac,
        &audit,
        developer_id,
        Resource::Agent,
        Operation::Create,
        None,
        None,
        "创建 Agent",
    );
    check_and_log(
        &rbac,
        &audit,
        developer_id,
        Resource::Agent,
        Operation::Read,
        None,
        None,
        "读取 Agent",
    );
    check_and_log(
        &rbac,
        &audit,
        developer_id,
        Resource::User,
        Operation::Delete,
        None,
        None,
        "删除用户 (应该失败)",
    );

    // 普通用户权限检查
    println!("\n普通用户权限:");
    check_and_log(
        &rbac,
        &audit,
        user_id,
        Resource::Agent,
        Operation::Read,
        None,
        None,
        "读取 Agent",
    );
    check_and_log(
        &rbac,
        &audit,
        user_id,
        Resource::Transaction,
        Operation::Create,
        None,
        None,
        "创建交易",
    );
    check_and_log(
        &rbac,
        &audit,
        user_id,
        Resource::Agent,
        Operation::Create,
        None,
        None,
        "创建 Agent (应该失败)",
    );

    println!("\n3. 自定义权限演示");
    println!("==================");

    // 授予普通用户对特定 Agent 的更新权限
    let agent_id = Uuid::new_v4();
    let custom_perm = CustomPermission::new(
        user_id,
        Permission::new(Resource::Agent, Operation::Update),
        granted_by,
    )
    .with_resource(agent_id);

    rbac.grant_permission(custom_perm.clone()).unwrap();
    audit.log(AuditLog::new(
        granted_by,
        AuditEventType::PermissionGranted {
            target_user_id: user_id,
            resource: Resource::Agent,
            operation: Operation::Update,
            resource_id: Some(agent_id),
        },
    ));
    println!("✓ 授予用户 {} 对 Agent {} 的更新权限", user_id, agent_id);

    // 检查自定义权限
    check_and_log(
        &rbac,
        &audit,
        user_id,
        Resource::Agent,
        Operation::Update,
        Some(agent_id),
        None,
        "更新特定 Agent",
    );

    let other_agent = Uuid::new_v4();
    check_and_log(
        &rbac,
        &audit,
        user_id,
        Resource::Agent,
        Operation::Update,
        Some(other_agent),
        None,
        "更新其他 Agent (应该失败)",
    );

    println!("\n4. 角色查询演示");
    println!("================");

    // 查询用户角色
    let roles = rbac.get_user_roles(developer_id);
    println!("开发者的角色: {:?}", roles.iter().map(|r| r.role).collect::<Vec<_>>());

    // 查询最高角色
    let highest = rbac.get_highest_role(super_admin_id);
    println!("超级管理员的最高角色: {:?}", highest);

    // 检查特定角色
    println!("用户 {} 是超级管理员吗? {}", super_admin_id, rbac.is_super_admin(super_admin_id));
    println!("用户 {} 是租户 {} 的管理员吗? {}",
        tenant_admin_id, tenant1, rbac.is_tenant_admin(tenant_admin_id, tenant1));

    println!("\n5. 审计日志演示");
    println!("================");

    // 显示审计日志统计
    println!("总审计日志数: {}", audit.count());

    // 显示特定用户的审计日志
    let user_logs = audit.get_user_logs(granted_by);
    println!("用户 {} 的审计日志数: {}", granted_by, user_logs.len());

    // 搜索失败的权限检查
    let failed_checks = audit.search_logs(|log| {
        matches!(log.event_type, AuditEventType::PermissionCheckFailed { .. })
    });
    println!("失败的权限检查数: {}", failed_checks.len());

    // 显示最近的几条审计日志
    println!("\n最近的审计日志:");
    let all_logs = audit.get_all_logs();
    for (i, log) in all_logs.iter().rev().take(5).enumerate() {
        println!("  {}. {:?} - {:?}", i + 1, log.timestamp.format("%H:%M:%S"), log.event_type);
    }

    println!("\n6. 角色撤销演示");
    println!("================");

    // 撤销开发者角色
    rbac.revoke_role(developer_id, Role::Developer, None).unwrap();
    audit.log(AuditLog::new(
        granted_by,
        AuditEventType::RoleRevoked {
            target_user_id: developer_id,
            role: Role::Developer,
            tenant_id: None,
        },
    ));
    println!("✓ 撤销用户 {} 的开发者角色", developer_id);

    // 验证角色已撤销
    check_and_log(
        &rbac,
        &audit,
        developer_id,
        Resource::Agent,
        Operation::Create,
        None,
        None,
        "创建 Agent (应该失败，因为角色已撤销)",
    );

    println!("\n=== 演示完成 ===");
    println!("\n总结:");
    println!("- 成功分配了 4 个角色 (SuperAdmin, TenantAdmin, Developer, User)");
    println!("- 演示了不同角色的权限检查");
    println!("- 演示了租户级别的权限隔离");
    println!("- 演示了自定义权限的授予和检查");
    println!("- 演示了角色查询和撤销");
    println!("- 记录了 {} 条审计日志", audit.count());
}

fn check_and_log(
    rbac: &RbacManager,
    audit: &AuditLogger,
    user_id: Uuid,
    resource: Resource,
    operation: Operation,
    resource_id: Option<Uuid>,
    tenant_id: Option<Uuid>,
    description: &str,
) {
    let result = rbac.check_permission(user_id, resource, operation, resource_id, tenant_id);

    match result {
        Ok(_) => {
            println!("  ✓ {}: 成功", description);
            audit.log(AuditLog::new(
                user_id,
                AuditEventType::PermissionCheckSuccess {
                    resource,
                    operation,
                    resource_id,
                },
            ));
        }
        Err(e) => {
            println!("  ✗ {}: 失败 ({})", description, e);
            audit.log(AuditLog::new(
                user_id,
                AuditEventType::PermissionCheckFailed {
                    resource,
                    operation,
                    resource_id,
                    reason: e.to_string(),
                },
            ));
        }
    }
}
