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

### System & Hardware
- `get_system_info`: Get CPU, RAM, storage, and network status.
- `control_volume`: Control system audio volume (get, set, up, down, mute, unmute).
- `toggle_network`: Toggle network interfaces (WiFi, Bluetooth, Ethernet).
- `control_bluetooth_device`: List, connect, or disconnect Bluetooth devices.

### File System
- `resolve_path`: Resolve user-friendly path names (downloads, documents, etc.).
- `list_directory`: List directory contents with allowlist security.
- `organize_folder`: Organize files by extension, type, or date.

### Spotify (Full Integration)
- **Auth:** `authorizeSpotify`, `checkSpotifyAuth`.
- **Playback:** `playMusic`, `pausePlayback`, `resumePlayback`, `skipToNext`, `skipToPrevious`, `setVolume`, `adjustVolume`, `addToQueue`.
- **Information:** `getNowPlaying`, `getRecentlyPlayed`, `getQueue`, `getAvailableDevices`, `searchSpotify`.
- **Library & Playlists:** `getMyPlaylists`, `getPlaylistTracks`, `getPlaylist`, `getUsersSavedTracks`, `getAlbumTracks`, `getAlbums`, `checkUsersSavedAlbums`, `createPlaylist`, `addTracksToPlaylist`, `updatePlaylist`, `removeTracksFromPlaylist`, `reorderPlaylistItems`, `saveOrRemoveAlbumForUser`, `removeUsersSavedTracks`.

## Notes

- **Windows Volume Control:** Uses the native `windows` crate (Core Audio APIs) for high-performance, tool-less volume management.
- **Spotify Control:** Uses the official Spotify Web API directly. This provides granular control (playlists, queue, specific tracks) and works independently of the local Spotify desktop client's focus.
- **Linux Support:** Volume and media controls require OS tools (e.g., `pactl`, `playerctl`, `nmcli`, `rfkill`) to be installed and available in `PATH`.
- **Spotify API Compatibility:** Playback control endpoints (volume, skip, etc.) require explicit `Content-Length: 0` and `Content-Type: application/json` headers for empty-body PUT/POST requests.
- **Security:** `list_directory` root access is restricted. The allowlist can be configured via `JARVIS_SKILLS_LIST_ALLOWED_ROOTS` (semicolon-separated).
