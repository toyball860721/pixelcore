use anyhow::Result;
use tracing_subscriber::EnvFilter;

use pixelcore_runtime::{Agent, AgentConfig, AgentState, Message};
use pixelcore_agents::ClaudeAgent;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let config = AgentConfig::new(
        "demo-agent",
        "You are a helpful assistant. Be concise.",
    );

    let mut agent = ClaudeAgent::new(config)?;
    agent.start().await?;

    let turns = [
        "What is 2 + 2?",
        "Now multiply that by 10.",
    ];

    for input in &turns {
        println!("\n[user] {input}");
        let reply = agent.process(Message::user(*input)).await?;
        println!("[assistant] {}", reply.content);
    }

    agent.stop().await?;
    assert_eq!(*agent.state(), AgentState::Stopped);

    Ok(())
}
