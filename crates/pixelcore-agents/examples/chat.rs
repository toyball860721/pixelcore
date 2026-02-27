use pixelcore_agents::ClaudeAgent;
use pixelcore_claw::ClawClient;
use pixelcore_runtime::{Agent, AgentConfig, Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("SILICONFLOW_API_KEY")
        .expect("SILICONFLOW_API_KEY not set");
    let model = std::env::var("SILICONFLOW_MODEL")
        .unwrap_or_else(|_| "Pro/MiniMaxAI/MiniMax-M2.5".to_string());

    let client = ClawClient::siliconflow(api_key);
    let config = AgentConfig::new("demo", "你是一个简洁的助手，用中文回答。")
        .with_model(model);

    let mut agent = ClaudeAgent::with_client(config, client);
    agent.start().await?;

    let reply = agent.process(Message::user("用一句话介绍你自己")).await?;
    println!("Assistant: {}", reply.content);

    agent.stop().await?;
    Ok(())
}
