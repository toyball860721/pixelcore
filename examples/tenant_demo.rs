use pixelcore_tenant::{TenantManager, TenantConfig, IsolationLevel};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Tenant System Demo ===\n");

    let manager = TenantManager::new();

    // Demo 1: 创建租户
    println!("--- Demo 1: Create Tenants ---");
    let owner1_id = Uuid::new_v4();
    let owner2_id = Uuid::new_v4();

    let tenant1 = manager
        .create_tenant(
            "Acme Corp".to_string(),
            "A software company".to_string(),
            owner1_id,
            None,
        )
        .await?;

    println!("✓ Created tenant: {} (ID: {})", tenant1.name, tenant1.id);
    println!("  Owner: {}", tenant1.owner_id);
    println!("  Status: {:?}", tenant1.status);
    println!("  Max Agents: {}", tenant1.config.max_agents);
    println!("  Max Storage: {} GB", tenant1.config.max_storage_gb);
    println!("  Max API Calls: {}/month\n", tenant1.config.max_api_calls_per_month);

    // 创建企业租户 (更高配额)
    let mut enterprise_config = TenantConfig::default();
    enterprise_config.max_agents = 100;
    enterprise_config.max_storage_gb = 1000.0;
    enterprise_config.max_api_calls_per_month = 1000000;
    enterprise_config.enable_advanced_features = true;

    let tenant2 = manager
        .create_tenant(
            "Enterprise Inc".to_string(),
            "An enterprise customer".to_string(),
            owner2_id,
            Some(enterprise_config),
        )
        .await?;

    println!("✓ Created enterprise tenant: {}", tenant2.name);
    println!("  Max Agents: {}", tenant2.config.max_agents);
    println!("  Max Storage: {} GB", tenant2.config.max_storage_gb);
    println!("  Advanced Features: {}\n", tenant2.config.enable_advanced_features);

    // Demo 2: 资源使用
    println!("--- Demo 2: Resource Usage ---");

    // 添加 Agents
    manager.add_agent(tenant1.id).await?;
    manager.add_agent(tenant1.id).await?;
    manager.add_agent(tenant1.id).await?;

    println!("✓ Added 3 agents to {}", tenant1.name);

    // 添加存储
    manager.add_storage(tenant1.id, 2.5).await?;
    manager.add_storage(tenant1.id, 1.5).await?;

    println!("✓ Added 4.0 GB storage");

    // 记录 API 调用
    for _ in 0..150 {
        manager.record_api_call(tenant1.id).await?;
    }

    println!("✓ Recorded 150 API calls");

    // 查看使用情况
    let usage = manager.get_tenant_usage(tenant1.id).await.unwrap();
    println!("\nCurrent Usage:");
    println!("  Agents: {}/{}", usage.current_agents, tenant1.config.max_agents);
    println!("  Storage: {:.1}/{} GB", usage.current_storage_gb, tenant1.config.max_storage_gb);
    println!("  API Calls: {}/{}\n", usage.current_api_calls, tenant1.config.max_api_calls_per_month);

    // Demo 3: 配额检查
    println!("--- Demo 3: Quota Enforcement ---");

    // 尝试超出配额
    println!("Attempting to exceed agent quota...");
    for i in 0..10 {
        match manager.add_agent(tenant1.id).await {
            Ok(_) => println!("  ✓ Added agent {}", i + 4),
            Err(e) => {
                println!("  ✗ Failed: {}", e);
                break;
            }
        }
    }
    println!();

    // Demo 4: 成员管理
    println!("--- Demo 4: Member Management ---");

    let dev1_id = Uuid::new_v4();
    let dev2_id = Uuid::new_v4();
    let viewer_id = Uuid::new_v4();

    // 添加成员
    manager.add_member(tenant1.id, dev1_id, "developer".to_string()).await?;
    manager.add_member(tenant1.id, dev2_id, "developer".to_string()).await?;
    manager.add_member(tenant1.id, viewer_id, "viewer".to_string()).await?;

    println!("✓ Added 3 members to {}", tenant1.name);

    let members = manager.get_tenant_members(tenant1.id).await;
    println!("\nTenant Members ({}):", members.len());
    for member in &members {
        println!("  - User {} ({})", member.user_id, member.role);
    }
    println!();

    // Demo 5: 数据隔离
    println!("--- Demo 5: Data Isolation ---");

    let isolation1 = manager.get_isolation(tenant1.id).await.unwrap();
    let isolation2 = manager.get_isolation(tenant2.id).await.unwrap();

    println!("Tenant 1 Isolation:");
    println!("  Level: {:?}", isolation1.isolation_level);
    println!("  Table: users -> {}", isolation1.get_table_name("users"));
    println!("  Table: agents -> {}", isolation1.get_table_name("agents"));

    println!("\nTenant 2 Isolation:");
    println!("  Level: {:?}", isolation2.isolation_level);
    println!("  Table: users -> {}", isolation2.get_table_name("users"));
    println!("  Table: agents -> {}\n", isolation2.get_table_name("agents"));

    // Demo 6: 租户暂停和激活
    println!("--- Demo 6: Tenant Suspension ---");

    println!("Suspending tenant: {}", tenant1.name);
    manager.suspend_tenant(tenant1.id).await?;

    let suspended = manager.get_tenant(tenant1.id).await.unwrap();
    println!("✓ Tenant status: {:?}", suspended.status);

    // 尝试在暂停状态下添加 Agent
    let result = manager.add_agent(tenant1.id).await;
    if result.is_err() {
        println!("✗ Cannot add agent: Tenant is suspended");
    }

    // 重新激活
    println!("\nReactivating tenant...");
    manager.activate_tenant(tenant1.id).await?;

    let activated = manager.get_tenant(tenant1.id).await.unwrap();
    println!("✓ Tenant status: {:?}\n", activated.status);

    // Demo 7: 用户的租户列表
    println!("--- Demo 7: User's Tenants ---");

    // 将 dev1 添加到 tenant2
    manager.add_member(tenant2.id, dev1_id, "developer".to_string()).await?;

    let dev1_tenants = manager.get_user_tenants(dev1_id).await;
    println!("User {} is member of {} tenant(s):", dev1_id, dev1_tenants.len());
    for tenant in &dev1_tenants {
        println!("  - {} ({})", tenant.name, tenant.id);
    }
    println!();

    // Demo 8: 月度重置
    println!("--- Demo 8: Monthly Reset ---");

    let usage_before = manager.get_tenant_usage(tenant1.id).await.unwrap();
    println!("API calls before reset: {}", usage_before.current_api_calls);

    manager.reset_monthly_usage(tenant1.id).await?;

    let usage_after = manager.get_tenant_usage(tenant1.id).await.unwrap();
    println!("API calls after reset: {}", usage_after.current_api_calls);
    println!("✓ Monthly usage reset\n");

    // Demo 9: 活跃租户列表
    println!("--- Demo 9: Active Tenants ---");

    let active_tenants = manager.get_active_tenants().await;
    println!("Active tenants: {}", active_tenants.len());
    for tenant in &active_tenants {
        println!("  - {} ({})", tenant.name, tenant.id);
    }
    println!();

    // Demo 10: 配置更新
    println!("--- Demo 10: Update Configuration ---");

    let mut new_config = tenant1.config.clone();
    new_config.max_agents = 20;
    new_config.max_storage_gb = 50.0;

    manager.update_tenant_config(tenant1.id, new_config).await?;

    let updated = manager.get_tenant(tenant1.id).await.unwrap();
    println!("✓ Updated {} configuration:", updated.name);
    println!("  Max Agents: {}", updated.config.max_agents);
    println!("  Max Storage: {} GB", updated.config.max_storage_gb);

    println!("\n=== Demo Complete ===");

    Ok(())
}
