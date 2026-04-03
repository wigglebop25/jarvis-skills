"""
Spotify Control Tool - Music playback control via Spotify.
"""

from typing import Optional
from jarvis_skills.models import ToolParameter, ToolParameterType


def control_spotify(
    action: str,
    uri: Optional[str] = None,
    query: Optional[str] = None,
) -> dict:
    """
    Control Spotify playback.
    
    Args:
        action: One of "play", "pause", "next", "previous", "current", "search"
        uri: Spotify URI for specific track/playlist
        query: Search query for "search" action
    
    Returns:
        Dictionary with playback state or search results.
    """
    try:
        import spotipy
        from spotipy.oauth2 import SpotifyOAuth
        
        sp = spotipy.Spotify(auth_manager=SpotifyOAuth(
            scope="user-read-playback-state,user-modify-playback-state,user-read-currently-playing"
        ))
        
        if action == "play":
            if uri:
                sp.start_playback(uris=[uri])
            else:
                sp.start_playback()
            return {"action": "play", "success": True}
        
        elif action == "pause":
            sp.pause_playback()
            return {"action": "pause", "success": True}
        
        elif action == "next":
            sp.next_track()
            return {"action": "next", "success": True}
        
        elif action == "previous":
            sp.previous_track()
            return {"action": "previous", "success": True}
        
        elif action == "current":
            track = sp.current_playback()
            if track and track.get("item"):
                item = track["item"]
                return {
                    "track": {
                        "name": item["name"],
                        "artist": ", ".join(a["name"] for a in item["artists"]),
                        "album": item["album"]["name"],
                        "uri": item["uri"],
                    },
                    "playing": track["is_playing"],
                    "progress_ms": track["progress_ms"],
                    "duration_ms": item["duration_ms"],
                }
            return {"track": None, "playing": False}
        
        elif action == "search" and query:
            results = sp.search(q=query, type="track", limit=5)
            tracks = []
            for item in results["tracks"]["items"]:
                tracks.append({
                    "name": item["name"],
                    "artist": ", ".join(a["name"] for a in item["artists"]),
                    "uri": item["uri"],
                })
            return {"results": tracks}
        
        return {"error": f"Unknown action: {action}"}
    
    except ImportError:
        return _spotify_fallback(action)
    except Exception as e:
        error_msg = str(e)
        if "No active device" in error_msg:
            return {"error": "No active Spotify device. Open Spotify and try again."}
        return {"error": error_msg}


def _spotify_fallback(action: str) -> dict:
    """
    Fallback for when spotipy is not installed.
    Uses basic playback control without API.
    """
    import subprocess
    import sys
    
    if sys.platform == "win32":
        if action == "play":
            subprocess.run(["nircmd", "sendkeypress", "media_play_pause"], capture_output=True)
            return {"action": "play", "success": True, "mode": "media_key"}
        elif action == "pause":
            subprocess.run(["nircmd", "sendkeypress", "media_play_pause"], capture_output=True)
            return {"action": "pause", "success": True, "mode": "media_key"}
        elif action == "next":
            subprocess.run(["nircmd", "sendkeypress", "media_next"], capture_output=True)
            return {"action": "next", "success": True, "mode": "media_key"}
        elif action == "previous":
            subprocess.run(["nircmd", "sendkeypress", "media_prev"], capture_output=True)
            return {"action": "previous", "success": True, "mode": "media_key"}
    
    return {
        "error": "spotipy not installed and no fallback available. Run: uv add spotipy",
        "action": action,
    }


def register_spotify_tool(server) -> None:
    """Register the Spotify control tool with the MCP server."""
    parameters = [
        ToolParameter(
            name="action",
            type=ToolParameterType.STRING,
            description="Playback action: play, pause, next, previous, current, search",
            required=True,
            enum=["play", "pause", "next", "previous", "current", "search"],
        ),
        ToolParameter(
            name="uri",
            type=ToolParameterType.STRING,
            description="Spotify URI for specific track/playlist",
            required=False,
        ),
        ToolParameter(
            name="query",
            type=ToolParameterType.STRING,
            description="Search query for 'search' action",
            required=False,
        ),
    ]
    
    server.register_tool(
        name="control_spotify",
        description="Control Spotify music playback (play, pause, skip, search)",
        handler=control_spotify,
        parameters=parameters,
    )
