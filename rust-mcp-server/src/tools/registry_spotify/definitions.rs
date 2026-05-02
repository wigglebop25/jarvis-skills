use serde_json::{json, Value};

pub fn read_tool_definitions() -> Vec<Value> {
    vec![
        spotify_tool(
            "searchSpotify",
            "Search for tracks, albums, artists, or playlists on Spotify",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "type": {"type": "string"},
                    "limit": {"type": "integer"}
                },
                "required": ["query", "type"]
            }),
        ),
        spotify_tool(
            "getNowPlaying",
            "Get information about the currently playing track on Spotify, including device and volume info",
            json!({
                "type": "object",
                "properties": {},
            }),
        ),
        spotify_tool(
            "getRecentlyPlayed",
            "Get the user's recently played tracks on Spotify",
            json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Number of tracks to return (default: 10, max: 50)"}
                }
            }),
        ),
        spotify_tool(
            "getQueue",
            "Get the current playback queue on Spotify",
            json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Number of upcoming tracks to return (default: 10, max: 100)"}
                }
            }),
        ),
        spotify_tool(
            "getAvailableDevices",
            "Get information about the user's available Spotify Connect devices",
            json!({
                "type": "object",
                "properties": {},
            }),
        ),
    ]
}

pub fn library_tool_definitions() -> Vec<Value> {
    vec![
        spotify_tool(
            "getMyPlaylists",
            "Get a list of the current user's playlists on Spotify",
            json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Number of playlists to fetch (default: 20, max: 50)"},
                    "offset": {"type": "integer", "description": "The index of the first playlist to return (default: 0)"}
                }
            }),
        ),
        spotify_tool(
            "getPlaylistTracks",
            "Get all tracks from a specific Spotify playlist",
            json!({
                "type": "object",
                "properties": {
                    "playlistId": {"type": "string", "description": "The Spotify playlist ID"},
                    "limit": {"type": "integer", "description": "Number of tracks to fetch per request (default: 20, max: 50)"},
                    "offset": {"type": "integer", "description": "The index of the first track to return (default: 0)"}
                },
                "required": ["playlistId"]
            }),
        ),
        spotify_tool(
            "getUsersSavedTracks",
            "Get the user's saved (liked) tracks on Spotify",
            json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Number of tracks to fetch per request (default: 50, max: 50)"},
                    "offset": {"type": "integer", "description": "The index of the first track to return (default: 0)"}
                }
            }),
        ),
        spotify_tool(
            "getPlaylist",
            "Get details about a specific Spotify playlist",
            json!({
                "type": "object",
                "properties": {
                    "playlistId": {"type": "string", "description": "The Spotify playlist ID"}
                },
                "required": ["playlistId"]
            }),
        ),
        spotify_tool(
            "getAlbumTracks",
            "Get all tracks from a specific Spotify album",
            json!({
                "type": "object",
                "properties": {
                    "albumId": {"type": "string", "description": "The Spotify album ID"},
                    "limit": {"type": "integer", "description": "Number of tracks to fetch per request (default: 20, max: 50)"},
                    "offset": {"type": "integer", "description": "The index of the first track to return (default: 0)"}
                },
                "required": ["albumId"]
            }),
        ),
        spotify_tool(
            "getAlbums",
            "Get details about multiple Spotify albums",
            json!({
                "type": "object",
                "properties": {
                    "albumIds": {
                        "oneOf": [
                            {"type": "string", "description": "A single Spotify album ID"},
                            {"type": "array", "items": {"type": "string"}, "description": "Array of Spotify album IDs (max 20)"}
                        ]
                    }
                },
                "required": ["albumIds"]
            }),
        ),
        spotify_tool(
            "checkUsersSavedAlbums",
            "Check if the user has saved specific Spotify albums",
            json!({
                "type": "object",
                "properties": {
                    "albumIds": {
                        "oneOf": [
                            {"type": "string", "description": "A single Spotify album ID"},
                            {"type": "array", "items": {"type": "string"}, "description": "Array of Spotify album IDs (max 20)"}
                        ]
                    }
                },
                "required": ["albumIds"]
            }),
        ),
    ]
}

pub fn playback_tool_definitions() -> Vec<Value> {
    vec![
        spotify_tool(
            "playMusic",
            "Start playing a track, album, artist, or playlist on Spotify",
            json!({
                "type": "object",
                "properties": {
                    "uri": {"type": "string"},
                    "type": {"type": "string", "enum": ["track", "album", "artist", "playlist", "episode", "show"]},
                    "id": {"type": "string"},
                    "deviceId": {"type": "string"}
                },
                "anyOf": [
                    {"required": ["uri"]},
                    {"required": ["type", "id"]}
                ]
            }),
        ),
        spotify_tool(
            "pausePlayback",
            "Pause the currently playing track on Spotify",
            json!({
                "type": "object",
                "properties": {
                    "deviceId": {"type": "string"}
                },
            }),
        ),
        spotify_tool(
            "resumePlayback",
            "Resume the current Spotify playback",
            json!({
                "type": "object",
                "properties": {
                    "deviceId": {"type": "string"}
                },
            }),
        ),
        spotify_tool(
            "skipToNext",
            "Skip to the next track in the current playback queue",
            json!({
                "type": "object",
                "properties": {
                    "deviceId": {"type": "string"}
                },
            }),
        ),
        spotify_tool(
            "skipToPrevious",
            "Skip to the previous track in the current playback queue",
            json!({
                "type": "object",
                "properties": {
                    "deviceId": {"type": "string"}
                },
            }),
        ),
        spotify_tool(
            "setVolume",
            "Set the playback volume to a specific percentage (requires Spotify Premium)",
            json!({
                "type": "object",
                "properties": {
                    "volumePercent": {"type": "integer"},
                    "deviceId": {"type": "string"}
                },
                "required": ["volumePercent"]
            }),
        ),
        spotify_tool(
            "adjustVolume",
            "Adjust the current playback volume by a relative amount (requires Spotify Premium)",
            json!({
                "type": "object",
                "properties": {
                    "adjustment": {"type": "integer"},
                    "deviceId": {"type": "string"}
                },
                "required": ["adjustment"]
            }),
        ),
        spotify_tool(
            "addToQueue",
            "Add an item to the current playback queue by URI or type+id",
            json!({
                "type": "object",
                "properties": {
                    "uri": {"type": "string"},
                    "type": {"type": "string", "enum": ["track", "album", "artist", "playlist", "episode", "show"]},
                    "id": {"type": "string"},
                    "deviceId": {"type": "string"}
                },
                "anyOf": [
                    {"required": ["uri"]},
                    {"required": ["type", "id"]}
                ]
            }),
        ),
    ]
}

pub fn write_tool_definitions() -> Vec<Value> {
    vec![
        spotify_tool(
            "createPlaylist",
            "Create a new playlist for the current user",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "minLength": 1, "description": "The name of the new playlist"},
                    "description": {"type": "string", "description": "Optional description for the playlist"},
                    "public": {"type": "boolean", "description": "Optional: whether the playlist should be public (default: true)"},
                    "collaborative": {"type": "boolean", "description": "Optional: whether the playlist should be collaborative (default: false)"}
                },
                "required": ["name"]
            }),
        ),
        spotify_tool(
            "addTracksToPlaylist",
            "Add one or more tracks to a Spotify playlist",
            json!({
                "type": "object",
                "properties": {
                    "playlistId": {"type": "string", "description": "The Spotify playlist ID"},
                    "trackUris": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 100, "description": "Array of Spotify track URIs (or IDs/URLs) to add (max 100)"},
                    "position": {"type": "integer", "minimum": 0, "description": "Optional: zero-based position to insert tracks"}
                },
                "required": ["playlistId", "trackUris"]
            }),
        ),
        spotify_tool(
            "updatePlaylist",
            "Update a playlist's details (name, description, etc.)",
            json!({
                "type": "object",
                "properties": {
                    "playlistId": {"type": "string", "description": "The Spotify playlist ID"},
                    "name": {"type": "string", "minLength": 1, "description": "New name for the playlist"},
                    "description": {"type": "string", "description": "New description for the playlist"},
                    "public": {"type": "boolean", "description": "Whether the playlist should be public"},
                    "collaborative": {"type": "boolean", "description": "Whether the playlist should be collaborative"}
                },
                "required": ["playlistId"]
            }),
        ),
        spotify_tool(
            "removeTracksFromPlaylist",
            "Remove one or more tracks from a Spotify playlist",
            json!({
                "type": "object",
                "properties": {
                    "playlistId": {"type": "string", "description": "The Spotify playlist ID"},
                    "trackIds": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 100, "description": "Array of Spotify track IDs (or URIs/URLs) to remove (max 100)"},
                    "snapshotId": {"type": "string", "description": "Optional: The playlist's snapshot ID"}
                },
                "required": ["playlistId", "trackIds"]
            }),
        ),
        spotify_tool(
            "reorderPlaylistItems",
            "Reorder items in a Spotify playlist",
            json!({
                "type": "object",
                "properties": {
                    "playlistId": {"type": "string", "description": "The Spotify playlist ID"},
                    "rangeStart": {"type": "integer", "minimum": 0, "description": "The position of the first item to be reordered"},
                    "insertBefore": {"type": "integer", "minimum": 0, "description": "The position where the items should be inserted"},
                    "rangeLength": {"type": "integer", "minimum": 1, "description": "Optional: The number of items to be reordered (default: 1)"},
                    "snapshotId": {"type": "string", "description": "Optional: The playlist's snapshot ID"}
                },
                "required": ["playlistId", "rangeStart", "insertBefore"]
            }),
        ),
        spotify_tool(
            "saveOrRemoveAlbumForUser",
            "Save or remove one or more albums for the current user",
            json!({
                "type": "object",
                "properties": {
                    "albumIds": {
                        "oneOf": [
                            {"type": "string", "description": "A single Spotify album ID"},
                            {"type": "array", "items": {"type": "string"}, "description": "Array of Spotify album IDs (max 50)"}
                        ]
                    },
                    "action": {"type": "string", "enum": ["save", "remove"], "description": "Whether to save or remove the albums"}
                },
                "required": ["albumIds", "action"]
            }),
        ),
        spotify_tool(
            "removeUsersSavedTracks",
            "Remove one or more tracks from the user's saved (liked) tracks",
            json!({
                "type": "object",
                "properties": {
                    "trackIds": {
                        "oneOf": [
                            {"type": "string", "description": "A single Spotify track ID"},
                            {"type": "array", "items": {"type": "string"}, "description": "Array of Spotify track IDs (max 50)"}
                        ]
                    }
                },
                "required": ["trackIds"]
            }),
        ),
    ]
}

pub fn auth_tool_definitions() -> Vec<Value> {
    vec![
        spotify_tool(
            "checkSpotifyAuth",
            "Check if the user is authenticated with Spotify. Returns a login URL if not.",
            json!({
                "type": "object",
                "properties": {},
            }),
        ),
        spotify_tool(
            "authorizeSpotify",
            "Authorize with Spotify by automatically opening the browser. No user input needed!",
            json!({
                "type": "object",
                "properties": {},
            }),
        ),
    ]
}

fn spotify_tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": parameters
        }
    })
}
