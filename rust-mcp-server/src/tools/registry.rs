use serde_json::{json, Value};

use super::registry_spotify;

pub fn tool_definitions() -> Vec<Value> {
    let mut tools = vec![
        tool(
            "resolve_path",
            "Resolve user-friendly path names to full system paths. Use this FIRST when user mentions downloads, documents, etc.",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "enum": ["downloads","documents","desktop","home","project"], "description": "User-friendly name to resolve"}
                },
                "required": ["name"]
            }),
        ),
        tool(
            "get_system_info",
            "Get system information including CPU usage, RAM, storage, and network status",
            json!({
                "type": "object",
                "properties": {
                    "include": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Sections to include: cpu, ram, storage, network"
                    }
                }
            }),
        ),
        tool(
            "control_volume",
            "Control system audio volume (get, set, up, down, mute, unmute)",
            json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["get","set","up","down","mute","unmute"]},
                    "level": {"type": "integer"},
                    "step": {"type": "integer"}
                },
                "required": ["action"]
            }),
        ),
    ];

    tools.extend(registry_spotify::spotify_tool_definitions());

    tools.extend(vec![
        tool(
            "toggle_network",
            "Toggle network interfaces (WiFi, Bluetooth, Ethernet) on/off",
            json!({
                "type": "object",
                "properties": {
                    "interface": {"type": "string", "enum": ["wifi","bluetooth","ethernet"]},
                    "enable": {"type": "boolean"}
                },
                "required": ["interface","enable"]
            }),
        ),
        tool(
            "control_bluetooth_device",
            "List/connect/disconnect a specific Bluetooth device by name or instance ID (best-effort via device control)",
            json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["list","connect","disconnect"]},
                    "device_name": {"type": "string", "description": "Partial match against Bluetooth friendly name"},
                    "instance_id": {"type": "string", "description": "Exact PnP instance ID"},
                    "include_system": {"type": "boolean", "description": "Include enumerators/transports/system entries in list/search"}
                },
                "required": ["action"]
            }),
        ),
        tool(
            "list_directory",
            "List directory contents (files/folders) for an allowlisted path",
            json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "include_hidden": {"type": "boolean"},
                    "max_entries": {"type": "integer"},
                    "directories_only": {"type": "boolean"},
                    "files_only": {"type": "boolean"}
                },
                "required": ["path"]
            }),
        ),
        tool(
            "organize_folder",
            "Organize files in a folder by extension/type/date within an allowlisted scope (non-destructive move/rename only, no delete)",
            json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "strategy": {"type": "string", "enum": ["extension","type","date"]},
                    "recursive": {"type": "boolean"},
                    "dry_run": {"type": "boolean"},
                    "include_hidden": {"type": "boolean"},
                    "execute_plan": {"type": "boolean"}
                },
                "required": ["path"]
            }),
        ),
    ]);

    tools
}

pub fn mcp_tool_definitions() -> Vec<Value> {
    tool_definitions()
        .into_iter()
        .filter_map(|tool| {
            let function = tool.get("function")?;
            let name = function.get("name")?.clone();
            let description = function
                .get("description")
                .cloned()
                .unwrap_or_else(|| json!(""));
            let input_schema = function
                .get("parameters")
                .cloned()
                .unwrap_or_else(|| json!({"type":"object","properties":{}}));

            Some(json!({
                "name": name,
                "description": description,
                "inputSchema": input_schema
            }))
        })
        .collect()
}

fn tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": parameters
        }
    })
}
