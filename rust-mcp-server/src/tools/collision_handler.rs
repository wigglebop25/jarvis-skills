use std::path::{Path, PathBuf};

/// Generates a unique file path by appending a counter if the path already exists.
///
/// If the given path doesn't exist, it's returned as-is. Otherwise, this function
/// appends an incrementing numeric suffix to the filename (e.g., `file_1.txt`,
/// `file_2.txt`) until it finds an unused path or reaches the limit (10,000 attempts).
///
/// # Arguments
/// * `path` - The file path to check
///
/// # Returns
/// A `PathBuf` with either the original path (if it doesn't exist) or a new unique path
pub fn unique_path_if_exists(path: &Path) -> PathBuf {
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
