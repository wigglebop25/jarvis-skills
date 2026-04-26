# JARVIS Rust MCP Server

Rust implementation of the JARVIS MCP server (transport + tools).

## Stack

- Tokio
- Axum + Hyper
- Serde JSON

## Endpoints

- `GET /health`
- `GET /tools`
- `POST /jsonrpc`

## Phase 1 router migration interface

For deterministic routing hot-path migration from `jarvis-chat` to Rust, add JSON-RPC methods (additive, non-breaking):

- `jarvis/route` → returns route decision only (`intent`, `confidence`, `tool_name`, `arguments`, `should_execute`)
- `jarvis/route_and_call` → same decision payload, plus optional execution through existing `tools::execute_tool`

Compatibility requirements:
- `tools/list` and `tools/call`.
- Match Python intent-routing behavior (confidence threshold, tool mapping, argument mapping) for existing intents.
- Return structured errors so chat runtime can auto-fallback to Python routing when needed.

## Run

```bash
cd jarvis-skills\rust-mcp-server
cargo run
```

Run in stdio MCP mode (for Copilot CLI / Gemini CLI):

```bash
cargo run -- --stdio
```

Default bind:

- `RUST_MCP_HOST=127.0.0.1`
- `RUST_MCP_PORT=5050`

## Implemented tools

- `get_system_info`
- `control_volume`
- `control_spotify`
- `toggle_network`
- `list_directory`
- `organize_folder`

## Notes

- Windows media controls use `nircmd`; if missing, the server attempts automatic download/install into a local cache.
- Linux volume and media controls require OS tools (for example: `pactl`, `playerctl`, `nmcli`, `rfkill`) to be installed and in `PATH`.
- `list_directory` allowlist can be overridden with `JARVIS_SKILLS_LIST_ALLOWED_ROOTS` (semicolon-separated paths).
