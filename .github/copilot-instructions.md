# JARVIS Skills - Copilot Instructions

## Quick Reference

**Project**: Rust MCP server for hardware/system control and Spotify integration.
**Language**: Rust (high-performance backend)
**Platform**: Windows and Linux (macOS excluded)

## Copilot Tool Execution Policy (REQUIRED)

- Use **direct JARVIS MCP tools** for all system, hardware, and Spotify actions.
- **NEVER** use shell wrappers (like `uv run`) or standalone scripts.
- Only use shell commands for build (`cargo build --release`), setup, or system diagnostics.

## Available Skills (36 Tools)

### 1. Spotify Control (Premium Required for Playback)
- **Playback**: `playMusic`, `pausePlayback`, `resumePlayback`, `skipToNext`, `skipToPrevious`, `setVolume`, `adjustVolume`, `addToQueue`.
- **Status**: `getNowPlaying`, `getQueue`, `getAvailableDevices`.
- **Library**: `searchSpotify`, `getRecentlyPlayed`, `getUsersSavedTracks`, `getAlbums`, `getAlbumTracks`, `checkUsersSavedAlbums`, `saveOrRemoveAlbumForUser`, `removeUsersSavedTracks`.
- **Playlists**: `createPlaylist`, `addTracksToPlaylist`, `updatePlaylist`, `removeTracksFromPlaylist`, `reorderPlaylistItems`, `getMyPlaylists`, `getPlaylist`, `getPlaylistTracks`.
- **Auth**: `authorizeSpotify`, `checkSpotifyAuth`.

### 2. System & Hardware
- **System Info**: `get_system_info` (CPU, RAM, storage, network stats).
- **Audio**: `control_volume` (get, set, up, down, mute, unmute).
- **Network**: `toggle_network` (wifi, bluetooth, ethernet).
- **Bluetooth**: `control_bluetooth_device` (list, connect, disconnect).

### 3. File & Directory Management
- **Navigation**: `list_directory` (read-only, restricted to allowed roots).
- **Organization**: `organize_folder` (non-destructive, always try `dry_run=true` first).
- **Portability**: `resolve_path` (dynamically resolves `downloads`, `documents`, `desktop`, `home`, `project`).

## Key Implementation Patterns

### Intent Routing
The server includes a built-in intent router (`jarvis/route`). If a user's request is ambiguous, use this endpoint to determine the best tool and arguments.

### Zero-Delay Architecture
The Rust backend is optimized for sub-100ms response times:
- **In-Memory Caching**: Spotify tokens are cached in RAM.
- **Connection Pooling**: Keeps a persistent connection to the Spotify API.
- **Relative Pathing**: Configurations use `./rust-mcp-server` paths for instant, portable execution.

### Safety & Constraints
- No "Delete" or "Format" tools are implemented.
- `organize_folder` only moves/renames files.
- `list_directory` is restricted by the `JARVIS_SKILLS_ALLOWED_ROOTS` env var.

## Troubleshooting
- **Not Connected**: Ensure the Rust binary `jarvis-rust-mcp-server.exe` exists in the release folder.
- **Spotify 401**: Run `authorizeSpotify` to refresh the session.
- **Bluetooth Fail**: On Windows, ensure PowerShell is available and the user has permission to toggle PnP devices.
