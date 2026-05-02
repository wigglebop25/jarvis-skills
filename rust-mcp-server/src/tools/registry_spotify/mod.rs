use serde_json::Value;

mod definitions;

pub fn spotify_tool_definitions() -> Vec<Value> {
    let mut tools = Vec::new();
    
    tools.extend(definitions::auth_tool_definitions());
    tools.extend(definitions::read_tool_definitions());
    tools.extend(definitions::library_tool_definitions());
    tools.extend(definitions::playback_tool_definitions());
    tools.extend(definitions::write_tool_definitions());
    
    tools
}
