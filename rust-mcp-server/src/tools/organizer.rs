use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use serde_json::{json, Map, Value};
use walkdir::WalkDir;

#[derive(Clone)]
struct MoveOp {
    from: PathBuf,
    to: PathBuf,
}

pub fn organize_folder(args: &Map<String, Value>) -> Result<Value, String> {
    let path_str = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: path".to_string())?;
    let strategy = args
        .get("strategy")
        .and_then(Value::as_str)
        .unwrap_or("extension");
    let recursive = args.get("recursive").and_then(Value::as_bool).unwrap_or(false);
    let dry_run = args.get("dry_run").and_then(Value::as_bool).unwrap_or(true);
    let include_hidden = args
        .get("include_hidden")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let execute_plan = args
        .get("execute_plan")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let base = fs::canonicalize(path_str).map_err(|e| format!("Invalid path '{path_str}': {e}"))?;
    ensure_allowed_root(&base)?;

    let files = collect_files(&base, recursive, include_hidden)?;
    let ops = plan_moves(&files, strategy)?;

    if dry_run || !execute_plan {
        return Ok(json!({
            "phase": "planning",
            "path": base.to_string_lossy(),
            "strategy": strategy,
            "planned_operations": ops.len(),
            "operations": ops.iter().map(|op| {
                json!({"from": op.from.to_string_lossy(), "to": op.to.to_string_lossy()})
            }).collect::<Vec<Value>>()
        }));
    }

    let mut moved = 0usize;
    let mut errors: Vec<String> = Vec::new();
    for op in ops {
        if let Some(parent) = op.to.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                errors.push(format!("Failed to create '{}': {e}", parent.to_string_lossy()));
                continue;
            }
        }
        let target = unique_path_if_exists(&op.to);
        match fs::rename(&op.from, &target) {
            Ok(_) => moved += 1,
            Err(e) => errors.push(format!(
                "Failed moving '{}' -> '{}': {e}",
                op.from.to_string_lossy(),
                target.to_string_lossy()
            )),
        }
    }

    Ok(json!({
        "phase": "execution",
        "moved": moved,
        "errors": errors,
        "success": errors.is_empty()
    }))
}

fn collect_files(base: &Path, recursive: bool, include_hidden: bool) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    if recursive {
        for entry in WalkDir::new(base).into_iter().filter_map(Result::ok) {
            let p = entry.path();
            if p.is_file() {
                if !include_hidden && is_hidden_path(p) {
                    continue;
                }
                files.push(p.to_path_buf());
            }
        }
    } else {
        let entries = fs::read_dir(base).map_err(|e| format!("Failed reading dir: {e}"))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if !include_hidden && is_hidden_path(&p) {
                    continue;
                }
                files.push(p);
            }
        }
    }
    Ok(files)
}

fn plan_moves(files: &[PathBuf], strategy: &str) -> Result<Vec<MoveOp>, String> {
    let mut ops = Vec::new();
    for file in files {
        let parent = match file.parent() {
            Some(v) => v,
            None => continue,
        };
        let bucket = match strategy {
            "extension" => extension_bucket(file),
            "type" => type_bucket(file),
            "date" => date_bucket(file)?,
            other => return Err(format!("Unsupported strategy: {other}")),
        };
        let target_dir = parent.join(bucket);
        let Some(name) = file.file_name() else {
            continue;
        };
        let target = target_dir.join(name);
        if target != *file {
            ops.push(MoveOp {
                from: file.clone(),
                to: target,
            });
        }
    }
    Ok(ops)
}

fn extension_bucket(file: &Path) -> String {
    file.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| "no_extension".to_string())
}

fn type_bucket(file: &Path) -> String {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .map(|v| format!(".{}", v.to_ascii_lowercase()))
        .unwrap_or_default();
    let map = type_map();
    for (bucket, exts) in map {
        if exts.contains(&ext) {
            return bucket.to_string();
        }
    }
    "others".to_string()
}

fn date_bucket(file: &Path) -> Result<String, String> {
    let meta = fs::metadata(file).map_err(|e| format!("metadata failed: {e}"))?;
    let modified = meta
        .modified()
        .map_err(|e| format!("modified time failed: {e}"))?;
    let dt: DateTime<Local> = DateTime::<Local>::from(modified);
    Ok(dt.format("%Y-%m").to_string())
}

fn ensure_allowed_root(path: &Path) -> Result<(), String> {
    let allowed = allowed_roots()?;
    for root in &allowed {
        if path.starts_with(root) {
            return Ok(());
        }
    }
    Err(format!(
        "Path '{}' is outside allowed roots: {}",
        path.to_string_lossy(),
        allowed
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("; ")
    ))
}

fn allowed_roots() -> Result<Vec<PathBuf>, String> {
    if let Ok(v) = env::var("JARVIS_SKILLS_ALLOWED_ROOTS") {
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

    let mut roots = Vec::new();
    if let Some(p) = dirs::desktop_dir() {
        roots.push(p);
    }
    if let Some(p) = dirs::download_dir() {
        roots.push(p);
    }
    if let Some(p) = dirs::document_dir() {
        roots.push(p);
    }
    if roots.is_empty() {
        return Err("Could not resolve default allowed roots".to_string());
    }

    let mut canon = Vec::new();
    for r in roots {
        if let Ok(c) = fs::canonicalize(&r) {
            canon.push(c);
        } else {
            canon.push(r);
        }
    }
    Ok(canon)
}

fn unique_path_if_exists(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    for i in 1..10_000 {
        let name = if ext.is_empty() {
            format!("{stem}_{i}")
        } else {
            format!("{stem}_{i}.{ext}")
        };
        let candidate = parent.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}

fn type_map() -> HashMap<&'static str, Vec<String>> {
    HashMap::from([
        (
            "images",
            vec![".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".svg"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        (
            "documents",
            vec![".pdf", ".doc", ".docx", ".txt", ".md", ".rtf", ".odt"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        (
            "spreadsheets",
            vec![".xls", ".xlsx", ".csv", ".ods"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        (
            "presentations",
            vec![".ppt", ".pptx", ".odp"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        (
            "archives",
            vec![".zip", ".rar", ".7z", ".tar", ".gz"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        (
            "audio",
            vec![".mp3", ".wav", ".flac", ".aac", ".ogg"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        (
            "video",
            vec![".mp4", ".mkv", ".mov", ".avi", ".wmv"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        (
            "code",
            vec![
                ".py", ".js", ".ts", ".tsx", ".jsx", ".go", ".rs", ".java", ".cs", ".cpp", ".c",
                ".h", ".json", ".yml", ".yaml",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        ),
    ])
}

fn is_hidden_path(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    })
}
