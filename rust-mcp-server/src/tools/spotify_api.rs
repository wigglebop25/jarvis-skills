mod album;
mod library;
mod models;
mod playback;
mod playlist;
mod playlist_management;
mod query;

use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

pub use models::*;

use std::sync::{Arc, RwLock};

pub struct SpotifyClient {
    client_id: String,
    client_secret: String,
    http_client: Client,
    token_cache: Arc<RwLock<Option<TokenCache>>>,
}

struct TokenCache {
    token: String,
    expires_at: i64,
}

impl SpotifyClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http_client: Client::new(),
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub(crate) async fn get_access_token(&self) -> Result<String, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 1. Try memory cache first (lowest delay)
        {
            if let Ok(cache) = self.token_cache.read() {
                if let Some(c) = cache.as_ref() {
                    if c.expires_at > now + 60 {
                        return Ok(c.token.clone());
                    }
                }
            }
        }

        // 2. Try environment variable
        if let Ok(token) = std::env::var("SPOTIPY_ACCESS_TOKEN") {
            if !token.is_empty() {
                return Ok(token);
            }
        }

        // 3. Try .cache file
        let cache_paths = [".cache", "../.cache"];
        let mut cache_json: Option<serde_json::Value> = None;

        for path in cache_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    cache_json = Some(json);
                    break;
                }
            }
        }

        if let Some(cache) = cache_json {
            if let Some(token) = cache.get("access_token").and_then(|v| v.as_str()) {
                let expires_at = cache.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0);

                // If token is valid and not expired (with 1 minute buffer)
                if expires_at > now + 60 {
                    self.update_memory_cache(token.to_string(), expires_at);
                    return Ok(token.to_string());
                }

                // Try to refresh if we have a refresh token
                if let Some(refresh_token) = cache.get("refresh_token").and_then(|v| v.as_str()) {
                    if let Ok(new_token) = self.refresh_access_token(refresh_token).await {
                        return Ok(new_token);
                    }
                }

                // Last resort
                return Ok(token.to_string());
            }
        }

        // 4. Fallback to client_credentials
        self.get_client_credentials_token().await
    }

    fn update_memory_cache(&self, token: String, expires_at: i64) {
        if let Ok(mut cache) = self.token_cache.write() {
            *cache = Some(TokenCache { token, expires_at });
        }
    }

    async fn refresh_access_token(&self, refresh_token: &str) -> Result<String, String> {
        let auth_header = format!(
            "Basic {}",
            STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret))
        );

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .http_client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", auth_header)
            .form(&params)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Spotify refresh request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Refresh failed: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct RefreshResponse {
            access_token: String,
            expires_in: i64,
            refresh_token: Option<String>,
        }

        let data: RefreshResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = now + data.expires_in;

        // Update memory cache
        self.update_memory_cache(data.access_token.clone(), expires_at);

        // Update .cache file
        if let Ok(content) = std::fs::read_to_string(".cache") {
            if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                json["access_token"] = serde_json::json!(data.access_token);
                json["expires_at"] = serde_json::json!(expires_at);
                if let Some(new_rt) = data.refresh_token {
                    json["refresh_token"] = serde_json::json!(new_rt);
                }
                let _ = std::fs::write(".cache", serde_json::to_string_pretty(&json).unwrap_or_default());
            }
        }

        Ok(data.access_token)
    }

    async fn get_client_credentials_token(&self) -> Result<String, String> {
        let auth_header = format!(
            "Basic {}",
            STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret))
        );

        let params = [("grant_type", "client_credentials")];

        let response = self
            .http_client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", auth_header)
            .form(&params)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Spotify token request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Spotify auth failed: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token_data: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        Ok(token_data.access_token)
    }
}
