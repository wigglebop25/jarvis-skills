// Library exports for JARVIS Rust MCP Server
// This module exposes the public API for testing and integration

pub mod intent;
pub mod tools;

use reqwest::Client;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub http: Client,
}

// Re-exports for convenience
pub use intent::route_intent;
pub use tools::{execute_tool, tool_definitions};
