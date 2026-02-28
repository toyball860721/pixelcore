#!/usr/bin/env python3
"""
完整的 MCP 服务器测试程序

这个程序演示如何：
1. 启动多个 MCP 服务器
2. 测试每个服务器的所有工具
3. 展示实际使用场景
"""

import subprocess
import json
import sys
import time
from typing import Dict, Any, Optional


class McpTester:
    def __init__(self, command: str, args: list):
        self.command = command
        self.args = args
        self.process = None
        self.request_id = 1

    def start(self):
        """启动 MCP 服务器"""
        self.process = subprocess.Popen(
            [self.command] + self.args,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        time.sleep(0.1)  # 等待服务器启动

    def send_request(self, method: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """发送 JSON-RPC 请求"""
        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method
        }
        if params:
            request["params"] = params

        self.request_id += 1

        # 发送请求
        request_json = json.dumps(request)
        self.process.stdin.write(request_json + "\n")
        self.process.stdin.flush()

        # 读取响应
        response_line = self.process.stdout.readline()
        return json.loads(response_line)

    def initialize(self):
        """初始化连接"""
        params = {
            "protocol_version": "2024-11-05",
            "capabilities": {},
            "client_info": {"name": "test", "version": "1.0"}
        }
        response = self.send_request("initialize", params)

        # 发送 initialized 通知
        self.send_request("notifications/initialized")

        return response

    def list_tools(self):
        """列出所有工具"""
        response = self.send_request("tools/list")
        return response.get("result", {}).get("tools", [])

    def call_tool(self, name: str, arguments: Dict[str, Any]):
        """调用工具"""
        params = {"name": name, "arguments": arguments}
        response = self.send_request("tools/call", params)
        return response.get("result", {})

    def stop(self):
        """停止服务器"""
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=5)


def print_section(title: str):
    """打印分节标题"""
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}\n")


def print_result(tool_name: str, result: Dict[str, Any]):
    """打印工具调用结果"""
    is_error = result.get("is_error", False)
    content = result.get("content", [])

    if content:
        text = content[0].get("text", "")
        status = "❌ ERROR" if is_error else "✅ SUCCESS"
        print(f"{status} {tool_name}:")
        print(f"  {text[:200]}{'...' if len(text) > 200 else ''}")
    else:
        print(f"⚠️  {tool_name}: No content")


def test_filesystem_server():
    """测试文件系统服务器"""
    print_section("测试文件系统服务器")

    tester = McpTester("python3", ["examples/mcp_servers/filesystem_server.py", "/tmp"])
    tester.start()

    # 初始化
    init_result = tester.initialize()
    server_info = init_result.get("result", {}).get("server_info", {})
    print(f"服务器: {server_info.get('name')} v{server_info.get('version')}")

    # 列出工具
    tools = tester.list_tools()
    print(f"发现 {len(tools)} 个工具: {', '.join(t['name'] for t in tools)}\n")

    # 测试 write_file
    print("1. 写入文件...")
    result = tester.call_tool("write_file", {
        "path": "mcp_test.txt",
        "content": "Hello from MCP!\nThis is a test file.\n测试中文内容。"
    })
    print_result("write_file", result)

    # 测试 read_file
    print("\n2. 读取文件...")
    result = tester.call_tool("read_file", {"path": "mcp_test.txt"})
    print_result("read_file", result)

    # 测试 file_exists
    print("\n3. 检查文件是否存在...")
    result = tester.call_tool("file_exists", {"path": "mcp_test.txt"})
    print_result("file_exists", result)

    # 测试 get_file_info
    print("\n4. 获取文件信息...")
    result = tester.call_tool("get_file_info", {"path": "mcp_test.txt"})
    print_result("get_file_info", result)

    # 测试 list_dir
    print("\n5. 列出目录内容...")
    result = tester.call_tool("list_dir", {"path": "."})
    print_result("list_dir", result)

    tester.stop()


def test_time_server():
    """测试时间服务器"""
    print_section("测试时间工具服务器")

    tester = McpTester("python3", ["examples/mcp_servers/time_server.py"])
    tester.start()

    # 初始化
    init_result = tester.initialize()
    server_info = init_result.get("result", {}).get("server_info", {})
    print(f"服务器: {server_info.get('name')} v{server_info.get('version')}")

    # 列出工具
    tools = tester.list_tools()
    print(f"发现 {len(tools)} 个工具: {', '.join(t['name'] for t in tools)}\n")

    # 测试 get_current_time
    print("1. 获取当前时间...")
    result = tester.call_tool("get_current_time", {})
    print_result("get_current_time", result)

    # 测试 format_time
    print("\n2. 格式化时间...")
    result = tester.call_tool("format_time", {
        "time": "2024-01-01T12:00:00",
        "format": "%Y年%m月%d日 %H:%M:%S"
    })
    print_result("format_time", result)

    # 测试 parse_time
    print("\n3. 解析时间字符串...")
    result = tester.call_tool("parse_time", {
        "time_string": "2024-12-25 18:30:00",
        "format": "%Y-%m-%d %H:%M:%S"
    })
    print_result("parse_time", result)

    # 测试 time_diff
    print("\n4. 计算时间差...")
    result = tester.call_tool("time_diff", {
        "time1": "2024-01-01T00:00:00",
        "time2": "2024-12-31T23:59:59",
        "unit": "days"
    })
    print_result("time_diff", result)

    # 测试 add_time
    print("\n5. 添加时间间隔...")
    result = tester.call_tool("add_time", {
        "time": "2024-01-01T00:00:00",
        "days": 7,
        "hours": 3,
        "minutes": 30
    })
    print_result("add_time", result)

    tester.stop()


def test_http_server():
    """测试 HTTP 服务器"""
    print_section("测试 HTTP API 服务器")

    try:
        tester = McpTester("python3", ["examples/mcp_servers/http_server.py"])
        tester.start()

        # 初始化
        init_result = tester.initialize()
        server_info = init_result.get("result", {}).get("server_info", {})
        print(f"服务器: {server_info.get('name')} v{server_info.get('version')}")

        # 列出工具
        tools = tester.list_tools()
        print(f"发现 {len(tools)} 个工具: {', '.join(t['name'] for t in tools)}\n")

        # 测试 http_get
        print("1. 发送 GET 请求...")
        result = tester.call_tool("http_get", {
            "url": "https://httpbin.org/get"
        })
        print_result("http_get", result)

        # 测试 http_post
        print("\n2. 发送 POST 请求...")
        result = tester.call_tool("http_post", {
            "url": "https://httpbin.org/post",
            "body": json.dumps({"test": "data", "message": "Hello MCP"})
        })
        print_result("http_post", result)

        tester.stop()

    except Exception as e:
        print(f"⚠️  HTTP 服务器测试跳过: {e}")
        print("   提示: 运行 'pip install requests' 安装依赖")


def main():
    """主函数"""
    print("""
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║           MCP 服务器完整测试程序                              ║
║                                                              ║
║  测试所有 MCP 服务器的功能和工具                              ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
    """)

    try:
        # 测试文件系统服务器
        test_filesystem_server()

        # 测试时间服务器
        test_time_server()

        # 测试 HTTP 服务器
        test_http_server()

        # 总结
        print_section("测试完成")
        print("✅ 所有 MCP 服务器测试完成！")
        print("\n下一步:")
        print("  1. 运行 Rust 集成测试: cargo run --example test_all_servers")
        print("  2. 与 Agent 集成: cargo run --example mcp_skills_demo")
        print("  3. 创建自定义 MCP 服务器")

    except KeyboardInterrupt:
        print("\n\n⚠️  测试被用户中断")
    except Exception as e:
        print(f"\n\n❌ 测试失败: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
