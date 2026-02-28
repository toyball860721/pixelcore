#!/usr/bin/env python3
"""
时间工具 MCP 服务器

提供时间和日期相关的工具。
"""

import json
import sys
from typing import Any, Dict, Optional
from datetime import datetime, timedelta
import time


class TimeMcpServer:
    def __init__(self):
        self.tools = [
            {
                "name": "get_current_time",
                "description": "Get the current date and time",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "description": "Optional format string (e.g., '%Y-%m-%d %H:%M:%S')"
                        },
                        "timezone": {
                            "type": "string",
                            "description": "Optional timezone (e.g., 'UTC', 'Asia/Shanghai')"
                        }
                    }
                }
            },
            {
                "name": "format_time",
                "description": "Format a timestamp or datetime string",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "time": {
                            "type": "string",
                            "description": "Time to format (ISO format or timestamp)"
                        },
                        "format": {
                            "type": "string",
                            "description": "Format string (e.g., '%Y-%m-%d %H:%M:%S')"
                        }
                    },
                    "required": ["time", "format"]
                }
            },
            {
                "name": "parse_time",
                "description": "Parse a time string into ISO format",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "time_string": {
                            "type": "string",
                            "description": "Time string to parse"
                        },
                        "format": {
                            "type": "string",
                            "description": "Format of the input string (e.g., '%Y-%m-%d %H:%M:%S')"
                        }
                    },
                    "required": ["time_string"]
                }
            },
            {
                "name": "time_diff",
                "description": "Calculate the difference between two times",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "time1": {
                            "type": "string",
                            "description": "First time (ISO format)"
                        },
                        "time2": {
                            "type": "string",
                            "description": "Second time (ISO format)"
                        },
                        "unit": {
                            "type": "string",
                            "description": "Unit for result (seconds, minutes, hours, days)",
                            "enum": ["seconds", "minutes", "hours", "days"]
                        }
                    },
                    "required": ["time1", "time2"]
                }
            },
            {
                "name": "add_time",
                "description": "Add a duration to a time",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "time": {
                            "type": "string",
                            "description": "Base time (ISO format)"
                        },
                        "days": {"type": "number", "description": "Days to add"},
                        "hours": {"type": "number", "description": "Hours to add"},
                        "minutes": {"type": "number", "description": "Minutes to add"},
                        "seconds": {"type": "number", "description": "Seconds to add"}
                    },
                    "required": ["time"]
                }
            }
        ]

    def handle_initialize(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """处理初始化请求"""
        return {
            "protocol_version": "2024-11-05",
            "capabilities": {"tools": {}},
            "server_info": {
                "name": "time-mcp-server",
                "version": "0.1.0"
            }
        }

    def handle_list_tools(self, params: Optional[Dict[str, Any]]) -> Dict[str, Any]:
        """列出所有工具"""
        return {"tools": self.tools}

    def handle_get_current_time(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """获取当前时间"""
        try:
            now = datetime.now()
            format_str = args.get("format")

            if format_str:
                result = now.strftime(format_str)
            else:
                result = now.isoformat()

            return {
                "content": [{"type": "text", "text": result}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_format_time(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """格式化时间"""
        try:
            time_str = args["time"]
            format_str = args["format"]

            # 尝试解析时间
            try:
                # 尝试作为 ISO 格式解析
                dt = datetime.fromisoformat(time_str)
            except:
                # 尝试作为时间戳解析
                dt = datetime.fromtimestamp(float(time_str))

            result = dt.strftime(format_str)
            return {
                "content": [{"type": "text", "text": result}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_parse_time(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """解析时间字符串"""
        try:
            time_string = args["time_string"]
            format_str = args.get("format")

            if format_str:
                dt = datetime.strptime(time_string, format_str)
            else:
                # 尝试常见格式
                for fmt in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d", "%Y/%m/%d", "%d/%m/%Y"]:
                    try:
                        dt = datetime.strptime(time_string, fmt)
                        break
                    except:
                        continue
                else:
                    raise ValueError("Could not parse time string")

            result = dt.isoformat()
            return {
                "content": [{"type": "text", "text": result}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_time_diff(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """计算时间差"""
        try:
            time1 = datetime.fromisoformat(args["time1"])
            time2 = datetime.fromisoformat(args["time2"])
            unit = args.get("unit", "seconds")

            diff = (time2 - time1).total_seconds()

            if unit == "minutes":
                diff = diff / 60
            elif unit == "hours":
                diff = diff / 3600
            elif unit == "days":
                diff = diff / 86400

            result = {
                "difference": diff,
                "unit": unit,
                "time1": time1.isoformat(),
                "time2": time2.isoformat()
            }

            return {
                "content": [{"type": "text", "text": json.dumps(result, indent=2)}]
            }
        except Exception as e:
            return {
                "content": [{"type": "text", "text": f"Error: {str(e)}"}],
                "is_error": True
            }

    def handle_add_time(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """添加时间"""
        try:
            base_time = datetime.fromisoformat(args["time"])
            days = args.get("days", 0)
            hours = args.get("hours", 0)
            minutes = args.get("minutes", 0)
            seconds = args.get("seconds", 0)

            delta = timedelta(
                days=days,
                hours=hours,
                minutes=minutes,
                seconds=seconds
            )

            result_time = base_time + delta
            return {
                "content": [{"type": "text", "text": result_time.isoformat()}]
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
            "get_current_time": self.handle_get_current_time,
            "format_time": self.handle_format_time,
            "parse_time": self.handle_parse_time,
            "time_diff": self.handle_time_diff,
            "add_time": self.handle_add_time,
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
        print("Time MCP server started", file=sys.stderr, flush=True)

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
    server = TimeMcpServer()
    server.run()
