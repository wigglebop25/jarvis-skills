use serde_json::{Map, Value};

use crate::AppState;

mod network;
mod bluetooth_device;
mod directory_listing;
mod organizer;
mod path_security;
mod file_collector;
mod categorizer;
mod collision_handler;
mod registry;
mod registry_spotify;
mod shell;
mod spotify;
pub mod spotify_api;
mod spotify_auth;
mod system;
mod volume;
mod resolve_path;

pub use registry::{mcp_tool_definitions, tool_definitions};

pub async fn execute_tool(
    name: &str,
    args: Map<String, Value>,
    state: &AppState,
) -> Result<Value, String> {
    // Spotify tools are handled by a single dispatcher that needs the tool name
    if is_spotify_tool(name) {
        return spotify::handle_spotify_tool(name, &args, state).await;
    }

    match name {
        "resolve_path" => resolve_path::resolve_path(&args, state).await,
        "get_system_info" => system::get_system_info(&args, state).await,
        "control_volume" => volume::control_volume(&args, state).await,
        "toggle_network" => network::toggle_network(&args, state).await,
        "control_bluetooth_device" => bluetooth_device::control_bluetooth_device(&args, state).await,
        "list_directory" => directory_listing::list_directory(&args, state).await,
        "organize_folder" => organizer::organize_folder(&args, state).await,
        _ => Err(format!("Unknown tool: {name}")),
    }
}

fn is_spotify_tool(name: &str) -> bool {
    matches!(
        name,
        "authorizeSpotify"
            | "checkSpotifyAuth"
            | "logoutSpotify"
            | "searchSpotify"
            | "getNowPlaying"
            | "playMusic"
            | "pausePlayback"
            | "resumePlayback"
            | "skipToNext"
            | "skipToPrevious"
            | "setVolume"
            | "adjustVolume"
            | "addToQueue"
            | "getAvailableDevices"
            | "getMyPlaylists"
            | "getPlaylistTracks"
            | "getRecentlyPlayed"
            | "getUsersSavedTracks"
            | "getQueue"
            | "getPlaylist"
            | "getAlbumTracks"
            | "getAlbums"
            | "checkUsersSavedAlbums"
            | "createPlaylist"
            | "addTracksToPlaylist"
            | "updatePlaylist"
            | "removeTracksFromPlaylist"
            | "reorderPlaylistItems"
            | "saveOrRemoveAlbumForUser"
            | "removeUsersSavedTracks"
    )
}
