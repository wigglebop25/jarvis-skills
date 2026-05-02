use serde_json::{json, Map, Value};

use super::super::spotify_api::SpotifyClient;
use super::helpers::*;

const SPOTIFY_ALBUM_BATCH_LIMIT: usize = 50;
const SPOTIFY_TRACK_BATCH_LIMIT: usize = 50;

pub async fn handle_write_tool(
    name: &str,
    args: &Map<String, Value>,
    client: &SpotifyClient,
) -> Option<Result<Value, String>> {
    match name {
        "playMusic" => Some(play_music(args, client).await),
        "pausePlayback" => Some(pause_playback(args, client).await),
        "resumePlayback" => Some(resume_playback(args, client).await),
        "skipToNext" => Some(skip_to_next(args, client).await),
        "skipToPrevious" => Some(skip_to_previous(args, client).await),
        "setVolume" => Some(set_volume(args, client).await),
        "adjustVolume" => Some(adjust_volume(args, client).await),
        "addToQueue" => Some(add_to_queue(args, client).await),
        "createPlaylist" => Some(create_playlist(args, client).await),
        "addTracksToPlaylist" => Some(add_tracks_to_playlist(args, client).await),
        "updatePlaylist" => Some(update_playlist(args, client).await),
        "removeTracksFromPlaylist" => Some(remove_tracks_from_playlist(args, client).await),
        "reorderPlaylistItems" => Some(reorder_playlist_items(args, client).await),
        "saveOrRemoveAlbumForUser" => Some(save_or_remove_album(args, client).await),
        "removeUsersSavedTracks" => Some(remove_users_saved_tracks(args, client).await),
        _ => None,
    }
}

async fn create_playlist(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let name = parse_non_empty_string(args, "name")?;

    let public = args.get("public").and_then(Value::as_bool);
    let collaborative = args.get("collaborative").and_then(Value::as_bool);
    let description = args.get("description").and_then(|v| v.as_str().map(|s| s.to_string()));

    client
        .create_playlist(&name, public, collaborative, description)
        .await
}

async fn add_tracks_to_playlist(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let playlist_id = parse_playlist_id(args, "playlistId")?;
    let track_uris = parse_track_uris(args, "trackUris", 100)?;
    let position = parse_optional_u32(args, "position")?;

    let snapshot_id = client
        .add_tracks_to_playlist(&playlist_id, track_uris, position)
        .await?;
    Ok(json!({ "snapshot_id": snapshot_id, "success": true }))
}

async fn update_playlist(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let playlist_id = parse_playlist_id(args, "playlistId")?;

    let name = args.get("name").and_then(|v| v.as_str().map(|s| s.to_string()));
    let public = args.get("public").and_then(Value::as_bool);
    let collaborative = args.get("collaborative").and_then(Value::as_bool);
    let description = args.get("description").and_then(|v| v.as_str().map(|s| s.to_string()));

    if name.is_none() && public.is_none() && collaborative.is_none() && description.is_none() {
        return Err(
            "At least one field must be provided: name, description, public, collaborative"
                .to_string(),
        );
    }

    client
        .update_playlist(&playlist_id, name, public, collaborative, description)
        .await?;
    Ok(json!({ "success": true }))
}

async fn remove_tracks_from_playlist(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let playlist_id = parse_playlist_id(args, "playlistId")?;
    let track_uris = parse_track_ids(args, "trackIds", 100)?;
    let snapshot_id_param = parse_optional_snapshot_id(args, "snapshotId")?;

    let snapshot_id = client
        .remove_tracks_from_playlist(&playlist_id, track_uris, snapshot_id_param)
        .await?;
    Ok(json!({ "snapshot_id": snapshot_id, "success": true }))
}

async fn reorder_playlist_items(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let playlist_id = parse_playlist_id(args, "playlistId")?;
    let range_start = parse_required_u32(args, "rangeStart")?;
    let insert_before = parse_required_u32(args, "insertBefore")?;
    let range_length = parse_optional_u32(args, "rangeLength")?;
    if matches!(range_length, Some(0)) {
        return Err("rangeLength must be greater than 0 when provided".to_string());
    }
    let snapshot_id_param = parse_optional_snapshot_id(args, "snapshotId")?;

    let snapshot_id = client
        .reorder_playlist_items(
            &playlist_id,
            range_start,
            insert_before,
            range_length,
            snapshot_id_param,
        )
        .await?;
    Ok(json!({ "snapshot_id": snapshot_id, "success": true }))
}

async fn save_or_remove_album(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let album_ids = parse_spotify_ids(args, "albumIds", "album", SPOTIFY_ALBUM_BATCH_LIMIT)?;

    let action = args
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: action ('save' or 'remove')".to_string())?;

    if action != "save" && action != "remove" {
        return Err("action must be 'save' or 'remove'".to_string());
    }

    client.save_or_remove_albums(album_ids, action).await?;
    Ok(json!({ "success": true, "action": action }))
}

async fn remove_users_saved_tracks(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let track_ids = parse_spotify_ids(args, "trackIds", "track", SPOTIFY_TRACK_BATCH_LIMIT)?;

    client.remove_users_saved_tracks(track_ids).await?;
    Ok(json!({ "success": true, "action": "remove" }))
}

async fn play_music(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let uri = resolve_playback_uri(args)?;

    match client.play_on_best_device(&uri).await {
        Ok(_) => Ok(
            json!({"action": "play", "success": true, "mode": "api", "device_auto_select": true}),
        ),
        Err(_) => match client.play_playback(&uri).await {
            Ok(_) => Ok(json!({"action": "play", "success": true, "mode": "api"})),
            Err(error) => {
                // Local fallback bypass: Try to open the URI directly via the OS
                #[cfg(target_os = "windows")]
                {
                    if let Ok(_) = crate::tools::shell::run_command("cmd", &["/C", "start", &uri]) {
                        return Ok(json!({
                            "action": "play", 
                            "success": true, 
                            "mode": "local_fallback", 
                            "message": "API playback failed, bypassed via local URI launch",
                            "uri": uri
                        }));
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    if let Ok(_) = crate::tools::shell::run_command("xdg-open", &[&uri]) {
                        return Ok(json!({
                            "action": "play",
                            "success": true,
                            "mode": "local_fallback",
                            "message": "API playback failed, bypassed via local URI launch",
                            "uri": uri
                        }));
                    }
                }
                Err(error)
            }
        },
    }
}

async fn pause_playback(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let device_id = args.get("deviceId").and_then(Value::as_str);
    client.pause_playback(device_id).await?;
    let mut response = json!({"action": "pause", "success": true, "mode": "api"});
    if let Some(device_id) = device_id {
        response["device_id"] = json!(device_id);
    }
    Ok(response)
}

async fn resume_playback(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let device_id = args.get("deviceId").and_then(Value::as_str);
    client.resume_playback(device_id).await?;

    let mut response = json!({"action": "resume", "success": true, "mode": "api"});
    if let Some(device_id) = device_id {
        response["device_id"] = json!(device_id);
    }

    Ok(response)
}

async fn skip_to_next(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let device_id = args.get("deviceId").and_then(Value::as_str);
    client.next_track(device_id).await?;
    let mut response = json!({"action": "next", "success": true, "mode": "api"});
    if let Some(device_id) = device_id {
        response["device_id"] = json!(device_id);
    }
    Ok(response)
}

async fn skip_to_previous(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let device_id = args.get("deviceId").and_then(Value::as_str);
    client.previous_track(device_id).await?;
    let mut response = json!({"action": "previous", "success": true, "mode": "api"});
    if let Some(device_id) = device_id {
        response["device_id"] = json!(device_id);
    }
    Ok(response)
}

async fn set_volume(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let level = args
        .get("volumePercent")
        .and_then(Value::as_i64)
        .ok_or_else(|| "Missing required field: volumePercent".to_string())?;
    let clamped_level = level.clamp(0, 100) as u8;
    let device_id = args.get("deviceId").and_then(Value::as_str);
    client.set_volume_for_device(clamped_level, device_id).await?;
    Ok(json!({"action": "set_volume", "success": true, "level": clamped_level, "mode": "api"}))
}

async fn adjust_volume(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let adjustment = args
        .get("adjustment")
        .and_then(Value::as_i64)
        .ok_or_else(|| "Missing required field: adjustment".to_string())?;
    let device_id = args.get("deviceId").and_then(Value::as_str);

    let before = client.get_volume_for_device(device_id).await?;
    let after = (before as i64 + adjustment).clamp(0, 100) as u8;
    client.set_volume_for_device(after, device_id).await?;

    let mut response = json!({
        "action": "adjust_volume",
        "success": true,
        "mode": "api",
        "adjustment": adjustment,
        "before": before,
        "after": after
    });
    if let Some(device_id) = device_id {
        response["device_id"] = json!(device_id);
    }

    Ok(response)
}

async fn add_to_queue(args: &Map<String, Value>, client: &SpotifyClient) -> Result<Value, String> {
    let uri = resolve_queue_uri(args)?;
    let device_id = args.get("deviceId").and_then(Value::as_str);

    client.add_to_queue(&uri, device_id).await?;

    let mut response = json!({
        "action": "add_to_queue",
        "success": true,
        "mode": "api",
        "uri": uri
    });
    if let Some(device_id) = device_id {
        response["device_id"] = json!(device_id);
    }

    Ok(response)
}
