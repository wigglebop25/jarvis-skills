"""
Built-in tools initialization.
"""

from jarvis_skills.tools.system_info import register_system_info_tool
from jarvis_skills.tools.volume import register_volume_tool
from jarvis_skills.tools.spotify import register_spotify_tool
from jarvis_skills.tools.network import register_network_tool
from jarvis_skills.tools.folder_organizer import register_folder_organizer_tool


def register_all_tools(server):
    """Register all built-in tools with the server."""
    register_system_info_tool(server)
    register_volume_tool(server)
    register_spotify_tool(server)
    register_network_tool(server)
    register_folder_organizer_tool(server)


__all__ = [
    "register_all_tools",
    "register_system_info_tool",
    "register_volume_tool",
    "register_spotify_tool",
    "register_network_tool",
    "register_folder_organizer_tool",
]
