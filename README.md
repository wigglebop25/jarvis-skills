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

### Spotify OAuth Setup (Automated)

**New in this version:** OAuth callback server automatically handles the redirect from Spotify.

1. **Get Spotify credentials:**
   - Go to https://developer.spotify.com/dashboard
   - Create an app, copy `Client ID` and `Client Secret`

2. **Set environment variables:**
   ```bash
   $env:SPOTIPY_CLIENT_ID = "your_client_id"
   $env:SPOTIPY_CLIENT_SECRET = "your_client_secret"
   $env:SPOTIPY_REDIRECT_URI = "http://localhost:8888/callback"  # Optional, default shown
   ```

3. **Trigger authorization (one-time):**
   - Run `checkSpotifyAuth` tool
   - A login URL is returned
   - Visit the URL in your browser and click "Authorize"
   - **The callback server automatically:**
     - Captures the authorization code
     - Exchanges it for an access token
     - Stores the token in `.cache` file
     - Shows success confirmation page
   - Close the browser window—you're done!

4. **Future commands use cached token:**
    - `get_recently_played`: Get user's recently played tracks
    - `getUsersSavedTracks`: Get user's liked tracks
    - `getQueue`: Get current playback queue
    - `getPlaylist`: Get details about a playlist
    - `getAlbumTracks`: Get tracks from an album
    - `getAlbums`: Get details about multiple albums
    - `checkUsersSavedAlbums`: Check if albums are in library

    ### Playlist Management
    - `getMyPlaylists`: List current user's playlists
    - `getPlaylistTracks`: Get tracks from a specific playlist
    - `createPlaylist`: Create a new playlist
    - `addTracksToPlaylist`: Add tracks to a playlist (max 100)
    - `updatePlaylist`: Update name, description, or privacy
    - `removeTracksFromPlaylist`: Remove specific tracks
    - `reorderPlaylistItems`: Reorder tracks within a playlist

    ### Album & Track Operations
    - `saveOrRemoveAlbumForUser`: Save or remove albums from library
    - `removeUsersSavedTracks`: Remove tracks from liked songs

    ### Requirements & Limits
    - **Spotify Premium:** Required for playback control tools (`playMusic`, `pausePlayback`, `skipToNext`, `skipToPrevious`, `setVolume`).
    - **Batch Limits:**
      - `addTracksToPlaylist`: Max 100 tracks per call.
      - `saveOrRemoveAlbumForUser`: Max 50 albums per call.
      - `removeUsersSavedTracks`: Max 50 tracks per call.

    ### Examples
    - **Search & Play:** `searchSpotify(query="Interstellar", type="album")` -> `playMusic(uri="spotify:album:...")`
    - **Playlist Management:** `createPlaylist(name="My New Mix")` -> `addTracksToPlaylist(playlistId="...", uris=["spotify:track:..."])`

    ### Troubleshooting
- No delete tools are exposed.
- `list_directory` is read-only.
- `organize_folder` is non-destructive and supports dry-run planning.
