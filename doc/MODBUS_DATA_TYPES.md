# Modbus 数据类型使用指南

## 概述

Modbus 协议现在支持多种数据类型，可以正确处理不同格式的数据（整数、浮点数、布尔值等）。

## 支持的数据类型

| 数据类型 | 别名 | 寄存器数 | 字节序 | 说明 |
|---------|------|---------|--------|------|
| `uint16` | `u16` | 1 | - | 无符号16位整数 (0-65535) |
| `int16` | `i16` | 1 | - | 有符号16位整数 (-32768 ~ 32767) |
| `uint32` | `u32` | 2 | Big Endian | 无符号32位整数 |
| `int32` | `i32` | 2 | Big Endian | 有符号32位整数 |
| `uint32le` | `u32le` | 2 | Little Endian | 无符号32位整数（小端） |
| `int32le` | `i32le` | 2 | Little Endian | 有符号32位整数（小端） |
| `float32` | `float`, `f32` | 2 | Big Endian | 32位浮点数 |
| `float32le` | `floatle`, `f32le` | 2 | Little Endian | 32位浮点数（小端） |
| `float64` | `double`, `f64` | 4 | Big Endian | 64位浮点数 |
| `bool` | `boolean`, `bit` | - | - | 布尔值（使用线圈） |

## 配置示例

### 基本配置（与之前相同）

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
  ]
}
```

## 使用示例

### 1. 读取不同数据类型

#### 读取 Int16（温度传感器）

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read",
    "params": {
      "addr": 100,
      "type": "int16"
    }
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "value": -25,
    "type": "int16",
    "registers": [65511]
  },
  "msg": "success"
}
```

#### 读取 Float32（压力传感器）

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read",
    "params": {
      "addr": 200,
      "type": "float32"
    }
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "value": 123.456,
    "type": "float32",
    "registers": [17000, 58982]
  },
  "msg": "success"
}
```

#### 读取 UInt32（计数器）

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read",
    "params": {
      "addr": 300,
      "type": "uint32"
    }
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "value": 1234567890,
    "type": "uint32",
    "registers": [18838, 722]
  },
  "msg": "success"
}
```

#### 读取 Float64（高精度数据）

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read",
    "params": {
      "addr": 400,
      "type": "float64"
    }
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "value": 3.141592653589793,
    "type": "float64",
    "registers": [16457, 8699, 33462, 50848]
  },
  "msg": "success"
}
```

#### 读取 Bool（线圈状态）

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read",
    "params": {
      "addr": 5,
      "type": "bool"
    }
  }'
```

**响应：**
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "value": true,
    "type": "bool"
  },
  "msg": "success"
}
```

### 2. 写入不同数据类型

#### 写入 Int16

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write",
    "params": {
      "addr": 100,
      "type": "int16",
      "value": -123
    }
  }'
```

#### 写入 Float32

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write",
    "params": {
      "addr": 200,
      "type": "float32",
      "value": 98.765
    }
  }'
```

#### 写入 UInt32

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write",
    "params": {
      "addr": 300,
      "type": "uint32",
      "value": 9876543210
    }
  }'
```

#### 写入 UInt32LE（小端序）

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write",
    "params": {
      "addr": 350,
      "type": "uint32le",
      "value": 1234567
    }
  }'
```

#### 写入 Float32LE（小端序）

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write",
    "params": {
      "addr": 250,
      "type": "float32le",
      "value": 45.678
    }
  }'
```

#### 写入 Bool

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write",
    "params": {
      "addr": 5,
      "type": "bool",
      "value": true
    }
  }'
```

#### 写入 Float64

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "write",
    "params": {
      "addr": 400,
      "type": "float64",
      "value": 3.141592653589793
    }
  }'
```

## 默认行为

如果不指定 `type` 参数，默认使用 `uint16`：

```bash
# 等同于 type: "uint16"
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 1,
    "command": "read",
    "params": {
      "addr": 100
    }
  }'
```

## 字节序说明

### Big Endian（大端序，默认）

- 高位字节在前，低位字节在后
- 适用于大多数工业设备
- 类型：`uint32`, `int32`, `float32`, `float64`

**示例：** 值 `0x12345678` 存储为：
- 寄存器1: `0x1234`
- 寄存器2: `0x5678`

### Little Endian（小端序）

- 低位字节在前，高位字节在后
- 适用于某些特定设备（如部分 PLC）
- 类型：`uint32le`, `int32le`, `float32le`

**示例：** 值 `0x12345678` 存储为：
- 寄存器1: `0x5678`
- 寄存器2: `0x1234`

## 完整示例

### Python 脚本

```python
import requests
import json

BASE_URL = "http://localhost:8080"

def read_modbus_typed(channel, addr, data_type):
    """读取指定类型的 Modbus 数据"""
    response = requests.post(f"{BASE_URL}/device/execute", json={
        "channel": channel,
        "command": "read",
        "params": {
            "addr": addr,
            "type": data_type
        }
    })
    
    if response.status_code == 200:
        result = response.json()
        if result["code"] == 0:
            return result["data"]["value"]
    return None

def write_modbus_typed(channel, addr, value, data_type):
    """写入指定类型的 Modbus 数据"""
    response = requests.post(f"{BASE_URL}/device/execute", json={
        "channel": channel,
        "command": "write",
        "params": {
            "addr": addr,
            "type": data_type,
            "value": value
        }
    })
    
    return response.status_code == 200 and response.json()["code"] == 0

# 使用示例
channel = 1

# 读取温度（Int16）
temperature = read_modbus_typed(channel, 100, "int16")
print(f"温度: {temperature}°C")

# 读取压力（Float32）
pressure = read_modbus_typed(channel, 200, "float32")
print(f"压力: {pressure} Pa")

# 读取计数器（UInt32）
counter = read_modbus_typed(channel, 300, "uint32")
print(f"计数: {counter}")

# 写入设定值（Float32）
success = write_modbus_typed(channel, 250, 75.5, "float32")
print(f"写入{'成功' if success else '失败'}")

# 控制开关（Bool）
success = write_modbus_typed(channel, 5, True, "bool")
print(f"开关控制{'成功' if success else '失败'}")
```

### JavaScript 示例

```javascript
const axios = require('axios');

const BASE_URL = 'http://localhost:8080';

async function readModbusTyped(channel, addr, dataType) {
    try {
        const response = await axios.post(`${BASE_URL}/device/execute`, {
            channel: channel,
            command: 'read',
            params: {
                addr: addr,
                type: dataType
            }
        });
        
        if (response.data.code === 0) {
            return response.data.data.value;
        }
    } catch (error) {
        console.error('读取失败:', error.message);
    }
    return null;
}

async function writeModbusTyped(channel, addr, value, dataType) {
    try {
        const response = await axios.post(`${BASE_URL}/device/execute`, {
            channel: channel,
            command: 'write',
            params: {
                addr: addr,
                type: dataType,
                value: value
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
    const channel = 1;
    
    // 读取不同类型的数据
    const temp = await readModbusTyped(channel, 100, 'int16');
    console.log(`温度: ${temp}°C`);
    
    const pressure = await readModbusTyped(channel, 200, 'float32');
    console.log(`压力: ${pressure} Pa`);
    
    const counter = await readModbusTyped(channel, 300, 'uint32');
    console.log(`计数: ${counter}`);
    
    // 写入数据
    await writeModbusTyped(channel, 250, 85.5, 'float32');
    await writeModbusTyped(channel, 5, true, 'bool');
})();
```

## 数据类型选择指南

### 温度传感器
- **Int16**: 适用于 -327.68°C ~ 327.67°C（精度0.01°C）
- **Float32**: 适用于更大范围或更高精度

### 压力传感器
- **Float32**: 推荐用于精确测量

### 计数器/累加器
- **UInt32**: 0 ~ 4,294,967,295
- **UInt32LE**: 某些 PLC 使用小端序

### 开关/继电器
- **Bool**: 使用线圈操作

### 高精度科学数据
- **Float64**: 双精度浮点数

### 位置/编码器
- **Int32**: 支持正负值
- **UInt32**: 仅正值

## 错误处理

### 值超出范围

```json
{
  "code": 400,
  "msg": "配置错误: 值超出UInt16范围"
}
```

### 不支持的数据类型

```json
{
  "code": 400,
  "msg": "配置错误: 不支持的数据类型: xyz"
}
```

### 寄存器数据不足

```json
{
  "code": 30002,
  "msg": "协议错误: 寄存器数据不足"
}
```

## 测试脚本

```bash
#!/bin/bash

BASE_URL="http://localhost:8080"
CHANNEL=1

echo "=== Modbus 数据类型测试 ==="

# 测试 Int16
echo -e "\n1. 写入 Int16: -123"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"write","params":{"addr":100,"type":"int16","value":-123}}' | jq '.'

echo -e "\n2. 读取 Int16"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"read","params":{"addr":100,"type":"int16"}}' | jq '.'

# 测试 Float32
echo -e "\n3. 写入 Float32: 98.765"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"write","params":{"addr":200,"type":"float32","value":98.765}}' | jq '.'

echo -e "\n4. 读取 Float32"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"read","params":{"addr":200,"type":"float32"}}' | jq '.'

# 测试 UInt32
echo -e "\n5. 写入 UInt32: 123456789"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"write","params":{"addr":300,"type":"uint32","value":123456789}}' | jq '.'

echo -e "\n6. 读取 UInt32"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"read","params":{"addr":300,"type":"uint32"}}' | jq '.'

# 测试 Bool
echo -e "\n7. 写入 Bool: true"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"write","params":{"addr":5,"type":"bool","value":true}}' | jq '.'

echo -e "\n8. 读取 Bool"
curl -s -X POST "$BASE_URL/device/execute" -H "Content-Type: application/json" \
  -d '{"channel":1,"command":"read","params":{"addr":5,"type":"bool"}}' | jq '.'

echo -e "\n=== 测试完成 ==="
```

## 注意事项

1. **寄存器地址连续性**: 
   - 32位数据类型需要2个连续寄存器
   - 64位数据类型需要4个连续寄存器

2. **字节序匹配**:
   - 确保与设备的字节序设置一致
   - 如果读取的值不正确，尝试切换字节序

3. **浮点数精度**:
   - Float32 约7位有效数字
   - Float64 约15位有效数字

4. **Bool 类型特殊性**:
   - 使用 Modbus 线圈操作（FC01/FC05）
   - 不使用保持寄存器

5. **数据范围验证**:
   - 写入前会自动验证值是否在类型范围内
   - 超出范围会返回错误

## 兼容性

该数据类型支持与原有的读写接口完全兼容：

```bash
# 原有方式仍然可用（默认 UInt16）
curl -X POST http://localhost:8080/device/read \
  -H "Content-Type: application/json" \
  -d '{"channel": 1, "id": 100}'

# 新方式（支持类型）
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{"channel": 1, "command": "read", "params": {"addr": 100, "type": "int16"}}'
```
