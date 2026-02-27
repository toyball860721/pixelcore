use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use pixelcore_runtime::{Agent, AgentConfig, AgentState, Message};
use pixelcore_agents::ClaudeAgent;
use pixelcore_skills::{EchoSkill, StorageGetSkill, StorageSetSkill};
use pixelcore_storage::Storage;
use pixelcore_swarm::Swarm;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Shared storage backend.
    let store = Arc::new(Storage::new());

    // Build agent with skills.
    let config = AgentConfig::new(
        "demo-agent",
        "You are a helpful assistant with access to a key-value store. Be concise.",
    ).with_model("Pro/MiniMaxAI/MiniMax-M2.5");
    let client = pixelcore_claw::ClawClient::siliconflow(
        "sk-ewcpabkoyhlwimwcmqckbxhudfcdysviltqagvzsxlcjqrpe",
    );
    let mut agent = ClaudeAgent::with_client(config, client);
    agent.register_skill(Arc::new(EchoSkill));
    agent.register_skill(Arc::new(StorageGetSkill { store: Arc::clone(&store) }));
    agent.register_skill(Arc::new(StorageSetSkill { store: Arc::clone(&store) }));
    agent.start().await?;

    // Register in swarm.
    let swarm = Swarm::new();
    let agent_id = agent.id();
    swarm.add(Arc::new(Mutex::new(agent))).await;

    // Demo turns via swarm routing.
    let turns = [
        "Store the value 42 under the key 'answer'.",
        "What is stored under the key 'answer'?",
        "Echo back the phrase: hello pixelcore",
    ];

    for input in &turns {
        println!("\n[user] {input}");
        let reply = swarm.route(&agent_id, Message::user(*input)).await?;
        println!("[assistant] {}", reply.content);
    }

    // Stop agent.
    let agent_arc = swarm.get(&agent_id).await?;
    agent_arc.lock().await.stop().await?;
    assert_eq!(*agent_arc.lock().await.state(), AgentState::Stopped);

    Ok(())
}
