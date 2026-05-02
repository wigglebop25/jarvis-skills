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
    match name {
        "resolve_path" => resolve_path::resolve_path(&args),
        "get_system_info" => system::get_system_info(&args),
        "control_volume" => volume::control_volume(&args),
        "authorizeSpotify" => spotify::handle_spotify_tool("authorizeSpotify", &args, state).await,
        "checkSpotifyAuth" => spotify::handle_spotify_tool("checkSpotifyAuth", &args, state).await,
        "searchSpotify" => spotify::handle_spotify_tool("searchSpotify", &args, state).await,
        "getNowPlaying" => spotify::handle_spotify_tool("getNowPlaying", &args, state).await,
        "playMusic" => spotify::handle_spotify_tool("playMusic", &args, state).await,
        "pausePlayback" => spotify::handle_spotify_tool("pausePlayback", &args, state).await,
        "resumePlayback" => spotify::handle_spotify_tool("resumePlayback", &args, state).await,
        "skipToNext" => spotify::handle_spotify_tool("skipToNext", &args, state).await,
        "skipToPrevious" => spotify::handle_spotify_tool("skipToPrevious", &args, state).await,
        "setVolume" => spotify::handle_spotify_tool("setVolume", &args, state).await,
        "adjustVolume" => spotify::handle_spotify_tool("adjustVolume", &args, state).await,
        "addToQueue" => spotify::handle_spotify_tool("addToQueue", &args, state).await,
        "getAvailableDevices" => spotify::handle_spotify_tool("getAvailableDevices", &args, state).await,
        "getMyPlaylists" => spotify::handle_spotify_tool("getMyPlaylists", &args, state).await,
        "getPlaylistTracks" => spotify::handle_spotify_tool("getPlaylistTracks", &args, state).await,
        "getRecentlyPlayed" => spotify::handle_spotify_tool("getRecentlyPlayed", &args, state).await,
        "getUsersSavedTracks" => spotify::handle_spotify_tool("getUsersSavedTracks", &args, state).await,
        "getQueue" => spotify::handle_spotify_tool("getQueue", &args, state).await,
        "getPlaylist" => spotify::handle_spotify_tool("getPlaylist", &args, state).await,
        "getAlbumTracks" => spotify::handle_spotify_tool("getAlbumTracks", &args, state).await,
        "getAlbums" => spotify::handle_spotify_tool("getAlbums", &args, state).await,
        "checkUsersSavedAlbums" => spotify::handle_spotify_tool("checkUsersSavedAlbums", &args, state).await,
        "createPlaylist" => spotify::handle_spotify_tool("createPlaylist", &args, state).await,
        "addTracksToPlaylist" => spotify::handle_spotify_tool("addTracksToPlaylist", &args, state).await,
        "updatePlaylist" => spotify::handle_spotify_tool("updatePlaylist", &args, state).await,
        "removeTracksFromPlaylist" => spotify::handle_spotify_tool("removeTracksFromPlaylist", &args, state).await,
        "reorderPlaylistItems" => spotify::handle_spotify_tool("reorderPlaylistItems", &args, state).await,
        "saveOrRemoveAlbumForUser" => spotify::handle_spotify_tool("saveOrRemoveAlbumForUser", &args, state).await,
        "removeUsersSavedTracks" => spotify::handle_spotify_tool("removeUsersSavedTracks", &args, state).await,
        "toggle_network" => network::toggle_network(&args),
        "control_bluetooth_device" => bluetooth_device::control_bluetooth_device(&args),
        "list_directory" => directory_listing::list_directory(&args),
        "organize_folder" => organizer::organize_folder(&args),
        _ => Err(format!("Unknown tool: {name}")),
    }
}
