use serde_json::{json, Map, Value};

use super::shell::run_command;

pub fn control_volume(args: &Map<String, Value>) -> Result<Value, String> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: action".to_string())?;
    let level = args.get("level").and_then(Value::as_i64).unwrap_or(50) as i32;
    let step = args.get("step").and_then(Value::as_i64).unwrap_or(10) as i32;

    #[cfg(target_os = "windows")]
    {
        control_volume_windows(action, level, step)
    }
    #[cfg(target_os = "macos")]
    {
        control_volume_macos(action, level, step)
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        control_volume_linux(action, level, step)
    }
}

#[cfg(target_os = "windows")]
fn control_volume_windows(action: &str, level: i32, step: i32) -> Result<Value, String> {
    let nircmd = super::shell::find_nircmd()?;

    match action {
        "get" => {
            let raw = run_command(&nircmd, &["getsysvolume"])?;
            let parsed = raw.trim().parse::<f64>().unwrap_or(0.0);
            let pct = ((parsed / 65535.0) * 100.0).round() as i32;
            Ok(json!({"action":"get","level": clamp_0_100(pct)}))
        }
        "set" => {
            let clamped = clamp_0_100(level);
            let raw = ((clamped as f64 / 100.0) * 65535.0).round() as i32;
            let _ = run_command(&nircmd, &["setsysvolume", &raw.to_string()])?;
            Ok(json!({"action":"set","level": clamped, "success": true}))
        }
        "up" | "down" => {
            let raw = run_command(&nircmd, &["getsysvolume"])?;
            let parsed = raw.trim().parse::<f64>().unwrap_or(0.0);
            let mut pct = ((parsed / 65535.0) * 100.0).round() as i32;
            if action == "up" {
                pct += step;
            } else {
                pct -= step;
            }
            let clamped = clamp_0_100(pct);
            let raw_target = ((clamped as f64 / 100.0) * 65535.0).round() as i32;
            let _ = run_command(&nircmd, &["setsysvolume", &raw_target.to_string()])?;
            Ok(json!({"action":action,"level": clamped, "success": true}))
        }
        "mute" => {
            let _ = run_command(&nircmd, &["mutesysvolume", "1"])?;
            Ok(json!({"action":"mute","muted": true, "success": true}))
        }
        "unmute" => {
            let _ = run_command(&nircmd, &["mutesysvolume", "0"])?;
            Ok(json!({"action":"unmute","muted": false, "success": true}))
        }
        _ => Err(format!("Unsupported volume action: {action}")),
    }
}

#[cfg(target_os = "macos")]
fn control_volume_macos(action: &str, level: i32, step: i32) -> Result<Value, String> {
    match action {
        "get" => {
            let out = run_command("osascript", &["-e", "output volume of (get volume settings)"])?;
            let pct = out.trim().parse::<i32>().unwrap_or(0);
            Ok(json!({"action":"get","level": clamp_0_100(pct)}))
        }
        "set" => {
            let clamped = clamp_0_100(level);
            let _ = run_command("osascript", &["-e", &format!("set volume output volume {clamped}")])?;
            Ok(json!({"action":"set","level":clamped,"success":true}))
        }
        "up" | "down" => {
            let out = run_command("osascript", &["-e", "output volume of (get volume settings)"])?;
            let mut pct = out.trim().parse::<i32>().unwrap_or(0);
            if action == "up" {
                pct += step;
            } else {
                pct -= step;
            }
            let clamped = clamp_0_100(pct);
            let _ = run_command("osascript", &["-e", &format!("set volume output volume {clamped}")])?;
            Ok(json!({"action":action,"level":clamped,"success":true}))
        }
        "mute" => {
            let _ = run_command("osascript", &["-e", "set volume with output muted true"])?;
            Ok(json!({"action":"mute","muted":true,"success":true}))
        }
        "unmute" => {
            let _ = run_command("osascript", &["-e", "set volume with output muted false"])?;
            Ok(json!({"action":"unmute","muted":false,"success":true}))
        }
        _ => Err(format!("Unsupported volume action: {action}")),
    }
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn control_volume_linux(action: &str, level: i32, step: i32) -> Result<Value, String> {
    match action {
        "get" => {
            let out = run_command("pactl", &["get-sink-volume", "@DEFAULT_SINK@"])?;
            let pct = parse_percent_from_text(&out).unwrap_or(0);
            Ok(json!({"action":"get","level":pct}))
        }
        "set" => {
            let clamped = clamp_0_100(level);
            let _ = run_command("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &format!("{clamped}%")])?;
            Ok(json!({"action":"set","level":clamped,"success":true}))
        }
        "up" => {
            let _ = run_command("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &format!("+{step}%")])?;
            Ok(json!({"action":"up","success":true}))
        }
        "down" => {
            let _ = run_command("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &format!("-{step}%")])?;
            Ok(json!({"action":"down","success":true}))
        }
        "mute" => {
            let _ = run_command("pactl", &["set-sink-mute", "@DEFAULT_SINK@", "1"])?;
            Ok(json!({"action":"mute","muted":true,"success":true}))
        }
        "unmute" => {
            let _ = run_command("pactl", &["set-sink-mute", "@DEFAULT_SINK@", "0"])?;
            Ok(json!({"action":"unmute","muted":false,"success":true}))
        }
        _ => Err(format!("Unsupported volume action: {action}")),
    }
}

fn clamp_0_100(value: i32) -> i32 {
    value.clamp(0, 100)
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn parse_percent_from_text(input: &str) -> Option<i32> {
    for token in input.split_whitespace() {
        if let Some(stripped) = token.strip_suffix('%') {
            if let Ok(v) = stripped.trim().parse::<i32>() {
                return Some(clamp_0_100(v));
            }
        }
    }
    None
}
