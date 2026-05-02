use std::time::Duration;

use serde_json::{json, Value};

use super::SpotifyClient;

impl SpotifyClient {
    pub async fn create_playlist(
        &self,
        name: &str,
        public: Option<bool>,
        collaborative: Option<bool>,
        description: Option<String>,
    ) -> Result<Value, String> {
        if name.trim().is_empty() {
            return Err("name cannot be empty".to_string());
        }

        let token = self.get_access_token().await?;

        let user_response = self
            .http_client
            .get("https://api.spotify.com/v1/me")
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get user profile: {}", e))?;

        let user_id = if user_response.status().is_success() {
            let data: Value = user_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse user profile: {}", e))?;
            data["id"]
                .as_str()
                .ok_or("User ID not found in profile")?
                .to_string()
        } else {
            return Err(format!(
                "Failed to get user profile: {}",
                user_response.status()
            ));
        };

        let mut body = json!({ "name": name });
        if let Some(p) = public {
            body["public"] = json!(p);
        }
        if let Some(c) = collaborative {
            body["collaborative"] = json!(c);
        }
        if let Some(d) = description {
            body["description"] = json!(d);
        }

        let url = format!("https://api.spotify.com/v1/users/{}/playlists", user_id);
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to create playlist: {}", e))?;

        if response.status().is_success() || response.status().as_u16() == 201 {
            let data: Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse create playlist response: {}", e))?;
            Ok(data)
        } else {
            Err(format!("Failed to create playlist: {}", response.status()))
        }
    }

    pub async fn add_tracks_to_playlist(
        &self,
        playlist_id: &str,
        track_uris: Vec<String>,
        position: Option<u32>,
    ) -> Result<String, String> {
        if playlist_id.trim().is_empty() {
            return Err("playlistId cannot be empty".to_string());
        }
        if track_uris.is_empty() {
            return Err("trackUris cannot be empty".to_string());
        }
        if track_uris.len() > 100 {
            return Err("Cannot add more than 100 tracks at once".to_string());
        }

        let token = self.get_access_token().await?;

        let mut body = json!({ "uris": track_uris });
        if let Some(p) = position {
            body["position"] = json!(p);
        }

        let url = format!(
            "https://api.spotify.com/v1/playlists/{}/tracks",
            playlist_id
        );
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to add tracks: {}", e))?;

        if response.status().is_success() || response.status().as_u16() == 201 {
            let data: Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse add tracks response: {}", e))?;
            parse_snapshot_id(&data, "add tracks")
        } else {
            Err(format!("Failed to add tracks: {}", response.status()))
        }
    }

    pub async fn update_playlist(
        &self,
        playlist_id: &str,
        name: Option<String>,
        public: Option<bool>,
        collaborative: Option<bool>,
        description: Option<String>,
    ) -> Result<(), String> {
        if playlist_id.trim().is_empty() {
            return Err("playlistId cannot be empty".to_string());
        }
        if name.is_none() && public.is_none() && collaborative.is_none() && description.is_none() {
            return Err(
                "At least one field must be provided: name, description, public, collaborative"
                    .to_string(),
            );
        }

        let token = self.get_access_token().await?;

        let mut body = json!({});
        if let Some(n) = name {
            body["name"] = json!(n);
        }
        if let Some(p) = public {
            body["public"] = json!(p);
        }
        if let Some(c) = collaborative {
            body["collaborative"] = json!(c);
        }
        if let Some(d) = description {
            body["description"] = json!(d);
        }

        let url = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);
        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to update playlist: {}", e))?;

        if response.status().is_success() || response.status().as_u16() == 200 {
            Ok(())
        } else {
            Err(format!("Failed to update playlist: {}", response.status()))
        }
    }

    pub async fn remove_tracks_from_playlist(
        &self,
        playlist_id: &str,
        track_uris: Vec<String>,
        snapshot_id: Option<String>,
    ) -> Result<String, String> {
        if playlist_id.trim().is_empty() {
            return Err("playlistId cannot be empty".to_string());
        }
        if track_uris.is_empty() {
            return Err("trackIds cannot be empty".to_string());
        }
        if track_uris.len() > 100 {
            return Err("Cannot remove more than 100 tracks at once".to_string());
        }

        let token = self.get_access_token().await?;

        let tracks: Vec<Value> = track_uris
            .into_iter()
            .map(|uri| json!({ "uri": uri }))
            .collect();
        let mut body = json!({ "tracks": tracks });
        if let Some(s) = snapshot_id {
            if s.trim().is_empty() {
                return Err("snapshotId cannot be empty when provided".to_string());
            }
            body["snapshot_id"] = json!(s);
        }

        let url = format!(
            "https://api.spotify.com/v1/playlists/{}/tracks",
            playlist_id
        );
        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to remove tracks: {}", e))?;

        if response.status().is_success() {
            let data: Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse remove tracks response: {}", e))?;
            parse_snapshot_id(&data, "remove tracks")
        } else {
            Err(format!("Failed to remove tracks: {}", response.status()))
        }
    }

    pub async fn reorder_playlist_items(
        &self,
        playlist_id: &str,
        range_start: u32,
        insert_before: u32,
        range_length: Option<u32>,
        snapshot_id: Option<String>,
    ) -> Result<String, String> {
        if playlist_id.trim().is_empty() {
            return Err("playlistId cannot be empty".to_string());
        }
        if matches!(range_length, Some(0)) {
            return Err("rangeLength must be greater than 0 when provided".to_string());
        }

        let token = self.get_access_token().await?;

        let mut body = json!({
            "range_start": range_start,
            "insert_before": insert_before
        });
        if let Some(l) = range_length {
            body["range_length"] = json!(l);
        }
        if let Some(s) = snapshot_id {
            if s.trim().is_empty() {
                return Err("snapshotId cannot be empty when provided".to_string());
            }
            body["snapshot_id"] = json!(s);
        }

        let url = format!(
            "https://api.spotify.com/v1/playlists/{}/tracks",
            playlist_id
        );
        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to reorder playlist: {}", e))?;

        if response.status().is_success() {
            let data: Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse reorder playlist response: {}", e))?;
            parse_snapshot_id(&data, "reorder playlist items")
        } else {
            Err(format!("Failed to reorder playlist: {}", response.status()))
        }
    }
}

fn parse_snapshot_id(response_data: &Value, operation: &str) -> Result<String, String> {
    let snapshot_id = response_data
        .get("snapshot_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Spotify {operation} response missing snapshot_id"))?;

    Ok(snapshot_id.to_string())
}
