use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 工作流节点类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeType {
    /// 开始节点
    Start,
    /// 结束节点
    End,
    /// 任务节点
    Task {
        /// 任务名称
        task_name: String,
        /// 任务参数
        params: serde_json::Value,
    },
    /// 决策节点（条件分支）
    Decision {
        /// 条件表达式
        condition: String,
    },
    /// 循环节点
    Loop {
        /// 循环条件
        condition: String,
        /// 最大迭代次数
        max_iterations: usize,
    },
    /// 并行节点
    Parallel {
        /// 并行分支数
        branches: usize,
    },
}

/// 工作流节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub metadata: serde_json::Value,
}

impl WorkflowNode {
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            node_type,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn start(name: impl Into<String>) -> Self {
        Self::new(name, NodeType::Start)
    }

    pub fn end(name: impl Into<String>) -> Self {
        Self::new(name, NodeType::End)
    }

    pub fn task(name: impl Into<String>, task_name: impl Into<String>, params: serde_json::Value) -> Self {
        Self::new(name, NodeType::Task {
            task_name: task_name.into(),
            params,
        })
    }

    pub fn decision(name: impl Into<String>, condition: impl Into<String>) -> Self {
        Self::new(name, NodeType::Decision {
            condition: condition.into(),
        })
    }

    pub fn loop_node(name: impl Into<String>, condition: impl Into<String>, max_iterations: usize) -> Self {
        Self::new(name, NodeType::Loop {
            condition: condition.into(),
            max_iterations,
        })
    }

    pub fn parallel(name: impl Into<String>, branches: usize) -> Self {
        Self::new(name, NodeType::Parallel { branches })
    }
}
