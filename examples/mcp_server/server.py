#!/usr/bin/env python3
"""
简单的 MCP 服务器示例

提供以下工具：
- add: 两个数字相加
- multiply: 两个数字相乘
- echo: 回显输入的文本
"""

import json
import sys
from typing import Any, Dict, List, Optional


class McpServer:
    def __init__(self):
        self.tools = [
            {
                "name": "add",
                "description": "Add two numbers together",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number"}
                    },
                    "required": ["a", "b"]
                }
            },
            {
                "name": "multiply",
                "description": "Multiply two numbers",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number"}
                    },
                    "required": ["a", "b"]
                }
            },
            {
                "name": "echo",
                "description": "Echo back the input text",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to echo"}
                    },
                    "required": ["text"]
                }
            }
        ]

    def handle_initialize(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """处理初始化请求"""
        return {
            "protocol_version": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "server_info": {
                "name": "example-mcp-server",
                "version": "0.1.0"
            }
        }

    def handle_list_tools(self, params: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """列出所有工具"""
        return {"tools": self.tools}

    def handle_call_tool(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """调用工具"""
        name = params.get("name")
        arguments = params.get("arguments", {})

        if name == "add":
            a = arguments.get("a", 0)
            b = arguments.get("b", 0)
            result = a + b
            return {
                "content": [
                    {"type": "text", "text": str(result)}
                ]
            }
        elif name == "multiply":
            a = arguments.get("a", 0)
            b = arguments.get("b", 0)
            result = a * b
            return {
                "content": [
                    {"type": "text", "text": str(result)}
                ]
            }
        elif name == "echo":
            text = arguments.get("text", "")
            return {
                "content": [
                    {"type": "text", "text": text}
                ]
            }
        else:
            return {
                "content": [
                    {"type": "text", "text": f"Unknown tool: {name}"}
                ],
                "is_error": True
            }

    def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """处理 JSON-RPC 请求"""
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
                # 忽略通知
                return None
            else:
                return {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {
                        "code": -32601,
                        "message": f"Method not found: {method}"
                    }
                }

            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": result
            }
        except Exception as e:
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32603,
                    "message": str(e)
                }
            }

    def run(self):
        """运行服务器主循环"""
        # 写入启动日志到 stderr
        print("MCP server started", file=sys.stderr, flush=True)

        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue

            try:
                request = json.loads(line)
                response = self.handle_request(request)

                if response is not None:
                    # 写入响应到 stdout
                    print(json.dumps(response), flush=True)
            except json.JSONDecodeError as e:
                print(f"JSON decode error: {e}", file=sys.stderr, flush=True)
            except Exception as e:
                print(f"Error: {e}", file=sys.stderr, flush=True)


if __name__ == "__main__":
    server = McpServer()
    server.run()
