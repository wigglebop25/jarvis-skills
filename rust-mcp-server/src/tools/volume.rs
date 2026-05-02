use serde_json::{json, Map, Value};

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
    #[cfg(not(target_os = "windows"))]
    {
        control_volume_linux(action, level, step)
    }
}

#[cfg(target_os = "windows")]
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
};
#[cfg(target_os = "windows")]
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};

#[cfg(target_os = "windows")]
fn get_endpoint_volume() -> Result<IAudioEndpointVolume, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(
            &MMDeviceEnumerator,
            None,
            CLSCTX_ALL,
        ).map_err(|e| format!("CoCreateInstance failed: {}", e))?;

        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .map_err(|e| format!("GetDefaultAudioEndpoint failed: {}", e))?;

        let endpoint_volume: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|e| format!("Activate failed: {}", e))?;

        Ok(endpoint_volume)
    }
}

#[cfg(target_os = "windows")]
fn control_volume_windows(action: &str, level: i32, step: i32) -> Result<Value, String> {
    let endpoint_volume = get_endpoint_volume()?;

    unsafe {
        match action {
            "get" => {
                let current_level = endpoint_volume.GetMasterVolumeLevelScalar()
                    .map_err(|e| format!("Failed to get volume: {}", e))?;
                let is_muted = endpoint_volume.GetMute()
                    .map_err(|e| format!("Failed to get mute status: {}", e))?;
                
                let pct = (current_level * 100.0).round() as i32;
                Ok(json!({"action": "get", "level": clamp_0_100(pct), "muted": is_muted.as_bool()}))
            }
            "set" => {
                let clamped = clamp_0_100(level);
                endpoint_volume.SetMasterVolumeLevelScalar(clamped as f32 / 100.0, std::ptr::null())
                    .map_err(|e| format!("Failed to set volume: {}", e))?;
                Ok(json!({"action": "set", "level": clamped, "success": true}))
            }
            "up" | "down" => {
                let current_level = endpoint_volume.GetMasterVolumeLevelScalar()
                    .map_err(|e| format!("Failed to get volume: {}", e))?;
                let mut pct = (current_level * 100.0).round() as i32;
                
                if action == "up" {
                    pct += step;
                } else {
                    pct -= step;
                }
                
                let clamped = clamp_0_100(pct);
                endpoint_volume.SetMasterVolumeLevelScalar(clamped as f32 / 100.0, std::ptr::null())
                    .map_err(|e| format!("Failed to set volume: {}", e))?;
                Ok(json!({"action": action, "level": clamped, "success": true}))
            }
            "mute" => {
                endpoint_volume.SetMute(true, std::ptr::null())
                    .map_err(|e| format!("Failed to mute: {}", e))?;
                Ok(json!({"action": "mute", "muted": true, "success": true}))
            }
            "unmute" => {
                endpoint_volume.SetMute(false, std::ptr::null())
                    .map_err(|e| format!("Failed to unmute: {}", e))?;
                Ok(json!({"action": "unmute", "muted": false, "success": true}))
            }
            _ => Err(format!("Unsupported volume action: {action}")),
        }
    }
}

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
