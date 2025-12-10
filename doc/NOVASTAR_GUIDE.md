# Novastar LED 控制器协议 - 使用指南

## 协议概述

Novastar LED 控制器通讯协议支持 TCP 和 RS232 两种通信方式：

- **TCP 模式**: 端口 15200
- **RS232 模式**: 波特率 115200, 8N1

## 支持功能

1. **读取设备 Mode ID** - 检查设备连接和识别设备型号
2. **场景加载** - 加载预设场景 (1-10)

---

## 配置示例

### TCP 模式 (默认)

```json
{
  "channels": [{
    "channel_id": 1,
    "statute": "novastar",
    "arguments": {
      "use_tcp": true,
      "addr": "192.168.1.100",
      "port": 15200
    }
  }]
}
```

### RS232 串口模式

```json
{
  "channels": [{
    "channel_id": 1,
    "statute": "novastar",
    "arguments": {
      "use_tcp": false,
      "port_name": "/dev/ttyUSB0",
      "baud_rate": 115200
    }
  }]
}
```

---

## HTTP API 调用

### 1. 读取设备 Mode ID

```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "read_mode_id",
    "arguments": {}
  }'
```

**响应示例**:
```json
{
  "state": 10000,
  "message": "方法调用成功",
  "data": {
    "success": true,
    "mode_id": "[AA, 55, ...]"
  }
}
```

### 2. 加载场景 (1-10)

```bash
# 加载场景 1
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "load_scene",
    "arguments": {
      "scene_id": 1
    }
  }'
```

**响应示例**:
```json
{
  "state": 10000,
  "message": "方法调用成功",
  "data": {
    "success": true,
    "message": "场景 1 加载成功"
  }
}
```

### 3. 通过节点接口加载场景

```bash
# 加载场景 3 (通过 global_id=3)
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{
    "global_id": 3,
    "value": 1
  }'
```

---

## 场景命令对照表

| 场景编号 | 命令数据 (HEX) |
|---------|--------------|
| 1 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 00 ba 56 |
| 2 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 01 bb 56 |
| 3 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 02 bc 56 |
| 4 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 03 bd 56 |
| 5 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 04 be 56 |
| 6 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 05 bf 56 |
| 7 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 06 c0 56 |
| 8 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 07 c1 56 |
| 9 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 08 c2 56 |
| 10 | 55 aa 00 00 fe 00 00 00 00 00 01 00 00 01 51 13 01 00 09 c3 56 |

---

## 协议细节

### 帧格式

- **帧头**: `55 AA`
- **帧尾**: `56`
- **响应帧头**: `AA 55`

### 校验和计算

```
SUM = data[2..18] + 0x5555
SUM_L = SUM & 0xFF  (低8位)
SUM_H = (SUM >> 8) & 0xFF  (高8位)
```

### 场景加载成功响应

```
aa 55 00 00 00 fe 00 00 00 00 01 00 00 01 51 13 00 00 b9 56
```

---

## Python 示例

```python
import requests

class NovastarClient:
    def __init__(self, base_url="http://localhost:8080", channel_id=1):
        self.base_url = base_url
        self.channel_id = channel_id
    
    def read_mode_id(self):
        """读取设备 Mode ID"""
        response = requests.post(
            f"{self.base_url}/device/callMethod",
            json={
                "channel_id": self.channel_id,
                "method_name": "read_mode_id",
                "arguments": {}
            }
        )
        return response.json()
    
    def load_scene(self, scene_id):
        """加载场景 (1-10)"""
        response = requests.post(
            f"{self.base_url}/device/callMethod",
            json={
                "channel_id": self.channel_id,
                "method_name": "load_scene",
                "arguments": {"scene_id": scene_id}
            }
        )
        return response.json()

# 使用示例
client = NovastarClient()

# 读取设备信息
print(client.read_mode_id())

# 加载场景 1
print(client.load_scene(1))

# 切换到场景 5
print(client.load_scene(5))
```

---

## 故障排查

### TCP 模式

1. **连接失败**
   - 检查 IP 地址和端口 (默认 15200)
   - 确认网络连通性: `ping 192.168.1.100`
   - 检查防火墙设置

2. **响应超时**
   - 设备可能离线或未响应
   - 检查设备供电
   - 验证设备 TCP 端口是否开启

### RS232 模式

1. **串口打开失败**
   - 检查串口设备: `ls -l /dev/ttyUSB*`
   - 添加用户到 dialout 组: `sudo usermod -a -G dialout $USER`
   - 检查串口权限: `sudo chmod 666 /dev/ttyUSB0`

2. **波特率错误**
   - 确认设备波特率为 115200
   - 检查串口线质量

3. **通信失败**
   - 检查 RS232 接线 (TXD <-> RXD, GND <-> GND)
   - 验证串口参数: 115200, 8N1

---

## 测试脚本

```bash
#!/bin/bash

echo "=== Novastar LED 控制器测试 ==="
echo ""

# 1. 读取 Mode ID
echo "【测试1】读取设备 Mode ID"
curl -s -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"read_mode_id","arguments":{}}' | jq
echo ""

# 2. 加载场景 1
echo "【测试2】加载场景 1"
curl -s -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"load_scene","arguments":{"scene_id":1}}' | jq
sleep 2
echo ""

# 3. 加载场景 5
echo "【测试3】加载场景 5"
curl -s -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"load_scene","arguments":{"scene_id":5}}' | jq
sleep 2
echo ""

# 4. 通过节点接口加载场景 3
echo "【测试4】通过节点接口加载场景 3"
curl -s -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"global_id":3,"value":1}' | jq
echo ""

echo "=== 测试完成 ==="
```

---

## 快速开始

1. **配置文件**: 使用 `config.novastar.json`
2. **启动服务**: 
   ```bash
   ./target/release/dm-rust -c config.novastar.json -l info
   ```
3. **测试连接**:
   ```bash
   curl -X POST http://localhost:8080/device/callMethod \
     -H 'Content-Type: application/json' \
     -d '{"channel_id":1,"method_name":"read_mode_id","arguments":{}}'
   ```

---

## 技术支持

- 协议版本: V1.0
- TCP 端口: 15200
- RS232 波特率: 115200
- 支持场景数: 10 个

**更新日期**: 2025-11-11
