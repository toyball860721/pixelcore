use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 心流等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowLevel {
    /// 低心流：刚开始工作，还在热身
    Low,
    /// 中等心流：进入状态，工作顺畅
    Medium,
    /// 高心流：深度专注，效率很高
    High,
    /// 巅峰心流：完全沉浸，忘我状态
    Peak,
}

impl FlowLevel {
    /// 获取心流等级的数值表示（0.0-1.0）
    pub fn value(&self) -> f64 {
        match self {
            FlowLevel::Low => 0.25,
            FlowLevel::Medium => 0.5,
            FlowLevel::High => 0.75,
            FlowLevel::Peak => 1.0,
        }
    }

    /// 从数值创建心流等级
    pub fn from_value(value: f64) -> Self {
        if value >= 0.9 {
            FlowLevel::Peak
        } else if value >= 0.7 {
            FlowLevel::High
        } else if value >= 0.4 {
            FlowLevel::Medium
        } else {
            FlowLevel::Low
        }
    }
}

/// 心流状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowState {
    /// 空闲：没有任务在执行
    Idle,
    /// 工作中：正在执行任务，带有心流等级
    Working(FlowLevel),
    /// 深度心流：高度专注，任务执行非常顺畅
    DeepFlow,
    /// 超级专注：完全沉浸，达到最佳状态
    Hyperfocus,
}

impl FlowState {
    /// 获取当前状态的心流等级
    pub fn flow_level(&self) -> Option<FlowLevel> {
        match self {
            FlowState::Idle => None,
            FlowState::Working(level) => Some(*level),
            FlowState::DeepFlow => Some(FlowLevel::High),
            FlowState::Hyperfocus => Some(FlowLevel::Peak),
        }
    }

    /// 判断是否处于工作状态
    pub fn is_working(&self) -> bool {
        !matches!(self, FlowState::Idle)
    }
}

/// 心流指标
#[derive(Debug, Clone)]
pub struct FlowMetrics {
    /// 任务完成数量
    pub tasks_completed: u32,
    /// 任务失败数量
    pub tasks_failed: u32,
    /// 任务切换次数
    pub task_switches: u32,
    /// 最近任务的响应时间（毫秒）
    pub recent_response_times: Vec<u64>,
    /// 指标收集的时间窗口开始时间
    pub window_start: Instant,
    /// 当前任务开始时间
    pub current_task_start: Option<Instant>,
}

impl FlowMetrics {
    pub fn new() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            task_switches: 0,
            recent_response_times: Vec::new(),
            window_start: Instant::now(),
            current_task_start: None,
        }
    }

    /// 记录任务开始
    pub fn task_started(&mut self) {
        if self.current_task_start.is_some() {
            self.task_switches += 1;
        }
        self.current_task_start = Some(Instant::now());
    }

    /// 记录任务完成
    pub fn task_completed(&mut self) {
        if let Some(start) = self.current_task_start.take() {
            let duration = start.elapsed().as_millis() as u64;
            self.recent_response_times.push(duration);
            // 只保留最近 10 个响应时间
            if self.recent_response_times.len() > 10 {
                self.recent_response_times.remove(0);
            }
        }
        self.tasks_completed += 1;
    }

    /// 记录任务失败
    pub fn task_failed(&mut self) {
        self.current_task_start = None;
        self.tasks_failed += 1;
    }

    /// 计算任务完成速率（每分钟）
    pub fn completion_rate(&self) -> f64 {
        let elapsed = self.window_start.elapsed().as_secs_f64();
        if elapsed < 0.1 {
            return 0.0;
        }
        (self.tasks_completed as f64) / (elapsed / 60.0)
    }

    /// 计算错误率（0.0-1.0）
    pub fn error_rate(&self) -> f64 {
        let total = self.tasks_completed + self.tasks_failed;
        if total == 0 {
            return 0.0;
        }
        (self.tasks_failed as f64) / (total as f64)
    }

    /// 计算响应延迟的稳定性（标准差，越小越稳定）
    pub fn response_stability(&self) -> f64 {
        if self.recent_response_times.len() < 2 {
            return 1.0; // 数据不足，返回不稳定
        }

        let mean = self.recent_response_times.iter().sum::<u64>() as f64
            / self.recent_response_times.len() as f64;

        let variance = self
            .recent_response_times
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / self.recent_response_times.len() as f64;

        variance.sqrt() / mean.max(1.0) // 归一化的标准差
    }

    /// 计算任务切换频率（每分钟）
    pub fn switch_frequency(&self) -> f64 {
        let elapsed = self.window_start.elapsed().as_secs_f64();
        if elapsed < 0.1 {
            return 0.0;
        }
        (self.task_switches as f64) / (elapsed / 60.0)
    }

    /// 重置指标（开始新的时间窗口）
    pub fn reset(&mut self) {
        self.tasks_completed = 0;
        self.tasks_failed = 0;
        self.task_switches = 0;
        self.recent_response_times.clear();
        self.window_start = Instant::now();
        self.current_task_start = None;
    }
}

impl Default for FlowMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 心流状态机配置
#[derive(Debug, Clone)]
pub struct FlowStateMachineConfig {
    /// 进入 Working 状态的最小完成速率（每分钟）
    pub working_min_rate: f64,
    /// 进入 DeepFlow 状态的最小完成速率（每分钟）
    pub deep_flow_min_rate: f64,
    /// 进入 Hyperfocus 状态的最小完成速率（每分钟）
    pub hyperfocus_min_rate: f64,
    /// 最大允许错误率
    pub max_error_rate: f64,
    /// 最大允许响应不稳定性
    pub max_instability: f64,
    /// 最大允许切换频率（每分钟）
    pub max_switch_frequency: f64,
    /// 指标重置间隔
    pub metrics_reset_interval: Duration,
}

impl Default for FlowStateMachineConfig {
    fn default() -> Self {
        Self {
            working_min_rate: 1.0,      // 至少每分钟完成 1 个任务
            deep_flow_min_rate: 3.0,    // 至少每分钟完成 3 个任务
            hyperfocus_min_rate: 5.0,   // 至少每分钟完成 5 个任务
            max_error_rate: 0.1,        // 最多 10% 错误率
            max_instability: 0.3,       // 响应时间标准差不超过均值的 30%
            max_switch_frequency: 5.0,  // 每分钟最多切换 5 次任务
            metrics_reset_interval: Duration::from_secs(300), // 5 分钟重置一次
        }
    }
}

/// 心流状态机
pub struct FlowStateMachine {
    state: FlowState,
    metrics: FlowMetrics,
    config: FlowStateMachineConfig,
    last_reset: Instant,
}

impl FlowStateMachine {
    pub fn new(config: FlowStateMachineConfig) -> Self {
        Self {
            state: FlowState::Idle,
            metrics: FlowMetrics::new(),
            config,
            last_reset: Instant::now(),
        }
    }

    pub fn state(&self) -> &FlowState {
        &self.state
    }

    pub fn metrics(&self) -> &FlowMetrics {
        &self.metrics
    }

    /// 记录任务开始
    pub fn task_started(&mut self) {
        self.metrics.task_started();
        self.update_state();
    }

    /// 记录任务完成
    pub fn task_completed(&mut self) {
        self.metrics.task_completed();
        self.update_state();
    }

    /// 记录任务失败
    pub fn task_failed(&mut self) {
        self.metrics.task_failed();
        self.update_state();
    }

    /// 计算综合心流分数（0.0-1.0）
    fn calculate_flow_score(&self) -> f64 {
        let completion_rate = self.metrics.completion_rate();
        let error_rate = self.metrics.error_rate();
        let stability = 1.0 - self.metrics.response_stability().min(1.0);
        let switch_freq = self.metrics.switch_frequency();

        // 归一化完成速率（假设 10 个/分钟是最大值）
        let rate_score = (completion_rate / 10.0).min(1.0);

        // 错误率惩罚（错误率越高，分数越低）
        let error_penalty = 1.0 - (error_rate / self.config.max_error_rate).min(1.0);

        // 稳定性奖励
        let stability_score = stability;

        // 切换频率惩罚（切换太频繁会降低心流）
        let switch_penalty = 1.0 - (switch_freq / self.config.max_switch_frequency).min(1.0);

        // 综合分数（加权平均）
        rate_score * 0.4 + error_penalty * 0.2 + stability_score * 0.2 + switch_penalty * 0.2
    }

    /// 公开的计算心流分数方法（用于调试）
    pub fn calculate_flow_score_public(&self) -> f64 {
        self.calculate_flow_score()
    }

    /// 更新心流状态
    fn update_state(&mut self) {
        // 检查是否需要重置指标
        if self.last_reset.elapsed() >= self.config.metrics_reset_interval {
            self.metrics.reset();
            self.last_reset = Instant::now();
        }

        let completion_rate = self.metrics.completion_rate();
        let error_rate = self.metrics.error_rate();
        let flow_score = self.calculate_flow_score();

        // 调试输出
        tracing::debug!(
            "update_state: completion_rate={:.2}, error_rate={:.2}, flow_score={:.2}, \
             tasks_completed={}, tasks_failed={}, current_task={:?}",
            completion_rate,
            error_rate,
            flow_score,
            self.metrics.tasks_completed,
            self.metrics.tasks_failed,
            self.metrics.current_task_start.is_some()
        );

        // 如果没有任务在执行，回到 Idle
        if self.metrics.current_task_start.is_none()
            && self.metrics.tasks_completed == 0
            && self.metrics.tasks_failed == 0
        {
            self.state = FlowState::Idle;
            return;
        }

        // 状态转换逻辑
        self.state = if completion_rate >= self.config.hyperfocus_min_rate
            && error_rate <= self.config.max_error_rate * 0.5
            && flow_score >= 0.9
        {
            FlowState::Hyperfocus
        } else if completion_rate >= self.config.deep_flow_min_rate
            && error_rate <= self.config.max_error_rate
            && flow_score >= 0.7
        {
            FlowState::DeepFlow
        } else if completion_rate >= self.config.working_min_rate {
            let level = FlowLevel::from_value(flow_score);
            FlowState::Working(level)
        } else {
            FlowState::Idle
        };
    }

    /// 强制设置状态为 Idle
    pub fn set_idle(&mut self) {
        self.state = FlowState::Idle;
        self.metrics.reset();
        self.last_reset = Instant::now();
    }
}

impl Default for FlowStateMachine {
    fn default() -> Self {
        Self::new(FlowStateMachineConfig::default())
    }
}
