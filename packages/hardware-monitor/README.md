# jarvis-skills-hardware-monitor

Hardware monitoring and control tools.

## Features

- **System Info**: CPU, RAM, storage, network status
- **Volume Control**: Cross-platform audio management
- **Network Toggle**: WiFi, Bluetooth, Ethernet control

## Installation

```bash
uv pip install jarvis-skills-hardware-monitor

# For volume control on Windows
uv pip install jarvis-skills-hardware-monitor[windows]
```

## Usage

```python
from jarvis_skills_core import MCPServer
from jarvis_skills_hardware_monitor import register_all_tools

server = MCPServer()
register_all_tools(server)

# Execute tools
result = server.execute_tool_sync("get_system_info", include=["cpu"])
print(result.result)  # {"cpu": 25.0}

result = server.execute_tool_sync("control_volume", action="get")
print(result.result)  # {"level": 50, "muted": False}
```

## License

MIT

