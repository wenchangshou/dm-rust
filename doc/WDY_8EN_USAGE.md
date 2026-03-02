# WDY-8EN 8路电源时序器协议使用指南

## 概述

**设备**: WDY-8EN 8路电源时序器
**协议**: 自定义串口协议（通过串口转网口模块使用 TCP 通信）
**协议标识**: `wdy-8en`

### 功能列表

| 功能 | 命令 | 说明 |
|------|------|------|
| 全局开机 | `power_on` | 全部通道开机 |
| 全局关机 | `power_off` | 全部通道关机 |
| 通道开 | `channel_on` | 开启指定通道 (1-8) |
| 通道关 | `channel_off` | 关闭指定通道 (1-8) |
| 获取状态 | `get_status` | 获取设备完整状态（56字节） |
| 设置设备ID | `set_device_id` | 设置设备地址 |

---

## 通道配置

### 基本配置

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "wdy-8en",
      "description": "WDY-8EN 8路电源时序器",
      "arguments": {
        "addr": "172.168.1.246",
        "port": 1200,
        "device_id": 0
      }
    }
  ]
}
```

### 参数说明

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `addr` | string | ✅ | — | 设备 IP 地址（串口转网口模块地址） |
| `port` | number | — | `4196` | TCP 端口 |
| `device_id` | number | — | `0` | 设备 ID（0x00-0xFF） |

### 节点配置示例

```json
{
  "nodes": [
    { "global_id": 1, "channel_id": 1, "id": 1, "alias": "CH1" },
    { "global_id": 2, "channel_id": 1, "id": 2, "alias": "CH2" },
    { "global_id": 3, "channel_id": 1, "id": 3, "alias": "CH3" },
    { "global_id": 4, "channel_id": 1, "id": 4, "alias": "CH4" },
    { "global_id": 5, "channel_id": 1, "id": 5, "alias": "CH5" },
    { "global_id": 6, "channel_id": 1, "id": 6, "alias": "CH6" },
    { "global_id": 7, "channel_id": 1, "id": 7, "alias": "CH7" },
    { "global_id": 8, "channel_id": 1, "id": 8, "alias": "CH8" }
  ]
}
```

> **说明**: 节点的 `id` 对应通道编号（1-8），通过 `read`/`write` 接口可直接按节点读写。

---

## 协议原理

### 帧格式

所有命令均为 6 字节固定长度帧：

```
字节:  [0]    [1]    [2]    [3]    [4]    [5]
含义:  包头   类型   设备ID  命令   数据   包尾
固定:  0x55   --     --     --     --     0xAA
```

- **包头**: 固定 `0x55`
- **类型**: `0x5A`=控制命令, `0xFF`=查询命令
- **设备ID**: 默认 `0x00`，可通过 `set_device_id` 修改
- **包尾**: 固定 `0xAA`

### 控制命令帧

| 命令 | 帧内容 | 说明 |
|------|--------|------|
| 开机 | `55 5A ID 09 01 AA` | 全局开机 |
| 关机 | `55 5A ID 09 00 AA` | 全局关机 |
| CH1开 | `55 5A ID 01 01 AA` | 通道1开 |
| CH1关 | `55 5A ID 01 00 AA` | 通道1关 |
| CH2开 | `55 5A ID 02 01 AA` | 通道2开 |
| CH2关 | `55 5A ID 02 00 AA` | 通道2关 |
| CH3开 | `55 5A ID 03 01 AA` | 通道3开 |
| CH3关 | `55 5A ID 03 00 AA` | 通道3关 |
| CH4开 | `55 5A ID 04 01 AA` | 通道4开 |
| CH4关 | `55 5A ID 04 00 AA` | 通道4关 |
| CH5开 | `55 5A ID 05 01 AA` | 通道5开 |
| CH5关 | `55 5A ID 05 00 AA` | 通道5关 |
| CH6开 | `55 5A ID 06 01 AA` | 通道6开 |
| CH6关 | `55 5A ID 06 00 AA` | 通道6关 |
| CH7开 | `55 5A ID 07 01 AA` | 通道7开 |
| CH7关 | `55 5A ID 07 00 AA` | 通道7关 |
| CH8开 | `55 5A ID 08 01 AA` | 通道8开 |
| CH8关 | `55 5A ID 08 00 AA` | 通道8关 |
| 设置ID | `55 5A ID FF nn AA` | nn=新ID |

> **发送和返回内容相同**: 控制命令成功后，设备会回传与发送完全相同的帧。

### 查询命令帧

```
发送: 55 FF ID FF FF AA
返回: 56 字节状态数据
```

### 状态响应解析（56字节）

| 字节位置 | 长度 | 字段 | 说明 |
|---------|------|------|------|
| 1-2 | 2 | 包头 | 固定 `55 5A` |
| 3 | 1 | 设备ID | 当前设备地址 |
| 4-5 | 2 | CH1开机延时 | 高位在前 (例: `03 E7` = 999) |
| 6-7 | 2 | CH2开机延时 | 同上 |
| 8-9 | 2 | CH3开机延时 | 同上 |
| 10-11 | 2 | CH4开机延时 | 同上 |
| 12-13 | 2 | CH5开机延时 | 同上 |
| 14-15 | 2 | CH6开机延时 | 同上 |
| 16-17 | 2 | CH7开机延时 | 同上 |
| 18-19 | 2 | CH8开机延时 | 同上 |
| 20-21 | 2 | CH1关机延时 | 同上 |
| 22-23 | 2 | CH2关机延时 | 同上 |
| 24-25 | 2 | CH3关机延时 | 同上 |
| 26-27 | 2 | CH4关机延时 | 同上 |
| 28-29 | 2 | CH5关机延时 | 同上 |
| 30-31 | 2 | CH6关机延时 | 同上 |
| 32-33 | 2 | CH7关机延时 | 同上 |
| 34-35 | 2 | CH8关机延时 | 同上 |
| 36-43 | 8 | CH1-CH8开关状态 | 每字节: 01=开, 00=关 |
| 44 | 1 | 自启动状态 | 01=开, 00=关 |
| 45-46 | 2 | 电压 | 高位在前，单位 0.1V |
| 47-48 | 2 | 电流 | 高位在前，单位 0.01A |
| 49-52 | 4 | 功率 | 高位在前，单位 0.1W |
| 53-54 | 2 | 功率因数 | 高位在前，单位 0.01 |
| 55-56 | 2 | 包尾 | 固定 `FE AA` |

---

## HTTP API 使用

### 1. 简化接口 (read/write)

通过节点 ID 直接读写单个通道，ID 对应通道编号（1-8）。

#### 读取通道状态

```bash
curl -X POST http://localhost:18080/lspcapi/device/read \
  -H "Content-Type: application/json" \
  -d '{
    "global_id": 1
  }'
```

**响应**：
```json
{
  "state": 0,
  "message": "success",
  "data": { "value": 1 }
}
```
> `value` 为 `1` 表示开启，`0` 表示关闭。

#### 写入通道状态

```bash
# 开启 CH1
curl -X POST http://localhost:18080/lspcapi/device/write \
  -H "Content-Type: application/json" \
  -d '{
    "global_id": 1,
    "value": 1
  }'

# 关闭 CH3
curl -X POST http://localhost:18080/lspcapi/device/write \
  -H "Content-Type: application/json" \
  -d '{
    "global_id": 3,
    "value": 0
  }'
```

---

### 2. 扩展命令接口 (executeCommand)

#### power_on — 全局开机

```bash
curl -X POST http://localhost:18080/lspcapi/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "power_on",
    "params": {}
  }'
```

#### power_off — 全局关机

```bash
curl -X POST http://localhost:18080/lspcapi/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "power_off",
    "params": {}
  }'
```

#### channel_on — 开启指定通道

```bash
curl -X POST http://localhost:18080/lspcapi/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "channel_on",
    "params": { "channel": 3 }
  }'
```

#### channel_off — 关闭指定通道

```bash
curl -X POST http://localhost:18080/lspcapi/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "channel_off",
    "params": { "channel": 5 }
  }'
```

#### get_status — 获取设备完整状态

```bash
curl -X POST http://localhost:18080/lspcapi/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "get_status",
    "params": {}
  }'
```

**响应示例**：
```json
{
  "state": 0,
  "data": {
    "status": "success",
    "device_id": 0,
    "channels": {
      "1": { "on_delay": 1, "off_delay": 1, "state": true },
      "2": { "on_delay": 2, "off_delay": 2, "state": false },
      "3": { "on_delay": 3, "off_delay": 3, "state": true },
      "4": { "on_delay": 4, "off_delay": 4, "state": false },
      "5": { "on_delay": 5, "off_delay": 5, "state": true },
      "6": { "on_delay": 6, "off_delay": 6, "state": false },
      "7": { "on_delay": 7, "off_delay": 7, "state": true },
      "8": { "on_delay": 8, "off_delay": 8, "state": false }
    },
    "auto_start": false,
    "voltage": 220.5,
    "current": 1.23,
    "power": 271.2,
    "power_factor": 0.99
  }
}
```

#### set_device_id — 设置设备地址

```bash
curl -X POST http://localhost:18080/lspcapi/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "set_device_id",
    "params": { "new_id": 1 }
  }'
```

---

## 场景配置

通过场景实现自动化联动控制：

```json
{
  "scenes": [
    {
      "name": "open_all",
      "nodes": [
        { "id": 1, "value": 1, "delay": 0 },
        { "id": 2, "value": 1, "delay": 100 },
        { "id": 3, "value": 1, "delay": 200 },
        { "id": 4, "value": 1, "delay": 300 },
        { "id": 5, "value": 1, "delay": 400 },
        { "id": 6, "value": 1, "delay": 500 },
        { "id": 7, "value": 1, "delay": 600 },
        { "id": 8, "value": 1, "delay": 700 }
      ]
    },
    {
      "name": "close_all",
      "nodes": [
        { "id": 1, "value": 0, "delay": 0 },
        { "id": 2, "value": 0, "delay": 100 },
        { "id": 3, "value": 0, "delay": 200 },
        { "id": 4, "value": 0, "delay": 300 },
        { "id": 5, "value": 0, "delay": 400 },
        { "id": 6, "value": 0, "delay": 500 },
        { "id": 7, "value": 0, "delay": 600 },
        { "id": 8, "value": 0, "delay": 700 }
      ]
    }
  ]
}
```

---

## 注意事项

1. **通信方式**: 设备使用串口通信，需要通过串口转网口模块（如有人 USR 系列）转为 TCP。

2. **命令回显**: 所有控制命令（开机/关机/通道控制）发送后设备会回传相同的帧作为确认。如果回传内容与发送不一致，视为命令执行失败。

3. **状态查询**: 状态查询返回 56 字节数据，包含所有通道的延时设置、开关状态、以及电压/电流/功率等电气参数。

4. **设备ID**: 默认为 `0x00`。如果同一网段内有多台设备，需通过 `set_device_id` 设置不同的地址以避免冲突。

5. **连接方式**: 每次命令执行时创建 TCP 连接，命令完成后断开。对于高频操作场景建议适当增大命令间隔。
