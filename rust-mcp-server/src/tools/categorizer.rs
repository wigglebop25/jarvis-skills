use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct MoveOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Plan file move operations based on categorization strategy.
pub fn plan_moves(files: &[PathBuf], strategy: &str) -> Result<Vec<MoveOp>, String> {
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

/// Categorize file by extension.
pub fn extension_bucket(file: &Path) -> String {
    file.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| "no_extension".to_string())
}

/// Categorize file by type (extension-based categorization).
pub fn type_bucket(file: &Path) -> String {
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

/// Categorize file by modification date (YYYY-MM format).
pub fn date_bucket(file: &Path) -> Result<String, String> {
    let meta = fs::metadata(file).map_err(|e| format!("metadata failed: {e}"))?;
    let modified = meta
        .modified()
        .map_err(|e| format!("modified time failed: {e}"))?;
    let dt: DateTime<Local> = DateTime::<Local>::from(modified);
    Ok(dt.format("%Y-%m").to_string())
}

/// Predefined file type categories.
pub fn type_map() -> HashMap<&'static str, Vec<String>> {
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
