"""
MCP Server - Handles tool registration and invocation.

Supports:
- Direct handler registration (for local use)
- HTTP endpoint exposure (for remote clients)
- JSON-RPC protocol compliance
"""

import json
from typing import Any, Callable, Optional
from .registry import ToolRegistry
from .models import (
    ToolCall,
    ToolResult,
    MCPRequest,
    MCPResponse,
)


class MCPServer:
    """MCP Server for tool management and execution."""
    
    def __init__(self, name: str = "jarvis-skills"):
        self.name = name
        self.registry = ToolRegistry()
        self._running = False
    
    def register_tool(
        self,
        name: str,
        description: str,
        handler: Callable,
        parameters: Optional[list] = None,
    ) -> None:
        """Register a tool with the server."""
        self.registry.register(name, description, handler, parameters)
    
    def tool(self, name: str, description: str):
        """Decorator to register a tool."""
        def decorator(func: Callable) -> Callable:
            self.register_tool(name, description, func)
            return func
        return decorator
    
    def list_tools(self) -> list[dict]:
        """List all tools in OpenAI format."""
        return self.registry.get_openai_tools()
    
    async def handle_request(self, request: MCPRequest) -> MCPResponse:
        """Handle an MCP JSON-RPC request."""
        method = request.method
        
        if method == "tools/list":
            return MCPResponse(
                id=request.id,
                result={"tools": self.list_tools()},
            )
        
        elif method == "tools/call":
            tool_name = request.params.get("name")
            arguments = request.params.get("arguments", {})
            
            if not tool_name:
                return MCPResponse(
                    id=request.id,
                    error={"code": -32602, "message": "Missing tool name"},
                )
            
            call = ToolCall(
                id=request.id,
                name=tool_name,
                arguments=arguments,
            )
            result = await self.registry.execute(call)
            
            if result.success:
                return MCPResponse(id=request.id, result=result.result)
            else:
                return MCPResponse(
                    id=request.id,
                    error={"code": -32000, "message": result.error},
                )
        
        else:
            return MCPResponse(
                id=request.id,
                error={"code": -32601, "message": f"Unknown method: {method}"},
            )
    
    def handle_request_sync(self, request: MCPRequest) -> MCPResponse:
        """Handle an MCP request synchronously."""
        method = request.method
        
        if method == "tools/list":
            return MCPResponse(
                id=request.id,
                result={"tools": self.list_tools()},
            )
        
        elif method == "tools/call":
            tool_name = request.params.get("name")
            arguments = request.params.get("arguments", {})
            
            if not tool_name:
                return MCPResponse(
                    id=request.id,
                    error={"code": -32602, "message": "Missing tool name"},
                )
            
            call = ToolCall(
                id=request.id,
                name=tool_name,
                arguments=arguments,
            )
            result = self.registry.execute_sync(call)
            
            if result.success:
                return MCPResponse(id=request.id, result=result.result)
            else:
                return MCPResponse(
                    id=request.id,
                    error={"code": -32000, "message": result.error},
                )
        
        else:
            return MCPResponse(
                id=request.id,
                error={"code": -32601, "message": f"Unknown method: {method}"},
            )
    
    async def execute_tool(self, name: str, **kwargs) -> ToolResult:
        """Execute a tool directly by name."""
        call = ToolCall(name=name, arguments=kwargs)
        return await self.registry.execute(call)
    
    def execute_tool_sync(self, name: str, **kwargs) -> ToolResult:
        """Execute a tool synchronously."""
        call = ToolCall(name=name, arguments=kwargs)
        return self.registry.execute_sync(call)
    
    def parse_json_rpc(self, data: str | bytes | dict) -> MCPRequest:
        """Parse JSON-RPC request from various formats."""
        if isinstance(data, bytes):
            data = data.decode("utf-8")
        if isinstance(data, str):
            data = json.loads(data)
        
        return MCPRequest(
            id=data.get("id", "unknown"),
            method=data.get("method", ""),
            params=data.get("params", {}),
        )
    
    def format_response(self, response: MCPResponse) -> str:
        """Format response as JSON string."""
        return response.model_dump_json()
