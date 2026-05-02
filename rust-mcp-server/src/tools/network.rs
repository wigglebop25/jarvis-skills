use serde_json::{json, Map, Value};

use super::shell::run_command;

pub fn toggle_network(args: &Map<String, Value>) -> Result<Value, String> {
    let interface = args
        .get("interface")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required field: interface".to_string())?;
    let enable = args
        .get("enable")
        .and_then(Value::as_bool)
        .ok_or_else(|| "Missing required field: enable".to_string())?;

    #[cfg(target_os = "windows")]
    {
        let action = if enable { "enable" } else { "disable" };
        match interface {
            "wifi" => {
                let _ = run_command("netsh", &["interface", "set", "interface", "Wi-Fi", action])?;
                Ok(json!({"interface":"wifi","enabled":enable}))
            }
            "ethernet" => {
                let _ = run_command("netsh", &["interface", "set", "interface", "Ethernet", action])?;
                Ok(json!({"interface":"ethernet","enabled":enable}))
            }
            "bluetooth" => {
                let ps_action = if enable { "Enable-PnpDevice" } else { "Disable-PnpDevice" };
                let pnp_action = if enable { "enable-device" } else { "disable-device" };
                let script = bluetooth_toggle_script(ps_action, pnp_action, enable);
                match run_command("powershell", &["-NoProfile", "-Command", &script]) {
                    Ok(_) => {}
                    Err(err) => {
                        let normalized = normalize_bluetooth_toggle_error(&err);
                        let lowered = normalized.to_ascii_lowercase();
                        let needs_elevation =
                            lowered.contains("generic failure")
                                || lowered.contains("access is denied")
                                || lowered.contains("pnputil exit code 50");
                        if needs_elevation {
                            // Retry once via UAC elevation for environments where device toggles
                            // require an administrator token.
                            let elevated = elevated_bluetooth_toggle_script(ps_action, pnp_action, enable);
                            if let Err(elevated_err) = run_command(
                                "powershell",
                                &[
                                    "-NoProfile",
                                    "-ExecutionPolicy",
                                    "Bypass",
                                    "-Command",
                                    &elevated,
                                ],
                            ) {
                                return Err(normalize_bluetooth_toggle_error(&elevated_err));
                            }
                        } else {
                            return Err(normalized);
                        }
                    }
                }
                Ok(json!({"interface":"bluetooth","enabled":enable}))
            }
            _ => Err(format!("Unsupported interface: {interface}")),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let power = if enable { "on" } else { "off" };
        match interface {
            "wifi" => {
                let _ = run_command("nmcli", &["radio", "wifi", power])?;
                Ok(json!({"interface":"wifi","enabled":enable}))
            }
            "bluetooth" => {
                let action = if enable { "unblock" } else { "block" };
                let _ = run_command("rfkill", &[action, "bluetooth"])?;
                Ok(json!({"interface":"bluetooth","enabled":enable}))
            }
            "ethernet" => Ok(
                json!({"interface":"ethernet","enabled":enable,"note":"No direct generic command applied"}),
            ),
            _ => Err(format!("Unsupported interface: {interface}")),
        }
    }
}

#[cfg(target_os = "windows")]
fn bluetooth_toggle_script(ps_action: &str, pnp_action: &str, enable: bool) -> String {
    let verify_block = if enable {
        "$post = Get-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue | Where-Object { $targetIds -contains $_.InstanceId }; \
         $okCount = @($post | Where-Object { $_.Status -eq 'OK' }).Count; \
         if ($okCount -eq 0) { throw 'Bluetooth verification failed: adapter is still not enabled.' }"
            .to_string()
    } else {
        "$post = Get-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue | Where-Object { $targetIds -contains $_.InstanceId }; \
         $stillOn = @($post | Where-Object { $_.Status -eq 'OK' }); \
         if ($stillOn.Count -gt 0) { \
             $names = ($stillOn | ForEach-Object { $_.FriendlyName }) -join ', '; \
             throw ('Bluetooth verification failed: still ON for device(s): ' + $names) \
         }"
            .to_string()
    };
    format!(
        "$devices = Get-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue; \
         if (-not $devices) {{ throw 'No Bluetooth devices found.' }}; \
         $targets = $devices | Where-Object {{ \
            $_.FriendlyName -and \
            $_.FriendlyName -match 'Adapter|Radio|TP-Link|Intel|Realtek|Qualcomm|Mediatek|Broadcom' \
         }}; \
         if (-not $targets) {{ \
             $targets = $devices | Where-Object {{ \
                $_.FriendlyName -and $_.FriendlyName -notmatch 'Enumerator|RFCOMM|Transport|AVRCP|LE Enumerator|Bluetooth Device \\(' \
             }} \
         }}; \
         if (-not $targets) {{ \
             throw 'No controllable Bluetooth adapter device found.' \
         }}; \
         $targetIds = @($targets | ForEach-Object {{ $_.InstanceId }}); \
         $failed = @(); \
         foreach ($d in $targets) {{ \
            try {{ {ps_action} -InstanceId $d.InstanceId -Confirm:$false -ErrorAction Stop | Out-Null }} \
            catch {{ \
               try {{ \
                  pnputil /{pnp_action} \"$($d.InstanceId)\" /force | Out-Null; \
                  if ($LASTEXITCODE -ne 0) {{ throw ('pnputil exit code ' + $LASTEXITCODE) }} \
               }} catch {{ \
                  $failed += ('{ps_action} failed for ' + $d.FriendlyName + ': ' + $_.Exception.Message) \
               }} \
            }} \
         }}; \
         if ($failed.Count -gt 0) {{ throw ($failed -join \"`n\") }}; \
         {verify_block}"
    )
}

#[cfg(target_os = "windows")]
fn elevated_bluetooth_toggle_script(ps_action: &str, pnp_action: &str, enable: bool) -> String {
    let inner = bluetooth_toggle_script(ps_action, pnp_action, enable).replace('\'', "''");
    format!(
        "$inner = '{inner}'; \
         $tmp = Join-Path $env:TEMP ('jarvis_bt_' + [guid]::NewGuid().ToString() + '.ps1'); \
         Set-Content -Path $tmp -Value $inner -Encoding UTF8; \
         try {{ \
           $p = Start-Process -FilePath powershell -Verb RunAs -PassThru -Wait \
             -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',$tmp); \
           if ($p.ExitCode -ne 0) {{ throw ('Elevated bluetooth toggle failed with exit code ' + $p.ExitCode) }} \
         }} finally {{ Remove-Item -Path $tmp -Force -ErrorAction SilentlyContinue }}"
    )
}

#[cfg(target_os = "windows")]
fn normalize_bluetooth_toggle_error(err: &str) -> String {
    let lowered = err.to_ascii_lowercase();
    if lowered.contains("pnputil exit code 50") || lowered.contains("request is not supported") {
        return "Bluetooth adapter does not support software power toggle on this driver (pnputil exit code 50). Use Windows Settings toggle or unplug the Bluetooth dongle.".to_string();
    }
    if lowered.contains("access is denied") || lowered.contains("generic failure") {
        return format!(
            "{err}\nBluetooth toggle likely requires Administrator privileges on this system."
        );
    }
    err.to_string()
}
