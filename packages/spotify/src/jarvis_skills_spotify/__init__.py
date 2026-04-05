"""
JARVIS Skills - Spotify

Spotify music control tools:
- Playback control (play, pause, next, previous)
- Current track info
- Search functionality
"""

from .tools import (
    control_spotify,
    register_spotify_tool,
)

__version__ = "0.1.0"

__all__ = [
    "control_spotify",
    "register_spotify_tool",
]
