# Computer Control Protocol Usage Guide

This guide describes how to configure and use the Computer Control Protocol to manage computers (Wake-on-LAN and Remote UDP Commands) and monitor their status via heartbeats or UDP pings.

## 1. Configuration

Create or update your `config.json` with the following channel configuration:

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "computerControl",
      "arguments": {
        "mac_address": [
          {
            "id": 101,
            "mac": "00:11:22:33:44:55",
            "ip": "192.168.11.101",
            "port": 8888
          },
          {
            "id": 102,
            "mac": "AA:BB:CC:DD:EE:FF",
            "ip": "192.168.11.102",
            "port": 8888
          }
        ],
        "broadcast_addr": "192.168.11.255",
        "wol_port": 9,
        "shutdown_port": 9
      }
    }
  ]
}
```

*   **mac_address**: parameters array, each containing:
    *   `id`: unique ID for the computer within the channel.
    *   `mac`: MAC address string (required for WOL).
    *   `ip`: IP address string (required for direct UDP commands and Ping).
    *   `port`: UDP port the remote computer listens on (required for direct UDP commands and Ping).
*   **broadcast_addr**: Network broadcast address for WOL and legacy broadcast shutdown.
*   **wol_port**: UDP port for WOL packets (default: 9).
*   **shutdown_port**: Default UDP port for broadcast shutdown commands.

## 2. HTTP API Methods

All requests should be sent to `POST /lspcapi/device/executeCommand`.

### 2.1 Wake (WOL)
Sends a Magic Packet to the network.

**By ID:**
```bash
curl -X POST http://localhost:8080/lspcapi/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "powerOn",
    "params": { "id": 101 }
  }'
```

---

### 2.2 Shutdown
Sends a `"shutdown"` string via UDP. If an IP/Port is configured for the ID, it sends it directly to that IP. Otherwise, it broadcasts the MAC address.

**By ID:**
```bash
curl -X POST http://localhost:8080/lspcapi/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "powerOff",
    "params": { "id": 101 }
  }'
```

---

### 2.3 Custom Method (New)
Allows sending arbitrary text commands (e.g., mute, unmute) to a specific computer's IP and Port.

**Example: Mute PC**
```bash
curl -X POST http://localhost:8080/lspcapi/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "method",
    "params": {
      "id": 101,
      "method": "mute"
    }
  }'
```

---

### 2.4 Heartbeat Reporting (Optional)
Computers can actively report they are online by sending their MAC.

```bash
curl -X POST http://localhost:8080/lspcapi/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "heartbeat",
    "params": { "mac": "00:11:22:33:44:55" }
  }'
```

---

### 2.5 Status Check
Status (`1` for online, `0` for offline) is determined by:
1.  **UDP Ping:** The system sends `"ping"` to the computer's IP/Port. If it replies `"pong"` within 500ms, it is **Online**.
2.  **Heartbeat Fallback:** If Ping fails or is not configured, it checks if a heartbeat was received in the last 10 seconds.

**Read Status via Node API:** (Assuming node `global_id: 1` is mapped to ID `101`)
```bash
curl -X POST http://localhost:8080/lspcapi/device/read \
  -H 'Content-Type: application/json' \
  -d '{ "id": 1 }'
```

## 3. Remote Side Implementation (Reference)

You can test the connectivity using `nc` (netcat) on the target machines to simulate the agent:

#### Test Ping Response (Agent Simulation)
```bash
# Listen for ping and reply pong
while true; do echo -n "pong" | nc -u -l 8888 -q 0; done
```

#### Triggering commands from shell (Manual Test)
```bash
# Ping
echo -n "ping" | nc -u -w1 <TargetIP> 8888
# Expected response: pong

# Shutdown
echo -n "shutdown" | nc -u -w0 <TargetIP> 8888

# Mute/Unmute
echo -n "mute" | nc -u -w0 <TargetIP> 8888
echo -n "unmute" | nc -u -w0 <TargetIP> 8888
```

## 4. Node Mapping for Scene Control

To use these in scenes, map them in the `nodes` section:

```json
{
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 101,
      "category": "pc",
      "alias": "Main Office PC"
    }
  ]
}
```

*   `write(global_id, 1)` triggers `wake`.
*   `write(global_id, 0)` triggers `shutdown`.
