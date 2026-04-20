use serde_json::{json, Map, Value};

use super::shell::run_command;

pub fn control_bluetooth_device(args: &Map<String, Value>) -> Result<Value, String> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: action".to_string())?;
    let include_system = args
        .get("include_system")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    #[cfg(target_os = "windows")]
    {
        match action {
            "list" => {
                let devices = list_bluetooth_devices(include_system)?;
                Ok(json!({
                    "action": "list",
                    "count": devices.len(),
                    "devices": devices
                }))
            }
            "connect" | "disconnect" => {
                let name_query = args.get("device_name").and_then(Value::as_str).map(str::to_lowercase);
                let instance_id = args.get("instance_id").and_then(Value::as_str);
                if name_query.is_none() && instance_id.is_none() {
                    return Err("For connect/disconnect, provide 'device_name' or 'instance_id'.".to_string());
                }

                let all_devices = list_bluetooth_devices(include_system)?;
                let mut targets: Vec<Value> = all_devices
                    .into_iter()
                    .filter(|d| {
                        let id_match = instance_id
                            .and_then(|id| d.get("instance_id").and_then(Value::as_str).map(|v| v.eq_ignore_ascii_case(id)))
                            .unwrap_or(false);
                        let name_match = name_query
                            .as_ref()
                            .and_then(|needle| {
                                d.get("friendly_name")
                                    .and_then(Value::as_str)
                                    .map(|n| n.to_lowercase().contains(needle))
                            })
                            .unwrap_or(false);
                        id_match || name_match
                    })
                    .collect();

                if targets.is_empty() {
                    return Err("No matching Bluetooth device found.".to_string());
                }

                let enable = action == "connect";
                let mut results = Vec::new();
                let mut failed = Vec::new();

                for target in &mut targets {
                    let name = target
                        .get("friendly_name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown")
                        .to_string();
                    let instance = target
                        .get("instance_id")
                        .and_then(Value::as_str)
                        .ok_or_else(|| format!("Device '{name}' has no instance_id"))?
                        .to_string();

                    match toggle_device_instance(&instance, enable) {
                        Ok(_) => {
                            results.push(json!({
                                "friendly_name": name,
                                "instance_id": instance,
                                "command_succeeded": true
                            }));
                        }
                        Err(e) => {
                            failed.push(format!("{name}: {e}"));
                            results.push(json!({
                                "friendly_name": name,
                                "instance_id": instance,
                                "command_succeeded": false,
                                "error": e
                            }));
                        }
                    }
                }

                let post_devices = list_bluetooth_devices(true)?;
                let expected_connected = enable;
                let mut verification_failures = Vec::new();

                for result in &mut results {
                    let Some(instance) = result.get("instance_id").and_then(Value::as_str) else {
                        continue;
                    };
                    if let Some(post) = post_devices.iter().find(|d| {
                        d.get("instance_id")
                            .and_then(Value::as_str)
                            .map(|v| v.eq_ignore_ascii_case(instance))
                            .unwrap_or(false)
                    }) {
                        let connected = post.get("is_connected").cloned().unwrap_or(Value::Null);
                        if let Some(obj) = result.as_object_mut() {
                            obj.insert("is_connected".to_string(), connected.clone());
                        }
                        if let Some(c) = connected.as_bool() {
                            if c != expected_connected {
                                let name = result
                                    .get("friendly_name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("unknown");
                                verification_failures.push(format!(
                                    "{name}: expected is_connected={expected_connected}, actual={c}"
                                ));
                            }
                        }
                    }
                }

                if !failed.is_empty() {
                    return Err(failed.join("\n"));
                }
                if !verification_failures.is_empty() {
                    return Err(format!(
                        "Command executed but connection state did not match expectation:\n{}",
                        verification_failures.join("\n")
                    ));
                }

                Ok(json!({
                    "action": action,
                    "success": true,
                    "matched": results.len(),
                    "results": results
                }))
            }
            _ => Err(format!("Unsupported action: {action}. Use list/connect/disconnect.")),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(format!(
            "control_bluetooth_device is currently implemented for Windows only. Received action: {action}"
        ))
    }
}

#[cfg(target_os = "windows")]
fn toggle_device_instance(instance_id: &str, enable: bool) -> Result<(), String> {
    let ps_action = if enable { "Enable-PnpDevice" } else { "Disable-PnpDevice" };
    let pnp_action = if enable { "enable-device" } else { "disable-device" };
    let escaped = instance_id.replace('\'', "''");
    let script = format!("{ps_action} -InstanceId '{escaped}' -Confirm:$false -ErrorAction Stop | Out-Null");

    match run_command("powershell", &["-NoProfile", "-Command", &script]) {
        Ok(_) => Ok(()),
        Err(e) => {
            let pnputil_action = format!("/{pnp_action}");
            match run_command("pnputil", &[&pnputil_action, instance_id, "/force"]) {
                Ok(_) => Ok(()),
                Err(pnp_err) => Err(format!("{e}; fallback pnputil failed: {pnp_err}")),
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn list_bluetooth_devices(include_system: bool) -> Result<Vec<Value>, String> {
    let include_flag = if include_system { "$true" } else { "$false" };
    let script = format!(
        "$devices = Get-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue | Where-Object {{ $_.FriendlyName }}; \
         $rows = @(); \
         foreach ($d in $devices) {{ \
           $isConnected = $null; \
           try {{ $isConnected = (Get-PnpDeviceProperty -InstanceId $d.InstanceId -KeyName 'DEVPKEY_Device_IsConnected' -ErrorAction Stop).Data }} catch {{}}; \
           $isSystem = $d.FriendlyName -match 'Enumerator|RFCOMM|Transport|AVRCP|LE Enumerator|Bluetooth Device \\('; \
           $isAdapter = $d.FriendlyName -match 'Adapter|Radio|TP-Link|Intel|Realtek|Qualcomm|Mediatek|Broadcom'; \
           if ({include_flag} -or -not $isSystem) {{ \
             $rows += [PSCustomObject]@{{ \
               friendly_name = $d.FriendlyName; \
               instance_id = $d.InstanceId; \
               status = $d.Status; \
               is_connected = $isConnected; \
               is_adapter = $isAdapter; \
               is_system = $isSystem \
             }} \
           }} \
         }}; \
         $rows | ConvertTo-Json -Depth 4 -Compress"
    );
    let output = run_command("powershell", &["-NoProfile", "-Command", &script])?;
    parse_json_arrayish(&output)
}

#[cfg(target_os = "windows")]
fn parse_json_arrayish(raw: &str) -> Result<Vec<Value>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let v: Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("Failed to parse Bluetooth device JSON: {e}"))?;
    match v {
        Value::Array(arr) => Ok(arr),
        Value::Object(_) => Ok(vec![v]),
        _ => Err("Unexpected Bluetooth list output format.".to_string()),
    }
}
