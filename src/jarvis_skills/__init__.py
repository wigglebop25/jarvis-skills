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
    # result.result contains the tool output
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

# Import built-in tools and registration helpers
from .tools import (
    register_all_tools as register_builtin_tools,
    register_system_info_tool,
    register_volume_tool,
    register_spotify_tool as register_spotify_tool_local,
    register_network_tool,
    register_folder_organizer_tool,
)
from .tools.system_info import get_system_info
from .tools.volume import control_volume
from .tools.spotify import control_spotify
from .tools.network import toggle_network
from .tools.folder_organizer import organize_folder



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
    
    # Register all tools from the current built-in tools path
    register_builtin_tools(server)
    
    return server


def register_all_tools(server: MCPServer) -> None:
    """
    Register all JARVIS tools with an existing server.
    
    Args:
        server: MCPServer instance to register tools with
    """
    register_builtin_tools(server)


def register_hardware_tools(server: MCPServer) -> None:
    """
    Backward-compatible helper for hardware-related tools only.
    """
    register_system_info_tool(server)
    register_volume_tool(server)
    register_network_tool(server)


def register_spotify_tool(server: MCPServer) -> None:
    """
    Backward-compatible helper for Spotify tool registration.
    """
    register_spotify_tool_local(server)


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
    "organize_folder",
    
    # Registration functions
    "register_builtin_tools",
    "register_hardware_tools",
    "register_spotify_tool",
    "register_folder_organizer_tool",
]
