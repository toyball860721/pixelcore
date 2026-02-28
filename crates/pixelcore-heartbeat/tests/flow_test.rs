use pixelcore_heartbeat::{FlowState, FlowMetrics, FlowStateMachine, FlowStateMachineConfig};

#[test]
fn test_flow_state_machine_initialization() {
    let config = FlowStateMachineConfig::default();
    let fsm = FlowStateMachine::new(config);

    // 初始状态应该是 Idle
    assert!(matches!(fsm.state(), FlowState::Idle));
}

#[test]
fn test_flow_metrics_creation() {
    let mut metrics = FlowMetrics::new();

    // 测试任务开始
    metrics.task_started();
    assert!(metrics.current_task_start.is_some());

    // 测试任务完成
    metrics.task_completed();
    assert_eq!(metrics.tasks_completed, 1);
    assert!(metrics.current_task_start.is_none());
    assert!(!metrics.recent_response_times.is_empty());
}

#[test]
fn test_flow_metrics_task_switches() {
    let mut metrics = FlowMetrics::new();

    // 第一个任务
    metrics.task_started();
    assert_eq!(metrics.task_switches, 0);

    // 切换到第二个任务（没有完成第一个）
    metrics.task_started();
    assert_eq!(metrics.task_switches, 1);

    // 再切换
    metrics.task_started();
    assert_eq!(metrics.task_switches, 2);
}

#[test]
fn test_flow_metrics_failure_tracking() {
    let mut metrics = FlowMetrics::new();

    metrics.task_started();
    metrics.task_failed();

    assert_eq!(metrics.tasks_failed, 1);
    assert!(metrics.current_task_start.is_none());
}

#[test]
fn test_flow_state_machine_task_tracking() {
    let config = FlowStateMachineConfig::default();
    let mut fsm = FlowStateMachine::new(config);

    // 初始状态是 Idle
    assert!(matches!(fsm.state(), FlowState::Idle));

    // 完成多个任务
    for _ in 0..5 {
        fsm.task_started();
        fsm.task_completed();
    }

    // 验证任务被记录
    assert_eq!(fsm.metrics().tasks_completed, 5);
}
