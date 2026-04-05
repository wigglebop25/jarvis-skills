"""
JARVIS Skills - Hardware Monitor

Tools for system monitoring and hardware control:
- System information (CPU, RAM, storage, network)
- Volume control (cross-platform)
- Network toggle (WiFi, Bluetooth, Ethernet)
"""

from .tools import (
    get_system_info,
    control_volume,
    toggle_network,
    register_all_tools,
    register_system_info_tool,
    register_volume_tool,
    register_network_tool,
)

__version__ = "0.1.0"

__all__ = [
    # Tool functions
    "get_system_info",
    "control_volume",
    "toggle_network",
    # Registration
    "register_all_tools",
    "register_system_info_tool",
    "register_volume_tool",
    "register_network_tool",
]
