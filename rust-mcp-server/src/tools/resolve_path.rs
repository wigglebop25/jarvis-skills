use std::env;
use std::path::PathBuf;
use serde_json::{json, Map, Value};

pub fn resolve_path(args: &Map<String, Value>) -> Result<Value, String> {
    let name = args
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: name".to_string())?
        .to_lowercase();

    let path = match name.as_str() {
        "downloads" => dirs::download_dir(),
        "documents" => dirs::document_dir(),
        "desktop" => dirs::desktop_dir(),
        "home" => dirs::home_dir(),
        "project" => env::var("JARVIS_PROJECT_ROOT").ok().map(PathBuf::from),
        _ => return Err(format!("Unknown path name: {}. Supported: downloads, documents, desktop, home, project", name)),
    };

    let resolved_path = path.ok_or_else(|| format!("Could not resolve path for: {}", name))?;
    let path_str = resolved_path.to_string_lossy().to_string();
    
    if !resolved_path.exists() {
        return Err(format!("Path does not exist: {}", path_str));
    }

    Ok(json!({
        "resolved_path": path_str,
        "name": name,
        "exists": true
    }))
}
