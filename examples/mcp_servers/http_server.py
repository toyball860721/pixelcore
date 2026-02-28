#!/usr/bin/env python3
"""
HTTP API MCP 服务器

提供 HTTP 请求工具，支持 GET、POST、PUT、DELETE 等方法。
"""

import json
import sys
from typing import Any, Dict, Optional
try:
    import requests
except ImportError:
    print("Error: requests library not found. Install with: pip install requests", file=sys.stderr)
    sys.exit(1)


class HttpApiMcpServer:
    def __init__(self):
        self.tools = [
            {
                "name": "http_get",
                "description": "Send a GET request to a URL",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to send the request to"
                        },
                        "headers": {
                            "type": "object",
                            "description": "Optional headers to include",
                            "additionalProperties": {"type": "string"}
                        }
                    },
                    "required": ["url"]
                }
            },
            {
                "name": "http_post",
                "description": "Send a POST request to a URL",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to send the request to"
                        },
                        "body": {
                            "type": "string",
                            "description": "The request body (JSON string)"
                        },
                        "headers": {
                            "type": "object",
                            "description": "Optional headers to include",
                            "additionalProperties": {"type": "string"}
                        }
                    },
                    "required": ["url"]
                }
            },
            {
                "name": "http_put",
                "description": "Send a PUT request to a URL",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to send the request to"
                        },
                        "body": {
                            "type": "string",
                            "description": "The request body (JSON string)"
                        },
                        "headers": {
                            "type": "object",
                            "description": "Optional headers to include",
                            "additionalProperties": {"type": "string"}
                        }
                    },
                    "required": ["url"]
                }
            },
            {
                "name": "http_delete",
                "description": "Send a DELETE request to a URL",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to send the request to"
                        },
                        "headers": {
                            "type": "object",
                            "description": "Optional headers to include",
                            "additionalProperties": {"type": "string"}
                        }
                    },
                    "required": ["url"]
                }
            }
        ]

    def handle_initialize(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """处理初始化请求"""
        return {
            "protocol_version": "2024-11-05",
            "capabilities": {"tools": {}},
            "server_info": {
                "name": "http-api-mcp-server",
                "version": "0.1.0"
            }
        }

    def handle_list_tools(self, params: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """列出所有工具"""
        return {"tools": self.tools}

    def _format_response(self, response: requests.Response) -> str:
        """格式化 HTTP 响应"""
        result = {
            "status_code": response.status_code,
            "headers": dict(response.headers),
            "body": response.text
        }

        # 尝试解析 JSON
        try:
            result["json"] = response.json()
        except:
            pass

        return json.dumps(result, indent=2)

    def handle_http_get(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """处理 GET 请求"""
        try:
            url = args["url"]
            headers = args.get("headers", {})

            response = requests.get(url, headers=headers, timeout=30)
            return {
                "content": [{"type": "text", "text": self._format_response(response)}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_http_post(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """处理 POST 请求"""
        try:
            url = args["url"]
            body = args.get("body", "")
            headers = args.get("headers", {})

            # 如果 body 是 JSON 字符串，设置 Content-Type
            if body and "Content-Type" not in headers:
                try:
                    json.loads(body)
                    headers["Content-Type"] = "application/json"
                except:
                    pass

            response = requests.post(url, data=body, headers=headers, timeout=30)
            return {
                "content": [{"type": "text", "text": self._format_response(response)}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_http_put(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """处理 PUT 请求"""
        try:
            url = args["url"]
            body = args.get("body", "")
            headers = args.get("headers", {})

            if body and "Content-Type" not in headers:
                try:
                    json.loads(body)
                    headers["Content-Type"] = "application/json"
                except:
                    pass

            response = requests.put(url, data=body, headers=headers, timeout=30)
            return {
                "content": [{"type": "text", "text": self._format_response(response)}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_http_delete(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """处理 DELETE 请求"""
        try:
            url = args["url"]
            headers = args.get("headers", {})

            response = requests.delete(url, headers=headers, timeout=30)
            return {
                "content": [{"type": "text", "text": self._format_response(response)}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_call_tool(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """调用工具"""
        name = params.get("name")
        arguments = params.get("arguments", {})

        handlers = {
            "http_get": self.handle_http_get,
            "http_post": self.handle_http_post,
            "http_put": self.handle_http_put,
            "http_delete": self.handle_http_delete,
        }

        handler = handlers.get(name)
        if handler:
            return handler(arguments)
        else:
            return {
                "content": [{"type": "text", "text": f"Unknown tool: {name}"}],
                "is_error": True
            }

    def handle_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
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
        print("HTTP API MCP server started", file=sys.stderr, flush=True)

        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue

            try:
                request = json.loads(line)
                response = self.handle_request(request)

                if response is not None:
                    print(json.dumps(response), flush=True)
            except json.JSONDecodeError as e:
                print(f"JSON decode error: {e}", file=sys.stderr, flush=True)
            except Exception as e:
                print(f"Error: {e}", file=sys.stderr, flush=True)


if __name__ == "__main__":
    server = HttpApiMcpServer()
    server.run()
