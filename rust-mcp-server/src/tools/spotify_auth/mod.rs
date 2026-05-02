#![allow(dead_code, unused)]

use base64::Engine;
use serde_json::{json, Value};
use std::env;

mod server;

/// Spotify OAuth authentication handler
pub struct SpotifyAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

/// User profile information from Spotify
#[derive(Debug, Clone)]
pub struct UserProfile {
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub profile_url: Option<String>,
    pub product: Option<String>,
}

impl SpotifyAuth {
    /// Create a new SpotifyAuth instance
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Self {
        SpotifyAuth {
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    /// Generate OAuth authorization URL for user login
    pub fn get_login_url(&self) -> String {
        let scopes = vec![
            "user-read-private",
            "user-read-email",
            "user-read-playback-state",
            "user-modify-playback-state",
            "user-read-currently-playing",
            "playlist-read-private",
            "playlist-read-collaborative",
            "playlist-modify-private",
            "playlist-modify-public",
            "user-library-read",
            "user-read-recently-played",
            "user-top-read",
        ]
        .join("%20");

        format!(
            "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            scopes
        )
    }

    /// Check if a token is valid and fetch user profile
    pub async fn get_user_profile(&self, token: &str) -> Result<UserProfile, String> {
        let client = reqwest::Client::new();

        let response = client
            .get("https://api.spotify.com/v1/me")
            .header("Authorization", format!("Bearer {}", token))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.status() == 401 {
            return Err("Invalid or expired token".to_string());
        }

        if response.status() == 403 {
            // Token is valid, but user profile access is forbidden
            return Ok(UserProfile {
                id: "unknown".to_string(),
                display_name: Some("Spotify User".to_string()),
                email: None,
                profile_url: None,
                product: None,
            });
        }

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(UserProfile {
            id: data["id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            display_name: data["display_name"].as_str().map(|s| s.to_string()),
            email: data["email"].as_str().map(|s| s.to_string()),
            profile_url: data["external_urls"]["spotify"]
                .as_str()
                .map(|s| s.to_string()),
            product: data["product"].as_str().map(|s| s.to_string()),
        })
    }

    /// Try to refresh the token using the refresh_token from .cache
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));

        let response = client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .map_err(|e| format!("Network error during refresh: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Refresh error: {}", response.status()));
        }

        let mut data: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

        if let Some(new_access_token) = data.get("access_token").and_then(|v| v.as_str()) {
            // Update the .cache file with the new token
            if let Ok(cache_content) = std::fs::read_to_string(".cache") {
                if let Ok(mut cache_json) = serde_json::from_str::<Value>(&cache_content) {
                    cache_json["access_token"] = json!(new_access_token);
                    if let Some(expires_in) = data.get("expires_in").and_then(|v| v.as_i64()) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;
                        cache_json["expires_at"] = json!(now + expires_in);
                    }
                    if let Some(new_refresh_token) = data.get("refresh_token") {
                        cache_json["refresh_token"] = new_refresh_token.clone();
                    }
                    let _ = std::fs::write(
                        ".cache",
                        serde_json::to_string_pretty(&cache_json).unwrap_or_default(),
                    );
                }
            }
            return Ok(new_access_token.to_string());
        }

        Err("No access token in refresh response".to_string())
    }

    /// Get a valid access token (resolving from env, cache, and handling refresh)
    pub async fn get_valid_token(&self) -> Result<String, String> {
        // Try to get token from environment variable
        if let Ok(token) = env::var("SPOTIPY_ACCESS_TOKEN") {
            if !token.is_empty() {
                if self.get_user_profile(&token).await.is_ok() {
                    return Ok(token);
                }
            }
        }

        // Try to get token from .cache file
        match std::fs::read_to_string(".cache") {
            Ok(cache_content) => {
                if let Ok(cache_json) = serde_json::from_str::<Value>(&cache_content) {
                    if let Some(token) = cache_json.get("access_token").and_then(|v| v.as_str()) {
                        if self.get_user_profile(token).await.is_ok() {
                            return Ok(token.to_string());
                        }
                    }
                    
                    if let Some(refresh_token) = cache_json.get("refresh_token").and_then(|v| v.as_str()) {
                        if let Ok(new_token) = self.refresh_token(refresh_token).await {
                            if self.get_user_profile(&new_token).await.is_ok() {
                                return Ok(new_token);
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }

        Err("No valid token found".to_string())
    }

    /// Verify if user is authenticated
    pub async fn is_authenticated(&self) -> Result<UserProfile, String> {
        let token = self.get_valid_token().await?;
        self.get_user_profile(&token).await
    }

    /// Get authentication status
    pub async fn check_auth(&self) -> Value {
        match self.is_authenticated().await {
            Ok(profile) => {
                json!({
                    "authenticated": true,
                    "user": {
                        "id": profile.id,
                        "display_name": profile.display_name,
                        "email": profile.email,
                        "profile_url": profile.profile_url,
                        "product": profile.product,
                    },
                    "message": format!(
                        "Logged in as {} ({})",
                        profile.display_name.unwrap_or_else(|| "Spotify User".to_string()),
                        profile.product.unwrap_or_else(|| "unknown".to_string())
                    )
                })
            }
            Err(_) => {
                json!({
                    "authenticated": false,
                    "login_url": self.get_login_url(),
                    "message": "Not authenticated. Please visit the login URL to authorize."
                })
            }
        }
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(&self, code: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));

        let response = client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Token exchange failed: {}", response.status()));
        }

        let data: Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let access_token = data["access_token"]
            .as_str()
            .ok_or("No access token in response")?
            .to_string();

        let refresh_token = data["refresh_token"].as_str().map(|s| s.to_string());
        let expires_in = data["expires_in"].as_i64().unwrap_or(3600);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut cache_json = json!({
            "access_token": access_token,
            "expires_at": now + expires_in,
        });

        if let Some(refresh_token) = refresh_token {
            cache_json["refresh_token"] = json!(refresh_token);
        }

        let _ = std::fs::write(
            ".cache",
            serde_json::to_string_pretty(&cache_json).unwrap_or_default(),
        );

        Ok(access_token)
    }

    /// Start the OAuth callback server and open browser automatically
    pub async fn start_oauth_flow(&self) -> Result<Value, String> {
        self.start_callback_server().await?;

        let login_url = self.get_login_url();
        
        if cfg!(target_os = "windows") {
            let _ = std::process::Command::new("cmd")
                .args(&["/C", "start", "", &login_url])
                .spawn();
        } else {
            let _ = std::process::Command::new("xdg-open")
                .arg(&login_url)
                .spawn();
        }

        Ok(json!({
            "message": "Opening browser for Spotify authorization...",
            "login_url": login_url,
            "callback_server": "Started on http://127.0.0.1:8888/callback"
        }))
    }

    pub async fn start_callback_server(&self) -> Result<(), String> {
        server::start_callback_server(
            self.client_id.clone(),
            self.client_secret.clone(),
            self.redirect_uri.clone()
        ).await
    }
}
