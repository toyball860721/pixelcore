use pixelcore_registry::{
    AgentListing, AgentRegistry, AgentFilter,
    Capability, PricingModel, ServiceLevel,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Agent Registry Demo ===\n");

    // 创建注册表
    let registry = AgentRegistry::in_memory()?;
    println!("✅ Agent Registry created\n");

    // 场景 1: 注册一个计算器 Agent
    println!("--- Scenario 1: Register Calculator Agent ---");
    let owner_id = Uuid::new_v4();

    let calculator_agent = AgentListing::new(
        "Calculator Pro".to_string(),
        "Professional calculation agent with advanced math capabilities".to_string(),
        "1.0.0".to_string(),
        owner_id,
        vec![
            Capability {
                skill_name: "calculate".to_string(),
                description: "Perform basic arithmetic operations".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "expression": {"type": "string"}
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "number"
                }),
            },
            Capability {
                skill_name: "convert_units".to_string(),
                description: "Convert between different units".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "value": {"type": "number"},
                        "from": {"type": "string"},
                        "to": {"type": "string"}
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "number"
                }),
            },
        ],
        PricingModel::PerCall { price: 0.01 },
        ServiceLevel {
            response_time_ms: 500,
            availability_percent: 99.9,
            max_concurrent_requests: 100,
        },
    );

    let calc_id = registry.register(calculator_agent)?;
    println!("✅ Calculator Agent registered: {}", calc_id);
    println!("   Status: Draft\n");

    // 场景 2: 注册一个翻译 Agent
    println!("--- Scenario 2: Register Translator Agent ---");
    let translator_agent = AgentListing::new(
        "Universal Translator".to_string(),
        "Multi-language translation agent".to_string(),
        "2.0.0".to_string(),
        owner_id,
        vec![
            Capability {
                skill_name: "translate".to_string(),
                description: "Translate text between languages".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string"},
                        "from": {"type": "string"},
                        "to": {"type": "string"}
                    }
                }),
                output_schema: serde_json::json!({
                    "type": "string"
                }),
            },
        ],
        PricingModel::PerHour { price: 5.0 },
        ServiceLevel {
            response_time_ms: 2000,
            availability_percent: 99.5,
            max_concurrent_requests: 50,
        },
    );

    let trans_id = registry.register(translator_agent)?;
    println!("✅ Translator Agent registered: {}", trans_id);
    println!("   Status: Draft\n");

    // 场景 3: 发布 Agent
    println!("--- Scenario 3: Publish Agents ---");
    registry.publish(&calc_id)?;
    println!("✅ Calculator Agent published");

    registry.publish(&trans_id)?;
    println!("✅ Translator Agent published\n");

    // 场景 4: 列出所有 Agent
    println!("--- Scenario 4: List All Agents ---");
    let all_agents = registry.list(0, 10)?;
    println!("Total agents: {}", all_agents.len());
    for agent in &all_agents {
        println!("  - {} (v{}) - Status: {:?}", agent.name, agent.version, agent.status);
        println!("    Capabilities: {}", agent.capabilities.len());
        println!("    Pricing: {:?}", agent.pricing);
    }
    println!();

    // 场景 5: 搜索 Agent
    println!("--- Scenario 5: Search Agents ---");

    // 按名称搜索
    let filter = AgentFilter {
        name: Some("calc".to_string()),
        ..Default::default()
    };
    let results = registry.search(&filter)?;
    println!("Search by name 'calc': {} results", results.len());
    for agent in &results {
        println!("  - {}", agent.name);
    }
    println!();

    // 按技能搜索
    let filter = AgentFilter {
        skill_name: Some("translate".to_string()),
        ..Default::default()
    };
    let results = registry.search(&filter)?;
    println!("Search by skill 'translate': {} results", results.len());
    for agent in &results {
        println!("  - {}", agent.name);
        println!("    Skills: {:?}", agent.capabilities.iter().map(|c| &c.skill_name).collect::<Vec<_>>());
    }
    println!();

    // 场景 6: 获取已发布的 Agent
    println!("--- Scenario 6: List Published Agents ---");
    let published = registry.list_published(0, 10)?;
    println!("Published agents: {}", published.len());
    for agent in &published {
        println!("  - {} - Reputation: {:.1}/5.0", agent.name, agent.reputation_score);
    }
    println!();

    // 场景 7: 暂停和恢复 Agent
    println!("--- Scenario 7: Pause and Resume Agent ---");
    registry.pause(&calc_id)?;
    let paused = registry.get(&calc_id)?.unwrap();
    println!("✅ Calculator Agent paused");
    println!("   Status: {:?}", paused.status);

    registry.publish(&calc_id)?;
    let resumed = registry.get(&calc_id)?.unwrap();
    println!("✅ Calculator Agent resumed");
    println!("   Status: {:?}", resumed.status);
    println!();

    // 场景 8: 统计信息
    println!("--- Scenario 8: Statistics ---");
    let total_count = registry.count()?;
    println!("Total agents in registry: {}", total_count);

    let published_count = registry.list_published(0, 1000)?.len();
    println!("Published agents: {}", published_count);
    println!();

    println!("=== Agent Registry Demo Complete ===");
    Ok(())
}
