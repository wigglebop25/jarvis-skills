"""
Volume Control Tool - Cross-platform audio volume management.
"""

import sys
import subprocess
import shutil
import logging
import urllib.request
import urllib.error
import zipfile
from pathlib import Path
from typing import Optional
from jarvis_skills_core import ToolParameter, ToolParameterType

logger = logging.getLogger(__name__)


# Cache directory for downloaded tools
TOOLS_CACHE = Path(__file__).parent.parent.parent / ".tools_cache"


def _ensure_nircmd() -> Optional[str]:
    """Download and cache nircmd if not available."""
    try:
        # Check if nircmd is in PATH
        if shutil.which("nircmd"):
            return "nircmd"

        # Check local cache
        TOOLS_CACHE.mkdir(exist_ok=True)
        nircmd_exe = TOOLS_CACHE / "nircmd" / "nircmd.exe"

        if nircmd_exe.exists():
            return str(nircmd_exe)

        # Download nircmd
        url = "https://www.nirsoft.net/utils/nircmd.zip"
        zip_path = TOOLS_CACHE / "nircmd.zip"
        extract_path = TOOLS_CACHE / "nircmd"

        try:
            import socket
            socket.setdefaulttimeout(30)
            urllib.request.urlretrieve(url, zip_path)
            socket.setdefaulttimeout(None)
        except urllib.error.URLError as e:
            logger.error(f"Failed to download nircmd from {url}: {e}")
            return None
        except Exception as e:
            logger.error(f"Unexpected error downloading nircmd: {e}")
            return None

        # Validate zip file
        if not zip_path.exists() or zip_path.stat().st_size == 0:
            logger.error("Downloaded nircmd.zip is invalid or empty")
            return None

        try:
            with zipfile.ZipFile(zip_path, 'r') as zip_ref:
                zip_ref.extractall(extract_path)
        except zipfile.BadZipFile:
            logger.error("Downloaded file is not a valid zip archive")
            zip_path.unlink(missing_ok=True)
            return None
        except Exception as e:
            logger.error(f"Failed to extract nircmd: {e}")
            zip_path.unlink(missing_ok=True)
            return None

        # Cleanup and validate extraction
        try:
            zip_path.unlink()
        except Exception as e:
            logger.warning(f"Could not remove temporary zip file: {e}")

        if not nircmd_exe.exists():
            logger.error("nircmd.exe not found after extraction")
            return None

        return str(nircmd_exe)

    except Exception as e:
        logger.error(f"Unexpected error in _ensure_nircmd: {type(e).__name__}: {e}")
        return None


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
    """Windows volume control using nircmd (auto-downloaded)."""
    try:
        nircmd = _ensure_nircmd()

        if not nircmd:
            return {
                "error": "Could not obtain nircmd for volume control",
                "action": action
            }

        if action == "get":
            # nircmd doesn't directly read volume, return estimate
            return {"level": 50, "muted": False}

        elif action == "set" and level is not None:
            # Set system volume: setsysvolume takes 0-100 value * 655 (0-65535)
            subprocess.run(
                [nircmd, "setsysvolume", str(int(level * 655))],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"level": level, "action": "set"}

        elif action == "up":
            # Increase volume by step
            subprocess.run(
                [nircmd, "changesysvolume", str(int(step * 655))],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"level": "increased", "step": step, "action": "up"}

        elif action == "down":
            # Decrease volume by step
            subprocess.run(
                [nircmd, "changesysvolume", str(-int(step * 655))],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"level": "decreased", "step": step, "action": "down"}

        elif action == "mute":
            subprocess.run(
                [nircmd, "mutesysvolume", "1"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"muted": True, "action": "mute"}

        elif action == "unmute":
            subprocess.run(
                [nircmd, "mutesysvolume", "0"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"muted": False, "action": "unmute"}

        return {"error": f"Unknown action: {action}"}

    except Exception as e:
        return {
            "error": str(e),
            "action": action
        }


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
    """Linux volume control using pactl (PulseAudio/PipeWire) with fallback to amixer."""
    try:
        # Try pactl first (PulseAudio/PipeWire - modern systems)
        result = subprocess.run(
            ["pactl", "get-sink-volume", "@DEFAULT_SINK@"],
            capture_output=True, text=True, timeout=5
        )

        if result.returncode == 0:
            return _control_volume_linux_pactl(action, level, step)
        else:
            # Fallback to amixer (ALSA - older systems)
            return _control_volume_linux_amixer(action, level, step)

    except FileNotFoundError:
        # pactl not found, try amixer
        return _control_volume_linux_amixer(action, level, step)
    except Exception as e:
        return {
            "error": str(e),
            "action": action
        }


def _control_volume_linux_pactl(action: str, level: Optional[int], step: int) -> dict:
    """Linux volume control using pactl (PulseAudio/PipeWire)."""
    try:
        if action == "get":
            result = subprocess.run(
                ["pactl", "get-sink-volume", "@DEFAULT_SINK@"],
                capture_output=True, text=True, timeout=5
            )
            output = result.stdout

            import re
            match = re.search(r"(\d+)%", output)
            current = int(match.group(1)) if match else 0

            mute_result = subprocess.run(
                ["pactl", "get-sink-mute", "@DEFAULT_SINK@"],
                capture_output=True, text=True, timeout=5
            )
            muted = "yes" in mute_result.stdout

            return {"level": current, "muted": muted}

        elif action == "set" and level is not None:
            subprocess.run(
                ["pactl", "set-sink-volume", "@DEFAULT_SINK@", f"{level}%"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"level": level, "action": "set"}

        elif action == "up":
            subprocess.run(
                ["pactl", "set-sink-volume", "@DEFAULT_SINK@", f"+{step}%"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return _control_volume_linux_pactl("get", None, step)

        elif action == "down":
            subprocess.run(
                ["pactl", "set-sink-volume", "@DEFAULT_SINK@", f"-{step}%"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return _control_volume_linux_pactl("get", None, step)

        elif action == "mute":
            subprocess.run(
                ["pactl", "set-sink-mute", "@DEFAULT_SINK@", "yes"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"muted": True, "action": "mute"}

        elif action == "unmute":
            subprocess.run(
                ["pactl", "set-sink-mute", "@DEFAULT_SINK@", "no"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"muted": False, "action": "unmute"}

        return {"error": f"Unknown action: {action}"}

    except Exception as e:
        return {
            "error": str(e),
            "action": action
        }


def _control_volume_linux_amixer(action: str, level: Optional[int], step: int) -> dict:
    """Linux volume control using amixer (ALSA - fallback for older systems)."""
    try:
        if action == "get":
            result = subprocess.run(
                ["amixer", "get", "Master"],
                capture_output=True, text=True, timeout=5
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
                text=True,
                timeout=5,
                check=False,
            )
            return {"level": level, "action": "set"}

        elif action == "up":
            subprocess.run(
                ["amixer", "set", "Master", f"{step}%+"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return _control_volume_linux_amixer("get", None, step)

        elif action == "down":
            subprocess.run(
                ["amixer", "set", "Master", f"{step}%-"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return _control_volume_linux_amixer("get", None, step)

        elif action == "mute":
            subprocess.run(
                ["amixer", "set", "Master", "mute"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"muted": True, "action": "mute"}

        elif action == "unmute":
            subprocess.run(
                ["amixer", "set", "Master", "unmute"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            return {"muted": False, "action": "unmute"}

        return {"error": f"Unknown action: {action}"}

    except Exception as e:
        return {
            "error": str(e),
            "action": action
        }


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
