# jarvis-skills-spotify

Spotify music playback control.

## Installation

```bash
uv pip install jarvis-skills-spotify[spotify]
```

## Usage

```python
from jarvis_skills_core import MCPServer
from jarvis_skills_spotify import register_spotify_tool

server = MCPServer()
register_spotify_tool(server)

# Control playback
server.execute_tool_sync("control_spotify", action="play")
server.execute_tool_sync("control_spotify", action="next")

# Get current track
result = server.execute_tool_sync("control_spotify", action="current")
print(result.result)  # {"track": {...}, "playing": True}
```

## Configuration

Set up Spotify API credentials in `.env`:

```env
SPOTIPY_CLIENT_ID=your_client_id
SPOTIPY_CLIENT_SECRET=your_client_secret
SPOTIPY_REDIRECT_URI=http://localhost:8888/callback
```

## License

MIT

