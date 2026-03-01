import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'
import WorkflowEditor from './WorkflowEditor'
import MonitoringDashboard from './MonitoringDashboard'

interface AgentInfo {
  id: string
  name: string
  status: string
  model: string
  message_count: number
}

interface ChatMessage {
  role: string
  content: string
  timestamp: string
}

function App() {
  const [activeTab, setActiveTab] = useState<'agents' | 'workflows' | 'monitoring'>('agents')
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null)
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [inputMessage, setInputMessage] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  // 创建 Agent 表单状态
  const [showCreateForm, setShowCreateForm] = useState(false)
  const [agentName, setAgentName] = useState('')
  const [agentModel, setAgentModel] = useState('deepseek-ai/DeepSeek-V3')
  const [systemPrompt, setSystemPrompt] = useState('You are a helpful AI assistant with access to various tools.')
  const [apiKey, setApiKey] = useState('')

  // 加载 Agent 列表
  const loadAgents = async () => {
    try {
      const result = await invoke<AgentInfo[]>('get_agents')
      setAgents(result)
    } catch (error) {
      console.error('Failed to load agents:', error)
    }
  }

  // 创建 Agent
  const createAgent = async () => {
    if (!agentName.trim() || !apiKey.trim()) {
      alert('Please fill in all required fields')
      return
    }

    try {
      setIsLoading(true)
      const agentId = await invoke<string>('create_agent', {
        name: agentName,
        model: agentModel,
        systemPrompt,
        apiKey,
      })

      alert(`Agent created successfully! ID: ${agentId}`)
      setShowCreateForm(false)
      setAgentName('')
      setApiKey('')
      await loadAgents()
    } catch (error) {
      console.error('Failed to create agent:', error)
      alert(`Failed to create agent: ${error}`)
    } finally {
      setIsLoading(false)
    }
  }

  // 删除 Agent
  const deleteAgent = async (agentId: string) => {
    if (!confirm('Are you sure you want to delete this agent?')) return

    try {
      await invoke('delete_agent', { agentId })
      if (selectedAgentId === agentId) {
        setSelectedAgentId(null)
        setMessages([])
      }
      await loadAgents()
    } catch (error) {
      console.error('Failed to delete agent:', error)
      alert(`Failed to delete agent: ${error}`)
    }
  }

  // 选择 Agent
  const selectAgent = async (agentId: string) => {
    setSelectedAgentId(agentId)
    try {
      const history = await invoke<ChatMessage[]>('get_history', { agentId })
      setMessages(history)
    } catch (error) {
      console.error('Failed to load history:', error)
    }
  }

  // 发送消息
  const sendMessage = async () => {
    if (!selectedAgentId || !inputMessage.trim()) return

    const userMessage = inputMessage
    setInputMessage('')
    setIsLoading(true)

    // 添加用户消息到界面
    setMessages(prev => [...prev, {
      role: 'user',
      content: userMessage,
      timestamp: new Date().toISOString()
    }])

    try {
      const response = await invoke<ChatMessage>('send_message', {
        agentId: selectedAgentId,
        content: userMessage,
      })

      setMessages(prev => [...prev, response])
    } catch (error) {
      console.error('Failed to send message:', error)
      alert(`Failed to send message: ${error}`)
    } finally {
      setIsLoading(false)
    }
  }

  // 初始加载
  useEffect(() => {
    loadAgents()
    const interval = setInterval(loadAgents, 5000) // 每5秒刷新
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="app">
      {/* 顶部导航栏 */}
      <div style={{
        display: 'flex',
        borderBottom: '2px solid #ddd',
        backgroundColor: '#f8f9fa',
        padding: '0'
      }}>
        <button
          onClick={() => setActiveTab('agents')}
          style={{
            padding: '15px 30px',
            border: 'none',
            backgroundColor: activeTab === 'agents' ? '#007bff' : 'transparent',
            color: activeTab === 'agents' ? 'white' : '#333',
            cursor: 'pointer',
            fontSize: '16px',
            fontWeight: activeTab === 'agents' ? 'bold' : 'normal',
            borderBottom: activeTab === 'agents' ? '3px solid #0056b3' : 'none',
          }}
        >
          🤖 Agents
        </button>
        <button
          onClick={() => setActiveTab('workflows')}
          style={{
            padding: '15px 30px',
            border: 'none',
            backgroundColor: activeTab === 'workflows' ? '#007bff' : 'transparent',
            color: activeTab === 'workflows' ? 'white' : '#333',
            cursor: 'pointer',
            fontSize: '16px',
            fontWeight: activeTab === 'workflows' ? 'bold' : 'normal',
            borderBottom: activeTab === 'workflows' ? '3px solid #0056b3' : 'none',
          }}
        >
          🔄 Workflows
        </button>
        <button
          onClick={() => setActiveTab('monitoring')}
          style={{
            padding: '15px 30px',
            border: 'none',
            backgroundColor: activeTab === 'monitoring' ? '#007bff' : 'transparent',
            color: activeTab === 'monitoring' ? 'white' : '#333',
            cursor: 'pointer',
            fontSize: '16px',
            fontWeight: activeTab === 'monitoring' ? 'bold' : 'normal',
            borderBottom: activeTab === 'monitoring' ? '3px solid #0056b3' : 'none',
          }}
        >
          📊 Monitoring
        </button>
      </div>

      {/* 根据选中的标签页显示不同内容 */}
      {activeTab === 'monitoring' ? (
        <MonitoringDashboard />
      ) : activeTab === 'workflows' ? (
        <WorkflowEditor />
      ) : (
        <>
          {/* 原有的 Agent 聊天界面 */}
          {/* 侧边栏 */}
          <div className="sidebar">
        <div className="sidebar-header">
          <h1>PixelCore</h1>
          <button
            className="create-btn"
            onClick={() => setShowCreateForm(true)}
          >
            + New Agent
          </button>
        </div>

        <div className="agents-list">
          <h2>Agents ({agents.length})</h2>
          {agents.length === 0 ? (
            <p className="empty-state">No agents yet</p>
          ) : (
            agents.map((agent) => (
              <div
                key={agent.id}
                className={`agent-item ${selectedAgentId === agent.id ? 'selected' : ''}`}
                onClick={() => selectAgent(agent.id)}
              >
                <div className="agent-item-info">
                  <h3>{agent.name}</h3>
                  <p className="agent-model">{agent.model}</p>
                  <p className="agent-stats">
                    {agent.message_count} messages • {agent.status}
                  </p>
                </div>
                <button
                  className="delete-btn"
                  onClick={(e) => {
                    e.stopPropagation()
                    deleteAgent(agent.id)
                  }}
                >
                  ×
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* 主内容区 */}
      <div className="main-content">
        {selectedAgentId ? (
          <>
            {/* 消息列表 */}
            <div className="messages-container">
              {messages.length === 0 ? (
                <div className="empty-chat">
                  <h2>Start a conversation</h2>
                  <p>Send a message to begin chatting with your agent</p>
                </div>
              ) : (
                messages.map((msg, idx) => (
                  <div key={idx} className={`message ${msg.role}`}>
                    <div className="message-role">{msg.role}</div>
                    <div className="message-content">{msg.content}</div>
                    <div className="message-time">
                      {new Date(msg.timestamp).toLocaleTimeString()}
                    </div>
                  </div>
                ))
              )}
            </div>

            {/* 输入框 */}
            <div className="input-container">
              <input
                type="text"
                value={inputMessage}
                onChange={(e) => setInputMessage(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && !isLoading && sendMessage()}
                placeholder="Type your message..."
                disabled={isLoading}
              />
              <button
                onClick={sendMessage}
                disabled={isLoading || !inputMessage.trim()}
              >
                {isLoading ? 'Sending...' : 'Send'}
              </button>
            </div>
          </>
        ) : (
          <div className="no-agent-selected">
            <h2>Welcome to PixelCore</h2>
            <p>Select an agent from the sidebar or create a new one to get started</p>
          </div>
        )}
      </div>

      {/* 创建 Agent 模态框 */}
      {showCreateForm && (
        <div className="modal-overlay" onClick={() => setShowCreateForm(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>Create New Agent</h2>

            <div className="form-group">
              <label>Agent Name *</label>
              <input
                type="text"
                value={agentName}
                onChange={(e) => setAgentName(e.target.value)}
                placeholder="My Assistant"
              />
            </div>

            <div className="form-group">
              <label>Model</label>
              <select
                value={agentModel}
                onChange={(e) => setAgentModel(e.target.value)}
              >
                <option value="deepseek-ai/DeepSeek-V3">DeepSeek V3</option>
                <option value="Qwen/Qwen2.5-72B-Instruct">Qwen 2.5 72B</option>
                <option value="meta-llama/Llama-3.3-70B-Instruct">Llama 3.3 70B</option>
              </select>
            </div>

            <div className="form-group">
              <label>System Prompt</label>
              <textarea
                value={systemPrompt}
                onChange={(e) => setSystemPrompt(e.target.value)}
                rows={4}
                placeholder="You are a helpful AI assistant..."
              />
            </div>

            <div className="form-group">
              <label>API Key *</label>
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Your SiliconFlow API Key"
              />
            </div>

            <div className="modal-actions">
              <button
                className="cancel-btn"
                onClick={() => setShowCreateForm(false)}
              >
                Cancel
              </button>
              <button
                className="create-btn"
                onClick={createAgent}
                disabled={isLoading}
              >
                {isLoading ? 'Creating...' : 'Create Agent'}
              </button>
            </div>
          </div>
        </div>
      )}
        </>
      )}
    </div>
  )
}

export default App
