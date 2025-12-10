# Modbus 协议使用指南

## 概述

本系统完整实现了 Modbus TCP 协议，支持所有标准的 Modbus 功能码：

- **读操作**：保持寄存器 (FC03)、输入寄存器 (FC04)、线圈 (FC01)、离散输入 (FC02)
- **写操作**：单个/多个保持寄存器 (FC06/FC16)、单个/多个线圈 (FC05/FC15)

## 配置示例

### 基本配置

```json
{
  "channels": [
    {
      "id": 1,
      "name": "Modbus设备",
      "statute": "modbus",
      "arguments": {
        "type": "tcp",
        "addr": "192.168.1.100",
        "port": 502,
        "slave_id": 1
      }
    }
  ],
  "nodes": [
    {
      "id": 1,
      "name": "温度传感器",
      "channel": 1
    }
  ]
}
```

### 参数说明

- `type`: 连接类型，目前仅支持 `"tcp"`
- `addr`: Modbus TCP 服务器 IP 地址
- `port`: Modbus TCP 端口（标准端口为 502）
- `slave_id`: 从站地址（可选，默认为 1）

## HTTP API 使用

### 1. 标准 read/write 接口

#### 读取单个寄存器

```bash
curl -X POST http://localhost:8080/device/read \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "id": 100
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": 42,
  "msg": "success"
}
```

#### 写入单个寄存器

```bash
curl -X POST http://localhost:8080/device/write \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "id": 100,
    "value": 42
  }'
```

**响应：**
```json
{
  "code": 0,
  "msg": "success"
}
```

### 2. 扩展命令接口

系统通过 `execute` 方法提供更多 Modbus 功能。

#### 读取多个保持寄存器

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read_holding_registers",
    "params": {
      "addr": 100,
      "count": 10
    }
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "data": [42, 43, 44, 45, 46, 47, 48, 49, 50, 51]
  },
  "msg": "success"
}
```

#### 读取输入寄存器

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read_input_registers",
    "params": {
      "addr": 200,
      "count": 5
    }
  }'
```

#### 写入多个寄存器

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write_multiple_registers",
    "params": {
      "addr": 100,
      "values": [10, 20, 30, 40, 50]
    }
  }'
```

#### 读取线圈

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read_coils",
    "params": {
      "addr": 0,
      "count": 16
    }
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "data": [true, false, true, true, false, false, true, false, true, true, false, true, false, false, false, true]
  },
  "msg": "success"
}
```

#### 写入单个线圈

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write_single_coil",
    "params": {
      "addr": 5,
      "value": true
    }
  }'
```

#### 写入多个线圈

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write_multiple_coils",
    "params": {
      "addr": 0,
      "values": [true, false, true, true, false, false, true, false]
    }
  }'
```

#### 读取离散输入

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read_discrete_inputs",
    "params": {
      "addr": 0,
      "count": 8
    }
  }'
```

## 支持的命令

| 命令 | 别名 | 功能码 | 说明 |
|------|------|--------|------|
| `read_holding_registers` | `read_holding` | FC03 | 读取保持寄存器 |
| `read_input_registers` | `read_input` | FC04 | 读取输入寄存器 |
| `write_single_register` | `write_single` | FC06 | 写单个保持寄存器 |
| `write_multiple_registers` | `write_multiple` | FC16 | 写多个保持寄存器 |
| `read_coils` | - | FC01 | 读取线圈状态 |
| `read_discrete_inputs` | `read_discrete` | FC02 | 读取离散输入 |
| `write_single_coil` | - | FC05 | 写单个线圈 |
| `write_multiple_coils` | - | FC15 | 写多个线圈 |

## 参数说明

### 读操作参数

- `addr`: 起始地址 (u16)
- `count`: 读取数量 (u16，可选，默认为 1)

### 写操作参数

**单个值：**
- `addr`: 寄存器/线圈地址 (u16)
- `value`: 值 (寄存器用 u16，线圈用 bool)

**多个值：**
- `addr`: 起始地址 (u16)
- `values`: 值数组 (寄存器用 u16[]，线圈用 bool[])

## 错误处理

当 Modbus 操作失败时，系统会返回相应的错误信息：

```json
{
  "code": 30002,
  "msg": "连接错误: Modbus TCP 连接失败: Connection refused"
}
```

常见错误代码：
- `30001`: 设备未找到
- `30002`: 连接错误
- `400`: 参数错误

## 完整示例

### Python 示例

```python
import requests
import json

BASE_URL = "http://localhost:8080"

# 读取多个寄存器
response = requests.post(f"{BASE_URL}/device/execute", json={
    "channel": 1,
    "command": "read_holding_registers",
    "params": {
        "addr": 100,
        "count": 10
    }
})

if response.status_code == 200:
    result = response.json()
    if result["code"] == 0:
        data = result["data"]["data"]
        print(f"读取到的数据: {data}")
    else:
        print(f"错误: {result['msg']}")
else:
    print(f"HTTP错误: {response.status_code}")

# 写入多个寄存器
response = requests.post(f"{BASE_URL}/device/execute", json={
    "channel": 1,
    "command": "write_multiple_registers",
    "params": {
        "addr": 100,
        "values": [10, 20, 30, 40, 50]
    }
})

if response.status_code == 200:
    result = response.json()
    print(f"写入结果: {result['msg']}")
```

### JavaScript/Node.js 示例

```javascript
const axios = require('axios');

const BASE_URL = 'http://localhost:8080';

async function readModbusRegisters(channel, addr, count) {
    try {
        const response = await axios.post(`${BASE_URL}/device/execute`, {
            channel: channel,
            command: 'read_holding_registers',
            params: {
                addr: addr,
                count: count
            }
        });
        
        if (response.data.code === 0) {
            return response.data.data.data;
        } else {
            throw new Error(response.data.msg);
        }
    } catch (error) {
        console.error('读取失败:', error.message);
        return null;
    }
}

async function writeModbusRegisters(channel, addr, values) {
    try {
        const response = await axios.post(`${BASE_URL}/device/execute`, {
            channel: channel,
            command: 'write_multiple_registers',
            params: {
                addr: addr,
                values: values
            }
        });
        
        return response.data.code === 0;
    } catch (error) {
        console.error('写入失败:', error.message);
        return false;
    }
}

// 使用示例
(async () => {
    // 读取
    const data = await readModbusRegisters(1, 100, 10);
    console.log('读取的数据:', data);
    
    // 写入
    const success = await writeModbusRegisters(1, 100, [10, 20, 30, 40, 50]);
    console.log('写入成功:', success);
})();
```

### Bash/curl 测试脚本

```bash
#!/bin/bash

BASE_URL="http://localhost:8080"
CHANNEL=1

echo "=== 测试 Modbus 协议 ==="

# 1. 读取保持寄存器
echo -e "\n1. 读取保持寄存器 (地址100, 数量10)"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"read_holding_registers\",
    \"params\": {
      \"addr\": 100,
      \"count\": 10
    }
  }" | jq '.'

# 2. 写入单个寄存器
echo -e "\n2. 写入单个寄存器 (地址100, 值42)"
curl -s -X POST "$BASE_URL/device/write" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"id\": 100,
    \"value\": 42
  }" | jq '.'

# 3. 读取单个寄存器
echo -e "\n3. 读取单个寄存器 (地址100)"
curl -s -X POST "$BASE_URL/device/read" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"id\": 100
  }" | jq '.'

# 4. 写入多个寄存器
echo -e "\n4. 写入多个寄存器 (地址100-104)"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"write_multiple_registers\",
    \"params\": {
      \"addr\": 100,
      \"values\": [10, 20, 30, 40, 50]
    }
  }" | jq '.'

# 5. 读取线圈
echo -e "\n5. 读取线圈 (地址0, 数量16)"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"read_coils\",
    \"params\": {
      \"addr\": 0,
      \"count\": 16
    }
  }" | jq '.'

# 6. 写入单个线圈
echo -e "\n6. 写入单个线圈 (地址5, 值true)"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"write_single_coil\",
    \"params\": {
      \"addr\": 5,
      \"value\": true
    }
  }" | jq '.'

# 7. 检查状态
echo -e "\n7. 检查通道状态"
curl -s -X GET "$BASE_URL/channel/status/$CHANNEL" | jq '.'

echo -e "\n=== 测试完成 ==="
```

## 注意事项

1. **连接管理**: 每次操作都会建立新的TCP连接，适用于大多数场景。如需保持长连接，可修改代码使用连接池。

2. **超时设置**: 默认使用 tokio-modbus 的超时设置。如需自定义，可在连接时设置超时参数。

3. **异常处理**: Modbus 协议异常（Exception Code）会被转换为 `ProtocolError` 返回。

4. **地址范围**: Modbus 地址范围为 0-65535 (u16)，请确保在有效范围内。

5. **串口支持**: 目前仅实现了 TCP 模式，串口模式（Modbus RTU）待后续实现。

## 测试工具推荐

- **diagslave**: Modbus 从站模拟器
- **modpoll**: Modbus 主站测试工具
- **PyModbus**: Python Modbus 库

启动测试服务器：
```bash
# 使用 diagslave 启动 Modbus TCP 从站模拟器
diagslave -m tcp -p 502
```
