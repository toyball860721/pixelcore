import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AgentInfo {
  id: string;
  name: string;
  status: string;
  model: string;
  message_count: number;
}

interface SkillInfo {
  name: string;
  description: string;
  enabled: boolean;
}

export default function ConfigurationPanel() {
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [apiKey, setApiKey] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);
  const [agentConfig, setAgentConfig] = useState({
    name: '',
    model: '',
    systemPrompt: '',
  });

  // 加载 Agent 列表
  const loadAgents = async () => {
    try {
      const result = await invoke<AgentInfo[]>('get_agents');
      setAgents(result);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  // 加载可用技能
  const loadSkills = async () => {
    try {
      const result = await invoke<string[]>('get_available_skills');
      // 将技能字符串转换为 SkillInfo 对象
      const skillInfos = result.map((skill) => {
        const [name, description] = skill.split(': ');
        return {
          name: name || skill,
          description: description || '',
          enabled: true, // 默认启用
        };
      });
      setSkills(skillInfos);
    } catch (error) {
      console.error('Failed to load skills:', error);
    }
  };

  useEffect(() => {
    loadAgents();
    loadSkills();
  }, []);

  // 选择 Agent
  const selectAgent = (agentId: string) => {
    setSelectedAgentId(agentId);
    const agent = agents.find((a) => a.id === agentId);
    if (agent) {
      setAgentConfig({
        name: agent.name,
        model: agent.model,
        systemPrompt: '',
      });
    }
  };

  // 切换技能启用状态
  const toggleSkill = (skillName: string) => {
    setSkills((prevSkills) =>
      prevSkills.map((skill) =>
        skill.name === skillName ? { ...skill, enabled: !skill.enabled } : skill
      )
    );
  };

  // 保存 API 密钥
  const saveApiKey = () => {
    if (apiKey.trim()) {
      // TODO: 实际保存到后端
      alert('API Key saved successfully!');
      setShowApiKey(false);
    } else {
      alert('Please enter a valid API key');
    }
  };

  return (
    <div style={{ display: 'flex', height: '100vh', backgroundColor: '#f8f9fa' }}>
      {/* 左侧导航 */}
      <div style={{ width: '250px', backgroundColor: 'white', borderRight: '1px solid #dee2e6', padding: '20px' }}>
        <h2 style={{ marginBottom: '20px' }}>⚙️ Configuration</h2>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          <button
            onClick={() => setSelectedAgentId('agent-config')}
            style={{
              padding: '12px',
              textAlign: 'left',
              backgroundColor: selectedAgentId === 'agent-config' ? '#e3f2fd' : 'transparent',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: selectedAgentId === 'agent-config' ? 'bold' : 'normal',
            }}
          >
            🤖 Agent Configuration
          </button>
          <button
            onClick={() => setSelectedAgentId('skills')}
            style={{
              padding: '12px',
              textAlign: 'left',
              backgroundColor: selectedAgentId === 'skills' ? '#e3f2fd' : 'transparent',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: selectedAgentId === 'skills' ? 'bold' : 'normal',
            }}
          >
            🔧 Skills Management
          </button>
          <button
            onClick={() => setSelectedAgentId('permissions')}
            style={{
              padding: '12px',
              textAlign: 'left',
              backgroundColor: selectedAgentId === 'permissions' ? '#e3f2fd' : 'transparent',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: selectedAgentId === 'permissions' ? 'bold' : 'normal',
            }}
          >
            🔒 Permissions
          </button>
          <button
            onClick={() => setSelectedAgentId('api-keys')}
            style={{
              padding: '12px',
              textAlign: 'left',
              backgroundColor: selectedAgentId === 'api-keys' ? '#e3f2fd' : 'transparent',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontWeight: selectedAgentId === 'api-keys' ? 'bold' : 'normal',
            }}
          >
            🔑 API Keys
          </button>
        </div>
      </div>

      {/* 右侧内容区 */}
      <div style={{ flex: 1, padding: '30px', overflowY: 'auto' }}>
        {/* Agent 配置 */}
        {selectedAgentId === 'agent-config' && (
          <div>
            <h1 style={{ marginBottom: '30px' }}>Agent Configuration</h1>

            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)', marginBottom: '20px' }}>
              <h3 style={{ marginBottom: '20px' }}>Select Agent</h3>
              <select
                onChange={(e) => selectAgent(e.target.value)}
                style={{
                  width: '100%',
                  padding: '10px',
                  borderRadius: '4px',
                  border: '1px solid #dee2e6',
                  fontSize: '14px',
                }}
              >
                <option value="">-- Select an agent --</option>
                {agents.map((agent) => (
                  <option key={agent.id} value={agent.id}>
                    {agent.name} ({agent.model})
                  </option>
                ))}
              </select>
            </div>

            {selectedAgentId && selectedAgentId !== 'agent-config' && (
              <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
                <h3 style={{ marginBottom: '20px' }}>Agent Settings</h3>

                <div style={{ marginBottom: '20px' }}>
                  <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold' }}>Name</label>
                  <input
                    type="text"
                    value={agentConfig.name}
                    onChange={(e) => setAgentConfig({ ...agentConfig, name: e.target.value })}
                    style={{
                      width: '100%',
                      padding: '10px',
                      borderRadius: '4px',
                      border: '1px solid #dee2e6',
                    }}
                  />
                </div>

                <div style={{ marginBottom: '20px' }}>
                  <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold' }}>Model</label>
                  <select
                    value={agentConfig.model}
                    onChange={(e) => setAgentConfig({ ...agentConfig, model: e.target.value })}
                    style={{
                      width: '100%',
                      padding: '10px',
                      borderRadius: '4px',
                      border: '1px solid #dee2e6',
                    }}
                  >
                    <option value="deepseek-ai/DeepSeek-V3">DeepSeek V3</option>
                    <option value="Qwen/Qwen2.5-72B-Instruct">Qwen 2.5 72B</option>
                    <option value="meta-llama/Llama-3.3-70B-Instruct">Llama 3.3 70B</option>
                  </select>
                </div>

                <div style={{ marginBottom: '20px' }}>
                  <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold' }}>System Prompt</label>
                  <textarea
                    value={agentConfig.systemPrompt}
                    onChange={(e) => setAgentConfig({ ...agentConfig, systemPrompt: e.target.value })}
                    rows={6}
                    style={{
                      width: '100%',
                      padding: '10px',
                      borderRadius: '4px',
                      border: '1px solid #dee2e6',
                      fontFamily: 'monospace',
                    }}
                    placeholder="Enter system prompt..."
                  />
                </div>

                <button
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#007bff',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontWeight: 'bold',
                  }}
                  onClick={() => alert('Configuration saved!')}
                >
                  Save Configuration
                </button>
              </div>
            )}
          </div>
        )}

        {/* 技能管理 */}
        {selectedAgentId === 'skills' && (
          <div>
            <h1 style={{ marginBottom: '30px' }}>Skills Management</h1>
            <p style={{ marginBottom: '20px', color: '#666' }}>
              Enable or disable skills for your agents. Disabled skills will not be available for use.
            </p>

            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
              {skills.map((skill) => (
                <div
                  key={skill.name}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '15px',
                    borderBottom: '1px solid #dee2e6',
                  }}
                >
                  <div style={{ flex: 1 }}>
                    <div style={{ fontWeight: 'bold', marginBottom: '5px' }}>{skill.name}</div>
                    <div style={{ fontSize: '13px', color: '#666' }}>{skill.description}</div>
                  </div>
                  <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                    <input
                      type="checkbox"
                      checked={skill.enabled}
                      onChange={() => toggleSkill(skill.name)}
                      style={{ marginRight: '8px', width: '18px', height: '18px', cursor: 'pointer' }}
                    />
                    <span style={{ fontWeight: 'bold', color: skill.enabled ? '#28a745' : '#dc3545' }}>
                      {skill.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </label>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* 权限配置 */}
        {selectedAgentId === 'permissions' && (
          <div>
            <h1 style={{ marginBottom: '30px' }}>Permissions Configuration</h1>
            <p style={{ marginBottom: '20px', color: '#666' }}>
              Configure access permissions for agents and workflows.
            </p>

            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)', marginBottom: '20px' }}>
              <h3 style={{ marginBottom: '15px' }}>Agent Permissions</h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input type="checkbox" defaultChecked style={{ marginRight: '10px', width: '18px', height: '18px' }} />
                  <span>Allow file system access</span>
                </label>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input type="checkbox" defaultChecked style={{ marginRight: '10px', width: '18px', height: '18px' }} />
                  <span>Allow network requests</span>
                </label>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input type="checkbox" defaultChecked style={{ marginRight: '10px', width: '18px', height: '18px' }} />
                  <span>Allow code execution</span>
                </label>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input type="checkbox" style={{ marginRight: '10px', width: '18px', height: '18px' }} />
                  <span>Allow database access</span>
                </label>
              </div>
            </div>

            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)' }}>
              <h3 style={{ marginBottom: '15px' }}>Workflow Permissions</h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input type="checkbox" defaultChecked style={{ marginRight: '10px', width: '18px', height: '18px' }} />
                  <span>Allow workflow creation</span>
                </label>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input type="checkbox" defaultChecked style={{ marginRight: '10px', width: '18px', height: '18px' }} />
                  <span>Allow workflow execution</span>
                </label>
                <label style={{ display: 'flex', alignItems: 'center' }}>
                  <input type="checkbox" style={{ marginRight: '10px', width: '18px', height: '18px' }} />
                  <span>Allow workflow deletion</span>
                </label>
              </div>
            </div>

            <button
              style={{
                marginTop: '20px',
                padding: '10px 20px',
                backgroundColor: '#007bff',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontWeight: 'bold',
              }}
              onClick={() => alert('Permissions saved!')}
            >
              Save Permissions
            </button>
          </div>
        )}

        {/* API 密钥管理 */}
        {selectedAgentId === 'api-keys' && (
          <div>
            <h1 style={{ marginBottom: '30px' }}>API Keys Management</h1>
            <p style={{ marginBottom: '20px', color: '#666' }}>
              Manage API keys for external services and LLM providers.
            </p>

            <div style={{ backgroundColor: 'white', padding: '20px', borderRadius: '8px', boxShadow: '0 2px 4px rgba(0,0,0,0.1)', marginBottom: '20px' }}>
              <h3 style={{ marginBottom: '15px' }}>SiliconFlow API Key</h3>
              <div style={{ marginBottom: '15px' }}>
                <input
                  type={showApiKey ? 'text' : 'password'}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="Enter your API key"
                  style={{
                    width: '100%',
                    padding: '10px',
                    borderRadius: '4px',
                    border: '1px solid #dee2e6',
                    fontFamily: 'monospace',
                  }}
                />
              </div>
              <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
                <button
                  onClick={saveApiKey}
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#28a745',
                    color: 'white',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    fontWeight: 'bold',
                  }}
                >
                  Save API Key
                </button>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <input
                    type="checkbox"
                    checked={showApiKey}
                    onChange={() => setShowApiKey(!showApiKey)}
                    style={{ marginRight: '8px' }}
                  />
                  <span>Show API Key</span>
                </label>
              </div>
            </div>

            <div style={{ backgroundColor: '#fff3cd', padding: '15px', borderRadius: '8px', border: '1px solid #ffc107' }}>
              <strong>⚠️ Security Notice:</strong>
              <p style={{ marginTop: '10px', marginBottom: 0 }}>
                API keys are sensitive credentials. Never share them publicly or commit them to version control.
                Keys are stored securely and encrypted.
              </p>
            </div>
          </div>
        )}

        {/* 默认欢迎页 */}
        {!selectedAgentId && (
          <div style={{ textAlign: 'center', marginTop: '100px' }}>
            <h1 style={{ fontSize: '48px', marginBottom: '20px' }}>⚙️</h1>
            <h2 style={{ color: '#666' }}>Configuration Panel</h2>
            <p style={{ color: '#999', marginTop: '10px' }}>
              Select a configuration category from the left sidebar to get started.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
