use std::time::Duration;

use serde::Deserialize;

use super::{PlaylistInfo, PlaylistTrack, SpotifyClient};

impl SpotifyClient {
    pub async fn get_playlists(&self, limit: u32, offset: u32) -> Result<Vec<PlaylistInfo>, String> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://api.spotify.com/v1/me/playlists?limit={}&offset={}",
            limit.min(50),
            offset
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get playlists: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to get playlists: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct PlaylistsResponse {
            items: Vec<PlaylistInfo>,
        }

        let data: PlaylistsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse playlists response: {}", e))?;

        Ok(data.items)
    }

    pub async fn get_playlist_tracks(
        &self,
        playlist_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PlaylistTrack>, String> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://api.spotify.com/v1/playlists/{}/items?limit={}&offset={}",
            playlist_id,
            limit.min(50),
            offset
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get playlist tracks: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            // Handle specific 403 errors which often occur for empty playlists
            if status == reqwest::StatusCode::FORBIDDEN {
                return Ok(Vec::new());
            }
            let error_text = response.text().await.unwrap_or_else(|_| "No error body".to_string());
            return Err(format!("Failed to get playlist tracks: {} - {}", status, error_text));
        }

        #[derive(Deserialize)]
        struct PlaylistTracksResponse {
            items: Vec<PlaylistTrackItem>,
        }

        #[derive(Deserialize)]
        struct PlaylistTrackItem {
            #[serde(alias = "item")]
            track: Option<TrackObject>,
            added_at: String,
        }

        #[derive(Deserialize)]
        struct TrackObject {
            id: String,
            name: String,
            artists: Vec<ArtistObject>,
            album: AlbumObject,
            duration_ms: u32,
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

        let data: PlaylistTracksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse playlist tracks response: {}", e))?;

        let tracks = data
            .items
            .into_iter()
            .filter_map(|item| {
                item.track.map(|track| PlaylistTrack {
                    id: track.id,
                    name: track.name,
                    artist: track
                        .artists
                        .iter()
                        .map(|artist| artist.name.clone())
                        .collect::<Vec<_>>()
                        .join(", "),
                    album: track.album.name,
                    duration: track.duration_ms,
                    added_at: item.added_at,
                    uri: track.uri,
                })
            })
            .collect();

        Ok(tracks)
    }

    pub async fn get_playlist(&self, playlist_id: &str) -> Result<PlaylistInfo, String> {
        let token = self.get_access_token().await?;
        let url = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get playlist: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "No error body".to_string());
            return Err(format!("Failed to get playlist: {} - {}", status, error_text));
        }

        let data: PlaylistInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse playlist response: {}", e))?;

        Ok(data)
    }
}
