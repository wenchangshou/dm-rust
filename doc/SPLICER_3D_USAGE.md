# 3D拼接处理器协议使用指南

本文档描述如何配置和使用 3D拼接处理器协议来控制拼接处理器设备切换场景。

## 1. 连接设置

### 网口方式 (TCP)
- 默认 TCP 端口: **5000**
- 默认 UDP 端口: **5002** (TCP端口 + 2)

### 串口方式
- 波特率: **115200**
- 数据位: **8**
- 停止位: **1**
- 校验位: **无**

## 2. 配置示例

在 `config.json` 中添加通道配置:

### TCP 连接方式

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "splicer3d",
      "arguments": {
        "use_tcp": true,
        "addr": "192.168.1.100",
        "port": 5000,
        "group": 1
      }
    }
  ],
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 1,
      "alias": "场景控制"
    }
  ],
  "scenes": [],
  "task_settings": {},
  "web_server": {
    "port": 8080
  }
}
```

### UDP 连接方式

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "splicer3d",
      "arguments": {
        "use_udp": true,
        "addr": "192.168.1.100",
        "port": 5002,
        "group": 1,
        "local_port": 5002
      }
    }
  ],
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 1,
      "alias": "场景控制"
    }
  ],
  "scenes": [],
  "task_settings": {},
  "web_server": {
    "port": 8080
  }
}
```

### 串口连接方式

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "splicer3d",
      "arguments": {
        "use_tcp": false,
        "port_name": "/dev/ttyUSB0",
        "baud_rate": 115200,
        "group": 1
      }
    }
  ],
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 1,
      "alias": "场景控制"
    }
  ],
  "scenes": [],
  "task_settings": {},
  "web_server": {
    "port": 8080
  }
}
```

### 参数说明

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `use_tcp` | boolean | 否 | 使用TCP连接(默认true)，否则使用串口 |
| `use_udp` | boolean | 否 | 使用UDP连接(默认false) |
| `addr` / `ip` | string | TCP模式必填 | 设备IP地址 |
| `port` | number | 否 | TCP端口(默认5000)，UDP端口(默认5002) |
| `local_port` / `udp_local_port` / `bind_port` | number | 否 | UDP本地绑定端口(默认随机端口) |
| `port_name` / `serial_port` | string | 串口模式必填 | 串口设备路径 |
| `baud_rate` | number | 否 | 波特率(默认115200) |
| `group` | number | 否 | 用户组(默认1) |

## 3. HTTP API 使用方法

所有请求发送到 `POST /lspcapi/device/executeCommand`

### 3.1 切换场景 (SetPreset)

通过 execute 命令切换场景:

```bash
curl -X POST http://localhost:8080/lspcapi/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "setPreset",
    "params": {
      "scene_id": 1
    }
  }'
```

**响应示例 (成功):**
```json
{
  "success": true,
  "message": "场景 1 切换成功"
}
```

**响应示例 (失败):**
```json
{
  "success": false,
  "message": "场景 1 切换失败"
}
```

---

### 3.2 通过 write 接口切换场景

使用节点写入方式切换场景 (更简便):

```bash
curl -X POST http://localhost:8080/lspcapi/device/write \
  -H 'Content-Type: application/json' \
  -d '{
    "id": 1,
    "value": 2
  }'
```

**说明:**
- `id`: 节点的 global_id (配置中定义)
- `value`: 场景编号 (从1开始)

---

### 3.3 读取当前场景

```bash
curl -X POST http://localhost:8080/lspcapi/device/read \
  -H 'Content-Type: application/json' \
  -d '{
    "id": 1
  }'
```

**响应示例:**
```json
{
  "value": 2
}
```

---

### 3.4 设置窗口信号源 (SetWinSrc)

设置指定窗口的信号源:

```bash
curl -X POST http://localhost:8080/lspcapi/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "setWinSrc",
    "params": {
      "window": 3,
      "slot": 5,
      "interface": 1,
      "type": 8
    }
  }'
```

**参数说明:**
- `window`: 窗口编号
- `slot`: 槽位编号
- `interface`: 接口编号
- `type`: 信号类型 (参考板卡接口代码表)

**信号类型参考:**
| 代码 | 类型 |
|------|------|
| 8 | DVI |
| ... | 参考设备手册 |

**注意:** setWinSrc 命令无返回信息。

---

## 4. 协议指令格式说明

### 切换场景指令
```
/SetPreset:d,{scene_id},{group};
```

**示例:**
- `/SetPreset:d,1,1;` - 切换到用户组1的场景1
- `/SetPreset:d,2,1;` - 切换到用户组1的场景2

**返回:**
- 成功: `/ack:d,1;`
- 失败: `/ack:d,0;`

### 设置窗口信号源指令
```
/setWinSrc:d,{window},{slot},{interface},{type},{group};
```

**示例:**
- `/setWinSrc:d,3,5,1,8,1;` - 把用户组1窗口3的信号源切换为槽位5的接口1，类型为DVI(8)

**返回:** 无返回信息

---

## 5. 运行示例

```bash
cargo run --features swagger -- -c config_splicer3d.json
```

## 6. 场景预设

如果需要定时或批量切换场景，可以使用场景配置:

```json
{
  "scenes": [
    {
      "name": "开机场景",
      "nodes": [
        { "id": 1, "value": 1 }
      ]
    },
    {
      "name": "展示场景",
      "nodes": [
        { "id": 1, "value": 2 }
      ]
    }
  ]
}
```

然后调用场景 API:
```bash
curl -X POST http://localhost:8080/lspcapi/device/playScene \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "展示场景"
  }'
```
