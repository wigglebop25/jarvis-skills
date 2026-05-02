use std::{
    io::ErrorKind,
    process::Command,
};

pub fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                if let Some(hint) = install_hint(program) {
                    format!("Required command '{program}' not found. {hint}")
                } else {
                    format!("Required command '{program}' not found in PATH.")
                }
            } else {
                format!("Failed to run '{program}': {e}")
            }
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stderr.is_empty() {
            Err(stderr)
        } else if !stdout.is_empty() {
            Err(stdout)
        } else {
            Err(format!(
                "Command '{program}' failed with exit code {:?}",
                output.status.code()
            ))
        }
    }
}

fn install_hint(program: &str) -> Option<&'static str> {
    match program {
        "pactl" => Some("Install PulseAudio/PipeWire utilities (package usually named 'pulseaudio-utils' or equivalent)."),
        "playerctl" => Some("Install playerctl from your distro package manager."),
        "nmcli" => Some("Install NetworkManager CLI tools."),
        "rfkill" => Some("Install rfkill from your distro package manager."),
        "powershell" => Some("Install PowerShell and ensure it is available in PATH."),
        _ => None,
    }
}
