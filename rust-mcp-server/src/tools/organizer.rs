use serde_json::{json, Map, Value};
use super::path_security::ensure_allowed_root;
use super::file_collector::collect_files;
use super::categorizer::plan_moves;
use super::collision_handler::unique_path_if_exists;

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

    let base = std::fs::canonicalize(path_str).map_err(|e| format!("Invalid path '{path_str}': {e}"))?;
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
            if let Err(e) = std::fs::create_dir_all(parent) {
                errors.push(format!("Failed to create '{}': {e}", parent.to_string_lossy()));
                continue;
            }
        }
        let target = unique_path_if_exists(&op.to);
        match std::fs::rename(&op.from, &target) {
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
