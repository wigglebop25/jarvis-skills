"""
JARVIS Skills Core - MCP Infrastructure

Provides the core components for building JARVIS MCP skills:
- MCPServer: JSON-RPC compliant server
- ToolRegistry: Tool registration and execution
- Models: Protocol data types
"""

from .models import (
    MCPRequest,
    MCPResponse,
    ToolCall,
    ToolDefinition,
    ToolParameter,
    ToolParameterType,
    ToolResult,
)
from .registry import ToolRegistry
from .server import MCPServer

__version__ = "0.1.0"

__all__ = [
    # Server
    "MCPServer",
    "ToolRegistry",
    # Models
    "MCPRequest",
    "MCPResponse",
    "ToolCall",
    "ToolDefinition",
    "ToolParameter",
    "ToolParameterType",
    "ToolResult",
]
