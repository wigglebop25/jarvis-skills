use serde_json::{Map, Value};

use super::spotify_api::SpotifyClient;

mod auth_handler;
mod helpers;
mod read_handler;
mod write_handler;

pub async fn handle_spotify_tool(
    name: &str,
    args: &Map<String, Value>,
    state: &crate::AppState,
) -> Result<Value, String> {
    if let Some(result) = auth_handler::maybe_handle_auth_tool(name).await {
        return result;
    }

    let client = if let Some(ref c) = state.spotify {
        c.clone()
    } else {
        std::sync::Arc::new(spotify_client_from_env()?)
    };

    if let Some(result) = read_handler::handle_read_tool(name, args, &client).await {
        return result;
    }

    if let Some(result) = write_handler::handle_write_tool(name, args, &client).await {
        return result;
    }

    Err(format!("Unsupported spotify action: {name}"))
}

fn spotify_client_from_env() -> Result<SpotifyClient, String> {
    let client_id = std::env::var("SPOTIPY_CLIENT_ID")
        .map_err(|_| "Missing SPOTIPY_CLIENT_ID environment variable".to_string())?;
    let client_secret = std::env::var("SPOTIPY_CLIENT_SECRET")
        .map_err(|_| "Missing SPOTIPY_CLIENT_SECRET environment variable".to_string())?;

    Ok(SpotifyClient::new(client_id, client_secret))
}
