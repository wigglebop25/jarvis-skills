use std::time::Duration;

use serde::Deserialize;

use super::{DeviceInfo, SpotifyClient, TrackInfo};

impl SpotifyClient {
    pub async fn search_tracks(&self, query: &str, limit: u32) -> Result<Vec<TrackInfo>, String> {
        let token = self.get_access_token().await?;

        let url = format!(
            "https://api.spotify.com/v1/search?q={}&type=track&limit={}",
            urlencoding::encode(query),
            limit
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Spotify search request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Spotify search failed: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct SearchResponse {
            tracks: Option<TracksObject>,
        }

        #[derive(Deserialize)]
        struct TracksObject {
            items: Vec<TrackObject>,
        }

        #[derive(Deserialize)]
        struct TrackObject {
            name: String,
            artists: Vec<ArtistObject>,
            album: AlbumObject,
            uri: String,
        }

        #[derive(Deserialize)]
        struct ArtistObject {
            name: String,
        }

        #[derive(Deserialize)]
        struct AlbumObject {
            name: String,
        }

        let search_data: SearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse search response: {}", e))?;

        let tracks = search_data
            .tracks
            .unwrap_or_else(|| TracksObject { items: Vec::new() })
            .items
            .into_iter()
            .map(|track| TrackInfo {
                name: track.name,
                artist: track
                    .artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                album: track.album.name,
                uri: track.uri,
                progress_ms: None,
                duration_ms: None,
            })
            .collect();

        Ok(tracks)
    }

    pub async fn get_current_track(&self) -> Result<TrackInfo, String> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .get("https://api.spotify.com/v1/me/player/currently-playing")
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Spotify current track request failed: {}", e))?;

        if response.status() == 204 {
            return Err("No track currently playing".to_string());
        }

        if !response.status().is_success() {
            return Err(format!(
                "Spotify current track failed: {} (may require Premium account)",
                response.status()
            ));
        }

        #[derive(Deserialize)]
        struct CurrentPlayback {
            item: Option<TrackObject>,
            progress_ms: Option<u32>,
        }

        #[derive(Deserialize)]
        struct TrackObject {
            name: String,
            artists: Vec<ArtistObject>,
            album: AlbumObject,
            uri: String,
            duration_ms: u32,
        }

        #[derive(Deserialize)]
        struct ArtistObject {
            name: String,
        }

        #[derive(Deserialize)]
        struct AlbumObject {
            name: String,
        }

        let playback: CurrentPlayback = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse current track response: {}", e))?;

        match playback.item {
            Some(track) => Ok(TrackInfo {
                name: track.name,
                artist: track
                    .artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                album: track.album.name,
                uri: track.uri,
                progress_ms: playback.progress_ms,
                duration_ms: Some(track.duration_ms),
            }),
            None => Err("No track information available".to_string()),
        }
    }

    pub async fn get_devices(&self) -> Result<Vec<DeviceInfo>, String> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .get("https://api.spotify.com/v1/me/player/devices")
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get devices: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to get devices: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct DevicesResponse {
            devices: Vec<DeviceInfo>,
        }

        let data: DevicesResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse devices response: {}", e))?;

        Ok(data.devices)
    }
}
