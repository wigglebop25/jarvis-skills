"""
JARVIS Skills - MCP Server CLI

Usage:
    uv run main.py                     # Interactive mode
    uv run main.py --list              # List available tools
    uv run main.py --tool TOOL [ARGS]  # Execute a specific tool
"""

import sys
import json
import argparse
from jarvis_skills import MCPServer
from jarvis_skills.tools import register_all_tools


def create_server() -> MCPServer:
    """Create and configure the MCP server."""
    server = MCPServer(name="jarvis-skills")
    register_all_tools(server)
    return server


def list_tools(server: MCPServer) -> None:
    """List all available tools."""
    tools = server.list_tools()
    
    print("=" * 50)
    print("JARVIS Skills - Available Tools")
    print("=" * 50)
    
    for tool in tools:
        func = tool["function"]
        print(f"\n{func['name']}")
        print(f"  {func['description']}")
        
        params = func.get("parameters", {}).get("properties", {})
        required = func.get("parameters", {}).get("required", [])
        
        if params:
            print("  Parameters:")
            for name, info in params.items():
                req = "*" if name in required else ""
                ptype = info.get("type", "any")
                desc = info.get("description", "")
                enum = info.get("enum", [])
                
                if enum:
                    print(f"    {name}{req} ({ptype}): {desc}")
                    print(f"      Options: {', '.join(enum)}")
                else:
                    print(f"    {name}{req} ({ptype}): {desc}")


def execute_tool(server: MCPServer, tool_name: str, args: dict) -> None:
    """Execute a tool and print the result."""
    result = server.execute_tool_sync(tool_name, **args)
    
    if result.success:
        print(f"\nResult: {json.dumps(result.result, indent=2)}")
    else:
        print(f"\nError: {result.error}")


def interactive_mode(server: MCPServer) -> None:
    """Run in interactive mode."""
    print("=" * 50)
    print("JARVIS Skills - Interactive Mode")
    print("=" * 50)
    print("Commands: list, quit, or <tool_name> <json_args>")
    print("Example: get_system_info {}")
    print("Example: control_volume {\"action\": \"get\"}")
    print()
    
    while True:
        try:
            line = input("jarvis-skills> ").strip()
            
            if not line:
                continue
            
            if line.lower() == "quit":
                break
            
            if line.lower() == "list":
                list_tools(server)
                continue
            
            parts = line.split(" ", 1)
            tool_name = parts[0]
            
            if len(parts) > 1:
                try:
                    args = json.loads(parts[1])
                except json.JSONDecodeError:
                    print("Error: Invalid JSON arguments")
                    continue
            else:
                args = {}
            
            execute_tool(server, tool_name, args)
            
        except KeyboardInterrupt:
            print("\nGoodbye!")
            break
        except EOFError:
            break


def main():
    parser = argparse.ArgumentParser(description="JARVIS Skills MCP Server")
    parser.add_argument("--list", "-l", action="store_true", help="List available tools")
    parser.add_argument("--tool", "-t", type=str, help="Tool to execute")
    parser.add_argument("--args", "-a", type=str, default="{}", help="JSON arguments")
    
    args = parser.parse_args()
    
    server = create_server()
    
    if args.list:
        list_tools(server)
    elif args.tool:
        try:
            tool_args = json.loads(args.args)
        except json.JSONDecodeError:
            print("Error: Invalid JSON arguments")
            sys.exit(1)
        execute_tool(server, args.tool, tool_args)
    else:
        interactive_mode(server)


if __name__ == "__main__":
    main()
