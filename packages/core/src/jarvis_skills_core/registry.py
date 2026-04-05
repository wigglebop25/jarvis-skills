"""
Tool Registry - Manages available tools and their handlers.
"""

import inspect
from typing import Any, Callable, Optional
from .models import (
    ToolDefinition,
    ToolParameter,
    ToolParameterType,
    ToolCall,
    ToolResult,
)


class ToolRegistry:
    """Registry for MCP tools."""
    
    def __init__(self):
        self._tools: dict[str, ToolDefinition] = {}
    
    def register(
        self,
        name: str,
        description: str,
        handler: Callable,
        parameters: Optional[list[ToolParameter]] = None,
    ) -> None:
        """Register a tool with the registry."""
        if parameters is None:
            parameters = self._infer_parameters(handler)
        
        tool = ToolDefinition(
            name=name,
            description=description,
            parameters=parameters,
            handler=handler,
        )
        self._tools[name] = tool
    
    def register_tool(self, tool: ToolDefinition) -> None:
        """Register a pre-built ToolDefinition."""
        self._tools[tool.name] = tool
    
    def unregister(self, name: str) -> bool:
        """Remove a tool from the registry."""
        if name in self._tools:
            del self._tools[name]
            return True
        return False
    
    def get_tool(self, name: str) -> Optional[ToolDefinition]:
        """Get a tool by name."""
        return self._tools.get(name)
    
    def list_tools(self) -> list[ToolDefinition]:
        """List all registered tools."""
        return list(self._tools.values())
    
    def get_openai_tools(self) -> list[dict]:
        """Get all tools in OpenAI function calling format."""
        return [tool.to_openai_format() for tool in self._tools.values()]
    
    async def execute(self, call: ToolCall) -> ToolResult:
        """Execute a tool call."""
        tool = self._tools.get(call.name)
        
        if tool is None:
            return ToolResult(
                tool_call_id=call.id,
                success=False,
                error=f"Unknown tool: {call.name}",
            )
        
        if tool.handler is None:
            return ToolResult(
                tool_call_id=call.id,
                success=False,
                error=f"Tool {call.name} has no handler",
            )
        
        try:
            if inspect.iscoroutinefunction(tool.handler):
                result = await tool.handler(**call.arguments)
            else:
                result = tool.handler(**call.arguments)
            
            return ToolResult(
                tool_call_id=call.id,
                success=True,
                result=result,
            )
        except Exception as e:
            return ToolResult(
                tool_call_id=call.id,
                success=False,
                error=str(e),
            )
    
    def execute_sync(self, call: ToolCall) -> ToolResult:
        """Execute a tool call synchronously."""
        tool = self._tools.get(call.name)
        
        if tool is None:
            return ToolResult(
                tool_call_id=call.id,
                success=False,
                error=f"Unknown tool: {call.name}",
            )
        
        if tool.handler is None:
            return ToolResult(
                tool_call_id=call.id,
                success=False,
                error=f"Tool {call.name} has no handler",
            )
        
        try:
            result = tool.handler(**call.arguments)
            return ToolResult(
                tool_call_id=call.id,
                success=True,
                result=result,
            )
        except Exception as e:
            return ToolResult(
                tool_call_id=call.id,
                success=False,
                error=str(e),
            )
    
    def _infer_parameters(self, handler: Callable) -> list[ToolParameter]:
        """Infer parameters from function signature."""
        sig = inspect.signature(handler)
        params = []
        
        type_mapping = {
            str: ToolParameterType.STRING,
            int: ToolParameterType.INTEGER,
            float: ToolParameterType.NUMBER,
            bool: ToolParameterType.BOOLEAN,
            list: ToolParameterType.ARRAY,
            dict: ToolParameterType.OBJECT,
        }
        
        for name, param in sig.parameters.items():
            if name in ("self", "cls"):
                continue
            
            annotation = param.annotation
            param_type = ToolParameterType.STRING
            
            if annotation != inspect.Parameter.empty:
                param_type = type_mapping.get(annotation, ToolParameterType.STRING)
            
            has_default = param.default != inspect.Parameter.empty
            default_value = param.default if has_default else None
            
            params.append(
                ToolParameter(
                    name=name,
                    type=param_type,
                    description=f"Parameter {name}",
                    required=not has_default,
                    default=default_value,
                )
            )
        
        return params
