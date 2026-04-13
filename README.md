# JARVIS Skills

Modular MCP server for system control and automation.

## Packages

- **jarvis-skills-core**: MCP infrastructure
- **jarvis-skills-hardware-monitor**: System info, volume, network
- **jarvis-skills-spotify**: Spotify playback control
- **Built-in folder organizer**: Safe file organization in allowlisted paths

## Installation

```bash
cd jarvis-skills
uv sync
```

Or install individual packages:

```bash
uv pip install jarvis-skills-core
uv pip install jarvis-skills-hardware-monitor
uv pip install jarvis-skills-spotify
```

## Quick Start

```python
from jarvis_skills import create_server

# Create server with all tools
server = create_server()

# Execute tools
result = server.execute_tool_sync("get_system_info")
print(result.result)

result = server.execute_tool_sync("control_volume", action="get")
print(result.result)

result = server.execute_tool_sync(
    "organize_folder",
    path="C:/Users/YourUser/Downloads",
    strategy="extension",
    dry_run=True,
)
print(result.result)
```

## HTTP Server

```bash
uv run server.py
# Server runs on http://127.0.0.1:5050
```

`server.py` uses **Granian** as the ASGI runtime.

## Folder organizer safety scope

`organize_folder` is allowlist-scoped and non-destructive by default.

- Default writable roots: `Desktop`, `Downloads`, `Documents`
- Override roots with environment variable:

```env
JARVIS_SKILLS_ALLOWED_ROOTS=C:\Users\you\Desktop;C:\Users\you\Downloads
```

- `dry_run=true` previews moves without changing files

## License

MIT
