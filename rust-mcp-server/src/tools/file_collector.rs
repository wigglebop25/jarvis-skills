use std::{fs, path::{Path, PathBuf}};
use walkdir::WalkDir;

/// Collect files from a directory with optional recursion and hidden file filtering.
pub fn collect_files(base: &Path, recursive: bool, include_hidden: bool) -> Result<Vec<PathBuf>, String> {
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

/// Check if a path component starts with a dot (hidden file).
pub fn is_hidden_path(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    })
}
