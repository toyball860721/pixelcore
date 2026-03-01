use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::workflow::Workflow;
use super::node::{NodeType, WorkflowNode};
use super::edge::{EdgeCondition, WorkflowEdge};
use super::error_handling::{ErrorHandlingStrategy, RetryPolicy};

/// 执行状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Paused,
}

/// 执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub status: ExecutionStatus,
    pub current_node: Option<Uuid>,
    pub variables: HashMap<String, serde_json::Value>,
    pub node_results: HashMap<Uuid, serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl ExecutionContext {
    pub fn new(workflow_id: Uuid) -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            workflow_id,
            status: ExecutionStatus::Running,
            current_node: None,
            variables: HashMap::new(),
            node_results: HashMap::new(),
            started_at: Utc::now(),
            completed_at: None,
            error: None,
        }
    }

    pub fn set_variable(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.variables.insert(key.into(), value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    pub fn set_node_result(&mut self, node_id: Uuid, result: serde_json::Value) {
        self.node_results.insert(node_id, result);
    }

    pub fn get_node_result(&self, node_id: &Uuid) -> Option<&serde_json::Value> {
        self.node_results.get(node_id)
    }
}

/// 工作流执行器
pub struct WorkflowExecutor {
    workflow: Arc<RwLock<Workflow>>,
    context: Arc<RwLock<ExecutionContext>>,
}

impl WorkflowExecutor {
    pub fn new(workflow: Workflow) -> Self {
        let context = ExecutionContext::new(workflow.id);
        Self {
            workflow: Arc::new(RwLock::new(workflow)),
            context: Arc::new(RwLock::new(context)),
        }
    }

    /// 开始执行工作流
    pub async fn execute(&self) -> Result<ExecutionContext, String> {
        // 验证工作流
        let workflow = self.workflow.read().await;
        workflow.validate()?;

        // 查找开始节点
        let start_node = workflow.find_start_node()
            .ok_or("No start node found")?;

        let start_id = start_node.id;
        drop(workflow);

        // 从开始节点执行
        self.execute_from_node(start_id).await?;

        // 返回执行上下文
        let context = self.context.read().await;
        Ok(context.clone())
    }

    /// 从指定节点开始执行
    fn execute_from_node<'a>(&'a self, node_id: Uuid) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + 'a>> {
        Box::pin(async move {
            let workflow = self.workflow.read().await;
            let node = workflow.get_node(&node_id)
                .ok_or(format!("Node not found: {}", node_id))?
                .clone();
            drop(workflow);

            // 更新当前节点
            {
                let mut context = self.context.write().await;
                context.current_node = Some(node_id);
            }

            // 执行节点
            match &node.node_type {
                NodeType::Start => {
                    // 开始节点，直接继续
                    self.execute_next_nodes(node_id).await?;
                }
                NodeType::End => {
                    // 结束节点，标记完成
                    let mut context = self.context.write().await;
                    context.status = ExecutionStatus::Completed;
                    context.completed_at = Some(Utc::now());
                }
                NodeType::Task { task_name, params } => {
                    // 执行任务节点（带错误处理）
                    let result = self.execute_task_with_error_handling(
                        node_id,
                        task_name,
                        params,
                        &node.error_handling
                    ).await;

                    match result {
                        Ok(task_result) => {
                            // 保存结果
                            {
                                let mut context = self.context.write().await;
                                context.set_node_result(node_id, task_result);
                            }

                            // 继续执行下一个节点
                            self.execute_next_nodes(node_id).await?;
                        }
                        Err(e) => {
                            // 根据错误处理策略决定是否继续
                            match &node.error_handling {
                                ErrorHandlingStrategy::Fail => {
                                    let mut context = self.context.write().await;
                                    context.status = ExecutionStatus::Failed;
                                    context.error = Some(e.clone());
                                    return Err(e);
                                }
                                ErrorHandlingStrategy::Ignore => {
                                    // 忽略错误，继续执行
                                    self.execute_next_nodes(node_id).await?;
                                }
                                _ => {
                                    // 其他策略已在 execute_task_with_error_handling 中处理
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
                NodeType::Decision { condition } => {
                    // 执行决策节点
                    let result = self.evaluate_condition(condition).await?;

                    // 根据结果选择分支
                    self.execute_decision_branch(node_id, result).await?;
                }
                NodeType::Loop { .. } => {
                    // 暂时不支持循环
                    return Err("Loop nodes not yet supported".to_string());
                }
                NodeType::Parallel { .. } => {
                    // 暂时不支持并行
                    return Err("Parallel nodes not yet supported".to_string());
                }
            }

            Ok(())
        })
    }

    /// 执行下一个节点
    async fn execute_next_nodes(&self, current_node_id: Uuid) -> Result<(), String> {
        let next_node_id = {
            let workflow = self.workflow.read().await;
            let edges = workflow.get_outgoing_edges(&current_node_id);

            if edges.is_empty() {
                return Ok(());
            }

            // 找到第一个满足条件的边
            let mut next_id = None;
            for edge in edges {
                if self.evaluate_edge_condition(&edge.condition).await? {
                    next_id = Some(edge.to);
                    break;
                }
            }

            next_id
        };

        if let Some(node_id) = next_node_id {
            self.execute_from_node(node_id).await?;
        }

        Ok(())
    }

    /// 执行任务
    async fn execute_task(&self, task_name: &str, params: &serde_json::Value) -> Result<serde_json::Value, String> {
        // 这里是任务执行的占位符
        // 实际实现中，这里会调用 Agent 或其他执行器
        Ok(serde_json::json!({
            "task": task_name,
            "params": params,
            "result": "success"
        }))
    }

    /// 带错误处理的任务执行
    async fn execute_task_with_error_handling(
        &self,
        node_id: Uuid,
        task_name: &str,
        params: &serde_json::Value,
        strategy: &ErrorHandlingStrategy,
    ) -> Result<serde_json::Value, String> {
        match strategy {
            ErrorHandlingStrategy::Retry { policy } => {
                self.execute_task_with_retry(task_name, params, policy).await
            }
            ErrorHandlingStrategy::Fallback { fallback_node } => {
                match self.execute_task(task_name, params).await {
                    Ok(result) => Ok(result),
                    Err(_) => {
                        // 跳转到 fallback 节点
                        // 这里需要解析 fallback_node 字符串为 Uuid
                        Err(format!("Fallback to node: {}", fallback_node))
                    }
                }
            }
            _ => {
                // Fail 和 Ignore 策略在调用方处理
                self.execute_task(task_name, params).await
            }
        }
    }

    /// 带重试的任务执行
    async fn execute_task_with_retry(
        &self,
        task_name: &str,
        params: &serde_json::Value,
        policy: &RetryPolicy,
    ) -> Result<serde_json::Value, String> {
        let mut last_error = String::new();

        for attempt in 0..=policy.max_retries {
            match self.execute_task(task_name, params).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = e;

                    if attempt < policy.max_retries {
                        let delay = policy.calculate_delay(attempt);
                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(format!("Task failed after {} retries: {}", policy.max_retries, last_error))
    }

    /// 评估条件
    async fn evaluate_condition(&self, condition: &str) -> Result<bool, String> {
        // 简单的条件评估占位符
        // 实际实现中，这里会使用表达式解析器
        Ok(condition.contains("true"))
    }

    /// 评估边条件
    async fn evaluate_edge_condition(&self, condition: &EdgeCondition) -> Result<bool, String> {
        match condition {
            EdgeCondition::Always => Ok(true),
            EdgeCondition::Expression { expr } => self.evaluate_condition(expr).await,
            EdgeCondition::Branch { value } => Ok(*value),
            EdgeCondition::ParallelBranch { .. } => Ok(true),
        }
    }

    /// 执行决策分支
    async fn execute_decision_branch(&self, node_id: Uuid, result: bool) -> Result<(), String> {
        let next_node_id = {
            let workflow = self.workflow.read().await;
            let edges = workflow.get_outgoing_edges(&node_id);

            let mut next_id = None;
            for edge in edges {
                if let EdgeCondition::Branch { value } = edge.condition {
                    if value == result {
                        next_id = Some(edge.to);
                        break;
                    }
                }
            }

            next_id
        };

        if let Some(node_id) = next_node_id {
            self.execute_from_node(node_id).await?;
        }

        Ok(())
    }

    /// 获取执行上下文
    pub async fn get_context(&self) -> ExecutionContext {
        self.context.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::workflow::Workflow;
    use crate::workflow::node::WorkflowNode;

    #[tokio::test]
    async fn test_simple_workflow_execution() {
        let mut workflow = Workflow::new("Test", "Test workflow");

        let start = WorkflowNode::start("Start");
        let task = WorkflowNode::task("Task1", "test_task", serde_json::json!({"value": 10}));
        let end = WorkflowNode::end("End");

        let start_id = workflow.add_node(start);
        let task_id = workflow.add_node(task);
        let end_id = workflow.add_node(end);

        workflow.connect(start_id, task_id);
        workflow.connect(task_id, end_id);

        let executor = WorkflowExecutor::new(workflow);
        let result = executor.execute().await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.status, ExecutionStatus::Completed);
    }

    #[tokio::test]
    async fn test_workflow_with_variables() {
        let mut workflow = Workflow::new("Test", "Test");

        workflow.set_variable("count", serde_json::json!(5));

        let start = WorkflowNode::start("Start");
        let end = WorkflowNode::end("End");

        let start_id = workflow.add_node(start);
        let end_id = workflow.add_node(end);

        workflow.connect(start_id, end_id);

        let executor = WorkflowExecutor::new(workflow);
        let result = executor.execute().await;

        assert!(result.is_ok());
    }
}
