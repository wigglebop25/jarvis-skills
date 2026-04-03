"""Tests for the built-in tools."""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

import pytest
from jarvis_skills import MCPServer
from jarvis_skills.tools import register_all_tools
from jarvis_skills.tools.system_info import get_system_info


def test_system_info_all():
    """Test getting all system info."""
    result = get_system_info()
    
    assert "cpu" in result
    assert "ram" in result
    assert "storage" in result
    assert "network" in result
    
    assert isinstance(result["cpu"], float)
    assert "total_gb" in result["ram"]
    assert "percent" in result["ram"]


def test_system_info_partial():
    """Test getting partial system info."""
    result = get_system_info(include=["cpu", "ram"])
    
    assert "cpu" in result
    assert "ram" in result
    assert "storage" not in result
    assert "network" not in result


def test_register_all_tools():
    """Test all tools are registered."""
    server = MCPServer()
    register_all_tools(server)
    
    tools = server.list_tools()
    tool_names = [t["function"]["name"] for t in tools]
    
    assert "get_system_info" in tool_names
    assert "control_volume" in tool_names
    assert "control_spotify" in tool_names
    assert "toggle_network" in tool_names


def test_system_info_via_server():
    """Test system info through MCP server."""
    server = MCPServer()
    register_all_tools(server)
    
    result = server.execute_tool_sync("get_system_info", include=["cpu"])
    
    assert result.success is True
    assert "cpu" in result.result
    assert "ram" not in result.result
