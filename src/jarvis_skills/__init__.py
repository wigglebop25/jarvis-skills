"""
JARVIS Skills - MCP Server Aggregator

This is the main aggregator package that combines all JARVIS skill packages:
- jarvis-skills-core: MCP infrastructure
- jarvis-skills-hardware-monitor: System info, volume, network
- jarvis-skills-spotify: Spotify playback control

Usage:
    from jarvis_skills import create_server
    
    # Create server with all tools registered
    server = create_server()
    
    # Execute tools
    result = server.execute_tool_sync("get_system_info")
    print(result.result)
"""

# Re-export from core
from jarvis_skills_core import (
    MCPServer,
    ToolRegistry,
    ToolDefinition,
    ToolParameter,
    ToolParameterType,
    ToolCall,
    ToolResult,
    MCPRequest,
    MCPResponse,
)

# Import tool registration functions
from jarvis_skills_hardware_monitor import (
    register_all_tools as register_hardware_tools,
    get_system_info,
    control_volume,
    toggle_network,
)
from jarvis_skills_spotify import (
    register_spotify_tool,
    control_spotify,
)

# Keep legacy imports for backward compatibility
from .server import MCPServer as LegacyMCPServer
from .registry import ToolRegistry as LegacyToolRegistry
from .models import (
    ToolDefinition as LegacyToolDefinition,
    ToolCall as LegacyToolCall,
    ToolResult as LegacyToolResult,
    MCPRequest as LegacyMCPRequest,
    MCPResponse as LegacyMCPResponse,
)

__version__ = "0.2.0"


def create_server(name: str = "jarvis-skills") -> MCPServer:
    """
    Create an MCP server with all JARVIS tools registered.
    
    Args:
        name: Server name (default: "jarvis-skills")
        
    Returns:
        MCPServer with all tools registered
    """
    server = MCPServer(name=name)
    
    # Register all tools
    register_hardware_tools(server)
    register_spotify_tool(server)
    
    return server


def register_all_tools(server: MCPServer) -> None:
    """
    Register all JARVIS tools with an existing server.
    
    Args:
        server: MCPServer instance to register tools with
    """
    register_hardware_tools(server)
    register_spotify_tool(server)


__all__ = [
    # Factory
    "create_server",
    "register_all_tools",
    
    # Core classes
    "MCPServer",
    "ToolRegistry",
    "ToolDefinition",
    "ToolParameter",
    "ToolParameterType",
    "ToolCall",
    "ToolResult",
    "MCPRequest",
    "MCPResponse",
    
    # Tool functions
    "get_system_info",
    "control_volume",
    "toggle_network",
    "control_spotify",
    
    # Registration functions
    "register_hardware_tools",
    "register_spotify_tool",
]
