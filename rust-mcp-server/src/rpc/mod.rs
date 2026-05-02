use serde_json::{json, Value};
use tracing::{warn};
use crate::{AppState};
use jarvis_rust_mcp_server::intent;
use jarvis_rust_mcp_server::tools;

pub mod stdio;
pub use stdio::run_stdio;

#[derive(Copy, Clone)]
pub enum RpcMode {
    HttpCompat,
    StdioMcp,
}

pub async fn handle_jsonrpc(payload: &Value, state: &AppState, mode: RpcMode) -> Value {
    let id = payload.get("id").cloned().unwrap_or(Value::Null);
    let method = payload
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = payload
        .get("params")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    match method {
        "initialize" => {
            let negotiated_protocol = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or("2024-11-05");
            jsonrpc_success(
                id,
                json!({
                    "protocolVersion": negotiated_protocol,
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "jarvis-rust-mcp-server",
                        "version": "0.1.0"
                    }
                }),
            )
        }
        "notifications/initialized" => jsonrpc_success(id, json!({})),
        "ping" => jsonrpc_success(id, json!({})),
        "tools/list" => match mode {
            RpcMode::HttpCompat => jsonrpc_success(id, json!({ "tools": tools::tool_definitions() })),
            RpcMode::StdioMcp => jsonrpc_success(id, json!({ "tools": tools::mcp_tool_definitions() })),
        },
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str);
            let arguments = params
                .get("arguments")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();

            match name {
                Some(tool_name) => match tools::execute_tool(tool_name, arguments, state).await {
                    Ok(result) => match mode {
                        RpcMode::HttpCompat => jsonrpc_success(id, result),
                        RpcMode::StdioMcp => jsonrpc_success(id, mcp_tool_result_ok(result)),
                    },
                    Err(message) => match mode {
                        RpcMode::HttpCompat => jsonrpc_error(id, -32000, &message),
                        RpcMode::StdioMcp => jsonrpc_success(id, mcp_tool_result_err(&message)),
                    },
                },
                None => jsonrpc_error(id, -32602, "Missing tool name"),
            }
        }
        "jarvis/route" => {
            let text = params.get("text").and_then(Value::as_str).unwrap_or("");
            let decision = intent::route_intent(text);
            jsonrpc_success(id, json!({
                "intent": decision.intent,
                "confidence": decision.confidence,
                "tool_name": decision.tool_name,
                "arguments": decision.arguments,
                "should_execute": decision.should_execute,
            }))
        }
        "jarvis/route_and_call" => {
            let text = params.get("text").and_then(Value::as_str).unwrap_or("");
            let decision = intent::route_intent(text);
            let mut result = json!({
                "intent": decision.intent,
                "confidence": decision.confidence,
                "tool_name": decision.tool_name,
                "arguments": decision.arguments,
                "should_execute": decision.should_execute,
            });

            if decision.should_execute {
                if let Some(tool_name) = &decision.tool_name {
                    match tools::execute_tool(tool_name, decision.arguments.clone(), state).await {
                        Ok(res) => {
                            result["execution_result"] = res;
                        }
                        Err(err) => {
                            result["execution_error"] = json!(err);
                        }
                    }
                }
            }

            jsonrpc_success(id, result)
        }
        _ => {
            warn!("Unknown JSON-RPC method received: {method}");
            jsonrpc_error(id, -32601, &format!("Unknown method: {method}"))
        }
    }
}

pub fn jsonrpc_success(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

pub fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn mcp_tool_result_ok(result: Value) -> Value {
    let text = serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
    json!({
        "content": [{"type": "text", "text": text}],
        "isError": false
    })
}

fn mcp_tool_result_err(message: &str) -> Value {
    json!({
        "content": [{"type": "text", "text": message}],
        "isError": true
    })
}
