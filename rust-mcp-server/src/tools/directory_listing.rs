use std::{
    env, fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde_json::{json, Map, Value};

pub fn list_directory(args: &Map<String, Value>) -> Result<Value, String> {
    let path_str = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: path".to_string())?;
    let include_hidden = args
        .get("include_hidden")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let max_entries = args
        .get("max_entries")
        .and_then(Value::as_i64)
        .unwrap_or(200)
        .clamp(1, 2000) as usize;
    let directories_only = args
        .get("directories_only")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let files_only = args.get("files_only").and_then(Value::as_bool).unwrap_or(false);

    if directories_only && files_only {
        return Err("directories_only and files_only cannot both be true".to_string());
    }

    let base = fs::canonicalize(path_str).map_err(|e| format!("Invalid path '{path_str}': {e}"))?;
    if !base.is_dir() {
        return Err(format!("Path is not a directory: {}", base.to_string_lossy()));
    }

    ensure_allowed_listing_root(&base)?;

    let mut entries: Vec<Value> = Vec::new();
    let mut truncated = false;

    let read_dir = fs::read_dir(&base)
        .map_err(|e| format!("Failed reading directory '{}': {e}", base.to_string_lossy()))?;
    for entry_result in read_dir {
        let entry = match entry_result {
            Ok(v) => v,
            Err(_) => continue,
        };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !include_hidden && is_hidden_name(&name) {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let is_dir = file_type.is_dir();
        let is_file = file_type.is_file();

        if directories_only && !is_dir {
            continue;
        }
        if files_only && !is_file {
            continue;
        }

        let metadata = fs::metadata(&path).ok();
        let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified_unix = metadata
            .and_then(|m| m.modified().ok())
            .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        entries.push(json!({
            "name": name,
            "path": path.to_string_lossy(),
            "type": if is_dir { "directory" } else { "file" },
            "size_bytes": size_bytes,
            "modified_unix": modified_unix
        }));

        if entries.len() >= max_entries {
            truncated = true;
            break;
        }
    }

    entries.sort_by(|a, b| {
        let ta = a.get("type").and_then(Value::as_str).unwrap_or("file");
        let tb = b.get("type").and_then(Value::as_str).unwrap_or("file");
        let na = a.get("name").and_then(Value::as_str).unwrap_or("");
        let nb = b.get("name").and_then(Value::as_str).unwrap_or("");

        let type_cmp = ta.cmp(tb);
        if type_cmp == std::cmp::Ordering::Equal {
            na.to_ascii_lowercase().cmp(&nb.to_ascii_lowercase())
        } else {
            type_cmp
        }
    });

    Ok(json!({
        "path": base.to_string_lossy(),
        "count": entries.len(),
        "max_entries": max_entries,
        "truncated": truncated,
        "entries": entries
    }))
}

fn ensure_allowed_listing_root(path: &Path) -> Result<(), String> {
    let allowed = listing_allowed_roots()?;
    for root in &allowed {
        if path.starts_with(root) {
            return Ok(());
        }
    }
    Err(format!(
        "Path '{}' is outside allowed listing roots: {}",
        path.to_string_lossy(),
        allowed
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("; ")
    ))
}

fn listing_allowed_roots() -> Result<Vec<PathBuf>, String> {
    if let Ok(v) = env::var("JARVIS_SKILLS_LIST_ALLOWED_ROOTS") {
        let mut roots = Vec::new();
        for part in v.split(';') {
            let p = part.trim();
            if p.is_empty() {
                continue;
            }
            if let Ok(c) = fs::canonicalize(p) {
                roots.push(c);
            }
        }
        if !roots.is_empty() {
            return Ok(roots);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut roots = Vec::new();
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            let p = PathBuf::from(&drive);
            if p.exists() {
                if let Ok(c) = fs::canonicalize(&p) {
                    roots.push(c);
                } else {
                    roots.push(p);
                }
            }
        }
        if roots.is_empty() {
            return Err("Could not resolve any Windows drive roots".to_string());
        }
        return Ok(roots);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        return Ok(vec![home]);
    }
}

fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.')
}
