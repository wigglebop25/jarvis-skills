use serde_json::{Map, Value};

pub fn parse_playlist_id(args: &Map<String, Value>, field: &str) -> Result<String, String> {
    let value = parse_non_empty_string(args, field)?;
    normalize_spotify_id(&value, "playlist").map_err(|_| {
        format!("Invalid {field}: expected Spotify playlist ID, URI, or open.spotify.com URL")
    })
}

pub fn parse_track_uris(
    args: &Map<String, Value>,
    field: &str,
    max_batch_size: usize,
) -> Result<Vec<String>, String> {
    let values = parse_required_string_array(args, field)?;
    if values.len() > max_batch_size {
        return Err(format!("Cannot add more than {max_batch_size} tracks at once"));
    }

    values
        .into_iter()
        .map(|value| {
            normalize_spotify_id(&value, "track")
                .map(|id| format!("spotify:track:{id}"))
                .map_err(|_| {
                    format!(
                        "Invalid value in {field}: expected Spotify track URI, ID, or open.spotify.com URL"
                    )
                })
        })
        .collect()
}

pub fn parse_track_ids(
    args: &Map<String, Value>,
    field: &str,
    max_batch_size: usize,
) -> Result<Vec<String>, String> {
    let values = parse_required_string_array(args, field)?;
    if values.len() > max_batch_size {
        return Err(format!("Cannot remove more than {max_batch_size} tracks at once"));
    }

    values
        .into_iter()
        .map(|value| {
            normalize_spotify_id(&value, "track")
                .map(|id| format!("spotify:track:{id}"))
                .map_err(|_| {
                    format!(
                        "Invalid value in {field}: expected Spotify track ID, URI, or open.spotify.com URL"
                    )
                })
        })
        .collect()
}

pub fn parse_required_string_array(args: &Map<String, Value>, field: &str) -> Result<Vec<String>, String> {
    let values = args
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Missing required field: {field} (array)"))?;

    let strings: Vec<String> = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(|s| s.trim().to_string())
                .ok_or_else(|| format!("{field} entries must be strings"))
        })
        .collect::<Result<_, _>>()?;

    if strings.is_empty() {
        return Err(format!("{field} array cannot be empty"));
    }

    if strings.iter().any(|value| value.is_empty()) {
        return Err(format!("{field} entries cannot be empty"));
    }

    Ok(strings)
}

pub fn parse_string_or_array(args: &Map<String, Value>, field: &str) -> Result<Vec<String>, String> {
    let value = args
        .get(field)
        .ok_or_else(|| format!("Missing required field: {field}"))?;

    if let Some(single) = value.as_str() {
        let trimmed = single.trim();
        if trimmed.is_empty() {
            return Err(format!("{field} cannot be empty"));
        }
        return Ok(vec![trimmed.to_string()]);
    }

    let values = value
        .as_array()
        .ok_or_else(|| format!("{field} must be a string or array of strings"))?;

    let strings: Vec<String> = values
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(|s| s.trim().to_string())
                .ok_or_else(|| format!("{field} entries must be strings"))
        })
        .collect::<Result<_, _>>()?;

    if strings.is_empty() {
        return Err(format!("{field} array cannot be empty"));
    }
    if strings.iter().any(|entry| entry.is_empty()) {
        return Err(format!("{field} entries cannot be empty"));
    }

    Ok(strings)
}

pub fn parse_spotify_ids(
    args: &Map<String, Value>,
    field: &str,
    expected_kind: &str,
    max_batch_size: usize,
) -> Result<Vec<String>, String> {
    let values = parse_string_or_array(args, field)?;
    if values.len() > max_batch_size {
        return Err(format!(
            "Spotify limit exceeded for {field}: max {max_batch_size}, got {}",
            values.len()
        ));
    }

    values
        .into_iter()
        .map(|value| {
            normalize_spotify_id(&value, expected_kind).map_err(|_| {
                format!(
                    "Invalid value in {field}: expected Spotify {expected_kind} ID, URI, or open.spotify.com URL"
                )
            })
        })
        .collect()
}

pub fn parse_non_empty_string(args: &Map<String, Value>, field: &str) -> Result<String, String> {
    let value = args
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Missing required field: {field}"))?
        .trim()
        .to_string();

    if value.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }

    Ok(value)
}

pub fn parse_required_u32(args: &Map<String, Value>, field: &str) -> Result<u32, String> {
    let value = args
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Missing required field: {field}"))?;

    u32::try_from(value).map_err(|_| format!("{field} must be between 0 and {}", u32::MAX))
}

pub fn parse_optional_u32(args: &Map<String, Value>, field: &str) -> Result<Option<u32>, String> {
    args.get(field)
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| format!("{field} must be an integer"))
                .and_then(|n| {
                    u32::try_from(n).map_err(|_| format!("{field} must be between 0 and {}", u32::MAX))
                })
        })
        .transpose()
}

pub fn parse_optional_snapshot_id(args: &Map<String, Value>, field: &str) -> Result<Option<String>, String> {
    let value = args.get(field).and_then(Value::as_str).map(str::trim);
    if let Some(snapshot_id) = value {
        if snapshot_id.is_empty() {
            return Err(format!("{field} cannot be empty when provided"));
        }
        if snapshot_id.len() > 512 {
            return Err(format!("{field} is too long"));
        }
        Ok(Some(snapshot_id.to_string()))
    } else {
        Ok(None)
    }
}

pub fn normalize_spotify_id(input: &str, expected_kind: &str) -> Result<String, String> {
    let value = input.trim();
    if value.is_empty() {
        return Err("Value cannot be empty".to_string());
    }

    let candidate = if let Some(rest) = value.strip_prefix("spotify:") {
        let mut parts = rest.splitn(3, ':');
        let kind = parts.next().unwrap_or_default();
        let id = parts.next().unwrap_or_default();
        if kind != expected_kind || id.is_empty() {
            return Err("Unsupported Spotify URI".to_string());
        }
        id.split('?').next().unwrap_or_default()
    } else if let Some(rest) = value
        .strip_prefix("https://open.spotify.com/")
        .or_else(|| value.strip_prefix("http://open.spotify.com/"))
    {
        let mut parts = rest.splitn(3, '/');
        let kind = parts.next().unwrap_or_default();
        let id_with_query = parts.next().unwrap_or_default();
        if kind != expected_kind || id_with_query.is_empty() {
            return Err("Unsupported Spotify URL".to_string());
        }
        id_with_query.split('?').next().unwrap_or_default()
    } else {
        value
    };

    if candidate.len() != 22 || !candidate.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Invalid Spotify ID format".to_string());
    }

    Ok(candidate.to_string())
}

pub fn resolve_queue_uri(args: &Map<String, Value>) -> Result<String, String> {
    if let Some(uri) = args.get("uri").and_then(Value::as_str) {
        let trimmed_uri = uri.trim();
        if trimmed_uri.is_empty() {
            return Err("uri cannot be empty".to_string());
        }
        return Ok(trimmed_uri.to_string());
    }

    let item_type = args
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required fields: provide uri or type+id".to_string())?;
    let id = args
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required fields: provide uri or type+id".to_string())?;

    let item_type = item_type.trim();
    let id = id.trim();
    if item_type.is_empty() || id.is_empty() {
        return Err("type and id cannot be empty".to_string());
    }
    let item_type = normalize_item_type(item_type)?;

    Ok(format!("spotify:{}:{}", item_type, id))
}

pub fn resolve_playback_uri(args: &Map<String, Value>) -> Result<String, String> {
    if let Some(uri) = args.get("uri").and_then(Value::as_str) {
        let trimmed_uri = uri.trim();
        if trimmed_uri.is_empty() {
            return Err("uri cannot be empty".to_string());
        }
        return Ok(trimmed_uri.to_string());
    }

    let item_type = args
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required fields: provide uri or type+id".to_string())?;
    let id = args
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required fields: provide uri or type+id".to_string())?;

    let item_type = normalize_item_type(item_type.trim())?;
    let id = id.trim();
    if id.is_empty() {
        return Err("id cannot be empty".to_string());
    }

    Ok(format!("spotify:{}:{}", item_type, id))
}

pub fn normalize_item_type(item_type: &str) -> Result<&'static str, String> {
    match item_type.to_ascii_lowercase().as_str() {
        "track" => Ok("track"),
        "album" => Ok("album"),
        "artist" => Ok("artist"),
        "playlist" => Ok("playlist"),
        "episode" => Ok("episode"),
        "show" => Ok("show"),
        _ => Err("type must be one of: track, album, artist, playlist, episode, show".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalize_spotify_id_direct() {
        assert_eq!(normalize_spotify_id("6rqhFgOV96wjA7qRaqvC9v", "track").unwrap(), "6rqhFgOV96wjA7qRaqvC9v");
        assert_eq!(normalize_spotify_id(" 6rqhFgOV96wjA7qRaqvC9v ", "track").unwrap(), "6rqhFgOV96wjA7qRaqvC9v");
    }

    #[test]
    fn test_normalize_spotify_id_uri() {
        assert_eq!(normalize_spotify_id("spotify:track:6rqhFgOV96wjA7qRaqvC9v", "track").unwrap(), "6rqhFgOV96wjA7qRaqvC9v");
        assert!(normalize_spotify_id("spotify:album:6rqhFgOV96wjA7qRaqvC9v", "track").is_err());
    }

    #[test]
    fn test_normalize_spotify_id_url() {
        assert_eq!(normalize_spotify_id("https://open.spotify.com/track/6rqhFgOV96wjA7qRaqvC9v?si=abc", "track").unwrap(), "6rqhFgOV96wjA7qRaqvC9v");
        assert_eq!(normalize_spotify_id("http://open.spotify.com/track/6rqhFgOV96wjA7qRaqvC9v", "track").unwrap(), "6rqhFgOV96wjA7qRaqvC9v");
        assert!(normalize_spotify_id("https://open.spotify.com/album/6rqhFgOV96wjA7qRaqvC9v", "track").is_err());
    }

    #[test]
    fn test_normalize_spotify_id_invalid() {
        assert!(normalize_spotify_id("", "track").is_err());
        assert!(normalize_spotify_id("short", "track").is_err());
        assert!(normalize_spotify_id("too-long-id-that-is-more-than-22-chars", "track").is_err());
        assert!(normalize_spotify_id("invalid!char@in#id", "track").is_err());
    }

    #[test]
    fn test_parse_non_empty_string() {
        let mut args = Map::new();
        args.insert("name".to_string(), json!("My Playlist"));
        assert_eq!(parse_non_empty_string(&args, "name").unwrap(), "My Playlist");

        args.insert("empty".to_string(), json!("  "));
        assert!(parse_non_empty_string(&args, "empty").is_err());
        assert!(parse_non_empty_string(&args, "missing").is_err());
    }

    #[test]
    fn test_parse_required_u32() {
        let mut args = Map::new();
        args.insert("count".to_string(), json!(42));
        assert_eq!(parse_required_u32(&args, "count").unwrap(), 42);

        args.insert("negative".to_string(), json!(-1));
        assert!(parse_required_u32(&args, "negative").is_err());
    }

    #[test]
    fn test_normalize_item_type() {
        assert_eq!(normalize_item_type("Track").unwrap(), "track");
        assert_eq!(normalize_item_type("album").unwrap(), "album");
        assert!(normalize_item_type("invalid").is_err());
    }

    #[test]
    fn test_resolve_playback_uri_direct() {
        let mut args = Map::new();
        args.insert("uri".to_string(), json!("spotify:track:6rqhFgOV96wjA7qRaqvC9v"));
        assert_eq!(resolve_playback_uri(&args).unwrap(), "spotify:track:6rqhFgOV96wjA7qRaqvC9v");
    }

    #[test]
    fn test_resolve_playback_uri_type_id() {
        let mut args = Map::new();
        args.insert("type".to_string(), json!("track"));
        args.insert("id".to_string(), json!("6rqhFgOV96wjA7qRaqvC9v"));
        assert_eq!(resolve_playback_uri(&args).unwrap(), "spotify:track:6rqhFgOV96wjA7qRaqvC9v");
    }

    #[test]
    fn test_resolve_playback_uri_missing() {
        let args = Map::new();
        assert!(resolve_playback_uri(&args).is_err());
    }
}
