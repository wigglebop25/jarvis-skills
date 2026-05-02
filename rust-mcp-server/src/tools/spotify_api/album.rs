use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use super::{AlbumInfo, PlaylistTrack, SpotifyClient};

const SPOTIFY_ALBUM_BATCH_LIMIT: usize = 50;
const SPOTIFY_TRACK_BATCH_LIMIT: usize = 50;

impl SpotifyClient {
    pub async fn get_album_tracks(
        &self,
        album_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<PlaylistTrack>, String> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://api.spotify.com/v1/albums/{}/tracks?limit={}&offset={}",
            album_id,
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
            .map_err(|e| format!("Failed to get album tracks: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to get album tracks: {}",
                response.status()
            ));
        }

        #[derive(Deserialize)]
        struct AlbumTracksResponse {
            items: Vec<TrackObject>,
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

        let data: AlbumTracksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse album tracks response: {}", e))?;

        let tracks = data
            .items
            .into_iter()
            .map(|track| PlaylistTrack {
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
                added_at: String::new(),
                uri: track.uri,
            })
            .collect();

        Ok(tracks)
    }

    pub async fn get_albums(&self, album_ids: Vec<String>) -> Result<Value, String> {
        let token = self.get_access_token().await?;
        let ids_string = album_ids.join(",");
        let url = format!(
            "https://api.spotify.com/v1/albums?ids={}",
            urlencoding::encode(&ids_string)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get albums: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to get albums: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct AlbumsResponse {
            albums: Vec<AlbumObject>,
        }

        #[derive(Deserialize)]
        struct AlbumObject {
            id: String,
            name: String,
            artists: Vec<ArtistObject>,
            release_date: String,
            album_type: String,
            total_tracks: u32,
            uri: String,
        }

        #[derive(Deserialize)]
        struct ArtistObject {
            name: String,
        }

        let data: AlbumsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse albums response: {}", e))?;

        let albums: Vec<AlbumInfo> = data
            .albums
            .into_iter()
            .map(|album| AlbumInfo {
                id: album.id,
                name: album.name,
                artists: album
                    .artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect(),
                release_date: album.release_date,
                album_type: album.album_type,
                total_tracks: album.total_tracks,
                uri: album.uri,
            })
            .collect();

        if albums.len() == 1 {
            serde_json::to_value(&albums[0]).map_err(|e| format!("Failed to serialize album: {}", e))
        } else {
            serde_json::to_value(&albums).map_err(|e| format!("Failed to serialize albums: {}", e))
        }
    }

    pub async fn check_user_saved_albums(
        &self,
        album_ids: Vec<String>,
    ) -> Result<Vec<bool>, String> {
        let token = self.get_access_token().await?;
        let ids_string = album_ids.join(",");
        let url = format!(
            "https://api.spotify.com/v1/me/albums/contains?ids={}",
            urlencoding::encode(&ids_string)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to check saved albums: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to check saved albums: {}",
                response.status()
            ));
        }

        let data: Vec<bool> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse check saved albums response: {}", e))?;

        Ok(data)
    }

    pub async fn save_or_remove_albums(
        &self,
        album_ids: Vec<String>,
        action: &str,
    ) -> Result<(), String> {
        if album_ids.is_empty() {
            return Err("albumIds cannot be empty".to_string());
        }
        if album_ids.len() > SPOTIFY_ALBUM_BATCH_LIMIT {
            return Err(format!(
                "Spotify limit exceeded for albumIds: max {SPOTIFY_ALBUM_BATCH_LIMIT}, got {}",
                album_ids.len()
            ));
        }
        if action != "save" && action != "remove" {
            return Err("action must be 'save' or 'remove'".to_string());
        }

        let token = self.get_access_token().await?;
        let joined_ids = album_ids.join(",");
        let ids = urlencoding::encode(&joined_ids);
        let url = format!("https://api.spotify.com/v1/me/albums?ids={ids}");

        let mut request = if action == "save" {
            self.http_client.put(&url)
        } else {
            self.http_client.delete(&url)
        };

        if action == "save" {
            request = request
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(reqwest::header::CONTENT_LENGTH, "0")
                .body("");
        }

        let response = request
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to {} albums: {}", action, e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to {} albums: {}",
                action,
                response.status()
            ))
        }
    }

    pub async fn remove_users_saved_tracks(&self, track_ids: Vec<String>) -> Result<(), String> {
        if track_ids.is_empty() {
            return Err("trackIds cannot be empty".to_string());
        }
        if track_ids.len() > SPOTIFY_TRACK_BATCH_LIMIT {
            return Err(format!(
                "Spotify limit exceeded for trackIds: max {SPOTIFY_TRACK_BATCH_LIMIT}, got {}",
                track_ids.len()
            ));
        }

        let token = self.get_access_token().await?;
        let joined_ids = track_ids.join(",");
        let ids = urlencoding::encode(&joined_ids);
        let url = format!("https://api.spotify.com/v1/me/tracks?ids={ids}");

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to remove saved tracks: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to remove saved tracks: {}",
                response.status()
            ))
        }
    }
}
