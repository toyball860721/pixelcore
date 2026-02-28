# 示例 MCP 服务器

这是一个简单的 MCP (Model Context Protocol) 服务器示例，用于演示如何实现本地 MCP 服务器。

## 功能

提供以下工具：

- **add**: 两个数字相加
- **multiply**: 两个数字相乘
- **echo**: 回显输入的文本

## 运行

```bash
python3 server.py
```

服务器通过 stdin/stdout 进行 JSON-RPC 通信。

## 测试

可以手动测试服务器：

```bash
# 初始化
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol_version":"2024-11-05","capabilities":{},"client_info":{"name":"test","version":"1.0"}}}' | python3 server.py

# 列出工具
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | python3 server.py

# 调用工具
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"add","arguments":{"a":5,"b":3}}}' | python3 server.py
```

## 与 PixelCore 集成

在 Rust 代码中使用：

```rust
use pixelcore_claw::LocalMcpClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 启动 MCP 服务器
    let client = LocalMcpClient::new(
        "python3",
        &["examples/mcp_server/server.py"]
    ).await?;

    // 列出工具
    let tools = client.list_tools().await?;
    println!("Available tools: {:?}", tools);

    // 调用工具
    let result = client.call_tool(
        "add",
        Some(serde_json::json!({"a": 5, "b": 3}))
    ).await?;
    println!("Result: {:?}", result);

    Ok(())
}
```

## MCP 协议

本服务器实现了 MCP 协议的基本功能：

1. **initialize**: 初始化连接，协商协议版本和能力
2. **tools/list**: 列出所有可用的工具
3. **tools/call**: 调用指定的工具

更多信息请参考 [MCP 规范](https://modelcontextprotocol.io/)
