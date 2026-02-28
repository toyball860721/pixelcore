#!/usr/bin/env python3
"""
文件系统 MCP 服务器

提供文件和目录操作工具，支持安全的文件系统访问。
"""

import json
import sys
import os
from pathlib import Path
from typing import Any, Dict, List, Optional
from datetime import datetime


class FileSystemMcpServer:
    def __init__(self, base_dir: Optional[str] = None):
        """
        初始化文件系统 MCP 服务器

        Args:
            base_dir: 基础目录，限制文件访问范围（安全沙箱）
        """
        self.base_dir = Path(base_dir) if base_dir else Path.cwd()
        self.tools = [
            {
                "name": "read_file",
                "description": "Read the content of a file",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to read"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "write_file",
                "description": "Write content to a file",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }
            },
            {
                "name": "list_dir",
                "description": "List contents of a directory",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the directory to list"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "file_exists",
                "description": "Check if a file or directory exists",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to check"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "get_file_info",
                "description": "Get information about a file",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file"
                        }
                    },
                    "required": ["path"]
                }
            }
        ]

    def _resolve_path(self, path: str) -> Path:
        """解析路径并确保在基础目录内（安全检查）"""
        full_path = (self.base_dir / path).resolve()

        # 安全检查：确保路径在基础目录内
        try:
            full_path.relative_to(self.base_dir)
        except ValueError:
            raise ValueError(f"Access denied: path outside base directory")

        return full_path

    def handle_initialize(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """处理初始化请求"""
        return {
            "protocol_version": "2024-11-05",
            "capabilities": {"tools": {}},
            "server_info": {
                "name": "filesystem-mcp-server",
                "version": "0.1.0"
            }
        }

    def handle_list_tools(self, params: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """列出所有工具"""
        return {"tools": self.tools}

    def handle_read_file(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """读取文件内容"""
        try:
            path = self._resolve_path(args["path"])
            content = path.read_text(encoding="utf-8")
            return {
                "content": [{"type": "text", "text": content}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error reading file: {str(e)}"}],
                "is_error": True
            }

    def handle_write_file(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """写入文件内容"""
        try:
            path = self._resolve_path(args["path"])
            content = args["content"]

            # 确保父目录存在
            path.parent.mkdir(parents=True, exist_ok=True)

            path.write_text(content, encoding="utf-8")
            return {
                "content": [{"type": "text", "text": f"Successfully wrote to {path.name}"}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error writing file: {str(e)}"}],
                "is_error": True
            }

    def handle_list_dir(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """列出目录内容"""
        try:
            path = self._resolve_path(args["path"])

            if not path.is_dir():
                return {
                    "content": [{"type": "text", "text": f"Not a directory: {path}"}],
                    "is_error": True
                }

            items = []
            for item in sorted(path.iterdir()):
                item_type = "dir" if item.is_dir() else "file"
                items.append(f"{item_type}: {item.name}")

            result = "\n".join(items) if items else "(empty directory)"
            return {
                "content": [{"type": "text", "text": result}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error listing directory: {str(e)}"}],
                "is_error": True
            }

    def handle_file_exists(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """检查文件是否存在"""
        try:
            path = self._resolve_path(args["path"])
            exists = path.exists()
            return {
                "content": [{"type": "text", "text": str(exists)}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error checking file: {str(e)}"}],
                "is_error": True
            }

    def handle_get_file_info(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """获取文件信息"""
        try:
            path = self._resolve_path(args["path"])

            if not path.exists():
                return {
                    "content": [{"type": "text", "text": f"File not found: {path}"}],
                    "is_error": True
                }

            stat = path.stat()
            info = {
                "name": path.name,
                "path": str(path.relative_to(self.base_dir)),
                "type": "directory" if path.is_dir() else "file",
                "size": stat.st_size,
                "modified": datetime.fromtimestamp(stat.st_mtime).isoformat(),
                "created": datetime.fromtimestamp(stat.st_ctime).isoformat(),
            }

            return {
                "content": [{"type": "text", "text": json.dumps(info, indent=2)}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error getting file info: {str(e)}"}],
                "is_error": True
            }

    def handle_call_tool(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """调用工具"""
        name = params.get("name")
        arguments = params.get("arguments", {})

        handlers = {
            "read_file": self.handle_read_file,
            "write_file": self.handle_write_file,
            "list_dir": self.handle_list_dir,
            "file_exists": self.handle_file_exists,
            "get_file_info": self.handle_get_file_info,
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
        print(f"Filesystem MCP server started (base_dir: {self.base_dir})", file=sys.stderr, flush=True)

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
    # 可以通过命令行参数指定基础目录
    base_dir = sys.argv[1] if len(sys.argv) > 1 else None
    server = FileSystemMcpServer(base_dir)
    server.run()
