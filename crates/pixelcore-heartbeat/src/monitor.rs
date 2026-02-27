use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use pixelcore_runtime::event::{Event, EventBus, EventKind};
use pixelcore_runtime::AgentId;
use crate::flow::{FlowState, FlowStateMachine, FlowStateMachineConfig};

/// 心流监控器
/// 监听 Agent 的事件并更新心流状态
pub struct FlowMonitor {
    /// 每个 Agent 的心流状态机
    state_machines: Arc<RwLock<HashMap<AgentId, FlowStateMachine>>>,
    /// 事件总线
    event_bus: EventBus,
    /// 配置
    config: FlowStateMachineConfig,
}

impl FlowMonitor {
    pub fn new(event_bus: EventBus, config: FlowStateMachineConfig) -> Self {
        Self {
            state_machines: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            config,
        }
    }

    /// 注册一个 Agent 进行监控
    pub async fn register_agent(&self, agent_id: AgentId) {
        let mut machines = self.state_machines.write().await;
        machines.insert(agent_id, FlowStateMachine::new(self.config.clone()));
    }

    /// 取消注册 Agent
    pub async fn unregister_agent(&self, agent_id: &AgentId) {
        let mut machines = self.state_machines.write().await;
        machines.remove(agent_id);
    }

    /// 获取 Agent 的当前心流状态
    pub async fn get_flow_state(&self, agent_id: &AgentId) -> Option<FlowState> {
        let machines = self.state_machines.read().await;
        machines.get(agent_id).map(|m| m.state().clone())
    }

    /// 获取 Agent 的心流指标（用于调试）
    pub async fn get_metrics_debug(&self, agent_id: &AgentId) -> Option<String> {
        let machines = self.state_machines.read().await;
        machines.get(agent_id).map(|m| {
            let metrics = m.metrics();
            let flow_score = m.calculate_flow_score_public();
            format!(
                "tasks_completed={}, tasks_failed={}, completion_rate={:.2}/min, error_rate={:.2}, flow_score={:.2}",
                metrics.tasks_completed,
                metrics.tasks_failed,
                metrics.completion_rate(),
                metrics.error_rate(),
                flow_score
            )
        })
    }

    /// 启动监控（监听事件并更新状态）
    pub async fn run(&self) {
        let receiver = self.event_bus.subscribe();
        let state_machines = Arc::clone(&self.state_machines);
        let event_bus = self.event_bus.clone();

        tokio::spawn(async move {
            loop {
                match receiver.recv_async().await {
                    Ok(event) => {
                        Self::handle_event(event, &state_machines, &event_bus).await;
                    }
                    Err(_) => {
                        // Channel closed
                        break;
                    }
                }
            }
        });
    }

    async fn handle_event(
        event: Event,
        state_machines: &Arc<RwLock<HashMap<AgentId, FlowStateMachine>>>,
        event_bus: &EventBus,
    ) {
        // 从事件中提取 agent_id
        let agent_id = match Self::extract_agent_id(&event) {
            Some(id) => id,
            None => return,
        };

        let mut machines = state_machines.write().await;
        let machine = match machines.get_mut(&agent_id) {
            Some(m) => m,
            None => return,
        };

        let old_state = machine.state().clone();

        // 根据事件类型更新状态机
        match event.kind {
            EventKind::TaskStarted => {
                machine.task_started();
            }
            EventKind::TaskCompleted => {
                machine.task_completed();
            }
            EventKind::TaskFailed => {
                machine.task_failed();
            }
            EventKind::AgentStopped => {
                machine.set_idle();
            }
            _ => {}
        }

        let new_state = machine.state().clone();

        // 如果状态发生变化，发布事件
        if old_state != new_state {
            let flow_event = Event::new(
                EventKind::Custom("flow_state_changed".to_string()),
                format!("agent:{}", agent_id),
                serde_json::json!({
                    "agent_id": agent_id,
                    "old_state": old_state,
                    "new_state": new_state,
                }),
            );
            let _ = event_bus.publish(flow_event);
        }
    }

    fn extract_agent_id(event: &Event) -> Option<AgentId> {
        // 尝试从 payload 中提取 agent_id
        if let Some(id_str) = event.payload.get("agent_id").and_then(|v| v.as_str()) {
            return id_str.parse().ok();
        }

        // 尝试从 source 中提取（格式：agent:uuid）
        if event.source.starts_with("agent:") {
            let id_str = event.source.strip_prefix("agent:")?;
            return id_str.parse().ok();
        }

        None
    }
}

impl Default for FlowMonitor {
    fn default() -> Self {
        Self::new(EventBus::new(), FlowStateMachineConfig::default())
    }
}
