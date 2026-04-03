"""
JARVIS Skills - MCP Server and Tools

This module provides:
- MCP Server for tool registration and invocation
- Built-in tools (system info, volume, spotify, network)
- Tool registry for dynamic tool management
"""

from jarvis_skills.server import MCPServer
from jarvis_skills.registry import ToolRegistry
from jarvis_skills.models import (
    ToolDefinition,
    ToolCall,
    ToolResult,
    MCPRequest,
    MCPResponse,  # noqa: F401 - Used by external consumers
)

__version__ = "0.1.0"
__all__ = [
    "MCPServer",
    "ToolRegistry",
    "ToolDefinition",
    "ToolCall",
    "ToolResult",
    "MCPRequest",
    "MCPResponse",
]
