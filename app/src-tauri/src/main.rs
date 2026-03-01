// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use pixelcore_runtime::{Agent, AgentConfig, Message, RuntimeError};
use pixelcore_runtime::workflow::{Workflow, WorkflowNode, WorkflowStatus};
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
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
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

// ========== 工作流相关命令 ==========

// 工作流信息结构（用于前端显示）
#[derive(Clone, serde::Serialize)]
struct WorkflowInfo {
    id: String,
    name: String,
    description: String,
    status: String,
    node_count: usize,
    edge_count: usize,
    created_at: String,
    updated_at: String,
}

// Tauri 命令：创建新工作流
#[tauri::command]
async fn create_workflow(
    name: String,
    description: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let workflow = Workflow::new(name, description);
    let workflow_id = workflow.id.to_string();

    let mut workflows = state.workflows.write().await;
    workflows.insert(workflow_id.clone(), workflow);

    Ok(workflow_id)
}

// Tauri 命令：获取所有工作流
#[tauri::command]
async fn get_workflows(state: tauri::State<'_, AppState>) -> Result<Vec<WorkflowInfo>, String> {
    let workflows = state.workflows.read().await;

    let workflow_list: Vec<WorkflowInfo> = workflows
        .iter()
        .map(|(id, workflow)| WorkflowInfo {
            id: id.clone(),
            name: workflow.name.clone(),
            description: workflow.description.clone(),
            status: format!("{:?}", workflow.status),
            node_count: workflow.nodes.len(),
            edge_count: workflow.edges.len(),
            created_at: workflow.created_at.to_rfc3339(),
            updated_at: workflow.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(workflow_list)
}

// Tauri 命令：获取单个工作流详情
#[tauri::command]
async fn get_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let workflows = state.workflows.read().await;

    let workflow = workflows
        .get(&workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    serde_json::to_value(workflow).map_err(|e| e.to_string())
}

// Tauri 命令：删除工作流
#[tauri::command]
async fn delete_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut workflows = state.workflows.write().await;

    if workflows.remove(&workflow_id).is_some() {
        Ok(format!("Workflow {} deleted", workflow_id))
    } else {
        Err(format!("Workflow {} not found", workflow_id))
    }
}

// Tauri 命令：添加节点到工作流
#[tauri::command]
async fn add_workflow_node(
    workflow_id: String,
    node_name: String,
    node_type: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut workflows = state.workflows.write().await;

    let workflow = workflows
        .get_mut(&workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    let node = match node_type.as_str() {
        "start" => WorkflowNode::start(node_name),
        "end" => WorkflowNode::end(node_name),
        "task" => WorkflowNode::task(node_name, "default_task", serde_json::json!({})),
        _ => return Err(format!("Unknown node type: {}", node_type)),
    };

    let node_id = workflow.add_node(node);
    Ok(node_id.to_string())
}

// Tauri 命令：连接两个节点
#[tauri::command]
async fn connect_workflow_nodes(
    workflow_id: String,
    from_node_id: String,
    to_node_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut workflows = state.workflows.write().await;

    let workflow = workflows
        .get_mut(&workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    let from_uuid = Uuid::parse_str(&from_node_id).map_err(|e| e.to_string())?;
    let to_uuid = Uuid::parse_str(&to_node_id).map_err(|e| e.to_string())?;

    workflow.connect(from_uuid, to_uuid);
    Ok("Nodes connected successfully".to_string())
}

// 工作流执行状态
#[derive(Clone, serde::Serialize)]
struct WorkflowExecutionStatus {
    workflow_id: String,
    status: String,
    current_node: Option<String>,
    completed_nodes: Vec<String>,
    failed_nodes: Vec<String>,
    progress: f32,
}

// Tauri 命令：执行工作流
#[tauri::command]
async fn execute_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let workflows = state.workflows.read().await;

    let workflow = workflows
        .get(&workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    // 验证工作流
    workflow.validate().map_err(|e| e.to_string())?;

    // TODO: 实际执行工作流（这里只是模拟）
    // 在实际实现中，应该使用 WorkflowExecutor 来执行工作流

    Ok(format!("Workflow {} execution started", workflow_id))
}

// Tauri 命令：获取工作流执行状态
#[tauri::command]
async fn get_workflow_execution_status(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowExecutionStatus, String> {
    let workflows = state.workflows.read().await;

    let workflow = workflows
        .get(&workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    // TODO: 实际获取执行状态（这里只是模拟）
    let status = WorkflowExecutionStatus {
        workflow_id: workflow_id.clone(),
        status: format!("{:?}", workflow.status),
        current_node: None,
        completed_nodes: vec![],
        failed_nodes: vec![],
        progress: 0.0,
    };

    Ok(status)
}

// Tauri 命令：更新工作流状态
#[tauri::command]
async fn update_workflow_status(
    workflow_id: String,
    status: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut workflows = state.workflows.write().await;

    let workflow = workflows
        .get_mut(&workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    // 更新工作流状态
    use pixelcore_runtime::workflow::WorkflowStatus;
    workflow.status = match status.as_str() {
        "Draft" => WorkflowStatus::Draft,
        "Active" => WorkflowStatus::Active,
        "Paused" => WorkflowStatus::Paused,
        "Completed" => WorkflowStatus::Completed,
        "Failed" => WorkflowStatus::Failed,
        _ => return Err(format!("Unknown status: {}", status)),
    };

    Ok(format!("Workflow {} status updated to {}", workflow_id, status))
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
        workflows: Arc::new(RwLock::new(HashMap::new())),
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
            // 工作流命令
            create_workflow,
            get_workflows,
            get_workflow,
            delete_workflow,
            add_workflow_node,
            connect_workflow_nodes,
            execute_workflow,
            get_workflow_execution_status,
            update_workflow_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
