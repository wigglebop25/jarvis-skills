# JARVIS Skills

Rust MCP server for JARVIS hardware and system tools.

## Architecture

- **Runtime:** Rust (`tokio` + `axum`)
- **Transport:** HTTP JSON-RPC (`/jsonrpc`) and stdio (`--stdio`)
- **Default bind:** `127.0.0.1:5050`
- **Tool catalog:** returned by `tools/list`, consumed by `jarvis-chat`

## Setup

```bash
cd jarvis-skills\rust-mcp-server
cargo build --release
```

## Run

HTTP mode:

```bash
cargo run --release
```

stdio mode (for MCP clients):

```bash
.\target\release\jarvis-rust-mcp-server.exe --stdio
```

## Verify implementation

Health:

```bash
curl http://127.0.0.1:5050/health
```

Tool list:

```bash
curl http://127.0.0.1:5050/tools
```

JSON-RPC tools/list:

```bash
curl -X POST http://127.0.0.1:5050/jsonrpc ^
  -H "Content-Type: application/json" ^
  -d "{\"jsonrpc\":\"2.0\",\"id\":\"1\",\"method\":\"tools/list\",\"params\":{}}"
```

## MCP client configuration

This repo includes:

- `mcp.json`
- `.mcp.json`

Both are configured for stdio launch of `jarvis-rust-mcp-server.exe`.

## Troubleshooting

- **Cannot connect to port 5050:** confirm no stale process holds the port and restart server.
- **Bluetooth toggle failures:** Windows may require elevated privileges for PnP operations.
- **Spotify commands fail:** set `SPOTIPY_CLIENT_ID`, `SPOTIPY_CLIENT_SECRET`, `SPOTIPY_REDIRECT_URI`.

## Safety

- No delete tools are exposed.
- `list_directory` is read-only.
- `organize_folder` is non-destructive and supports dry-run planning.
