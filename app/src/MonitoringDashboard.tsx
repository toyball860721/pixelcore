import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AgentInfo {
  id: string;
  name: string;
  status: string;
  model: string;
  message_count: number;
}

interface SystemMetrics {
  cpu_usage: number;
  memory_usage: number;
  active_agents: number;
  total_messages: number;
  uptime: number;
}

export default function MonitoringDashboard() {
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [metrics, setMetrics] = useState<SystemMetrics>({
    cpu_usage: 0,
    memory_usage: 0,
    active_agents: 0,
    total_messages: 0,
    uptime: 0,
  });
  const [logs, setLogs] = useState<string[]>([]);

  // 加载 Agent 列表
  const loadAgents = async () => {
    try {
      const result = await invoke<AgentInfo[]>('get_agents');
      setAgents(result);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  // 模拟系统指标（实际应该从后端获取）
  const updateMetrics = () => {
    setMetrics({
      cpu_usage: Math.random() * 100,
      memory_usage: Math.random() * 100,
      active_agents: agents.filter(a => a.status === 'Active').length,
      total_messages: agents.reduce((sum, a) => sum + a.message_count, 0),
      uptime: Date.now() / 1000,
    });
  };

  useEffect(() => {
    loadAgents();
    const interval = setInterval(() => {
      loadAgents();
      updateMetrics();
    }, 2000); // 每2秒更新一次

    return () => clearInterval(interval);
  }, [agents.length]);

  return (
    <div style={{ padding: '20px', height: '100vh', overflowY: 'auto', backgroundColor: '#f8f9fa' }}>
      <h1 style={{ marginBottom: '30px' }}>📊 Monitoring Dashboard</h1>

      {/* 系统指标卡片 */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '20px', marginBottom: '30px' }}>
        {/* CPU 使用率 */}
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <div style={{ fontSize: '14px', color: '#666', marginBottom: '10px' }}>CPU Usage</div>
          <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#007bff' }}>
            {metrics.cpu_usage.toFixed(1)}%
          </div>
          <div style={{
            marginTop: '10px',
            height: '8px',
            backgroundColor: '#e9ecef',
            borderRadius: '4px',
            overflow: 'hidden',
          }}>
            <div style={{
              height: '100%',
              width: `${metrics.cpu_usage}%`,
              backgroundColor: metrics.cpu_usage > 80 ? '#dc3545' : metrics.cpu_usage > 50 ? '#ffc107' : '#28a745',
              transition: 'width 0.3s ease',
            }} />
          </div>
        </div>

        {/* 内存使用率 */}
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <div style={{ fontSize: '14px', color: '#666', marginBottom: '10px' }}>Memory Usage</div>
          <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#17a2b8' }}>
            {metrics.memory_usage.toFixed(1)}%
          </div>
          <div style={{
            marginTop: '10px',
            height: '8px',
            backgroundColor: '#e9ecef',
            borderRadius: '4px',
            overflow: 'hidden',
          }}>
            <div style={{
              height: '100%',
              width: `${metrics.memory_usage}%`,
              backgroundColor: metrics.memory_usage > 80 ? '#dc3545' : metrics.memory_usage > 50 ? '#ffc107' : '#28a745',
              transition: 'width 0.3s ease',
            }} />
          </div>
        </div>

        {/* 活跃 Agents */}
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <div style={{ fontSize: '14px', color: '#666', marginBottom: '10px' }}>Active Agents</div>
          <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#28a745' }}>
            {metrics.active_agents}
          </div>
          <div style={{ fontSize: '12px', color: '#999', marginTop: '5px' }}>
            Total: {agents.length}
          </div>
        </div>

        {/* 总消息数 */}
        <div style={{
          backgroundColor: 'white',
          padding: '20px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}>
          <div style={{ fontSize: '14px', color: '#666', marginBottom: '10px' }}>Total Messages</div>
          <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#6f42c1' }}>
            {metrics.total_messages}
          </div>
          <div style={{ fontSize: '12px', color: '#999', marginTop: '5px' }}>
            All agents combined
          </div>
        </div>
      </div>

      {/* Agent 状态表格 */}
      <div style={{
        backgroundColor: 'white',
        padding: '20px',
        borderRadius: '8px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        marginBottom: '30px',
      }}>
        <h2 style={{ marginBottom: '20px' }}>Agent Status</h2>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '2px solid #dee2e6' }}>
              <th style={{ padding: '12px', textAlign: 'left', color: '#666' }}>Name</th>
              <th style={{ padding: '12px', textAlign: 'left', color: '#666' }}>Model</th>
              <th style={{ padding: '12px', textAlign: 'left', color: '#666' }}>Status</th>
              <th style={{ padding: '12px', textAlign: 'right', color: '#666' }}>Messages</th>
            </tr>
          </thead>
          <tbody>
            {agents.length === 0 ? (
              <tr>
                <td colSpan={4} style={{ padding: '20px', textAlign: 'center', color: '#999' }}>
                  No agents available
                </td>
              </tr>
            ) : (
              agents.map((agent) => (
                <tr key={agent.id} style={{ borderBottom: '1px solid #dee2e6' }}>
                  <td style={{ padding: '12px', fontWeight: 'bold' }}>{agent.name}</td>
                  <td style={{ padding: '12px', fontSize: '14px', color: '#666' }}>{agent.model}</td>
                  <td style={{ padding: '12px' }}>
                    <span style={{
                      padding: '4px 12px',
                      borderRadius: '12px',
                      fontSize: '12px',
                      fontWeight: 'bold',
                      backgroundColor: agent.status === 'Active' ? '#d4edda' : '#f8d7da',
                      color: agent.status === 'Active' ? '#155724' : '#721c24',
                    }}>
                      {agent.status}
                    </span>
                  </td>
                  <td style={{ padding: '12px', textAlign: 'right', fontWeight: 'bold' }}>
                    {agent.message_count}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* 错误日志 */}
      <div style={{
        backgroundColor: 'white',
        padding: '20px',
        borderRadius: '8px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
      }}>
        <h2 style={{ marginBottom: '20px' }}>Error Logs</h2>
        <div style={{
          backgroundColor: '#f8f9fa',
          padding: '15px',
          borderRadius: '4px',
          fontFamily: 'monospace',
          fontSize: '13px',
          maxHeight: '300px',
          overflowY: 'auto',
        }}>
          {logs.length === 0 ? (
            <div style={{ color: '#999' }}>No errors logged</div>
          ) : (
            logs.map((log, index) => (
              <div key={index} style={{ marginBottom: '5px', color: '#dc3545' }}>
                {log}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
