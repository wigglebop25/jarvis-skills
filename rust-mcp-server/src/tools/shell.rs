use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
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

#[cfg(target_os = "windows")]
pub fn find_nircmd() -> Result<String, String> {
    if let Ok(path) = which::which("nircmd") {
        return Ok(path.to_string_lossy().to_string());
    }
    if let Ok(path) = which::which("nircmd.exe") {
        return Ok(path.to_string_lossy().to_string());
    }
    let local = Path::new("src")
        .join(".tools_cache")
        .join("nircmd")
        .join("nircmd.exe");
    if local.exists() {
        return Ok(local.to_string_lossy().to_string());
    }

    let installed = install_nircmd()?;
    Ok(installed.to_string_lossy().to_string())
}

#[cfg(target_os = "windows")]
fn install_nircmd() -> Result<PathBuf, String> {
    let cache_dir: PathBuf = dirs::cache_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("jarvis-skills")
        .join("nircmd");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed creating NirCmd cache directory '{}': {e}", cache_dir.to_string_lossy()))?;

    let exe_path = cache_dir.join("nircmd.exe");
    if exe_path.exists() {
        return Ok(exe_path);
    }

    let zip_path: PathBuf = cache_dir.join("nircmd.zip");
    let urls: [&str; 2] = [
        "https://www.nirsoft.net/utils/nircmd-x64.zip",
        "https://www.nirsoft.net/utils/nircmd.zip",
    ];
    let script: String = format!(
        "$ErrorActionPreference='Stop';\
         $urls=@('{}','{}');\
         $zip='{}';\
         $dest='{}';\
         $ok=$false;\
         foreach($u in $urls){{\
            try {{ Invoke-WebRequest -Uri $u -OutFile $zip -UseBasicParsing; $ok=$true; break }} catch {{}}\
         }};\
         if(-not $ok){{ throw 'download failed' }};\
         Expand-Archive -Path $zip -DestinationPath $dest -Force",
        urls[0],
        urls[1],
        ps_escape(&zip_path),
        ps_escape(&cache_dir)
    );
    let extract = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell to extract NirCmd archive: {e}"))?;

    if !extract.status.success() {
        let stderr = String::from_utf8_lossy(&extract.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&extract.stdout).trim().to_string();
        let details: String = if !stderr.is_empty() { stderr } else { stdout };
        return Err(format!(
            "Automatic NirCmd install failed: {}",
            if details.is_empty() {
                "unknown extraction error".to_string()
            } else {
                details
            }
        ));
    }

    if exe_path.exists() {
        return Ok(exe_path);
    }

    let nested = cache_dir.join("nircmd").join("nircmd.exe");
    if nested.exists() {
        return Ok(nested);
    }

    Err(format!(
        "NirCmd was downloaded but nircmd.exe was not found under '{}'. Install manually from https://www.nirsoft.net/utils/nircmd.html",
        cache_dir.to_string_lossy()
    ))
}

#[cfg(target_os = "windows")]
fn ps_escape(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

fn install_hint(program: &str) -> Option<&'static str> {
    match program {
        "pactl" => Some("Install PulseAudio/PipeWire utilities (package usually named 'pulseaudio-utils' or equivalent)."),
        "playerctl" => Some("Install playerctl from your distro package manager."),
        "nmcli" => Some("Install NetworkManager CLI tools."),
        "rfkill" => Some("Install rfkill from your distro package manager."),
        "osascript" => Some("This command should be available on macOS by default."),
        "powershell" => Some("Install PowerShell and ensure it is available in PATH."),
        _ => None,
    }
}
