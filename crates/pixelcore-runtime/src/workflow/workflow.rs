use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::node::WorkflowNode;
use super::edge::WorkflowEdge;

/// 工作流状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Failed,
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: WorkflowStatus,
    pub nodes: HashMap<Uuid, WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub variables: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workflow {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            status: WorkflowStatus::Draft,
            nodes: HashMap::new(),
            edges: Vec::new(),
            variables: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: WorkflowNode) -> Uuid {
        let node_id = node.id;
        self.nodes.insert(node_id, node);
        self.updated_at = Utc::now();
        node_id
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: WorkflowEdge) {
        self.edges.push(edge);
        self.updated_at = Utc::now();
    }

    /// 连接两个节点
    pub fn connect(&mut self, from: Uuid, to: Uuid) {
        self.add_edge(WorkflowEdge::always(from, to));
    }

    /// 条件连接
    pub fn connect_when(&mut self, from: Uuid, to: Uuid, condition: impl Into<String>) {
        self.add_edge(WorkflowEdge::when(from, to, condition));
    }

    /// 获取节点
    pub fn get_node(&self, id: &Uuid) -> Option<&WorkflowNode> {
        self.nodes.get(id)
    }

    /// 获取节点的出边
    pub fn get_outgoing_edges(&self, node_id: &Uuid) -> Vec<&WorkflowEdge> {
        self.edges.iter().filter(|e| &e.from == node_id).collect()
    }

    /// 获取节点的入边
    pub fn get_incoming_edges(&self, node_id: &Uuid) -> Vec<&WorkflowEdge> {
        self.edges.iter().filter(|e| &e.to == node_id).collect()
    }

    /// 查找开始节点
    pub fn find_start_node(&self) -> Option<&WorkflowNode> {
        self.nodes.values().find(|n| matches!(n.node_type, super::node::NodeType::Start))
    }

    /// 设置变量
    pub fn set_variable(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.variables.insert(key.into(), value);
        self.updated_at = Utc::now();
    }

    /// 获取变量
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// 验证工作流
    pub fn validate(&self) -> Result<(), String> {
        // 检查是否有开始节点
        if self.find_start_node().is_none() {
            return Err("Workflow must have a start node".to_string());
        }

        // 检查是否有结束节点
        let has_end = self.nodes.values().any(|n| matches!(n.node_type, super::node::NodeType::End));
        if !has_end {
            return Err("Workflow must have at least one end node".to_string());
        }

        // 检查边的有效性
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) {
                return Err(format!("Edge references non-existent from node: {}", edge.from));
            }
            if !self.nodes.contains_key(&edge.to) {
                return Err(format!("Edge references non-existent to node: {}", edge.to));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::node::{NodeType, WorkflowNode};

    #[test]
    fn test_create_workflow() {
        let workflow = Workflow::new("Test Workflow", "A test workflow");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.status, WorkflowStatus::Draft);
        assert!(workflow.nodes.is_empty());
        assert!(workflow.edges.is_empty());
    }

    #[test]
    fn test_add_nodes_and_edges() {
        let mut workflow = Workflow::new("Test", "Test");

        let start = WorkflowNode::start("Start");
        let task = WorkflowNode::task("Task1", "do_something", serde_json::json!({}));
        let end = WorkflowNode::end("End");

        let start_id = workflow.add_node(start);
        let task_id = workflow.add_node(task);
        let end_id = workflow.add_node(end);

        workflow.connect(start_id, task_id);
        workflow.connect(task_id, end_id);

        assert_eq!(workflow.nodes.len(), 3);
        assert_eq!(workflow.edges.len(), 2);
    }

    #[test]
    fn test_validate_workflow() {
        let mut workflow = Workflow::new("Test", "Test");

        // 没有节点，验证失败
        assert!(workflow.validate().is_err());

        // 添加开始和结束节点
        let start = WorkflowNode::start("Start");
        let end = WorkflowNode::end("End");

        let start_id = workflow.add_node(start);
        let end_id = workflow.add_node(end);

        workflow.connect(start_id, end_id);

        // 验证成功
        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_workflow_variables() {
        let mut workflow = Workflow::new("Test", "Test");

        workflow.set_variable("count", serde_json::json!(10));
        workflow.set_variable("name", serde_json::json!("Alice"));

        assert_eq!(workflow.get_variable("count"), Some(&serde_json::json!(10)));
        assert_eq!(workflow.get_variable("name"), Some(&serde_json::json!("Alice")));
        assert_eq!(workflow.get_variable("missing"), None);
    }
}
