use serde_json::{json, Value};

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

pub fn tool_definitions() -> Vec<Value> {
    vec![
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
                        "type": "string",
                        "description": "Comma-separated sections to include: cpu, ram, storage, network"
                    }
                }
            }),
        ),
    ]
}
