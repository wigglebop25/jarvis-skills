use std::{env, fs, path::{Path, PathBuf}};

/// Verify that a path is within allowed root directories.
pub fn ensure_allowed_root(path: &Path) -> Result<(), String> {
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

/// Load allowed root directories from environment or defaults.
pub fn allowed_roots() -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();

    // 1. Add from environment variable if set
    if let Ok(v) = env::var("JARVIS_SKILLS_ALLOWED_ROOTS") {
        for part in v.split(';') {
            let p = part.trim();
            if !p.is_empty() {
                roots.push(PathBuf::from(p));
            }
        }
    }

    // 2. If no env var, add standard user folders automatically
    if roots.is_empty() {
        if let Some(p) = dirs::home_dir() {
            roots.push(p);
        }
        if let Some(p) = dirs::desktop_dir() {
            roots.push(p);
        }
        if let Some(p) = dirs::download_dir() {
            roots.push(p);
        }
        if let Some(p) = dirs::document_dir() {
            roots.push(p);
        }
        // Always include current directory
        if let Ok(p) = env::current_dir() {
            roots.push(p);
        }
    }

    if roots.is_empty() {
        return Err("Could not resolve any allowed roots".to_string());
    }

    // 3. Canonicalize all roots for reliable starts_with comparison
    let mut canon = Vec::new();
    for r in roots {
        if let Ok(c) = fs::canonicalize(&r) {
            canon.push(c);
        } else {
            // Fallback for paths that don't exist yet but might be created
            canon.push(r);
        }
    }
    
    // Deduplicate
    canon.sort();
    canon.dedup();
    
    Ok(canon)
}
