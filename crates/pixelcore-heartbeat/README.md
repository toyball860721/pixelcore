# PixelCore Heartbeat - 心流状态机

心流（Flow）状态机用于监控和评估 Agent 的工作状态，根据任务完成情况自动识别 Agent 的心流等级。

## 核心概念

### 心流等级（FlowLevel）

- **Low**: 低心流，刚开始工作，还在热身
- **Medium**: 中等心流，进入状态，工作顺畅
- **High**: 高心流，深度专注，效率很高
- **Peak**: 巅峰心流，完全沉浸，忘我状态

### 心流状态（FlowState）

```
Idle → Working(FlowLevel) → DeepFlow → Hyperfocus
```

- **Idle**: 空闲，没有任务在执行
- **Working(FlowLevel)**: 工作中，带有心流等级
- **DeepFlow**: 深度心流，高度专注，任务执行非常顺畅
- **Hyperfocus**: 超级专注，完全沉浸，达到最佳状态

### 心流指标（FlowMetrics）

心流状态由以下指标综合计算：

1. **任务完成速率**: 每分钟完成的任务数量
2. **错误率**: 失败任务占总任务的比例（越低越好）
3. **响应延迟稳定性**: 任务响应时间的标准差（越小越稳定）
4. **任务切换频率**: 每分钟切换任务的次数（越低越专注）

## 使用方法

### 直接使用 FlowStateMachine

```rust
use pixelcore_heartbeat::{FlowStateMachine, FlowStateMachineConfig};
use std::time::Duration;

// 创建状态机
let config = FlowStateMachineConfig {
    working_min_rate: 1.0,      // 至少每分钟完成 1 个任务
    deep_flow_min_rate: 3.0,    // 至少每分钟完成 3 个任务
    hyperfocus_min_rate: 5.0,   // 至少每分钟完成 5 个任务
    max_error_rate: 0.1,        // 最多 10% 错误率
    max_instability: 0.3,       // 响应时间标准差不超过均值的 30%
    max_switch_frequency: 5.0,  // 每分钟最多切换 5 次任务
    metrics_reset_interval: Duration::from_secs(300), // 5 分钟重置一次
};

let mut machine = FlowStateMachine::new(config);

// 记录任务开始
machine.task_started();

// 记录任务完成
machine.task_completed();

// 记录任务失败
machine.task_failed();

// 获取当前状态
let state = machine.state();
println!("当前心流状态: {:?}", state);

// 获取指标
let metrics = machine.metrics();
println!("完成速率: {:.2}/分钟", metrics.completion_rate());
println!("错误率: {:.2}", metrics.error_rate());
```

### 使用 FlowMonitor（事件驱动）

```rust
use pixelcore_heartbeat::{FlowMonitor, FlowStateMachineConfig};
use pixelcore_runtime::event::{Event, EventBus, EventKind};
use pixelcore_runtime::AgentId;

// 创建事件总线和监控器
let event_bus = EventBus::new();
let config = FlowStateMachineConfig::default();
let monitor = FlowMonitor::new(event_bus.clone(), config);

// 注册 Agent
let agent_id = AgentId::new_v4();
monitor.register_agent(agent_id).await;

// 启动监控
monitor.run().await;

// 发布任务事件
event_bus.publish(Event::new(
    EventKind::TaskStarted,
    format!("agent:{}", agent_id),
    serde_json::json!({ "agent_id": agent_id.to_string() }),
)).unwrap();

event_bus.publish(Event::new(
    EventKind::TaskCompleted,
    format!("agent:{}", agent_id),
    serde_json::json!({ "agent_id": agent_id.to_string() }),
)).unwrap();

// 获取心流状态
let state = monitor.get_flow_state(&agent_id).await;
```

## 运行示例

### 直接测试

```bash
cargo run --package pixelcore-heartbeat --example flow_direct
```

这个示例直接使用 FlowStateMachine，演示了：
- 快速完成任务达到 Hyperfocus 状态
- 任务失败导致心流下降
- 手动设置为 Idle 状态

### 事件驱动测试

```bash
cargo run --package pixelcore-heartbeat --example flow_demo
```

这个示例使用 FlowMonitor 和事件总线，演示了事件驱动的心流监控。

## 配置建议

根据不同的使用场景，可以调整配置参数：

### 快节奏任务（如 API 调用）

```rust
FlowStateMachineConfig {
    working_min_rate: 5.0,
    deep_flow_min_rate: 10.0,
    hyperfocus_min_rate: 20.0,
    max_error_rate: 0.05,
    max_instability: 0.2,
    max_switch_frequency: 10.0,
    metrics_reset_interval: Duration::from_secs(60),
}
```

### 慢节奏任务（如数据分析）

```rust
FlowStateMachineConfig {
    working_min_rate: 0.5,
    deep_flow_min_rate: 1.0,
    hyperfocus_min_rate: 2.0,
    max_error_rate: 0.15,
    max_instability: 0.4,
    max_switch_frequency: 3.0,
    metrics_reset_interval: Duration::from_secs(600),
}
```

## 注意事项

1. **时间窗口**: 指标是在一个时间窗口内计算的，默认 5 分钟后会重置
2. **最小时间**: 完成速率和切换频率需要至少 0.1 秒的时间窗口才能计算
3. **响应稳定性**: 需要至少 2 个任务的响应时间才能计算稳定性
4. **EventBus 限制**: 当前 EventBus 实现是单播而不是广播，多个订阅者会竞争消息

## 未来改进

- [ ] 改进 EventBus 支持广播模式
- [ ] 添加心流状态持久化
- [ ] 添加心流历史记录和可视化
- [ ] 支持自定义心流指标权重
- [ ] 添加心流状态预测功能
