"""
Network Control Tool - Toggle WiFi/Bluetooth/Network interfaces.
"""

import subprocess
import sys
from jarvis_skills_core import ToolParameter, ToolParameterType


def toggle_network(
    interface: str,
    enable: bool,
) -> dict:
    """
    Toggle network interfaces on/off.
    
    Args:
        interface: Interface type - "wifi", "bluetooth", "ethernet"
        enable: True to enable, False to disable
    
    Returns:
        Dictionary with interface status.
    """
    if sys.platform == "win32":
        return _toggle_network_windows(interface, enable)
    elif sys.platform == "darwin":
        return _toggle_network_macos(interface, enable)
    else:
        return _toggle_network_linux(interface, enable)


def _toggle_network_windows(interface: str, enable: bool) -> dict:
    """Windows network control using netsh."""
    action = "enable" if enable else "disable"
    
    try:
        if interface == "wifi":
            result = subprocess.run(
                ["netsh", "interface", "set", "interface", "Wi-Fi", action],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                return {"interface": "wifi", "enabled": enable}
            return {"error": result.stderr or "Failed to toggle WiFi"}
        
        elif interface == "bluetooth":
            ps_script = f"""
            $bt = Get-PnpDevice -Class Bluetooth | Where-Object {{ $_.FriendlyName -like '*Bluetooth*' }}
            if ($bt) {{
                if ({'$true' if enable else '$false'}) {{
                    Enable-PnpDevice -InstanceId $bt.InstanceId -Confirm:$false
                }} else {{
                    Disable-PnpDevice -InstanceId $bt.InstanceId -Confirm:$false
                }}
            }}
            """
            result = subprocess.run(
                ["powershell", "-Command", ps_script],
                capture_output=True,
                text=True,
            )
            return {"interface": "bluetooth", "enabled": enable}
        
        elif interface == "ethernet":
            result = subprocess.run(
                ["netsh", "interface", "set", "interface", "Ethernet", action],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                return {"interface": "ethernet", "enabled": enable}
            return {"error": result.stderr or "Failed to toggle Ethernet"}
        
        return {"error": f"Unknown interface: {interface}"}
    
    except Exception as e:
        return {"error": str(e)}


def _toggle_network_macos(interface: str, enable: bool) -> dict:
    """macOS network control using networksetup."""
    action = "on" if enable else "off"
    
    try:
        if interface == "wifi":
            subprocess.run(
                ["networksetup", "-setairportpower", "en0", action],
                capture_output=True,
            )
            return {"interface": "wifi", "enabled": enable}
        
        elif interface == "bluetooth":
            ps_cmd = "defaults write /Library/Preferences/com.apple.Bluetooth ControllerPowerState -int " + ("1" if enable else "0")
            subprocess.run(
                ["sudo", "sh", "-c", ps_cmd],
                capture_output=True,
            )
            subprocess.run(
                ["sudo", "killall", "-HUP", "blued"],
                capture_output=True,
            )
            return {"interface": "bluetooth", "enabled": enable}
        
        return {"error": f"Unknown interface: {interface}"}
    
    except Exception as e:
        return {"error": str(e)}


def _toggle_network_linux(interface: str, enable: bool) -> dict:
    """Linux network control using nmcli or rfkill."""
    action = "on" if enable else "off"
    
    try:
        if interface == "wifi":
            subprocess.run(
                ["nmcli", "radio", "wifi", action],
                capture_output=True,
            )
            return {"interface": "wifi", "enabled": enable}
        
        elif interface == "bluetooth":
            rfkill_action = "unblock" if enable else "block"
            subprocess.run(
                ["rfkill", rfkill_action, "bluetooth"],
                capture_output=True,
            )
            return {"interface": "bluetooth", "enabled": enable}
        
        return {"error": f"Unknown interface: {interface}"}
    
    except Exception as e:
        return {"error": str(e)}


def register_network_tool(server) -> None:
    """Register the network control tool with the MCP server."""
    parameters = [
        ToolParameter(
            name="interface",
            type=ToolParameterType.STRING,
            description="Network interface: wifi, bluetooth, ethernet",
            required=True,
            enum=["wifi", "bluetooth", "ethernet"],
        ),
        ToolParameter(
            name="enable",
            type=ToolParameterType.BOOLEAN,
            description="True to enable, False to disable",
            required=True,
        ),
    ]
    
    server.register_tool(
        name="toggle_network",
        description="Toggle network interfaces (WiFi, Bluetooth, Ethernet) on/off",
        handler=toggle_network,
        parameters=parameters,
    )
