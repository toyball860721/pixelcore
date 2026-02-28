# MCP 集成快速参考

## 快速开始

### 1. 运行完整示例

```bash
# 只测试 MCP 服务器（无需 API Key）
cargo run --example complete_mcp_demo

# 完整测试（包括 Agent 集成）
export SILICONFLOW_API_KEY=your-api-key
cargo run --example complete_mcp_demo
```

### 2. 查看文档

- **完整示例**: [examples/COMPLETE_MCP_DEMO.md](examples/COMPLETE_MCP_DEMO.md)
- **MCP 服务器集合**: [MCP_SERVERS_COLLECTION.md](MCP_SERVERS_COLLECTION.md)
- **服务器文档**: [examples/mcp_servers/README.md](examples/mcp_servers/README.md)

## 代码示例

### 启动 MCP 服务器

```rust
use pixelcore_skills::McpSkillProvider;

// 文件系统服务器
let fs_provider = McpSkillProvider::new(
    "python3",
    &["examples/mcp_servers/filesystem_server.py", "/tmp"]
).await?;

// 时间服务器
let time_provider = McpSkillProvider::new(
    "python3",
    &["examples/mcp_servers/time_server.py"]
).await?;

// HTTP 服务器
let http_provider = McpSkillProvider::new(
    "python3",
    &["examples/mcp_servers/http_server.py"]
).await?;
```

### 创建 Agent 并集成 MCP 工具

```rust
use pixelcore_agents::ClaudeAgent;
use pixelcore_runtime::{Agent, AgentConfig, Message};
use pixelcore_claw::ClawClient;
use pixelcore_storage::Storage;

// 1. 创建 ClawClient
let client = ClawClient::siliconflow(api_key);

// 2. 创建 Agent 配置
let config = AgentConfig::new("My Agent", "You are a helpful assistant")
    .with_model("deepseek-ai/DeepSeek-V3");

// 3. 创建 Agent
let mut agent = ClaudeAgent::with_client(config, client)
    .with_storage(Storage::new());

// 4. 注册 MCP 技能
for skill in fs_provider.skills() {
    agent.register_skill(skill.clone());
}
for skill in time_provider.skills() {
    agent.register_skill(skill.clone());
}

// 5. 启动 Agent
agent.start().await?;

// 6. 处理消息
let message = Message::user("请帮我创建一个文件...");
let response = agent.process(message).await?;
println!("{}", response.content);
```

## 可用的 MCP 工具

### 文件系统工具（5 个）
- `read_file` - 读取文件内容
- `write_file` - 写入文件内容
- `list_dir` - 列出目录内容
- `file_exists` - 检查文件是否存在
- `get_file_info` - 获取文件信息

### HTTP API 工具（4 个）
- `http_get` - 发送 GET 请求
- `http_post` - 发送 POST 请求
- `http_put` - 发送 PUT 请求
- `http_delete` - 发送 DELETE 请求

### 时间工具（5 个）
- `get_current_time` - 获取当前时间
- `format_time` - 格式化时间
- `parse_time` - 解析时间字符串
- `time_diff` - 计算时间差
- `add_time` - 添加时间间隔

## 常见问题

### Q: HTTP 服务器启动失败？
A: 安装依赖：`pip install requests`

### Q: 文件访问被拒绝？
A: 文件系统服务器有安全沙箱，只能访问指定的基础目录（默认 `/tmp`）

### Q: Agent 处理失败？
A: 确保在调用 `agent.process()` 前调用了 `agent.start()`

## 文件位置

### 核心实现
- `crates/pixelcore-claw/src/mcp_types.rs` - MCP 协议类型
- `crates/pixelcore-claw/src/stdio_transport.rs` - Stdio 传输层
- `crates/pixelcore-claw/src/local_mcp.rs` - LocalMcpClient
- `crates/pixelcore-skills/src/builtins/mcp_skill.rs` - McpSkill 包装器
- `crates/pixelcore-skills/src/builtins/mcp_provider.rs` - McpSkillProvider

### MCP 服务器
- `examples/mcp_servers/filesystem_server.py` - 文件系统服务器
- `examples/mcp_servers/http_server.py` - HTTP API 服务器
- `examples/mcp_servers/time_server.py` - 时间工具服务器

### 示例和测试
- `examples/complete_mcp_demo.rs` - 完整的 Agent 集成示例 ⭐
- `examples/test_all_servers.rs` - Rust 测试程序
- `examples/test_mcp_servers.py` - Python 测试脚本

## 下一步

1. 运行完整示例：`cargo run --example complete_mcp_demo`
2. 阅读文档：`examples/COMPLETE_MCP_DEMO.md`
3. 创建自己的 MCP 服务器：参考 `examples/mcp_servers/README.md`
4. 集成到你的 Agent：参考上面的代码示例
