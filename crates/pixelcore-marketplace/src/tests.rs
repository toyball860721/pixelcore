use super::*;
use pixelcore_registry::{
    AgentListing, AgentRegistry, Capability, PricingModel, ServiceLevel,
};
use uuid::Uuid;
use anyhow::Result;

fn create_test_agent(
    name: &str,
    skills: Vec<&str>,
    price: f64,
    reputation: f64,
    transactions: u64,
) -> AgentListing {
    let capabilities: Vec<Capability> = skills
        .iter()
        .map(|skill| Capability {
            skill_name: skill.to_string(),
            description: format!("{} capability", skill),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
        })
        .collect();

    let pricing = if price == 0.0 {
        PricingModel::Free
    } else {
        PricingModel::PerCall { price }
    };

    let mut agent = AgentListing::new(
        name.to_string(),
        format!("{} agent", name),
        "1.0.0".to_string(),
        Uuid::new_v4(),
        capabilities,
        pricing,
        ServiceLevel {
            response_time_ms: 1000,
            availability_percent: 99.0,
            max_concurrent_requests: 10,
        },
    );

    agent.reputation_score = reputation;
    agent.total_transactions = transactions;
    agent.publish();

    agent
}

#[test]
fn test_service_catalog() -> Result<()> {
    let registry = AgentRegistry::in_memory()?;

    // 创建测试 Agent
    let agent1 = create_test_agent("Calculator", vec!["calculate"], 0.01, 4.5, 100);
    let agent2 = create_test_agent("Translator", vec!["translate"], 0.05, 4.8, 200);
    let agent3 = create_test_agent("DataProcessor", vec!["json_parse", "csv_parse"], 0.02, 4.2, 50);

    registry.register(agent1)?;
    registry.register(agent2)?;
    registry.register(agent3)?;

    let catalog = ServiceCatalog::new(registry);

    // 测试浏览所有服务
    let all = catalog.browse_all(0, 10)?;
    assert_eq!(all.len(), 3);

    // 测试按类别浏览
    let calculators = catalog.browse_by_category("calculate")?;
    assert_eq!(calculators.len(), 1);
    assert_eq!(calculators[0].name, "Calculator");

    // 测试获取热门服务
    let popular = catalog.get_popular(2)?;
    assert_eq!(popular.len(), 2);
    assert_eq!(popular[0].name, "Translator"); // 200 transactions

    // 测试获取高评分服务
    let top_rated = catalog.get_top_rated(2)?;
    assert_eq!(top_rated.len(), 2);
    assert_eq!(top_rated[0].name, "Translator"); // 4.8 rating

    // 测试统计信息
    let stats = catalog.get_statistics()?;
    assert_eq!(stats.total_services, 3);
    assert_eq!(stats.total_transactions, 350);

    Ok(())
}

#[test]
fn test_service_discovery() -> Result<()> {
    let registry = AgentRegistry::in_memory()?;

    // 创建测试 Agent
    let agent1 = create_test_agent("Calculator", vec!["calculate", "convert_units"], 0.01, 4.5, 100);
    let agent2 = create_test_agent("Translator", vec!["translate"], 0.05, 4.8, 200);
    let agent3 = create_test_agent("Free Calculator", vec!["calculate"], 0.0, 3.5, 10);

    registry.register(agent1)?;
    registry.register(agent2)?;
    registry.register(agent3)?;

    let discovery = ServiceDiscovery::new(registry);

    // 测试按技能发现
    let calculators = discovery.discover_by_skill("calculate")?;
    assert_eq!(calculators.len(), 2);

    // 测试发现免费服务
    let free = discovery.discover_free_services()?;
    assert_eq!(free.len(), 1);
    assert_eq!(free[0].name, "Free Calculator");

    // 测试需求匹配
    let requirement = ServiceRequirement {
        required_skills: vec!["calculate".to_string()],
        optional_skills: vec!["convert_units".to_string()],
        max_budget: Some(0.02),
        min_reputation: Some(4.0),
        max_response_time: None,
        min_availability: None,
    };

    let matched = discovery.discover(&requirement)?;
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].name, "Calculator");

    Ok(())
}

#[test]
fn test_smart_matcher() -> Result<()> {
    let agent1 = create_test_agent("Calculator Pro", vec!["calculate", "convert_units"], 0.01, 4.5, 100);
    let agent2 = create_test_agent("Basic Calculator", vec!["calculate"], 0.005, 3.5, 10);
    let agent3 = create_test_agent("Translator", vec!["translate"], 0.05, 4.8, 200);

    let matcher = SmartMatcher::new();

    // 测试匹配分数计算
    let result = matcher.calculate_match_score(
        &agent1,
        &["calculate".to_string()],
        &["convert_units".to_string()],
    );

    assert!(result.score > 0.8); // 应该有很高的匹配分数
    assert!(!result.reasons.is_empty());

    // 测试排序和推荐
    let agents = vec![agent1.clone(), agent2.clone(), agent3.clone()];
    let ranked = matcher.match_and_rank(
        &agents,
        &["calculate".to_string()],
        &["convert_units".to_string()],
    );

    assert_eq!(ranked.len(), 3);
    assert_eq!(ranked[0].0.name, "Calculator Pro"); // 最佳匹配

    // 测试相似度计算
    let similarity = matcher.calculate_similarity(&agent1, &agent2);
    assert!(similarity > 0.0); // 有共同技能

    let similarity2 = matcher.calculate_similarity(&agent1, &agent3);
    assert_eq!(similarity2, 0.0); // 没有共同技能

    // 测试相似推荐
    let similar = matcher.recommend_similar(&agent1, &agents, 2);
    assert!(similar.len() >= 1); // 至少有 agent2 相似

    Ok(())
}
