// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use pixelcore_runtime::{Agent, AgentConfig, Message, RuntimeError};
use pixelcore_agents::ClaudeAgent;
use pixelcore_storage::Storage;
use pixelcore_claw::ClawClient;
use pixelcore_skills::{create_compute_skills, create_data_skills};

// Agent 信息结构（用于前端显示）
#[derive(Clone, serde::Serialize)]
struct AgentInfo {
    id: String,
    name: String,
    status: String,
    model: String,
    message_count: usize,
}

// 消息结构
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: String,
}

// Agent 包装器（包含 Agent 和历史记录）
struct AgentWrapper {
    agent: ClaudeAgent,
    history: Vec<ChatMessage>,
}

// 应用状态
struct AppState {
    agents: Arc<RwLock<HashMap<String, AgentWrapper>>>,
}

// Tauri 命令：获取所有 Agent 信息
#[tauri::command]
async fn get_agents(state: tauri::State<'_, AppState>) -> Result<Vec<AgentInfo>, String> {
    let agents = state.agents.read().await;

    let mut agent_list = Vec::new();
    for (id, wrapper) in agents.iter() {
        let agent = &wrapper.agent;
        agent_list.push(AgentInfo {
            id: id.clone(),
            name: agent.name().to_string(),
            status: format!("{:?}", agent.state()),
            model: agent.config().model.clone(),
            message_count: wrapper.history.len(),
        });
    }

    Ok(agent_list)
}

// Tauri 命令：创建新 Agent
#[tauri::command]
async fn create_agent(
    name: String,
    model: String,
    system_prompt: String,
    api_key: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let agent_id = Uuid::new_v4().to_string();

    // 创建 Storage
    let storage = Storage::new();

    // 创建 ClawClient
    let client = ClawClient::siliconflow(&api_key);

    // 创建 Agent 配置
    let config = AgentConfig::new(&name, &system_prompt)
        .with_model(&model);

    // 创建 Agent
    let mut agent = ClaudeAgent::with_client(config, client)
        .with_storage(storage);

    // 注册 Skills
    for skill in create_compute_skills() {
        agent.register_skill(skill);
    }
    for skill in create_data_skills() {
        agent.register_skill(skill);
    }

    // 启动 Agent
    agent.start().await.map_err(|e| e.to_string())?;

    // 保存 Agent
    let wrapper = AgentWrapper {
        agent,
        history: Vec::new(),
    };

    let mut agents = state.agents.write().await;
    agents.insert(agent_id.clone(), wrapper);

    Ok(agent_id)
}

// Tauri 命令：删除 Agent
#[tauri::command]
async fn delete_agent(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut agents = state.agents.write().await;

    if let Some(mut wrapper) = agents.remove(&agent_id) {
        wrapper.agent.stop().await.map_err(|e: RuntimeError| e.to_string())?;
        Ok(format!("Agent {} deleted", agent_id))
    } else {
        Err(format!("Agent {} not found", agent_id))
    }
}

// Tauri 命令：发送消息给 Agent
#[tauri::command]
async fn send_message(
    agent_id: String,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<ChatMessage, String> {
    let mut agents = state.agents.write().await;

    let wrapper = agents.get_mut(&agent_id)
        .ok_or_else(|| format!("Agent {} not found", agent_id))?;

    // 添加用户消息到历史
    let user_msg = ChatMessage {
        role: "user".to_string(),
        content: content.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    wrapper.history.push(user_msg);

    // 发送消息给 Agent
    let message = Message::user(&content);
    let response: Message = wrapper.agent.process(message).await
        .map_err(|e: pixelcore_runtime::RuntimeError| e.to_string())?;

    // 添加 Agent 响应到历史
    let assistant_msg = ChatMessage {
        role: "assistant".to_string(),
        content: response.content.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    wrapper.history.push(assistant_msg.clone());

    Ok(assistant_msg)
}

// Tauri 命令：获取对话历史
#[tauri::command]
async fn get_history(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    let agents = state.agents.read().await;

    let wrapper = agents.get(&agent_id)
        .ok_or_else(|| format!("Agent {} not found", agent_id))?;

    Ok(wrapper.history.clone())
}

// Tauri 命令：获取可用的 Skills
#[tauri::command]
async fn get_available_skills() -> Result<Vec<String>, String> {
    let mut skills = Vec::new();

    // 计算 Skills
    for skill in create_compute_skills() {
        skills.push(format!("{}: {}", skill.name(), skill.description()));
    }

    // 数据处理 Skills
    for skill in create_data_skills() {
        skills.push(format!("{}: {}", skill.name(), skill.description()));
    }

    Ok(skills)
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // 创建应用状态
    let app_state = AppState {
        agents: Arc::new(RwLock::new(HashMap::new())),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_agents,
            create_agent,
            delete_agent,
            send_message,
            get_history,
            get_available_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
