# 完整的 MCP 集成示例

这个示例展示了如何将多个 MCP 服务器完整集成到 PixelCore Agent 系统中。

## 功能特性

### 1. MCP 服务器集成
- **文件系统服务器**: 提供 5 个文件操作工具（读、写、列表、检查、信息）
- **时间工具服务器**: 提供 5 个时间处理工具（获取、格式化、解析、计算差值、添加时间）
- **HTTP API 服务器**: 提供 4 个 HTTP 请求工具（GET、POST、PUT、DELETE）

### 2. Agent 集成
- 自动发现和注册 MCP 工具
- Agent 可以智能选择和使用工具
- 支持多轮对话和工具调用
- 完整的错误处理和资源清理

### 3. 测试模式
- **无 API Key 模式**: 只测试 MCP 服务器功能
- **完整模式**: 测试 Agent 与 MCP 工具的集成

## 使用方法

### 准备工作

1. 安装 Python 依赖：
```bash
pip install requests  # HTTP 服务器需要
```

2. 设置环境变量（可选，用于 Agent 集成测试）：
```bash
export SILICONFLOW_API_KEY=your-api-key
```

### 运行示例

```bash
# 从项目根目录运行
cargo run --example complete_mcp_demo
```

### 运行模式

#### 模式 1: 只测试 MCP 服务器（无需 API Key）

如果没有设置 `SILICONFLOW_API_KEY`，程序会自动进入此模式：

```
═══ 第一部分：启动 MCP 服务器 ═══

1. 启动文件系统服务器...
   ✅ 文件系统服务器已启动，提供 5 个工具
      - read_file: Read content from a file
      - write_file: Write content to a file
      - list_dir: List files in a directory
      - file_exists: Check if a file exists
      - get_file_info: Get file metadata

2. 启动时间工具服务器...
   ✅ 时间服务器已启动，提供 5 个工具
      - get_current_time: Get current time
      - format_time: Format time string
      - parse_time: Parse time string
      - time_diff: Calculate time difference
      - add_time: Add duration to time

═══ 第二部分：测试工具调用 ═══

测试文件系统工具:
  ✅ write_file: 成功
  ✅ read_file: 成功
     内容: "Hello from PixelCore MCP!..."

测试时间工具:
  ✅ get_current_time: 2024-12-25T18:30:00
  ✅ format_time: 2024年12月25日 18时30分
```

#### 模式 2: 完整测试（包括 Agent 集成）

设置 API Key 后，程序会测试 Agent 使用 MCP 工具：

```
═══ 第一部分：启动 MCP 服务器 ═══
...

═══ 第二部分：创建 Agent 并集成 MCP 工具 ═══

注册 MCP 技能到 Agent...
✅ 已注册 10 个技能

═══ 第三部分：测试 Agent 使用 MCP 工具 ═══

【测试 1】让 Agent 创建并读取文件

Agent 响应:
我已经帮你创建了文件 /tmp/agent_test.txt，内容如下：
这是由 Agent 创建的测试文件。
创建时间：2024-12-25

【测试 2】让 Agent 处理时间信息

Agent 响应:
当前时间是：2024-12-25T18:30:00
格式化后的中文时间：2024年12月25日 18时30分

【测试 3】让 Agent 执行组合任务

Agent 响应:
我已经创建了文件 /tmp/time_log.txt，内容包含：
当前时间：2024-12-25T18:30:00
日志信息：系统运行正常
```

## 代码结构

### 主要函数

1. **main()**: 入口函数，检查 API Key 并选择运行模式
2. **test_mcp_servers_only()**: 只测试 MCP 服务器（无需 API Key）
3. **test_with_agent()**: 完整测试（包括 Agent 集成）
4. **test_filesystem_tools()**: 测试文件系统工具
5. **test_time_tools()**: 测试时间工具

### 关键组件

```rust
// 1. 启动 MCP 服务器
let fs_provider = McpSkillProvider::new(
    "python3",
    &["examples/mcp_servers/filesystem_server.py", "/tmp"]
).await?;

// 2. 创建 Agent
let client = ClawClient::siliconflow(api_key);
let config = AgentConfig::new("MCP Demo Agent", system_prompt)
    .with_model("deepseek-ai/DeepSeek-V3");
let mut agent = ClaudeAgent::with_client(config, client)
    .with_storage(storage);

// 3. 注册 MCP 技能
for skill in fs_provider.skills() {
    agent.register_skill(skill.clone());
}

// 4. 启动 Agent 并处理消息
agent.start().await?;
let response = agent.process(message).await?;
```

## 测试场景

### 场景 1: 文件操作
```
用户: "请帮我创建一个文件 /tmp/agent_test.txt，内容是：
      这是由 Agent 创建的测试文件。
      创建时间：2024-12-25
      然后读取这个文件的内容给我看。"

Agent 会：
1. 调用 write_file 工具创建文件
2. 调用 read_file 工具读取文件
3. 返回文件内容
```

### 场景 2: 时间操作
```
用户: "请告诉我现在的时间，并将 2024-12-25T18:30:00 格式化为中文格式。"

Agent 会：
1. 调用 get_current_time 工具获取当前时间
2. 调用 format_time 工具格式化时间
3. 返回格式化后的结果
```

### 场景 3: 组合操作
```
用户: "请创建一个文件 /tmp/time_log.txt，内容包含当前时间和一条日志信息：'系统运行正常'。"

Agent 会：
1. 调用 get_current_time 工具获取当前时间
2. 组合时间和日志信息
3. 调用 write_file 工具创建文件
4. 返回操作结果
```

## 扩展建议

### 添加更多 MCP 服务器

1. 创建新的 MCP 服务器（参考 `examples/mcp_servers/` 中的示例）
2. 在 `test_with_agent()` 中启动新服务器
3. 注册新服务器的技能到 Agent
4. 更新 Agent 的 system_prompt 说明新工具的用途

### 自定义测试场景

修改 `test_with_agent()` 函数中的测试消息，添加更复杂的任务：

```rust
let message = Message::user(
    "请帮我分析 /tmp 目录下的所有文件，\
     统计文件数量，并将结果保存到 /tmp/analysis.txt"
);
```

## 故障排除

### 问题 1: HTTP 服务器启动失败
```
⚠️  HTTP 服务器启动失败: ...
   提示: 运行 'pip install requests' 安装依赖
```
**解决方案**: 安装 requests 库
```bash
pip install requests
```

### 问题 2: 文件访问被拒绝
```
Access denied: path outside base directory
```
**解决方案**: 文件系统服务器有安全沙箱，只能访问指定的基础目录（默认 `/tmp`）

### 问题 3: Agent 处理失败
```
❌ Agent 处理失败: agent 'MCP Demo Agent' is not running
```
**解决方案**: 确保在调用 `agent.process()` 前调用了 `agent.start()`

## 性能优化

1. **并行启动服务器**: 可以使用 `tokio::join!` 并行启动多个 MCP 服务器
2. **复用 Agent**: 对于多个测试，可以复用同一个 Agent 实例
3. **批量注册技能**: 可以一次性注册所有技能，而不是循环注册

## 安全注意事项

1. **API Key 保护**: 不要在代码中硬编码 API Key，使用环境变量
2. **文件系统沙箱**: 文件系统服务器限制了访问路径，防止恶意访问
3. **工具权限**: 谨慎授予 Agent 访问敏感工具的权限

## 相关文档

- [MCP 服务器集合](../MCP_SERVERS_COLLECTION.md)
- [MCP 服务器 README](mcp_servers/README.md)
- [Python 测试脚本](test_mcp_servers.py)
