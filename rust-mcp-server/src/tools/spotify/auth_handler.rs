use serde_json::Value;

use super::super::spotify_auth;

pub async fn maybe_handle_auth_tool(name: &str) -> Option<Result<Value, String>> {
    match name {
        "authorizeSpotify" | "authorize" => Some(authorize().await),
        "checkSpotifyAuth" | "check_auth" => Some(check_auth().await),
        _ => None,
    }
}

async fn authorize() -> Result<Value, String> {
    let auth = build_auth()?;
    auth.start_oauth_flow().await
}

async fn check_auth() -> Result<Value, String> {
    let auth = build_auth()?;
    let _ = auth.start_callback_server().await;
    Ok(auth.check_auth().await)
}

fn build_auth() -> Result<spotify_auth::SpotifyAuth, String> {
    let client_id = std::env::var("SPOTIPY_CLIENT_ID")
        .map_err(|_| "Missing SPOTIPY_CLIENT_ID environment variable".to_string())?;
    let client_secret = std::env::var("SPOTIPY_CLIENT_SECRET")
        .map_err(|_| "Missing SPOTIPY_CLIENT_SECRET environment variable".to_string())?;
    let mut redirect_uri = std::env::var("SPOTIPY_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8888/callback".to_string());
    
    // Automation: Spotify Dashboard often strictly requires 'localhost' over '127.0.0.1'
    if redirect_uri.contains("127.0.0.1") {
        redirect_uri = redirect_uri.replace("127.0.0.1", "localhost");
    }

    Ok(spotify_auth::SpotifyAuth::new(
        client_id,
        client_secret,
        redirect_uri,
    ))
}
