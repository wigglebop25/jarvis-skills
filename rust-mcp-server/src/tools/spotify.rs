use serde_json::{json, Map, Value};

use super::shell::run_command;

pub fn control_spotify(args: &Map<String, Value>) -> Result<Value, String> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: action".to_string())?;

    #[cfg(target_os = "windows")]
    {
        let nircmd = super::shell::find_nircmd()?;
        match action {
            "play" | "pause" => {
                let _ = run_command(&nircmd, &["sendkeypress", "0xB3"])?;
                Ok(json!({"action":action,"success":true,"mode":"media_key"}))
            }
            "next" => {
                let _ = run_command(&nircmd, &["sendkeypress", "0xB0"])?;
                Ok(json!({"action":"next","success":true,"mode":"media_key"}))
            }
            "previous" => {
                let _ = run_command(&nircmd, &["sendkeypress", "0xB1"])?;
                Ok(json!({"action":"previous","success":true,"mode":"media_key"}))
            }
            "current" => Ok(json!({"error":"Current track metadata requires Spotify API integration in Rust path","action":"current"})),
            "search" => Ok(json!({"error":"Search requires Spotify API integration in Rust path","action":"search"})),
            _ => Err(format!("Unsupported spotify action: {action}")),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        match action {
            "play" | "pause" => {
                let _ = run_command("playerctl", &["play-pause"])?;
                Ok(json!({"action":action,"success":true,"mode":"playerctl"}))
            }
            "next" => {
                let _ = run_command("playerctl", &["next"])?;
                Ok(json!({"action":"next","success":true,"mode":"playerctl"}))
            }
            "previous" => {
                let _ = run_command("playerctl", &["previous"])?;
                Ok(json!({"action":"previous","success":true,"mode":"playerctl"}))
            }
            "current" => {
                let out = run_command("playerctl", &["metadata", "--format", "{{artist}} - {{title}}"])?;
                Ok(json!({"action":"current","track":out.trim()}))
            }
            "search" => Ok(json!({"error":"Search requires Spotify API integration in Rust path","action":"search"})),
            _ => Err(format!("Unsupported spotify action: {action}")),
        }
    }
}
