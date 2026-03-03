use super::*;
use uuid::Uuid;

#[tokio::test]
async fn test_create_tenant() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    assert_eq!(tenant.name, "Test Tenant");
    assert_eq!(tenant.owner_id, owner_id);
    assert_eq!(tenant.status, TenantStatus::Active);
}

#[tokio::test]
async fn test_get_tenant() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    let retrieved = manager.get_tenant(tenant.id).await.unwrap();
    assert_eq!(retrieved.id, tenant.id);
    assert_eq!(retrieved.name, tenant.name);
}

#[tokio::test]
async fn test_suspend_and_activate_tenant() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Suspend
    manager.suspend_tenant(tenant.id).await.unwrap();
    let suspended = manager.get_tenant(tenant.id).await.unwrap();
    assert_eq!(suspended.status, TenantStatus::Suspended);

    // Activate
    manager.activate_tenant(tenant.id).await.unwrap();
    let activated = manager.get_tenant(tenant.id).await.unwrap();
    assert_eq!(activated.status, TenantStatus::Active);
}

#[tokio::test]
async fn test_tenant_config() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let mut config = TenantConfig::default();
    config.max_agents = 50;
    config.max_storage_gb = 100.0;

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            Some(config.clone()),
        )
        .await
        .unwrap();

    assert_eq!(tenant.config.max_agents, 50);
    assert_eq!(tenant.config.max_storage_gb, 100.0);
}

#[tokio::test]
async fn test_add_agent() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Add agent
    manager.add_agent(tenant.id).await.unwrap();

    let usage = manager.get_tenant_usage(tenant.id).await.unwrap();
    assert_eq!(usage.current_agents, 1);
}

#[tokio::test]
async fn test_agent_quota_exceeded() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let mut config = TenantConfig::default();
    config.max_agents = 2;

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            Some(config),
        )
        .await
        .unwrap();

    // Add 2 agents (should succeed)
    manager.add_agent(tenant.id).await.unwrap();
    manager.add_agent(tenant.id).await.unwrap();

    // Try to add 3rd agent (should fail)
    let result = manager.add_agent(tenant.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_storage_usage() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Add storage
    manager.add_storage(tenant.id, 5.0).await.unwrap();

    let usage = manager.get_tenant_usage(tenant.id).await.unwrap();
    assert_eq!(usage.current_storage_gb, 5.0);
}

#[tokio::test]
async fn test_api_call_tracking() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Record API calls
    manager.record_api_call(tenant.id).await.unwrap();
    manager.record_api_call(tenant.id).await.unwrap();

    let usage = manager.get_tenant_usage(tenant.id).await.unwrap();
    assert_eq!(usage.current_api_calls, 2);
}

#[tokio::test]
async fn test_add_member() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Add member
    let member = manager
        .add_member(tenant.id, user_id, "developer".to_string())
        .await
        .unwrap();

    assert_eq!(member.user_id, user_id);
    assert_eq!(member.role, "developer");

    // Check membership
    assert!(manager.is_member(tenant.id, user_id).await);
}

#[tokio::test]
async fn test_get_tenant_members() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();
    let user1_id = Uuid::new_v4();
    let user2_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Add members
    manager
        .add_member(tenant.id, user1_id, "developer".to_string())
        .await
        .unwrap();
    manager
        .add_member(tenant.id, user2_id, "viewer".to_string())
        .await
        .unwrap();

    let members = manager.get_tenant_members(tenant.id).await;
    // Should have 3 members (owner + 2 added)
    assert_eq!(members.len(), 3);
}

#[tokio::test]
async fn test_remove_member() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Add and remove member
    manager
        .add_member(tenant.id, user_id, "developer".to_string())
        .await
        .unwrap();

    manager.remove_member(tenant.id, user_id).await.unwrap();

    assert!(!manager.is_member(tenant.id, user_id).await);
}

#[tokio::test]
async fn test_get_user_tenants() {
    let manager = TenantManager::new();
    let user_id = Uuid::new_v4();

    // Create tenant 1 with user as owner
    let tenant1 = manager
        .create_tenant(
            "Tenant 1".to_string(),
            "First tenant".to_string(),
            user_id,
            None,
        )
        .await
        .unwrap();

    // Create tenant 2 with different owner, add user as member
    let other_owner = Uuid::new_v4();
    let tenant2 = manager
        .create_tenant(
            "Tenant 2".to_string(),
            "Second tenant".to_string(),
            other_owner,
            None,
        )
        .await
        .unwrap();

    manager
        .add_member(tenant2.id, user_id, "developer".to_string())
        .await
        .unwrap();

    // Get user's tenants
    let user_tenants = manager.get_user_tenants(user_id).await;
    assert_eq!(user_tenants.len(), 2);
}

#[tokio::test]
async fn test_tenant_isolation() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    let isolation = manager.get_isolation(tenant.id).await.unwrap();
    assert_eq!(isolation.isolation_level, IsolationLevel::SeparateTable);

    // Test table name generation
    let table_name = isolation.get_table_name("users");
    assert!(table_name.starts_with("tenant_"));
    assert!(table_name.ends_with("_users"));
}

#[tokio::test]
async fn test_reset_monthly_usage() {
    let manager = TenantManager::new();
    let owner_id = Uuid::new_v4();

    let tenant = manager
        .create_tenant(
            "Test Tenant".to_string(),
            "A test tenant".to_string(),
            owner_id,
            None,
        )
        .await
        .unwrap();

    // Record some API calls
    manager.record_api_call(tenant.id).await.unwrap();
    manager.record_api_call(tenant.id).await.unwrap();

    // Reset
    manager.reset_monthly_usage(tenant.id).await.unwrap();

    let usage = manager.get_tenant_usage(tenant.id).await.unwrap();
    assert_eq!(usage.current_api_calls, 0);
}
