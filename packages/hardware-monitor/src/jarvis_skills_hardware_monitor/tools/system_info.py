"""
System Info Tool - Provides system resource information.
"""

import psutil
from typing import Optional
from jarvis_skills_core import ToolParameter, ToolParameterType


def get_system_info(include: Optional[list[str]] = None) -> dict:
    """
    Get system information including CPU, RAM, storage, and network.
    
    Args:
        include: Optional list of sections to include. 
                 Options: "cpu", "ram", "storage", "network"
                 If None, includes all sections.
    
    Returns:
        Dictionary with system information.
    """
    sections = include or ["cpu", "ram", "storage", "network"]
    result = {}
    
    if "cpu" in sections:
        result["cpu"] = psutil.cpu_percent(interval=0.1)
    
    if "ram" in sections:
        mem = psutil.virtual_memory()
        result["ram"] = {
            "total_gb": round(mem.total / (1024**3), 2),
            "used_gb": round(mem.used / (1024**3), 2),
            "available_gb": round(mem.available / (1024**3), 2),
            "percent": mem.percent,
        }
    
    if "storage" in sections:
        partitions = []
        for part in psutil.disk_partitions():
            try:
                usage = psutil.disk_usage(part.mountpoint)
                partitions.append({
                    "mount": part.mountpoint,
                    "total_gb": round(usage.total / (1024**3), 2),
                    "used_gb": round(usage.used / (1024**3), 2),
                    "free_gb": round(usage.free / (1024**3), 2),
                    "percent": usage.percent,
                })
            except PermissionError:
                continue
        result["storage"] = partitions
    
    if "network" in sections:
        net_if = psutil.net_if_stats()
        active_interface = None
        for name, stats in net_if.items():
            if stats.isup and name not in ("lo", "Loopback Pseudo-Interface 1"):
                active_interface = name
                break
        
        result["network"] = {
            "connected": active_interface is not None,
            "interface": active_interface or "None",
        }
    
    return result


def register_system_info_tool(server) -> None:
    """Register the system info tool with the MCP server."""
    parameters = [
        ToolParameter(
            name="include",
            type=ToolParameterType.ARRAY,
            description="Sections to include: cpu, ram, storage, network. If not specified, returns all.",
            required=False,
        ),
    ]
    
    server.register_tool(
        name="get_system_info",
        description="Get system information including CPU usage, RAM, storage, and network status",
        handler=get_system_info,
        parameters=parameters,
    )
