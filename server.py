"""
HTTP Server for JARVIS Skills MCP Server.

Provides a FastAPI-based HTTP server that exposes MCP tools via JSON-RPC.
This avoids STDOUT noise issues that come with stdio-based transports.

Usage:
    uv run server.py                    # Start server on port 5050
    uv run server.py --port 8080        # Custom port
"""

import argparse
import logging
import sys
from pathlib import Path
from typing import Any, Optional
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

# Add src to path for module resolution
sys.path.insert(0, str(Path(__file__).parent / "src"))

from jarvis_skills import MCPServer
from jarvis_skills.models import MCPRequest, MCPResponse
from jarvis_skills.tools import register_all_tools

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
    arguments: dict[str, Any] = {}


class JSONRPCRequest(BaseModel):
    jsonrpc: str = "2.0"
    id: str
    method: str
    params: dict[str, Any] = {}


class JSONRPCResponse(BaseModel):
    jsonrpc: str = "2.0"
    id: str
    result: Any = None
    error: dict | None = None


@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "healthy", "server": "jarvis-skills"}


@app.get("/tools")
async def list_tools():
    """List all available tools."""
    return {"tools": mcp_server.list_tools()}


@app.post("/tools/{tool_name}")
async def execute_tool(tool_name: str, request: dict[str, Any] = {}):
    """Execute a tool by name with arguments."""
    result = await mcp_server.execute_tool(tool_name, **request)
    
    if result.success:
        return result.result
    else:
        raise HTTPException(status_code=400, detail=result.error)


@app.post("/jsonrpc")
async def jsonrpc_endpoint(request: JSONRPCRequest) -> JSONRPCResponse:
    """
    JSON-RPC 2.0 endpoint for MCP protocol.
    
    Methods:
        - tools/list: List available tools
        - tools/call: Execute a tool
    """
    mcp_request = MCPRequest(
        id=request.id,
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
    import uvicorn
    
    parser = argparse.ArgumentParser(description="JARVIS Skills MCP Server")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    parser.add_argument("--port", type=int, default=5050, help="Port to bind to")
    parser.add_argument("--reload", action="store_true", help="Enable auto-reload")
    
    args = parser.parse_args()
    
    logger.info(f"Starting JARVIS Skills MCP Server on {args.host}:{args.port}")
    
    uvicorn.run(
        "server:app",
        host=args.host,
        port=args.port,
        reload=args.reload,
    )


if __name__ == "__main__":
    main()
