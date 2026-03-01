use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use super::workflow::Workflow;
use super::executor::ExecutionContext;

/// 工作流持久化错误
#[derive(Debug)]
pub enum PersistenceError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::IoError(e) => write!(f, "IO error: {}", e),
            PersistenceError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::IoError(e)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(e: serde_json::Error) -> Self {
        PersistenceError::SerializationError(e)
    }
}

/// 工作流持久化管理器
pub struct WorkflowPersistence;

impl WorkflowPersistence {
    /// 保存工作流到文件
    pub async fn save_workflow(
        workflow: &Workflow,
        path: impl AsRef<Path>,
    ) -> Result<(), PersistenceError> {
        let json = serde_json::to_string_pretty(workflow)?;
        fs::write(path, json).await?;
        Ok(())
    }

    /// 从文件加载工作流
    pub async fn load_workflow(
        path: impl AsRef<Path>,
    ) -> Result<Workflow, PersistenceError> {
        let json = fs::read_to_string(path).await?;
        let workflow = serde_json::from_str(&json)?;
        Ok(workflow)
    }

    /// 保存执行上下文
    pub async fn save_context(
        context: &ExecutionContext,
        path: impl AsRef<Path>,
    ) -> Result<(), PersistenceError> {
        let json = serde_json::to_string_pretty(context)?;
        fs::write(path, json).await?;
        Ok(())
    }

    /// 加载执行上下文
    pub async fn load_context(
        path: impl AsRef<Path>,
    ) -> Result<ExecutionContext, PersistenceError> {
        let json = fs::read_to_string(path).await?;
        let context = serde_json::from_str(&json)?;
        Ok(context)
    }

    /// 保存工作流和执行上下文到目录
    pub async fn save_workflow_with_context(
        workflow: &Workflow,
        context: &ExecutionContext,
        dir: impl AsRef<Path>,
    ) -> Result<(), PersistenceError> {
        let dir = dir.as_ref();

        // 创建目录（如果不存在）
        fs::create_dir_all(dir).await?;

        // 保存工作流
        let workflow_path = dir.join("workflow.json");
        Self::save_workflow(workflow, workflow_path).await?;

        // 保存执行上下文
        let context_path = dir.join("context.json");
        Self::save_context(context, context_path).await?;

        Ok(())
    }

    /// 从目录加载工作流和执行上下文
    pub async fn load_workflow_with_context(
        dir: impl AsRef<Path>,
    ) -> Result<(Workflow, ExecutionContext), PersistenceError> {
        let dir = dir.as_ref();

        // 加载工作流
        let workflow_path = dir.join("workflow.json");
        let workflow = Self::load_workflow(workflow_path).await?;

        // 加载执行上下文
        let context_path = dir.join("context.json");
        let context = Self::load_context(context_path).await?;

        Ok((workflow, context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::node::WorkflowNode;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_load_workflow() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_workflow.json");

        // 创建工作流
        let mut workflow = Workflow::new("Test Workflow", "Test description");
        let start = WorkflowNode::start("Start");
        let end = WorkflowNode::end("End");
        let start_id = workflow.add_node(start);
        let end_id = workflow.add_node(end);
        workflow.connect(start_id, end_id);

        // 保存
        WorkflowPersistence::save_workflow(&workflow, file_path.as_path())
            .await
            .unwrap();

        // 加载
        let loaded = WorkflowPersistence::load_workflow(file_path.as_path())
            .await
            .unwrap();

        assert_eq!(loaded.name, workflow.name);
        assert_eq!(loaded.nodes.len(), workflow.nodes.len());
        assert_eq!(loaded.edges.len(), workflow.edges.len());
    }

    #[tokio::test]
    async fn test_save_and_load_context() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_context.json");

        // 创建执行上下文
        let workflow_id = uuid::Uuid::new_v4();
        let mut context = ExecutionContext::new(workflow_id);
        context.set_variable("test_var", serde_json::json!(42));

        // 保存
        WorkflowPersistence::save_context(&context, file_path.as_path())
            .await
            .unwrap();

        // 加载
        let loaded = WorkflowPersistence::load_context(file_path.as_path())
            .await
            .unwrap();

        assert_eq!(loaded.workflow_id, context.workflow_id);
        assert_eq!(loaded.get_variable("test_var"), Some(&serde_json::json!(42)));
    }

    #[tokio::test]
    async fn test_save_and_load_workflow_with_context() {
        let dir = tempdir().unwrap();

        // 创建工作流和上下文
        let mut workflow = Workflow::new("Test", "Test");
        let start = WorkflowNode::start("Start");
        let end = WorkflowNode::end("End");
        let start_id = workflow.add_node(start);
        let end_id = workflow.add_node(end);
        workflow.connect(start_id, end_id);

        let mut context = ExecutionContext::new(workflow.id);
        context.set_variable("count", serde_json::json!(10));

        // 保存
        WorkflowPersistence::save_workflow_with_context(&workflow, &context, dir.path())
            .await
            .unwrap();

        // 加载
        let (loaded_workflow, loaded_context) =
            WorkflowPersistence::load_workflow_with_context(dir.path())
                .await
                .unwrap();

        assert_eq!(loaded_workflow.name, workflow.name);
        assert_eq!(loaded_context.workflow_id, context.workflow_id);
        assert_eq!(loaded_context.get_variable("count"), Some(&serde_json::json!(10)));
    }
}
