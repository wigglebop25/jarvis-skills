# jarvis-skills-core

Core MCP infrastructure for JARVIS skills.

## Features

- **MCPServer**: Tool registry and JSON-RPC server
- **ToolRegistry**: Dynamic tool registration
- **Pydantic Models**: Type-safe tool definitions

## Installation

```bash
uv pip install jarvis-skills-core
```

## Usage

```python
from jarvis_skills_core import MCPServer

server = MCPServer()

@server.tool("hello", "Say hello")
def hello(name: str) -> dict:
    return {"message": f"Hello, {name}!"}

result = server.execute_tool_sync("hello", name="World")
print(result.result)
```

## License

MIT

