use serde_json::{json, Map, Value};
use sysinfo::{Disks, Networks, System};

pub fn get_system_info(args: &Map<String, Value>) -> Result<Value, String> {
    let include = args
        .get("include")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(|| vec!["cpu".into(), "ram".into(), "storage".into(), "network".into()]);

    let mut system = System::new_all();
    system.refresh_all();

    let mut result = Map::new();

    if include.iter().any(|v| v == "cpu") {
        result.insert("cpu".to_string(), json!(system.global_cpu_usage()));
    }

    if include.iter().any(|v| v == "ram") {
        let total = bytes_to_gb(system.total_memory());
        let used = bytes_to_gb(system.used_memory());
        let available = bytes_to_gb(system.available_memory());
        let percent = if total > 0.0 {
            ((used / total) * 100.0 * 100.0).round() / 100.0
        } else {
            0.0
        };

        result.insert(
            "ram".to_string(),
            json!({
                "total_gb": total,
                "used_gb": used,
                "available_gb": available,
                "percent": percent
            }),
        );
    }

    if include.iter().any(|v| v == "storage") {
        let disks = Disks::new_with_refreshed_list();
        let partitions: Vec<Value> = disks
            .iter()
            .map(|disk| {
                let total = bytes_to_gb(disk.total_space());
                let free = bytes_to_gb(disk.available_space());
                let used = (total - free).max(0.0);
                let percent = if total > 0.0 {
                    ((used / total) * 100.0 * 100.0).round() / 100.0
                } else {
                    0.0
                };
                json!({
                    "mount": disk.mount_point().to_string_lossy(),
                    "total_gb": total,
                    "used_gb": used,
                    "free_gb": free,
                    "percent": percent
                })
            })
            .collect();
        result.insert("storage".to_string(), Value::Array(partitions));
    }

    if include.iter().any(|v| v == "network") {
        let networks = Networks::new_with_refreshed_list();
        let first = networks.keys().next().cloned().unwrap_or_default();
        let connected = !first.is_empty();
        result.insert(
            "network".to_string(),
            json!({
                "connected": connected,
                "interface": first
            }),
        );
    }

    Ok(Value::Object(result))
}

fn bytes_to_gb(bytes: u64) -> f64 {
    ((bytes as f64 / 1024.0 / 1024.0 / 1024.0) * 100.0).round() / 100.0
}
