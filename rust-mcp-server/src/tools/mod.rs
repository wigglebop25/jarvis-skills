use serde_json::{Map, Value};

use crate::AppState;

mod network;
mod bluetooth_device;
mod directory_listing;
mod organizer;
mod registry;
mod screen_description;
mod screen_recording;
mod shell;
mod spotify;
mod system;
mod translation;
mod volume;

pub use registry::{mcp_tool_definitions, tool_definitions};

pub async fn execute_tool(
    name: &str,
    args: Map<String, Value>,
    state: &AppState,
) -> Result<Value, String> {
    match name {
        "get_system_info" => system::get_system_info(&args),
        "control_volume" => volume::control_volume(&args),
        "control_spotify" => spotify::control_spotify(&args),
        "toggle_network" => network::toggle_network(&args),
        "control_bluetooth_device" => bluetooth_device::control_bluetooth_device(&args),
        "list_directory" => directory_listing::list_directory(&args),
        "organize_folder" => organizer::organize_folder(&args),
        "control_screen_recording" => screen_recording::control_screen_recording(&args),
        "translate_text" => translation::translate_text(&args, state).await,
        "describe_screen" => screen_description::describe_screen(&args),
        _ => Err(format!("Unknown tool: {name}")),
    }
}
