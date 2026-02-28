use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 边的条件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeCondition {
    /// 无条件（总是执行）
    Always,
    /// 条件表达式
    Expression {
        /// 表达式字符串
        expr: String,
    },
    /// 决策分支（true/false）
    Branch {
        /// 分支值
        value: bool,
    },
    /// 并行分支索引
    ParallelBranch {
        /// 分支索引
        index: usize,
    },
}

/// 工作流边（连接）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub condition: EdgeCondition,
    pub metadata: serde_json::Value,
}

impl WorkflowEdge {
    pub fn new(from: Uuid, to: Uuid, condition: EdgeCondition) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            condition,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn always(from: Uuid, to: Uuid) -> Self {
        Self::new(from, to, EdgeCondition::Always)
    }

    pub fn when(from: Uuid, to: Uuid, expr: impl Into<String>) -> Self {
        Self::new(from, to, EdgeCondition::Expression {
            expr: expr.into(),
        })
    }

    pub fn branch(from: Uuid, to: Uuid, value: bool) -> Self {
        Self::new(from, to, EdgeCondition::Branch { value })
    }

    pub fn parallel_branch(from: Uuid, to: Uuid, index: usize) -> Self {
        Self::new(from, to, EdgeCondition::ParallelBranch { index })
    }
}
