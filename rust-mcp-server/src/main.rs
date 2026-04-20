mod tools;

use std::{env, net::SocketAddr};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::{info, warn};

#[derive(Clone)]
pub struct AppState {
    pub http: Client,
}

#[derive(Copy, Clone)]
enum RpcMode {
    HttpCompat,
    StdioMcp,
}

#[tokio::main]
async fn main() {
    let stdio_mode = env::args().any(|a| a == "--stdio");
    init_tracing(stdio_mode);

    let host = env::var("RUST_MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("RUST_MCP_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(5050);

    let state = AppState {
        http: Client::new(),
    };

    if stdio_mode {
        info!("Starting JARVIS Rust MCP server in stdio mode");
        if let Err(err) = run_stdio(state).await {
            eprintln!("stdio server failed: {err}");
            std::process::exit(1);
        }
        return;
    }

    let app = Router::new()
        .route("/", get(root_info).post(jsonrpc))
        .route("/health", get(health))
        .route("/tools", get(list_tools_http))
        .route("/jsonrpc", post(jsonrpc))
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("invalid bind address");
    info!("Starting JARVIS Rust MCP server on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind socket");
    axum::serve(listener, app).await.expect("server failed");
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "server": "jarvis-rust-mcp-server"
        })),
    )
}

async fn root_info() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "server": "jarvis-rust-mcp-server",
            "endpoints": ["/health", "/tools", "/jsonrpc"]
        })),
    )
}

async fn list_tools_http() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "tools": tools::tool_definitions() })))
}

async fn jsonrpc(State(state): State<AppState>, Json(payload): Json<Value>) -> impl IntoResponse {
    let response = handle_jsonrpc(&payload, &state, RpcMode::HttpCompat).await;
    (StatusCode::OK, Json(response))
}

async fn handle_jsonrpc(payload: &Value, state: &AppState, mode: RpcMode) -> Value {
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
        _ => {
            warn!("Unknown JSON-RPC method received: {method}");
            jsonrpc_error(id, -32601, &format!("Unknown method: {method}"))
        }
    }
}

async fn run_stdio(state: AppState) -> Result<(), String> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = io::stdout();
    let mut line = String::new();

    loop {
        let mut content_length: Option<usize> = None;
        let mut inline_payload: Option<Value> = None;

        loop {
            line.clear();
            let read = reader
                .read_line(&mut line)
                .await
                .map_err(|e| format!("Failed reading stdio header: {e}"))?;

            if read == 0 {
                return Ok(());
            }

            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }

            let lower = trimmed.to_ascii_lowercase();
            if let Some(value) = lower.strip_prefix("content-length:") {
                let parsed = value
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| format!("Invalid Content-Length header: {e}"))?;
                content_length = Some(parsed);
                continue;
            }

            // Compatibility fallback for clients that send line-delimited JSON-RPC
            // instead of Content-Length framed stdio messages.
            if trimmed.starts_with('{') {
                match serde_json::from_str::<Value>(trimmed) {
                    Ok(payload) => {
                        inline_payload = Some(payload);
                        break;
                    }
                    Err(e) => {
                        let err = jsonrpc_error(Value::Null, -32700, &format!("Parse error: {e}"));
                        write_stdio_message(&mut stdout, &err).await?;
                        inline_payload = None;
                        break;
                    }
                }
            }
        }

        if let Some(payload) = inline_payload {
            handle_stdio_payload(&mut stdout, &state, payload, false).await?;
            continue;
        }

        let Some(length) = content_length else {
            continue;
        };

        let mut body = vec![0u8; length];
        reader
            .read_exact(&mut body)
            .await
            .map_err(|e| format!("Failed reading stdio body: {e}"))?;

        let payload: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(e) => {
                let err = jsonrpc_error(Value::Null, -32700, &format!("Parse error: {e}"));
                write_stdio_message(&mut stdout, &err).await?;
                continue;
            }
        };

        handle_stdio_payload(&mut stdout, &state, payload, true).await?;
    }
}

async fn handle_stdio_payload(
    stdout: &mut io::Stdout,
    state: &AppState,
    payload: Value,
    use_content_length_framing: bool,
) -> Result<(), String> {
    let response = handle_jsonrpc(&payload, state, RpcMode::StdioMcp).await;
    if payload.get("id").is_some() {
        if use_content_length_framing {
            write_stdio_message(stdout, &response).await?;
        } else {
            write_stdio_jsonline(stdout, &response).await?;
        }
    }
    Ok(())
}

async fn write_stdio_message(stdout: &mut io::Stdout, payload: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(payload).map_err(|e| format!("Failed to encode JSON: {e}"))?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());

    stdout
        .write_all(header.as_bytes())
        .await
        .map_err(|e| format!("Failed writing stdio header: {e}"))?;
    stdout
        .write_all(&body)
        .await
        .map_err(|e| format!("Failed writing stdio body: {e}"))?;
    stdout
        .flush()
        .await
        .map_err(|e| format!("Failed flushing stdio output: {e}"))?;
    Ok(())
}

async fn write_stdio_jsonline(stdout: &mut io::Stdout, payload: &Value) -> Result<(), String> {
    let mut body = serde_json::to_vec(payload).map_err(|e| format!("Failed to encode JSON: {e}"))?;
    body.push(b'\n');

    stdout
        .write_all(&body)
        .await
        .map_err(|e| format!("Failed writing stdio JSON line: {e}"))?;
    stdout
        .flush()
        .await
        .map_err(|e| format!("Failed flushing stdio output: {e}"))?;
    Ok(())
}

fn jsonrpc_success(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
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
        "structuredContent": result,
        "isError": false
    })
}

fn mcp_tool_result_err(message: &str) -> Value {
    json!({
        "content": [{"type": "text", "text": message}],
        "isError": true
    })
}

fn init_tracing(stdio_mode: bool) {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    if stdio_mode {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .compact()
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .compact()
            .init();
    }
}
