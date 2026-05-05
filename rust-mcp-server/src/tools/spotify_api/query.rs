use std::time::Duration;

use serde::Deserialize;

use super::{DeviceInfo, SearchResult, SpotifyClient, TrackInfo, PlaylistInfo, AlbumInfo};

impl SpotifyClient {
    pub async fn search(
        &self,
        query: &str,
        search_type: &str,
        limit: u32,
    ) -> Result<SearchResult, String> {
        let token = self.get_access_token().await?;

        let url = format!(
            "https://api.spotify.com/v1/search?q={}&type={}&limit={}",
            urlencoding::encode(query),
            search_type,
            limit.min(50)
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
            playlists: Option<PlaylistsObject>,
            albums: Option<AlbumsObject>,
            _artists: Option<ArtistsObject>,
        }

        #[derive(Deserialize)]
        struct TracksObject {
            items: Vec<TrackObject>,
        }

        #[derive(Deserialize)]
        struct PlaylistsObject {
            items: Vec<PlaylistObject>,
        }

        #[derive(Deserialize)]
        struct AlbumsObject {
            items: Vec<AlbumObject>,
        }

        #[derive(Deserialize)]
        struct ArtistsObject {
            _items: Vec<ArtistObject>,
        }

        #[derive(Deserialize)]
        struct TrackObject {
            _id: String,
            name: String,
            artists: Vec<ArtistMinimal>,
            album: AlbumMinimal,
            uri: String,
        }

        #[derive(Deserialize)]
        struct PlaylistObject {
            id: String,
            name: String,
            description: Option<String>,
            uri: String,
            images: Option<Vec<super::PlaylistImage>>,
        }

        #[derive(Deserialize)]
        struct AlbumObject {
            id: String,
            name: String,
            artists: Vec<ArtistMinimal>,
            uri: String,
            release_date: String,
            album_type: String,
            total_tracks: u32,
        }

        #[derive(Deserialize)]
        struct ArtistObject {
            _id: String,
            _name: String,
            _uri: String,
            _genres: Option<Vec<String>>,
        }

        #[derive(Deserialize)]
        struct ArtistMinimal {
            name: String,
        }

        #[derive(Deserialize)]
        struct AlbumMinimal {
            name: String,
        }

        let search_data: SearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse search response: {}", e))?;

        let mut result = SearchResult {
            tracks: None,
            playlists: None,
            albums: None,
            artists: None,
        };

        if let Some(tracks) = search_data.tracks {
            result.tracks = Some(
                tracks
                    .items
                    .into_iter()
                    .map(|track| TrackInfo {
                        name: track.name,
                        artist: track
                            .artists
                            .iter()
                            .map(|a| a.name.clone())
                            .collect::<Vec<_>>()
                            .join(", "),
                        album: track.album.name,
                        uri: track.uri,
                        progress_ms: None,
                        duration_ms: None,
                    })
                    .collect(),
            );
        }

        if let Some(playlists) = search_data.playlists {
            result.playlists = Some(
                playlists
                    .items
                    .into_iter()
                    .map(|p| PlaylistInfo {
                        id: p.id,
                        name: p.name,
                        uri: p.uri,
                        description: p.description.unwrap_or_default(),
                        images: p.images,
                    })
                    .collect(),
            );
        }

        if let Some(albums) = search_data.albums {
            result.albums = Some(
                albums
                    .items
                    .into_iter()
                    .map(|a| AlbumInfo {
                        id: a.id,
                        name: a.name,
                        artists: a.artists.into_iter().map(|ar| ar.name).collect(),
                        release_date: a.release_date,
                        album_type: a.album_type,
                        total_tracks: a.total_tracks,
                        uri: a.uri,
                    })
                    .collect(),
            );
        }

        Ok(result)
    }

    pub async fn search_tracks(&self, query: &str, limit: u32) -> Result<Vec<TrackInfo>, String> {
        let result = self.search(query, "track", limit).await?;
        Ok(result.tracks.unwrap_or_default())
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
