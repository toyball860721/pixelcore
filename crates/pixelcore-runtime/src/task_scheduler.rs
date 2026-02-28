use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub payload: serde_json::Value,
    pub assigned_to: Option<Uuid>, // Agent ID
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl Task {
    pub fn new(name: impl Into<String>, priority: TaskPriority, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            priority,
            status: TaskStatus::Pending,
            payload,
            assigned_to: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
}

/// 用于优先级队列的任务包装
#[derive(Debug, Clone)]
struct PriorityTask {
    task: Task,
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority && self.task.created_at == other.task.created_at
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先级高的排前面，如果优先级相同则早创建的排前面
        match self.task.priority.cmp(&other.task.priority) {
            Ordering::Equal => other.task.created_at.cmp(&self.task.created_at),
            other => other,
        }
    }
}

/// 任务调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 任务队列最大长度
    pub max_queue_size: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            max_queue_size: 1000,
        }
    }
}

/// 任务调度器
#[derive(Clone)]
pub struct TaskScheduler {
    config: SchedulerConfig,
    /// 待处理任务队列（优先级队列）
    pending_queue: Arc<RwLock<BinaryHeap<PriorityTask>>>,
    /// 所有任务状态
    tasks: Arc<RwLock<HashMap<Uuid, Task>>>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 任务完成通知
    task_tx: mpsc::UnboundedSender<Task>,
    task_rx: Arc<RwLock<mpsc::UnboundedReceiver<Task>>>,
}

impl TaskScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        let (task_tx, task_rx) = mpsc::unbounded_channel();

        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            config,
            pending_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_tx,
            task_rx: Arc::new(RwLock::new(task_rx)),
        }
    }

    /// 提交任务
    pub async fn submit(&self, task: Task) -> Result<Uuid, String> {
        let task_id = task.id;

        // 检查队列是否已满
        let queue_size = self.pending_queue.read().await.len();
        if queue_size >= self.config.max_queue_size {
            return Err(format!("Task queue is full (max: {})", self.config.max_queue_size));
        }

        // 添加到任务列表
        self.tasks.write().await.insert(task_id, task.clone());

        // 添加到优先级队列
        self.pending_queue.write().await.push(PriorityTask { task });

        Ok(task_id)
    }

    /// 获取下一个待执行任务
    pub async fn next_task(&self) -> Option<Task> {
        let mut queue = self.pending_queue.write().await;
        queue.pop().map(|pt| pt.task)
    }

    /// 更新任务状态
    pub async fn update_task_status(&self, task_id: Uuid, status: TaskStatus) {
        if let Some(task) = self.tasks.write().await.get_mut(&task_id) {
            task.status = status.clone();

            match status {
                TaskStatus::Running => {
                    task.started_at = Some(Utc::now());
                }
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                    task.completed_at = Some(Utc::now());
                }
                _ => {}
            }
        }
    }

    /// 设置任务结果
    pub async fn set_task_result(&self, task_id: Uuid, result: serde_json::Value) {
        if let Some(task) = self.tasks.write().await.get_mut(&task_id) {
            task.result = Some(result);
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Utc::now());
        }
    }

    /// 设置任务错误
    pub async fn set_task_error(&self, task_id: Uuid, error: String) {
        if let Some(task) = self.tasks.write().await.get_mut(&task_id) {
            task.error = Some(error);
            task.status = TaskStatus::Failed;
            task.completed_at = Some(Utc::now());
        }
    }

    /// 分配任务给 Agent
    pub async fn assign_task(&self, task_id: Uuid, agent_id: Uuid) {
        if let Some(task) = self.tasks.write().await.get_mut(&task_id) {
            task.assigned_to = Some(agent_id);
        }
    }

    /// 获取任务信息
    pub async fn get_task(&self, task_id: &Uuid) -> Option<Task> {
        self.tasks.read().await.get(task_id).cloned()
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// 获取指定状态的任务
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Task> {
        self.tasks.read().await
            .values()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    /// 获取队列长度
    pub async fn queue_length(&self) -> usize {
        self.pending_queue.read().await.len()
    }

    /// 获取运行中的任务数
    pub async fn running_tasks_count(&self) -> usize {
        self.tasks.read().await
            .values()
            .filter(|t| t.status == TaskStatus::Running)
            .count()
    }

    /// 获取可用的并发槽位数
    pub fn available_slots(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// 获取信号量（用于并发控制）
    pub fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(&task_id) {
            if task.status == TaskStatus::Running {
                return Err("Cannot cancel running task".to_string());
            }

            task.status = TaskStatus::Cancelled;
            task.completed_at = Some(Utc::now());
            Ok(())
        } else {
            Err("Task not found".to_string())
        }
    }

    /// 清理已完成的任务
    pub async fn cleanup_completed(&self) {
        let mut tasks = self.tasks.write().await;
        tasks.retain(|_, task| {
            !matches!(task.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_and_next_task() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());

        let task = Task::new("test-task", TaskPriority::Normal, serde_json::json!({}));
        let task_id = scheduler.submit(task.clone()).await.unwrap();

        assert_eq!(scheduler.queue_length().await, 1);

        let next = scheduler.next_task().await.unwrap();
        assert_eq!(next.id, task_id);
        assert_eq!(scheduler.queue_length().await, 0);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());

        let low = Task::new("low", TaskPriority::Low, serde_json::json!({}));
        let high = Task::new("high", TaskPriority::High, serde_json::json!({}));
        let normal = Task::new("normal", TaskPriority::Normal, serde_json::json!({}));

        scheduler.submit(low).await.unwrap();
        scheduler.submit(high.clone()).await.unwrap();
        scheduler.submit(normal).await.unwrap();

        let first = scheduler.next_task().await.unwrap();
        assert_eq!(first.name, "high");
        assert_eq!(first.priority, TaskPriority::High);
    }

    #[tokio::test]
    async fn test_update_task_status() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());

        let task = Task::new("test", TaskPriority::Normal, serde_json::json!({}));
        let task_id = scheduler.submit(task).await.unwrap();

        scheduler.update_task_status(task_id, TaskStatus::Running).await;

        let updated = scheduler.get_task(&task_id).await.unwrap();
        assert_eq!(updated.status, TaskStatus::Running);
        assert!(updated.started_at.is_some());
    }

    #[tokio::test]
    async fn test_queue_size_limit() {
        let config = SchedulerConfig {
            max_concurrent_tasks: 10,
            max_queue_size: 2,
        };
        let scheduler = TaskScheduler::new(config);

        let task1 = Task::new("task1", TaskPriority::Normal, serde_json::json!({}));
        let task2 = Task::new("task2", TaskPriority::Normal, serde_json::json!({}));
        let task3 = Task::new("task3", TaskPriority::Normal, serde_json::json!({}));

        assert!(scheduler.submit(task1).await.is_ok());
        assert!(scheduler.submit(task2).await.is_ok());
        assert!(scheduler.submit(task3).await.is_err());
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let scheduler = TaskScheduler::new(SchedulerConfig::default());

        let task = Task::new("test", TaskPriority::Normal, serde_json::json!({}));
        let task_id = scheduler.submit(task).await.unwrap();

        assert!(scheduler.cancel_task(task_id).await.is_ok());

        let cancelled = scheduler.get_task(&task_id).await.unwrap();
        assert_eq!(cancelled.status, TaskStatus::Cancelled);
    }
}
