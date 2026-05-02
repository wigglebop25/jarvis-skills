use serde_json::{json, Map, Value};

use super::super::spotify_api::SpotifyClient;

const SPOTIFY_READ_ALBUM_BATCH_LIMIT: usize = 20;

pub async fn handle_read_tool(
    name: &str,
    args: &Map<String, Value>,
    client: &SpotifyClient,
) -> Option<Result<Value, String>> {
    match name {
        "searchSpotify" => Some(search_spotify(args, client).await),
        "getNowPlaying" => Some(get_now_playing(client).await),
        "getAvailableDevices" => Some(get_available_devices(client).await),
        "getMyPlaylists" => Some(get_my_playlists(args, client).await),
        "getPlaylistTracks" => Some(get_playlist_tracks(args, client).await),
        "getRecentlyPlayed" => Some(get_recently_played(args, client).await),
        "getUsersSavedTracks" => Some(get_users_saved_tracks(args, client).await),
        "getQueue" => Some(get_queue(args, client).await),
        "getPlaylist" => Some(get_playlist(args, client).await),
        "getAlbumTracks" => Some(get_album_tracks(args, client).await),
        "getAlbums" => Some(get_albums(args, client).await),
        "checkUsersSavedAlbums" => Some(check_users_saved_albums(args, client).await),
        _ => None,
    }
}

async fn search_spotify(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: query".to_string())?;

    let tracks = client
        .search_tracks(query, 5)
        .await
        .map_err(|error| format!("Spotify search failed: {}", error))?;

    let results: Vec<Value> = tracks
        .iter()
        .map(|track| {
            json!({
                "name": track.name,
                "artist": track.artist,
                "uri": track.uri,
            })
        })
        .collect();

    Ok(json!({"results": results}))
}

async fn get_now_playing(client: &SpotifyClient) -> Result<Value, String> {
    match client.get_current_track().await {
        Ok(track) => Ok(json!({
            "action": "current",
            "track": {
                "name": track.name,
                "artist": track.artist,
                "album": track.album,
                "uri": track.uri,
                "progress_ms": track.progress_ms,
                "duration_ms": track.duration_ms,
            },
            "playing": true,
        })),
        Err(error) => Ok(json!({
            "error": format!("Unable to fetch current track. Detail: {}", error),
            "action": "current"
        })),
    }
}

async fn get_available_devices(client: &SpotifyClient) -> Result<Value, String> {
    let devices = client.get_devices().await?;
    let device_list: Vec<Value> = devices
        .iter()
        .map(|device| {
            json!({
                "id": device.id,
                "name": device.name,
                "is_active": device.is_active,
                "device_type": device.device_type
            })
        })
        .collect();
    Ok(json!({"devices": device_list}))
}

async fn get_my_playlists(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(20) as u32;
    let offset = args.get("offset").and_then(Value::as_i64).unwrap_or(0) as u32;
    let playlists = client.get_playlists(limit, offset).await?;

    let playlist_list: Vec<Value> = playlists
        .iter()
        .map(|playlist| {
            json!({
                "id": playlist.id,
                "name": playlist.name,
                "uri": playlist.uri,
                "description": playlist.description,
                "image": playlist.images.as_ref().and_then(|imgs| imgs.first()).map(|img| img.url.clone()).unwrap_or_default()
            })
        })
        .collect();

    Ok(json!({"playlists": playlist_list}))
}

async fn get_playlist_tracks(
    args: &Map<String, Value>,
    client: &SpotifyClient,
) -> Result<Value, String> {
    let playlist_id = args
        .get("playlistId")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: playlistId".to_string())?;
    let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(20) as u32;
    let offset = args.get("offset").and_then(Value::as_i64).unwrap_or(0) as u32;

    let tracks = client.get_playlist_tracks(playlist_id, limit, offset).await?;
    let track_list: Vec<Value> = tracks
        .iter()
        .map(|track| {
            json!({
                "id": track.id,
                "name": track.name,
                "artist": track.artist,
                "album": track.album,
                "duration": track.duration,
                "added_at": track.added_at,
                "uri": track.uri
            })
        })
        .collect();

    Ok(json!({"tracks": track_list}))
}

async fn get_recently_played(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(10) as u32;
    let tracks = client.get_recently_played(limit).await?;

    let track_list: Vec<Value> = tracks
        .iter()
        .map(|track| {
            json!({
                "id": track.id,
                "name": track.name,
                "artist": track.artist,
                "album": track.album,
                "duration": track.duration,
                "saved_at": track.saved_at,
                "uri": track.uri
            })
        })
        .collect();

    Ok(json!({"tracks": track_list}))
}

async fn get_users_saved_tracks(
    args: &Map<String, Value>,
    client: &SpotifyClient,
) -> Result<Value, String> {
    let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(50) as u32;
    let offset = args.get("offset").and_then(Value::as_i64).unwrap_or(0) as u32;
    let tracks = client.get_user_saved_tracks(limit, offset).await?;

    let track_list: Vec<Value> = tracks
        .iter()
        .map(|track| {
            json!({
                "id": track.id,
                "name": track.name,
                "artist": track.artist,
                "album": track.album,
                "duration": track.duration,
                "saved_at": track.saved_at,
                "uri": track.uri
            })
        })
        .collect();

    Ok(json!({"tracks": track_list}))
}

async fn get_queue(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(10) as u32;
    let queue = client.get_queue(limit).await?;

    let currently_playing = queue.currently_playing.as_ref().map(|track| {
        json!({
            "name": track.name,
            "artist": track.artist,
            "album": track.album,
            "uri": track.uri,
            "duration_ms": track.duration_ms
        })
    });
    let queue_list: Vec<Value> = queue
        .queue
        .iter()
        .map(|track| {
            json!({
                "name": track.name,
                "artist": track.artist,
                "album": track.album,
                "uri": track.uri,
                "duration_ms": track.duration_ms
            })
        })
        .collect();

    Ok(json!({
        "currently_playing": currently_playing,
        "queue": queue_list
    }))
}

async fn get_playlist(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let playlist_id = args
        .get("playlistId")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: playlistId".to_string())?;
    let playlist = client.get_playlist(playlist_id).await?;
    Ok(json!({
        "id": playlist.id,
        "name": playlist.name,
        "uri": playlist.uri,
        "description": playlist.description,
        "image": playlist.images.as_ref().and_then(|imgs| imgs.first()).map(|img| img.url.clone()).unwrap_or_default()
    }))
}

async fn get_album_tracks(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let album_id = args
        .get("albumId")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: albumId".to_string())?;
    let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(20) as u32;
    let offset = args.get("offset").and_then(Value::as_i64).unwrap_or(0) as u32;

    let tracks = client.get_album_tracks(album_id, limit, offset).await?;
    let track_list: Vec<Value> = tracks
        .iter()
        .map(|track| {
            json!({
                "id": track.id,
                "name": track.name,
                "artist": track.artist,
                "album": track.album,
                "duration": track.duration,
                "uri": track.uri
            })
        })
        .collect();

    Ok(json!({"tracks": track_list}))
}

async fn get_albums(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let album_ids = parse_album_ids(args.get("albumIds"))?;
    let result = client.get_albums(album_ids).await?;
    Ok(json!({"albums": result}))
}

async fn check_users_saved_albums(
    args: &Map<String, Value>,
    client: &SpotifyClient,
) -> Result<Value, String> {
    let album_ids = parse_album_ids(args.get("albumIds"))?;
    let saved = client.check_user_saved_albums(album_ids).await?;
    Ok(json!({"saved": saved}))
}

fn parse_album_ids(album_ids_value: Option<&Value>) -> Result<Vec<String>, String> {
    let value = album_ids_value.ok_or_else(|| "Missing required field: albumIds".to_string())?;

    let album_ids: Vec<String> = if let Some(array) = value.as_array() {
        array
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .map(|id| id.to_string())
                    .ok_or_else(|| "albumIds entries must be non-empty strings".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?
    } else if let Some(id) = value.as_str() {
        let id = id.trim();
        if id.is_empty() {
            return Err("albumIds cannot be empty".to_string());
        }
        vec![id.to_string()]
    } else {
        return Err("albumIds must be an array or string".to_string());
    };

    if album_ids.is_empty() {
        return Err("albumIds cannot be empty".to_string());
    }
    if album_ids.len() > SPOTIFY_READ_ALBUM_BATCH_LIMIT {
        return Err(format!(
            "Spotify limit exceeded for albumIds: max {SPOTIFY_READ_ALBUM_BATCH_LIMIT}, got {}",
            album_ids.len()
        ));
    }

    Ok(album_ids)
}
