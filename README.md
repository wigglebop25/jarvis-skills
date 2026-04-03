# JARVIS Skills / MCP Server

HTTP/JSON-RPC server exposing hardware control tools via Model Context Protocol (MCP). Manages system info, volume control, Spotify integration, and network management.

## Features

- JSON-RPC 2.0 protocol over HTTP
- System information tools (CPU, RAM, disk, network)
- Cross-platform volume control
- Spotify playback control
- Network device management (WiFi, Bluetooth)
- Tool registry and automatic discovery
- Error handling and request validation

## Installation

```bash
cd jarvis-skills
uv sync
```

## Quick Start

### Start the Server

```bash
python server.py
```

Server runs on http://127.0.0.1:5050

### Test a Tool Call

```bash
curl -X POST http://127.0.0.1:5050/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "get_system_info",
    "params": {}
  }'
```

## Available Tools

| Tool | Purpose | Params |
|------|---------|--------|
| `get_system_info` | CPU, RAM, storage, network info | None |
| `control_volume` | Adjust volume | `action`: up/down/mute |
| `control_spotify` | Playback control | `action`: play/pause/next/previous |
| `toggle_network` | WiFi/Bluetooth on/off | `device`, `enable` |

## Protocol

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tool_name",
  "params": { "key": "value" }
}
```

### Response Format

Success:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { "status": "success", "data": {...} }
}
```

Error:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Invalid request"
  }
}
```

## Configuration

Create `.env` file:

```env
MCP_PORT=5050
LOG_LEVEL=INFO
SPOTIFY_CLIENT_ID=your-id          # Optional
SPOTIFY_CLIENT_SECRET=your-secret  # Optional
```

## Testing

```bash
uv run pytest tests/ -v
```

Tests: 12/12 passing

## Troubleshooting

**Port already in use:**
```bash
python server.py --port 5051
```

**Tool execution fails:**
Enable debug logging:
```bash
export LOG_LEVEL=DEBUG
python server.py
```

**Spotify not working:**
- Tool works without credentials
- Add SPOTIFY_CLIENT_ID and SPOTIFY_CLIENT_SECRET to enable full features

## Requirements

- Python 3.13+
- uv package manager

## Development

```bash
uv add <package-name>        # Add dependency
uv run pytest                # Run tests
```

## License

MIT