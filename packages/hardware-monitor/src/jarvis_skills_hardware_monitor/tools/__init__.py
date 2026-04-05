"""
Hardware Monitor Tools

Exports all hardware monitoring and control tools.
"""

from .system_info import get_system_info, register_system_info_tool
from .volume import control_volume, register_volume_tool
from .network import toggle_network, register_network_tool


def register_all_tools(server) -> None:
    """Register all hardware monitor tools with an MCP server."""
    register_system_info_tool(server)
    register_volume_tool(server)
    register_network_tool(server)


__all__ = [
    "get_system_info",
    "control_volume",
    "toggle_network",
    "register_all_tools",
    "register_system_info_tool",
    "register_volume_tool",
    "register_network_tool",
]
