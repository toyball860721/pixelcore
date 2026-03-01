import { useState, useCallback, useEffect } from 'react';
import {
  ReactFlow,
  MiniMap,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  Node,
  BackgroundVariant,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { invoke } from '@tauri-apps/api/core';

interface WorkflowInfo {
  id: string;
  name: string;
  description: string;
  status: string;
  node_count: number;
  edge_count: number;
  created_at: string;
  updated_at: string;
}

interface WorkflowExecutionStatus {
  workflow_id: string;
  status: string;
  current_node: string | null;
  completed_nodes: string[];
  failed_nodes: string[];
  progress: number;
}

const initialNodes: Node[] = [
  {
    id: '1',
    type: 'input',
    data: { label: 'Start' },
    position: { x: 250, y: 5 },
  },
];

const initialEdges: Edge[] = [];

export default function WorkflowEditor() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [workflows, setWorkflows] = useState<WorkflowInfo[]>([]);
  const [selectedWorkflowId, setSelectedWorkflowId] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [workflowName, setWorkflowName] = useState('');
  const [workflowDescription, setWorkflowDescription] = useState('');
  const [executionStatus, setExecutionStatus] = useState<WorkflowExecutionStatus | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);

  // 加载工作流列表
  const loadWorkflows = useCallback(async () => {
    try {
      const result = await invoke<WorkflowInfo[]>('get_workflows');
      setWorkflows(result);
    } catch (error) {
      console.error('Failed to load workflows:', error);
    }
  }, []);

  useEffect(() => {
    loadWorkflows();
  }, [loadWorkflows]);

  // 创建新工作流
  const createWorkflow = async () => {
    if (!workflowName.trim()) {
      alert('Please enter a workflow name');
      return;
    }

    try {
      const workflowId = await invoke<string>('create_workflow', {
        name: workflowName,
        description: workflowDescription,
      });

      alert(`Workflow created successfully! ID: ${workflowId}`);
      setShowCreateForm(false);
      setWorkflowName('');
      setWorkflowDescription('');
      await loadWorkflows();
    } catch (error) {
      console.error('Failed to create workflow:', error);
      alert(`Failed to create workflow: ${error}`);
    }
  };

  // 删除工作流
  const deleteWorkflow = async (workflowId: string) => {
    if (!confirm('Are you sure you want to delete this workflow?')) return;

    try {
      await invoke('delete_workflow', { workflowId });
      if (selectedWorkflowId === workflowId) {
        setSelectedWorkflowId(null);
        setNodes(initialNodes);
        setEdges(initialEdges);
      }
      await loadWorkflows();
    } catch (error) {
      console.error('Failed to delete workflow:', error);
      alert(`Failed to delete workflow: ${error}`);
    }
  };

  // 选择工作流
  const selectWorkflow = async (workflowId: string) => {
    setSelectedWorkflowId(workflowId);
    try {
      const workflow = await invoke<any>('get_workflow', { workflowId });
      console.log('Loaded workflow:', workflow);
      // TODO: 将工作流数据转换为 React Flow 节点和边
    } catch (error) {
      console.error('Failed to load workflow:', error);
    }
  };

  // 连接节点
  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  // 添加新节点
  const addNode = (type: 'start' | 'task' | 'end') => {
    const newNode: Node = {
      id: `${nodes.length + 1}`,
      type: type === 'start' ? 'input' : type === 'end' ? 'output' : 'default',
      data: { label: `${type.charAt(0).toUpperCase() + type.slice(1)} Node` },
      position: { x: Math.random() * 400, y: Math.random() * 400 },
    };
    setNodes((nds) => nds.concat(newNode));
  };

  // 执行工作流
  const executeWorkflow = async () => {
    if (!selectedWorkflowId) {
      alert('Please select a workflow first');
      return;
    }

    try {
      setIsExecuting(true);
      await invoke('execute_workflow', { workflowId: selectedWorkflowId });

      // 开始轮询执行状态
      const intervalId = setInterval(async () => {
        try {
          const status = await invoke<WorkflowExecutionStatus>('get_workflow_execution_status', {
            workflowId: selectedWorkflowId,
          });
          setExecutionStatus(status);

          // 如果执行完成或失败，停止轮询
          if (status.status === 'Completed' || status.status === 'Failed') {
            clearInterval(intervalId);
            setIsExecuting(false);
          }
        } catch (error) {
          console.error('Failed to get execution status:', error);
          clearInterval(intervalId);
          setIsExecuting(false);
        }
      }, 1000); // 每秒更新一次

      // 5分钟后自动停止轮询
      setTimeout(() => {
        clearInterval(intervalId);
        setIsExecuting(false);
      }, 300000);
    } catch (error) {
      console.error('Failed to execute workflow:', error);
      alert(`Failed to execute workflow: ${error}`);
      setIsExecuting(false);
    }
  };

  // 停止工作流执行
  const stopWorkflow = async () => {
    if (!selectedWorkflowId) return;

    try {
      await invoke('update_workflow_status', {
        workflowId: selectedWorkflowId,
        status: 'Paused',
      });
      setIsExecuting(false);
      alert('Workflow paused');
    } catch (error) {
      console.error('Failed to stop workflow:', error);
      alert(`Failed to stop workflow: ${error}`);
    }
  };

  return (
    <div style={{ width: '100%', height: '100vh', display: 'flex' }}>
      {/* 左侧工作流列表 */}
      <div style={{ width: '300px', borderRight: '1px solid #ddd', padding: '20px', overflowY: 'auto' }}>
        <h2>Workflows</h2>

        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          style={{
            width: '100%',
            padding: '10px',
            marginBottom: '20px',
            backgroundColor: '#007bff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {showCreateForm ? 'Cancel' : 'Create New Workflow'}
        </button>

        {showCreateForm && (
          <div style={{ marginBottom: '20px', padding: '15px', backgroundColor: '#f5f5f5', borderRadius: '4px' }}>
            <input
              type="text"
              placeholder="Workflow Name"
              value={workflowName}
              onChange={(e) => setWorkflowName(e.target.value)}
              style={{ width: '100%', padding: '8px', marginBottom: '10px', borderRadius: '4px', border: '1px solid #ddd' }}
            />
            <textarea
              placeholder="Description"
              value={workflowDescription}
              onChange={(e) => setWorkflowDescription(e.target.value)}
              style={{ width: '100%', padding: '8px', marginBottom: '10px', borderRadius: '4px', border: '1px solid #ddd', minHeight: '60px' }}
            />
            <button
              onClick={createWorkflow}
              style={{
                width: '100%',
                padding: '8px',
                backgroundColor: '#28a745',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Create
            </button>
          </div>
        )}

        <div>
          {workflows.map((workflow) => (
            <div
              key={workflow.id}
              style={{
                padding: '10px',
                marginBottom: '10px',
                backgroundColor: selectedWorkflowId === workflow.id ? '#e3f2fd' : '#f9f9f9',
                borderRadius: '4px',
                cursor: 'pointer',
                border: '1px solid #ddd',
              }}
              onClick={() => selectWorkflow(workflow.id)}
            >
              <div style={{ fontWeight: 'bold', marginBottom: '5px' }}>{workflow.name}</div>
              <div style={{ fontSize: '12px', color: '#666' }}>
                {workflow.node_count} nodes, {workflow.edge_count} edges
              </div>
              <div style={{ fontSize: '11px', color: '#999', marginTop: '5px' }}>
                Status: {workflow.status}
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  deleteWorkflow(workflow.id);
                }}
                style={{
                  marginTop: '8px',
                  padding: '4px 8px',
                  backgroundColor: '#dc3545',
                  color: 'white',
                  border: 'none',
                  borderRadius: '3px',
                  cursor: 'pointer',
                  fontSize: '12px',
                }}
              >
                Delete
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* 右侧工作流画布 */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        {/* 工具栏 */}
        <div style={{ padding: '10px', borderBottom: '1px solid #ddd', backgroundColor: '#f5f5f5', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div>
            <button onClick={() => addNode('start')} style={{ marginRight: '10px', padding: '8px 16px' }}>
              Add Start Node
            </button>
            <button onClick={() => addNode('task')} style={{ marginRight: '10px', padding: '8px 16px' }}>
              Add Task Node
            </button>
            <button onClick={() => addNode('end')} style={{ marginRight: '10px', padding: '8px 16px' }}>
              Add End Node
            </button>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            {/* 执行状态显示 */}
            {executionStatus && (
              <div style={{
                padding: '8px 16px',
                backgroundColor: executionStatus.status === 'Completed' ? '#d4edda' :
                                 executionStatus.status === 'Failed' ? '#f8d7da' :
                                 executionStatus.status === 'Active' ? '#fff3cd' : '#e2e3e5',
                borderRadius: '4px',
                fontSize: '14px',
                fontWeight: 'bold',
              }}>
                Status: {executionStatus.status} | Progress: {Math.round(executionStatus.progress * 100)}%
              </div>
            )}

            {/* 执行/停止按钮 */}
            {!isExecuting ? (
              <button
                onClick={executeWorkflow}
                disabled={!selectedWorkflowId}
                style={{
                  padding: '8px 16px',
                  backgroundColor: selectedWorkflowId ? '#28a745' : '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: selectedWorkflowId ? 'pointer' : 'not-allowed',
                  fontWeight: 'bold',
                }}
              >
                ▶ Execute
              </button>
            ) : (
              <button
                onClick={stopWorkflow}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#dc3545',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                  fontWeight: 'bold',
                }}
              >
                ⏸ Pause
              </button>
            )}
          </div>
        </div>

        {/* React Flow 画布 */}
        <div style={{ flex: 1 }}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            fitView
          >
            <Controls />
            <MiniMap />
            <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
          </ReactFlow>
        </div>
      </div>
    </div>
  );
}
