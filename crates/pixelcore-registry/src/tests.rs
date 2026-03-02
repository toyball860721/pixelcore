use super::*;
use uuid::Uuid;
use anyhow::Result;

#[test]
fn test_agent_listing_creation() {
    let owner_id = Uuid::new_v4();
    let capabilities = vec![
        Capability {
            skill_name: "calculate".to_string(),
            description: "Perform calculations".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({"type": "number"}),
        },
    ];

    let pricing = PricingModel::PerCall { price: 0.01 };
    let sla = ServiceLevel {
        response_time_ms: 1000,
        availability_percent: 99.9,
        max_concurrent_requests: 10,
    };

    let listing = AgentListing::new(
        "Calculator Agent".to_string(),
        "A simple calculator agent".to_string(),
        "1.0.0".to_string(),
        owner_id,
        capabilities,
        pricing,
        sla,
    );

    assert_eq!(listing.name, "Calculator Agent");
    assert_eq!(listing.status, AgentStatus::Draft);
    assert_eq!(listing.reputation_score, 0.0);
}

#[test]
fn test_agent_registry() -> Result<()> {
    let registry = AgentRegistry::in_memory()?;

    // 创建 Agent
    let owner_id = Uuid::new_v4();
    let capabilities = vec![
        Capability {
            skill_name: "echo".to_string(),
            description: "Echo back the input".to_string(),
            input_schema: serde_json::json!({"type": "string"}),
            output_schema: serde_json::json!({"type": "string"}),
        },
    ];

    let listing = AgentListing::new(
        "Echo Agent".to_string(),
        "An agent that echoes input".to_string(),
        "1.0.0".to_string(),
        owner_id,
        capabilities,
        PricingModel::Free,
        ServiceLevel {
            response_time_ms: 100,
            availability_percent: 99.9,
            max_concurrent_requests: 100,
        },
    );

    // 注册 Agent
    let agent_id = registry.register(listing)?;

    // 获取 Agent
    let retrieved = registry.get(&agent_id)?;
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.name, "Echo Agent");
    assert_eq!(retrieved.status, AgentStatus::Draft);

    // 发布 Agent
    registry.publish(&agent_id)?;
    let published = registry.get(&agent_id)?.unwrap();
    assert_eq!(published.status, AgentStatus::Published);

    // 暂停 Agent
    registry.pause(&agent_id)?;
    let paused = registry.get(&agent_id)?.unwrap();
    assert_eq!(paused.status, AgentStatus::Paused);

    // 下架 Agent
    registry.archive(&agent_id)?;
    let archived = registry.get(&agent_id)?.unwrap();
    assert_eq!(archived.status, AgentStatus::Archived);

    // 删除 Agent
    let deleted = registry.delete(&agent_id)?;
    assert!(deleted);

    let not_found = registry.get(&agent_id)?;
    assert!(not_found.is_none());

    Ok(())
}

#[test]
fn test_agent_search() -> Result<()> {
    let registry = AgentRegistry::in_memory()?;

    // 创建多个 Agent
    let owner1 = Uuid::new_v4();
    let owner2 = Uuid::new_v4();

    let agent1 = AgentListing::new(
        "Calculator".to_string(),
        "Math agent".to_string(),
        "1.0.0".to_string(),
        owner1,
        vec![Capability {
            skill_name: "calculate".to_string(),
            description: "Calculate".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
        }],
        PricingModel::PerCall { price: 0.01 },
        ServiceLevel {
            response_time_ms: 1000,
            availability_percent: 99.0,
            max_concurrent_requests: 10,
        },
    );

    let agent2 = AgentListing::new(
        "Translator".to_string(),
        "Translation agent".to_string(),
        "1.0.0".to_string(),
        owner2,
        vec![Capability {
            skill_name: "translate".to_string(),
            description: "Translate".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
        }],
        PricingModel::Free,
        ServiceLevel {
            response_time_ms: 2000,
            availability_percent: 95.0,
            max_concurrent_requests: 5,
        },
    );

    let id1 = registry.register(agent1)?;
    let id2 = registry.register(agent2)?;

    registry.publish(&id1)?;
    registry.publish(&id2)?;

    // 按名称搜索
    let filter = AgentFilter {
        name: Some("calc".to_string()),
        ..Default::default()
    };
    let results = registry.search(&filter)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Calculator");

    // 按所有者搜索
    let filter = AgentFilter {
        owner_id: Some(owner1),
        ..Default::default()
    };
    let results = registry.search(&filter)?;
    assert_eq!(results.len(), 1);

    // 按技能搜索
    let filter = AgentFilter {
        skill_name: Some("translate".to_string()),
        ..Default::default()
    };
    let results = registry.search(&filter)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Translator");

    Ok(())
}
