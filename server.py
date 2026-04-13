"""
HTTP Server for JARVIS Skills MCP Server.

Provides a FastAPI-based HTTP server that exposes MCP tools via JSON-RPC.
This avoids STDOUT noise issues that come with stdio-based transports.

Usage:
    uv run server.py                    # Start Granian server on port 5050
    uv run server.py --port 8080        # Custom port
"""

import argparse
import logging
import shutil
import socket
import subprocess
import sys
from pathlib import Path
from typing import Any, Optional
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Add src to path for development/workspace mode (sys.path modifications are runtime-only)
sys.path.insert(0, str(Path(__file__).parent / "src"))
sys.path.insert(0, str(Path(__file__).parent / "packages" / "core" / "src"))
sys.path.insert(0, str(Path(__file__).parent / "packages" / "hardware-monitor" / "src"))
sys.path.insert(0, str(Path(__file__).parent / "packages" / "spotify" / "src"))

try:
    from jarvis_skills_core import MCPServer, MCPRequest  # type: ignore[import-not-found]
    from jarvis_skills import register_all_tools  # type: ignore[import-not-found]
except ImportError as e:
    raise RuntimeError(
        "Failed to import JARVIS skills modules. "
        "Install dependencies: uv sync"
    ) from e

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s"
)
logger = logging.getLogger("jarvis-skills-server")

mcp_server: Optional[MCPServer] = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global mcp_server
    mcp_server = MCPServer(name="jarvis-skills")
    register_all_tools(mcp_server)
    if mcp_server:
        logger.info(f"Registered {len(mcp_server.list_tools())} tools")
    yield
    logger.info("Shutting down MCP server")


app = FastAPI(
    title="JARVIS Skills MCP Server",
    description="MCP Server for JARVIS AI Assistant tools",
    version="0.1.0",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class ToolCallRequest(BaseModel):
    name: str
    arguments: dict[str, Any] = {}  # Pydantic handles mutable defaults safely


class JSONRPCRequest(BaseModel):
    jsonrpc: str = "2.0"
    id: str | int
    method: str
    params: dict[str, Any] = {}  # Pydantic handles mutable defaults safely


class JSONRPCResponse(BaseModel):
    jsonrpc: str = "2.0"
    id: str | int
    result: Any = None
    error: dict | None = None


@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "healthy", "server": "jarvis-skills"}


@app.get("/tools")
async def list_tools():
    """List all available tools."""
    if mcp_server:
        return {"tools": mcp_server.list_tools()}
    return {"tools": []}


@app.post("/tools/call")
async def execute_tool_by_name(request: dict[str, Any]) -> dict[str, Any]:
    """Execute a tool by name from request body."""
    if mcp_server:
        tool_name = request.get("name")
        arguments = request.get("arguments", {})
        
        if not tool_name:
            raise HTTPException(status_code=400, detail="Missing tool name")
        
        result = await mcp_server.execute_tool(tool_name, **arguments)
        
        if result.success:
            return {"result": result.result}
        else:
            raise HTTPException(status_code=400, detail=result.error)
    raise HTTPException(status_code=503, detail="MCP server not ready")


@app.post("/tools/{tool_name}")
async def execute_tool(tool_name: str, request: dict[str, Any] | None = None) -> dict[str, Any]:
    """Execute a tool by name with arguments."""
    if request is None:
        request = {}
    if mcp_server:
        # Extract arguments from request, not the whole request dict
        arguments = request.get("arguments", {})
        result = await mcp_server.execute_tool(tool_name, **arguments)
        
        if result.success:
            return {"result": result.result}
        else:
            raise HTTPException(status_code=400, detail=result.error)
    raise HTTPException(status_code=503, detail="MCP server not ready")


@app.post("/jsonrpc")
async def jsonrpc_endpoint(request: JSONRPCRequest) -> JSONRPCResponse:
    """
    JSON-RPC 2.0 endpoint for MCP protocol.
    
    Methods:
        - tools/list: List available tools
        - tools/call: Execute a tool
    """
    if not mcp_server:
        return JSONRPCResponse(
            id=request.id,
            error={"code": -32603, "message": "Internal error: MCP server not initialized"},
        )
    
    mcp_request = MCPRequest(
        id=str(request.id),
        method=request.method,
        params=request.params,
    )
    
    response = await mcp_server.handle_request(mcp_request)
    
    return JSONRPCResponse(
        id=response.id,
        result=response.result,
        error=response.error,
    )


def main():
    parser = argparse.ArgumentParser(description="JARVIS Skills MCP Server")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    parser.add_argument("--port", type=int, default=5050, help="Port to bind to")
    parser.add_argument(
        "--reload",
        action="store_true",
        help="Enable auto-reload (requires granian[reload])",
    )
    
    args = parser.parse_args()

    # Detect occupied host/port before launching the runtime to provide clear guidance.
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        try:
            sock.bind((args.host, args.port))
        except OSError:
            logger.error(
                "Port %s is already in use on %s. Cannot start MCP server.",
                args.port,
                args.host,
            )
            print(
                (
                    f"Port {args.port} on {args.host} is already in use.\n"
                    "Check listener PID:\n"
                    f"  Get-NetTCPConnection -LocalPort {args.port} -State Listen\n"
                    "Stop exact process by PID:\n"
                    "  Stop-Process -Id <PID>\n"
                ),
                file=sys.stderr,
            )
            sys.exit(1)

    logger.info(
        "Starting JARVIS Skills MCP Server with Granian on %s:%s",
        args.host,
        args.port,
    )

    uv_executable = shutil.which("uv")
    if uv_executable:
        command = [uv_executable, "run", "granian"]
    else:
        command = [sys.executable, "-m", "granian"]

    command.extend(
        [
            "--interface",
            "asgi",
            "--host",
            args.host,
            "--port",
            str(args.port),
        ]
    )
    if args.reload:
        command.append("--reload")
    command.append("server:app")

    result = subprocess.run(command, check=False)
    if result.returncode != 0:
        sys.exit(result.returncode)


if __name__ == "__main__":
    main()
