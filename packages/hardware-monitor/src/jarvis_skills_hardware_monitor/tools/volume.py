"""
Volume Control Tool - Cross-platform audio volume management.
"""

import sys
import subprocess
from typing import Optional
from jarvis_skills_core import ToolParameter, ToolParameterType


def control_volume(
    action: str,
    level: Optional[int] = None,
    step: int = 10,
) -> dict:
    """
    Control system audio volume.
    
    Args:
        action: One of "get", "set", "up", "down", "mute", "unmute"
        level: Volume level (0-100) for "set" action
        step: Volume step for up/down (default 10)
    
    Returns:
        Dictionary with current volume state.
    """
    if sys.platform == "win32":
        return _control_volume_windows(action, level, step)
    elif sys.platform == "darwin":
        return _control_volume_macos(action, level, step)
    else:
        return _control_volume_linux(action, level, step)


def _control_volume_windows(action: str, level: Optional[int], step: int) -> dict:
    """Windows volume control using pycaw."""
    try:
        from ctypes import cast, POINTER
        from comtypes import CLSCTX_ALL
        from pycaw.pycaw import AudioUtilities, IAudioEndpointVolume
        
        devices = AudioUtilities.GetSpeakers()
        interface = devices.Activate(IAudioEndpointVolume._iid_, CLSCTX_ALL, None)
        volume = cast(interface, POINTER(IAudioEndpointVolume))
        
        if action == "get":
            current = int(volume.GetMasterVolumeLevelScalar() * 100)
            muted = volume.GetMute()
            return {"level": current, "muted": bool(muted)}
        
        elif action == "set" and level is not None:
            volume.SetMasterVolumeLevelScalar(level / 100.0, None)
            return {"level": level, "muted": bool(volume.GetMute())}
        
        elif action == "up":
            current = volume.GetMasterVolumeLevelScalar() * 100
            new_level = min(100, current + step)
            volume.SetMasterVolumeLevelScalar(new_level / 100.0, None)
            return {"level": int(new_level), "muted": False}
        
        elif action == "down":
            current = volume.GetMasterVolumeLevelScalar() * 100
            new_level = max(0, current - step)
            volume.SetMasterVolumeLevelScalar(new_level / 100.0, None)
            return {"level": int(new_level), "muted": False}
        
        elif action == "mute":
            volume.SetMute(1, None)
            return {"muted": True}
        
        elif action == "unmute":
            volume.SetMute(0, None)
            return {"muted": False}
        
        return {"error": f"Unknown action: {action}"}
    
    except Exception:
        return _control_volume_windows_ps(action, level, step)


def _control_volume_windows_ps(action: str, level: Optional[int], step: int) -> dict:
    """Windows volume control fallback using PowerShell."""
    try:
        if action == "get":
            result = subprocess.run(
                ["powershell", "-Command", 
                 "(Get-AudioDevice -PlaybackVolume).Volume"],
                capture_output=True, text=True
            )
            try:
                current = int(float(result.stdout.strip()))
                return {"level": current, "muted": False}
            except ValueError:
                return {"level": 50, "muted": False, "note": "Could not read volume"}
        
        elif action == "set" and level is not None:
            subprocess.run(
                ["powershell", "-Command", 
                 f"Set-AudioDevice -PlaybackVolume {level}"],
                capture_output=True
            )
            return {"level": level}
        
        elif action == "up":
            subprocess.run(
                ["powershell", "-Command",
                 f"$v = (Get-AudioDevice -PlaybackVolume).Volume; Set-AudioDevice -PlaybackVolume ([Math]::Min(100, $v + {step}))"],
                capture_output=True
            )
            return {"level": "increased", "step": step}
        
        elif action == "down":
            subprocess.run(
                ["powershell", "-Command",
                 f"$v = (Get-AudioDevice -PlaybackVolume).Volume; Set-AudioDevice -PlaybackVolume ([Math]::Max(0, $v - {step}))"],
                capture_output=True
            )
            return {"level": "decreased", "step": step}
        
        elif action == "mute":
            subprocess.run(
                ["powershell", "-Command", "Set-AudioDevice -PlaybackMute 1"],
                capture_output=True
            )
            return {"muted": True}
        
        elif action == "unmute":
            subprocess.run(
                ["powershell", "-Command", "Set-AudioDevice -PlaybackMute 0"],
                capture_output=True
            )
            return {"muted": False}
        
        return {"error": f"Unknown action: {action}"}
    
    except Exception as e:
        return {"error": str(e), "note": "Volume control not available"}


def _control_volume_macos(action: str, level: Optional[int], step: int) -> dict:
    """macOS volume control using osascript."""
    try:
        if action == "get":
            result = subprocess.run(
                ["osascript", "-e", "output volume of (get volume settings)"],
                capture_output=True,
                text=True,
            )
            current = int(result.stdout.strip())
            
            muted_result = subprocess.run(
                ["osascript", "-e", "output muted of (get volume settings)"],
                capture_output=True,
                text=True,
            )
            muted = muted_result.stdout.strip() == "true"
            
            return {"level": current, "muted": muted}
        
        elif action == "set" and level is not None:
            subprocess.run(
                ["osascript", "-e", f"set volume output volume {level}"],
                capture_output=True,
            )
            return {"level": level}
        
        elif action == "up":
            result = subprocess.run(
                ["osascript", "-e", "output volume of (get volume settings)"],
                capture_output=True,
                text=True,
            )
            current = int(result.stdout.strip())
            new_level = min(100, current + step)
            subprocess.run(
                ["osascript", "-e", f"set volume output volume {new_level}"],
                capture_output=True,
            )
            return {"level": new_level}
        
        elif action == "down":
            result = subprocess.run(
                ["osascript", "-e", "output volume of (get volume settings)"],
                capture_output=True,
                text=True,
            )
            current = int(result.stdout.strip())
            new_level = max(0, current - step)
            subprocess.run(
                ["osascript", "-e", f"set volume output volume {new_level}"],
                capture_output=True,
            )
            return {"level": new_level}
        
        elif action == "mute":
            subprocess.run(
                ["osascript", "-e", "set volume output muted true"],
                capture_output=True,
            )
            return {"muted": True}
        
        elif action == "unmute":
            subprocess.run(
                ["osascript", "-e", "set volume output muted false"],
                capture_output=True,
            )
            return {"muted": False}
        
        return {"error": f"Unknown action: {action}"}
    
    except Exception as e:
        return {"error": str(e)}


def _control_volume_linux(action: str, level: Optional[int], step: int) -> dict:
    """Linux volume control using amixer."""
    try:
        if action == "get":
            result = subprocess.run(
                ["amixer", "get", "Master"],
                capture_output=True,
                text=True,
            )
            output = result.stdout
            
            import re
            match = re.search(r"\[(\d+)%\]", output)
            current = int(match.group(1)) if match else 0
            
            muted = "[off]" in output
            return {"level": current, "muted": muted}
        
        elif action == "set" and level is not None:
            subprocess.run(
                ["amixer", "set", "Master", f"{level}%"],
                capture_output=True,
            )
            return {"level": level}
        
        elif action == "up":
            subprocess.run(
                ["amixer", "set", "Master", f"{step}%+"],
                capture_output=True,
            )
            return _control_volume_linux("get", None, step)
        
        elif action == "down":
            subprocess.run(
                ["amixer", "set", "Master", f"{step}%-"],
                capture_output=True,
            )
            return _control_volume_linux("get", None, step)
        
        elif action == "mute":
            subprocess.run(
                ["amixer", "set", "Master", "mute"],
                capture_output=True,
            )
            return {"muted": True}
        
        elif action == "unmute":
            subprocess.run(
                ["amixer", "set", "Master", "unmute"],
                capture_output=True,
            )
            return {"muted": False}
        
        return {"error": f"Unknown action: {action}"}
    
    except Exception as e:
        return {"error": str(e)}


def register_volume_tool(server) -> None:
    """Register the volume control tool with the MCP server."""
    parameters = [
        ToolParameter(
            name="action",
            type=ToolParameterType.STRING,
            description="Volume action: get, set, up, down, mute, unmute",
            required=True,
            enum=["get", "set", "up", "down", "mute", "unmute"],
        ),
        ToolParameter(
            name="level",
            type=ToolParameterType.INTEGER,
            description="Volume level (0-100) for 'set' action",
            required=False,
        ),
        ToolParameter(
            name="step",
            type=ToolParameterType.INTEGER,
            description="Volume change step for up/down (default: 10)",
            required=False,
            default=10,
        ),
    ]
    
    server.register_tool(
        name="control_volume",
        description="Control system audio volume (get, set, up, down, mute, unmute)",
        handler=control_volume,
        parameters=parameters,
    )
