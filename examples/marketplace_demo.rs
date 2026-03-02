use pixelcore_registry::{
    AgentListing, AgentRegistry, Capability, PricingModel, ServiceLevel,
};
use pixelcore_marketplace::{
    ServiceCatalog, ServiceDiscovery, ServiceRequirement, SmartMatcher,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Marketplace Service Discovery Demo ===\n");

    // 创建注册表
    let registry = AgentRegistry::in_memory()?;
    println!("✅ Registry created\n");

    // 注册多个 Agent
    println!("--- Registering Agents ---");

    let agents_data = vec![
        ("Calculator Pro", vec!["calculate", "convert_units"], 0.01, 4.5, 150),
        ("Basic Calculator", vec!["calculate"], 0.005, 3.8, 50),
        ("Universal Translator", vec!["translate"], 0.05, 4.8, 300),
        ("Data Processor", vec!["json_parse", "csv_parse", "xml_parse"], 0.02, 4.3, 120),
        ("Free Echo Service", vec!["echo"], 0.0, 4.0, 80),
        ("Advanced Translator", vec!["translate", "detect_language"], 0.08, 4.9, 500),
    ];

    for (name, skills, price, reputation, transactions) in &agents_data {
        let capabilities: Vec<Capability> = skills
            .iter()
            .map(|skill| Capability {
                skill_name: skill.to_string(),
                description: format!("{} capability", skill),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({"type": "object"}),
            })
            .collect();

        let pricing = if *price == 0.0 {
            PricingModel::Free
        } else {
            PricingModel::PerCall { price: *price }
        };

        let mut agent = AgentListing::new(
            name.to_string(),
            format!("A {} service", name),
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

        agent.reputation_score = *reputation;
        agent.total_transactions = *transactions;

        let id = registry.register(agent)?;
        registry.publish(&id)?;
        println!("  ✅ {} registered", name);
    }
    println!();

    // 场景 1: 服务目录浏览
    println!("--- Scenario 1: Browse Service Catalog ---");
    let catalog = ServiceCatalog::new(registry);

    let all_services = catalog.browse_all(0, 10)?;
    println!("Total services: {}", all_services.len());

    let stats = catalog.get_statistics()?;
    println!("Statistics:");
    println!("  Total transactions: {}", stats.total_transactions);
    println!("  Average reputation: {:.2}/5.0", stats.average_reputation);
    println!("  Categories: {:?}", stats.categories);
    println!();

    // 场景 2: 获取热门和高评分服务
    println!("--- Scenario 2: Popular & Top Rated Services ---");

    let popular = catalog.get_popular(3)?;
    println!("Top 3 Popular Services:");
    for (i, agent) in popular.iter().enumerate() {
        println!("  {}. {} - {} transactions", i + 1, agent.name, agent.total_transactions);
    }
    println!();

    let top_rated = catalog.get_top_rated(3)?;
    println!("Top 3 Rated Services:");
    for (i, agent) in top_rated.iter().enumerate() {
        println!("  {}. {} - {:.1}/5.0", i + 1, agent.name, agent.reputation_score);
    }
    println!();

    // 场景 3: 服务发现
    println!("--- Scenario 3: Service Discovery ---");
    // 创建新的 registry 用于 discovery (因为 catalog 已经拥有了 registry)
    let registry2 = AgentRegistry::in_memory()?;
    for (name, skills, price, reputation, transactions) in &agents_data {
        let capabilities: Vec<Capability> = skills
            .iter()
            .map(|skill| Capability {
                skill_name: skill.to_string(),
                description: format!("{} capability", skill),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({"type": "object"}),
            })
            .collect();

        let mut agent = AgentListing::new(
            name.to_string(),
            format!("A {} service", name),
            "1.0.0".to_string(),
            Uuid::new_v4(),
            capabilities,
            if *price == 0.0 {
                PricingModel::Free
            } else {
                PricingModel::PerCall { price: *price }
            },
            ServiceLevel {
                response_time_ms: 1000,
                availability_percent: 99.0,
                max_concurrent_requests: 10,
            },
        );

        agent.reputation_score = *reputation;
        agent.total_transactions = *transactions;

        let id = registry2.register(agent)?;
        registry2.publish(&id)?;
    }

    let discovery = ServiceDiscovery::new(registry2);

    // 按技能发现
    let calculators = discovery.discover_by_skill("calculate")?;
    println!("Services with 'calculate' skill: {}", calculators.len());
    for agent in &calculators {
        println!("  - {} (${:.3}/call)", agent.name, match &agent.pricing {
            PricingModel::PerCall { price } => *price,
            PricingModel::Free => 0.0,
            _ => 0.0,
        });
    }
    println!();

    // 发现免费服务
    let free_services = discovery.discover_free_services()?;
    println!("Free services: {}", free_services.len());
    for agent in &free_services {
        println!("  - {}", agent.name);
    }
    println!();

    // 场景 4: 需求匹配
    println!("--- Scenario 4: Requirement Matching ---");
    let requirement = ServiceRequirement {
        required_skills: vec!["translate".to_string()],
        optional_skills: vec!["detect_language".to_string()],
        max_budget: Some(0.1),
        min_reputation: Some(4.5),
        max_response_time: None,
        min_availability: None,
    };

    let matched = discovery.discover(&requirement)?;
    println!("Matched services for translation requirement:");
    println!("  Required: translate");
    println!("  Optional: detect_language");
    println!("  Max budget: $0.1");
    println!("  Min reputation: 4.5");
    println!();
    println!("Found {} matching services:", matched.len());
    for agent in &matched {
        println!("  - {} - {:.1}/5.0", agent.name, agent.reputation_score);
    }
    println!();

    // 场景 5: 智能匹配和推荐
    println!("--- Scenario 5: Smart Matching & Recommendation ---");
    let matcher = SmartMatcher::new();

    // 使用 catalog 的 registry 获取 agents
    let all_agents = catalog.browse_all(0, 100)?;
    let ranked = matcher.match_and_rank(
        &all_agents,
        &["calculate".to_string()],
        &["convert_units".to_string()],
    );

    println!("Smart matching for calculation services:");
    println!("  Required: calculate");
    println!("  Optional: convert_units");
    println!();
    println!("Top 3 recommendations:");
    for (i, (agent, result)) in ranked.iter().take(3).enumerate() {
        println!("  {}. {} - Score: {:.2}", i + 1, agent.name, result.score);
        println!("     Reasons: {}", result.reasons.join(", "));
    }
    println!();

    // 场景 6: 相似服务推荐
    println!("--- Scenario 6: Similar Service Recommendation ---");
    if let Some(target_agent) = all_agents.iter().find(|a| a.name == "Calculator Pro") {
        let similar = matcher.recommend_similar(target_agent, &all_agents, 3);
        println!("Services similar to 'Calculator Pro':");
        for (agent, similarity) in &similar {
            println!("  - {} - Similarity: {:.2}", agent.name, similarity);
        }
    }
    println!();

    println!("=== Marketplace Service Discovery Demo Complete ===");
    Ok(())
}
