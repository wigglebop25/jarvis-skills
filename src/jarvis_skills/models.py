"""
Pydantic models for MCP protocol messages.
"""

from enum import Enum
from typing import Any, Callable, Optional
from pydantic import BaseModel, Field


class ToolParameterType(str, Enum):
    STRING = "string"
    NUMBER = "number"
    INTEGER = "integer"
    BOOLEAN = "boolean"
    ARRAY = "array"
    OBJECT = "object"


class ToolParameter(BaseModel):
    name: str
    type: ToolParameterType
    description: str
    required: bool = False
    default: Optional[Any] = None
    enum: Optional[list[str]] = Field(default=None)


class ToolDefinition(BaseModel):
    name: str
    description: str
    parameters: list[ToolParameter] = Field(default_factory=list)
    handler: Optional[Callable] = Field(default=None, exclude=True)
    
    class Config:
        arbitrary_types_allowed = True
    
    def to_openai_format(self) -> dict:
        """Convert to OpenAI function calling format."""
        properties: dict[str, dict[str, Any]] = {}
        required: list[str] = []
        
        for param in self.parameters:
            prop: dict[str, Any] = {
                "type": param.type.value,
                "description": param.description,
            }
            if param.enum:
                prop["enum"] = param.enum
            if param.default is not None:
                prop["default"] = param.default
            
            properties[param.name] = prop
            
            if param.required:
                required.append(param.name)
        
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                },
            },
        }


class ToolCall(BaseModel):
    id: str = Field(default_factory=lambda: f"call_{id(object())}")
    name: str
    arguments: dict[str, Any] = Field(default_factory=dict)


class ToolResult(BaseModel):
    tool_call_id: str
    success: bool
    result: Optional[Any] = None
    error: Optional[str] = None


class MCPRequest(BaseModel):
    jsonrpc: str = "2.0"
    id: str
    method: str
    params: dict[str, Any] = Field(default_factory=dict)


class MCPResponse(BaseModel):
    jsonrpc: str = "2.0"
    id: str
    result: Optional[Any] = None
    error: Optional[dict] = None
