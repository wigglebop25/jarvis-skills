use jarvis_rust_mcp_server::{tools, AppState};

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
use tracing::info;

mod rpc;

use rpc::{handle_jsonrpc, run_stdio, RpcMode};

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    let _ = dotenv::dotenv();

    let stdio_mode = env::args().any(|a| a == "--stdio");
    init_tracing(stdio_mode);

    let host = env::var("RUST_MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("RUST_MCP_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(5050);

    let client_id = env::var("SPOTIPY_CLIENT_ID").ok();
    let client_secret = env::var("SPOTIPY_CLIENT_SECRET").ok();
    let spotify = if let (Some(id), Some(secret)) = (client_id, client_secret) {
        Some(std::sync::Arc::new(jarvis_rust_mcp_server::tools::spotify_api::SpotifyClient::new(id, secret.to_string())))
    } else {
        None
    };

    let state = AppState {
        http: Client::new(),
        spotify,
    };

    if stdio_mode {
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
