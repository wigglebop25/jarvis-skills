use regex::Regex;
use serde_json::{Map, Value};
use std::path::Path;
use crate::intent::{IntentType, RouteDecision};

pub fn map_params_to_args(
    intent_type: &IntentType,
    raw_params: &Map<String, Value>,
    text_lower: &str,
    confidence: f64,
) -> RouteDecision {
    let mut final_args = raw_params.clone();
    
    match intent_type {
        IntentType::VolumeControl => {
            if let Some(direction) = raw_params.get("direction").and_then(|v| v.as_str()) {
                final_args.insert("action".to_string(), Value::String(direction.to_string()));
                final_args.remove("direction");
            } else if raw_params.contains_key("level") {
                final_args.insert("action".to_string(), Value::String("set".to_string()));
            } else {
                final_args.insert("action".to_string(), Value::String("get".to_string()));
            }

            if let Some(level) = raw_params.get("level").and_then(|v| v.as_str()) {
                if let Ok(num) = level.parse::<i32>() {
                    final_args.insert("level".to_string(), Value::Number(num.into()));
                } else {
                    final_args.remove("level");
                }
            }
        }
        IntentType::MusicControl => {
            let mut final_tool = "playMusic".to_string();
            if let Some(action) = raw_params.get("action").and_then(|v| v.as_str()) {
                match action {
                    "pause" => final_tool = "pausePlayback".to_string(),
                    "next" => final_tool = "skipToNext".to_string(),
                    "previous" => final_tool = "skipToPrevious".to_string(),
                    "current" => final_tool = "getNowPlaying".to_string(),
                    "search" => final_tool = "searchSpotify".to_string(),
                    "queue" => final_tool = "addToQueue".to_string(),
                    _ => final_tool = "playMusic".to_string(),
                }
                final_args.remove("action");
            }
            let should_exec = confidence >= 0.85;
            return RouteDecision {
                intent: intent_type.as_str().to_string(),
                confidence,
                tool_name: Some(final_tool),
                arguments: final_args,
                should_execute: should_exec,
            };
        }
        IntentType::NetworkToggle => {
            if let Some(device) = raw_params.get("device").and_then(|v| v.as_str()) {
                final_args.insert("interface".to_string(), Value::String(device.to_string()));
                final_args.remove("device");
            }
            if let Some(state) = raw_params.get("state").and_then(|v| v.as_str()) {
                final_args.insert("enable".to_string(), Value::Bool(state == "on"));
                final_args.remove("state");
            }
        }
        IntentType::SystemInfo => {
            let mut include = Vec::new();
            if Regex::new(r"(?i)\b(cpu|processor)\b").unwrap().is_match(text_lower) {
                include.push(Value::String("cpu".to_string()));
            }
            if Regex::new(r"(?i)\b(ram|memory)\b").unwrap().is_match(text_lower) {
                include.push(Value::String("ram".to_string()));
            }
            if Regex::new(r"(?i)\b(storage|disk|drive|ssd|space)\b").unwrap().is_match(text_lower) {
                include.push(Value::String("storage".to_string()));
            }
            if Regex::new(r"(?i)\b(network|internet|wifi|wi-fi|connected|connection)\b").unwrap().is_match(text_lower) {
                include.push(Value::String("network".to_string()));
            }
            if !include.is_empty() {
                final_args.insert("include".to_string(), Value::Array(include));
            }
        }
        IntentType::DirectoryList => {
            let mut path_str = None;

            let drive_root_fn = |letter: &str, require_exists: bool| -> Option<String> {
                if cfg!(windows) {
                    let candidate = format!("{}:\\", letter.to_ascii_uppercase());
                    if !require_exists || Path::new(&candidate).exists() {
                        Some(candidate)
                    } else {
                        None
                    }
                } else {
                    let candidate = format!("/mnt/{}", letter.to_lowercase());
                    if Path::new(&candidate).exists() {
                        return Some(candidate);
                    }
                    let candidate = format!("/media/{}", letter.to_lowercase());
                    if Path::new(&candidate).exists() {
                        return Some(candidate);
                    }
                    None
                }
            };

            // Check for common aliases
            if Regex::new(r"(?i)\bdownloads?\b").unwrap().is_match(text_lower) {
                if let Some(p) = dirs::download_dir() {
                    path_str = Some(p.to_string_lossy().to_string());
                } else if let Some(home) = dirs::home_dir() {
                    path_str = Some(home.join("Downloads").to_string_lossy().to_string());
                }
            } else if Regex::new(r"(?i)\bdesktop\b").unwrap().is_match(text_lower) {
                if let Some(p) = dirs::desktop_dir() {
                    path_str = Some(p.to_string_lossy().to_string());
                } else if let Some(home) = dirs::home_dir() {
                    path_str = Some(home.join("Desktop").to_string_lossy().to_string());
                }
            } else if Regex::new(r"(?i)\bdocuments?\b").unwrap().is_match(text_lower) {
                if let Some(p) = dirs::document_dir() {
                    path_str = Some(p.to_string_lossy().to_string());
                } else if let Some(home) = dirs::home_dir() {
                    path_str = Some(home.join("Documents").to_string_lossy().to_string());
                }
            }

            // Explicit drive formats
            if path_str.is_none() {
                for pattern in &[r"(?i)\b([a-z])\s*:", r"(?i)\bdrive\s*([a-z])\b", r"(?i)\b([a-z])\s*drive\b"] {
                    if let Some(caps) = Regex::new(pattern).unwrap().captures(text_lower) {
                        if let Some(m) = caps.get(1) {
                            if let Some(drive) = drive_root_fn(m.as_str(), false) {
                                path_str = Some(drive);
                                break;
                            }
                        }
                    }
                }
            }

            if path_str.is_none() {
                if let Some(caps) = Regex::new(r"(?i)\b(?:inside|in)\s*(?:the\s*)?([a-z])\b").unwrap().captures(text_lower) {
                    if let Some(m) = caps.get(1) {
                        if let Some(drive) = drive_root_fn(m.as_str(), true) {
                            path_str = Some(drive);
                        }
                    }
                }
            }

            if path_str.is_none() {
                if let Some(caps) = Regex::new(r"(?i)\bi\s+the\s+([a-z])\b").unwrap().captures(text_lower) {
                    if let Some(m) = caps.get(1) {
                        if let Some(drive) = drive_root_fn(m.as_str(), true) {
                            path_str = Some(drive);
                        }
                    }
                }
            }

            if path_str.is_none() {
                if let Some(home) = dirs::home_dir() {
                    path_str = Some(home.to_string_lossy().to_string());
                } else {
                    path_str = Some(".".to_string());
                }
            }

            final_args.insert("path".to_string(), Value::String(path_str.unwrap()));
            
            let is_hidden = raw_params.get("include_hidden").and_then(|v| v.as_str()) == Some("true");
            final_args.insert("include_hidden".to_string(), Value::Bool(is_hidden));
            final_args.insert("max_entries".to_string(), Value::Number(200.into()));

            let has_folders = Regex::new(r"(?i)\bfolders?\b").unwrap().is_match(text_lower);
            let has_files = Regex::new(r"(?i)\bfiles?\b").unwrap().is_match(text_lower);

            final_args.insert("directories_only".to_string(), Value::Bool(has_folders && !has_files));
            final_args.insert("files_only".to_string(), Value::Bool(has_files && !has_folders));
        }
        IntentType::FileOrganization => {
            let folder_alias = raw_params.get("target_folder").and_then(|v| v.as_str()).unwrap_or("downloads");
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            
            let path_val = match folder_alias {
                "downloads" => home.join("Downloads"),
                "desktop" => home.join("Desktop"),
                "documents" => home.join("Documents"),
                _ => home.join("Downloads"),
            };
            final_args.insert("path".to_string(), Value::String(path_val.to_string_lossy().to_string()));
            
            if !final_args.contains_key("strategy") {
                final_args.insert("strategy".to_string(), Value::String("extension".to_string()));
            }

            let is_dry = raw_params.get("dry_run").and_then(|v| v.as_str()) != Some("false");
            final_args.insert("dry_run".to_string(), Value::Bool(is_dry));
        }
        _ => {}
    }

    let should_execute = confidence >= 0.85 && intent_type.tool_name().is_some();

    RouteDecision {
        intent: intent_type.as_str().to_string(),
        confidence,
        tool_name: intent_type.tool_name().map(|s| s.to_string()),
        arguments: final_args,
        should_execute,
    }
}
