"""Tests for the MCP Server."""

import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from jarvis_skills import MCPServer, ToolCall, MCPRequest


def test_server_creation():
    """Test server can be created."""
    server = MCPServer(name="test-server")
    assert server.name == "test-server"
    assert len(server.list_tools()) == 0


def test_tool_registration():
    """Test tools can be registered."""
    server = MCPServer()
    
    def my_tool(x: int) -> int:
        return x * 2
    
    server.register_tool(
        name="my_tool",
        description="Doubles a number",
        handler=my_tool,
    )
    
    tools = server.list_tools()
    assert len(tools) == 1
    assert tools[0]["function"]["name"] == "my_tool"


def test_tool_decorator():
    """Test the @server.tool decorator."""
    server = MCPServer()
    
    @server.tool("greet", "Greets a person")
    def greet(name: str) -> str:
        return f"Hello, {name}!"
    
    tools = server.list_tools()
    assert len(tools) == 1
    assert tools[0]["function"]["name"] == "greet"


def test_tool_execution_sync():
    """Test synchronous tool execution."""
    server = MCPServer()
    
    def add(a: int, b: int) -> int:
        return a + b
    
    server.register_tool("add", "Add two numbers", add)
    
    result = server.execute_tool_sync("add", a=5, b=3)
    assert result.success is True
    assert result.result == 8


def test_unknown_tool():
    """Test executing unknown tool returns error."""
    server = MCPServer()
    
    result = server.execute_tool_sync("nonexistent")
    assert result.success is False
    assert "Unknown tool" in result.error


def test_json_rpc_tools_list():
    """Test JSON-RPC tools/list method."""
    server = MCPServer()
    
    def my_tool():
        return "ok"
    
    server.register_tool("my_tool", "Test tool", my_tool)
    
    request = MCPRequest(
        id="req-1",
        method="tools/list",
        params={},
    )
    
    response = server.handle_request_sync(request)
    assert response.error is None
    assert "tools" in response.result
    assert len(response.result["tools"]) == 1


def test_json_rpc_tools_call():
    """Test JSON-RPC tools/call method."""
    server = MCPServer()
    
    def echo(message: str) -> str:
        return message
    
    server.register_tool("echo", "Echo a message", echo)
    
    request = MCPRequest(
        id="req-2",
        method="tools/call",
        params={
            "name": "echo",
            "arguments": {"message": "hello"},
        },
    )
    
    response = server.handle_request_sync(request)
    assert response.error is None
    assert response.result == "hello"


def test_json_rpc_unknown_method():
    """Test JSON-RPC with unknown method."""
    server = MCPServer()
    
    request = MCPRequest(
        id="req-3",
        method="unknown/method",
        params={},
    )
    
    response = server.handle_request_sync(request)
    assert response.error is not None
    assert response.error["code"] == -32601
