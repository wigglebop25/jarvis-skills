use std::time::Duration;

use serde::Deserialize;

use super::{QueueItem, SavedTrack, SpotifyClient, TrackInfo};

impl SpotifyClient {
    pub async fn get_recently_played(&self, limit: u32) -> Result<Vec<SavedTrack>, String> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://api.spotify.com/v1/me/player/recently-played?limit={}",
            limit.min(50)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get recently played: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to get recently played: {}",
                response.status()
            ));
        }

        #[derive(Deserialize)]
        struct RecentlyPlayedResponse {
            items: Vec<PlayHistoryItem>,
        }

        #[derive(Deserialize)]
        struct PlayHistoryItem {
            track: TrackObject,
            played_at: String,
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

        let data: RecentlyPlayedResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse recently played response: {}", e))?;

        let tracks = data
            .items
            .into_iter()
            .map(|item| SavedTrack {
                id: item.track.id,
                name: item.track.name,
                artist: item
                    .track
                    .artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                album: item.track.album.name,
                duration: item.track.duration_ms,
                saved_at: item.played_at,
                uri: item.track.uri,
            })
            .collect();

        Ok(tracks)
    }

    pub async fn get_user_saved_tracks(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SavedTrack>, String> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://api.spotify.com/v1/me/tracks?limit={}&offset={}",
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
            .map_err(|e| format!("Failed to get saved tracks: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to get saved tracks: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct SavedTracksResponse {
            items: Vec<SavedTrackItem>,
        }

        #[derive(Deserialize)]
        struct SavedTrackItem {
            track: TrackObject,
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

        let data: SavedTracksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse saved tracks response: {}", e))?;

        let tracks = data
            .items
            .into_iter()
            .map(|item| SavedTrack {
                id: item.track.id,
                name: item.track.name,
                artist: item
                    .track
                    .artists
                    .iter()
                    .map(|artist| artist.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
                album: item.track.album.name,
                duration: item.track.duration_ms,
                saved_at: item.added_at,
                uri: item.track.uri,
            })
            .collect();

        Ok(tracks)
    }

    pub async fn get_queue(&self, limit: u32) -> Result<QueueItem, String> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://api.spotify.com/v1/me/player/queue?limit={}",
            limit.min(100)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to get queue: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to get queue: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct QueueResponse {
            currently_playing: Option<TrackObject>,
            queue: Vec<TrackObject>,
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

        let data: QueueResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse queue response: {}", e))?;

        let currently_playing = data.currently_playing.map(|track| TrackInfo {
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
            duration_ms: Some(track.duration_ms),
        });

        let queue = data
            .queue
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
                duration_ms: Some(track.duration_ms),
            })
            .collect();

        Ok(QueueItem {
            currently_playing,
            queue,
        })
    }
}
