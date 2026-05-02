use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistInfo {
    pub id: String,
    pub name: String,
    pub uri: String,
    #[serde(default)]
    pub description: String,
    pub images: Option<Vec<PlaylistImage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistImage {
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: u32,
    pub added_at: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTrack {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: u32,
    pub saved_at: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumInfo {
    pub id: String,
    pub name: String,
    pub artists: Vec<String>,
    pub release_date: String,
    pub album_type: String,
    pub total_tracks: u32,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub currently_playing: Option<TrackInfo>,
    pub queue: Vec<TrackInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    #[serde(default)]
    pub volume_percent: Option<u8>,
    #[serde(rename = "type")]
    pub device_type: String,
}
