# MCP 服务器集合

这个目录包含多个实用的 MCP 服务器实现，可以与 PixelCore 集成使用。

## 可用的服务器

### 1. 文件系统服务器 (`filesystem_server.py`)

提供安全的文件系统操作工具。

**工具列表**:
- `read_file` - 读取文件内容
- `write_file` - 写入文件内容
- `list_dir` - 列出目录内容
- `file_exists` - 检查文件是否存在
- `get_file_info` - 获取文件信息（大小、修改时间等）

**安全特性**:
- 沙箱模式：限制文件访问在指定目录内
- 路径验证：防止目录遍历攻击

**使用方法**:
```bash
# 默认使用当前目录作为基础目录
python3 filesystem_server.py

# 指定基础目录
python3 filesystem_server.py /path/to/base/dir
```

**示例**:
```rust
let provider = McpSkillProvider::new(
    "python3",
    &["examples/mcp_servers/filesystem_server.py", "/tmp"]
).await?;
```

### 2. HTTP API 服务器 (`http_server.py`)

提供 HTTP 请求工具，支持常见的 HTTP 方法。

**工具列表**:
- `http_get` - 发送 GET 请求
- `http_post` - 发送 POST 请求
- `http_put` - 发送 PUT 请求
- `http_delete` - 发送 DELETE 请求

**特性**:
- 自动 JSON 处理
- 自定义请求头
- 完整的响应信息（状态码、头部、正文）

**依赖**:
```bash
pip install requests
```

**使用方法**:
```bash
python3 http_server.py
```

**示例**:
```rust
let provider = McpSkillProvider::new(
    "python3",
    &["examples/mcp_servers/http_server.py"]
).await?;

// Agent 可以调用 HTTP API
// "Get the weather from https://api.weather.com/current"
```

### 3. 时间工具服务器 (`time_server.py`)

提供时间和日期相关的工具。

**工具列表**:
- `get_current_time` - 获取当前时间
- `format_time` - 格式化时间
- `parse_time` - 解析时间字符串
- `time_diff` - 计算时间差
- `add_time` - 添加时间间隔

**特性**:
- 支持多种时间格式
- 时间计算和转换
- ISO 8601 标准支持

**使用方法**:
```bash
python3 time_server.py
```

**示例**:
```rust
let provider = McpSkillProvider::new(
    "python3",
    &["examples/mcp_servers/time_server.py"]
).await?;

// Agent 可以处理时间相关问题
// "What time is it now?"
// "Calculate the difference between 2024-01-01 and 2024-12-31"
```

## 集成到 PixelCore

### 方法 1：单个服务器

```rust
use pixelcore_skills::McpSkillProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 启动文件系统服务器
    let fs_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/filesystem_server.py"]
    ).await?;

    // 注册到 Agent
    for skill in fs_provider.skills() {
        agent.register_skill(Arc::clone(skill));
    }

    Ok(())
}
```

### 方法 2：多个服务器

```rust
use pixelcore_skills::McpSkillProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 启动多个服务器
    let fs_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/filesystem_server.py"]
    ).await?;

    let http_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/http_server.py"]
    ).await?;

    let time_provider = McpSkillProvider::new(
        "python3",
        &["examples/mcp_servers/time_server.py"]
    ).await?;

    // 注册所有 Skills
    for skill in fs_provider.skills() {
        agent.register_skill(Arc::clone(skill));
    }
    for skill in http_provider.skills() {
        agent.register_skill(Arc::clone(skill));
    }
    for skill in time_provider.skills() {
        agent.register_skill(Arc::clone(skill));
    }

    Ok(())
}
```

## 测试服务器

### 手动测试

每个服务器都可以通过 stdin/stdout 手动测试：

```bash
# 初始化
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol_version":"2024-11-05","capabilities":{},"client_info":{"name":"test","version":"1.0"}}}' | python3 filesystem_server.py

# 列出工具
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | python3 filesystem_server.py

# 调用工具
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"read_file","arguments":{"path":"README.md"}}}' | python3 filesystem_server.py
```

### 使用 Rust 测试

```bash
# 测试所有服务器
cargo run --example test_all_servers
```

## 创建自定义服务器

参考现有服务器的实现，创建自己的 MCP 服务器：

1. 定义工具列表（`tools`）
2. 实现 `initialize` 方法
3. 实现 `tools/list` 方法
4. 实现 `tools/call` 方法
5. 实现主循环（stdin/stdout 通信）

**模板**:
```python
#!/usr/bin/env python3
import json
import sys
from typing import Any, Dict, Optional

class MyMcpServer:
    def __init__(self):
        self.tools = [
            {
                "name": "my_tool",
                "description": "Description of my tool",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "param": {"type": "string"}
                    },
                    "required": ["param"]
                }
            }
        ]

    def handle_initialize(self, params):
        return {
            "protocol_version": "2024-11-05",
            "capabilities": {"tools": {}},
            "server_info": {"name": "my-server", "version": "0.1.0"}
        }

    def handle_list_tools(self, params):
        return {"tools": self.tools}

    def handle_my_tool(self, args):
        # 实现工具逻辑
        result = f"Processed: {args['param']}"
        return {"content": [{"type": "text", "text": result}]}

    def handle_call_tool(self, params):
        name = params.get("name")
        arguments = params.get("arguments", {})

        if name == "my_tool":
            return self.handle_my_tool(arguments)
        else:
            return {
                "content": [{"type": "text", "text": f"Unknown tool: {name}"}],
                "is_error": True
            }

    def handle_request(self, request):
        method = request.get("method")
        params = request.get("params")
        request_id = request.get("id")

        try:
            if method == "initialize":
                result = self.handle_initialize(params)
            elif method == "tools/list":
                result = self.handle_list_tools(params)
            elif method == "tools/call":
                result = self.handle_call_tool(params)
            elif method == "notifications/initialized":
                return None
            else:
                return {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {"code": -32601, "message": f"Method not found: {method}"}
                }

            return {"jsonrpc": "2.0", "id": request_id, "result": result}
        except Exception as e:
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {"code": -32603, "message": str(e)}
            }

    def run(self):
        print("My MCP server started", file=sys.stderr, flush=True)
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue
            try:
                request = json.loads(line)
                response = self.handle_request(request)
                if response is not None:
                    print(json.dumps(response), flush=True)
            except Exception as e:
                print(f"Error: {e}", file=sys.stderr, flush=True)

if __name__ == "__main__":
    server = MyMcpServer()
    server.run()
```

## 安全注意事项

1. **文件系统服务器**:
   - 始终使用沙箱模式限制访问范围
   - 验证所有路径，防止目录遍历
   - 不要以 root 权限运行

2. **HTTP 服务器**:
   - 注意 SSRF 攻击风险
   - 验证 URL 和请求参数
   - 设置合理的超时时间

3. **通用建议**:
   - 输入验证：验证所有用户输入
   - 错误处理：不要泄露敏感信息
   - 资源限制：限制内存和 CPU 使用
   - 日志记录：记录所有操作用于审计

## 性能优化

1. **连接复用**: 对于频繁调用的工具，考虑连接池
2. **缓存**: 缓存不变的结果（如文件元数据）
3. **异步处理**: 使用异步 I/O 提高并发性能
4. **批量操作**: 支持批量工具调用

## 故障排查

### 服务器无法启动
- 检查 Python 版本（需要 3.7+）
- 检查依赖是否安装（如 requests）
- 检查文件权限

### 工具调用失败
- 检查参数格式是否正确
- 查看 stderr 输出的错误信息
- 验证路径和权限

### 性能问题
- 检查是否有大量重复调用
- 考虑添加缓存
- 检查网络延迟（HTTP 服务器）

## 贡献

欢迎贡献新的 MCP 服务器！建议的服务器类型：
- 数据库服务器（SQLite, PostgreSQL）
- Git 操作服务器
- 图像处理服务器
- 文本处理服务器
- 系统信息服务器

## 许可证

MIT
