use serde_json::{Map, Value};

use crate::AppState;

mod bluetooth_device;
mod categorizer;
mod collision_handler;
mod directory_listing;
mod file_collector;
mod network;
mod organizer;
mod path_security;
mod registry;
mod resolve_path;
mod shell;
mod system;

pub use registry::tool_definitions;

pub async fn execute_tool(
    name: &str,
    args: Map<String, Value>,
    state: &AppState,
) -> Result<Value, String> {
    match name {
        "resolve_path" => resolve_path::resolve_path(&args, state).await,
        "get_system_info" => system::get_system_info(&args, state).await,
        "toggle_network" => network::toggle_network(&args, state).await,
        "control_bluetooth_device" => {
            bluetooth_device::control_bluetooth_device(&args, state).await
        }
        "list_directory" => directory_listing::list_directory(&args, state).await,
        "organize_folder" => organizer::organize_folder(&args, state).await,
        _ => Err(format!("Unknown tool: {name}")),
    }
}
