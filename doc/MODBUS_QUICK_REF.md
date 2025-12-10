# Modbus 数据类型快速参考

## 命令格式

### 读取（带类型）
```json
{
  "channel": 1,
  "command": "read",
  "params": {
    "addr": 100,
    "type": "int16"
  }
}
```

### 写入（带类型）
```json
{
  "channel": 1,
  "command": "write",
  "params": {
    "addr": 100,
    "type": "float32",
    "value": 123.456
  }
}
```

## 数据类型速查表

| 类型 | 寄存器数 | 范围/精度 | 典型应用 |
|------|---------|----------|----------|
| **uint16** | 1 | 0 ~ 65535 | 转速、计数器 |
| **int16** | 1 | -32768 ~ 32767 | 温度（×10） |
| **uint32** | 2 | 0 ~ 4,294,967,295 | 累计流量、里程 |
| **int32** | 2 | -2.1亿 ~ 2.1亿 | 位置、坐标 |
| **float32** | 2 | ±3.4×10³⁸, 7位精度 | 压力、温度 |
| **float64** | 4 | ±1.7×10³⁰⁸, 15位精度 | 高精度测量 |
| **bool** | - | true/false | 开关、指示灯 |

## 字节序

| 类型 | 字节序 | 用途 |
|------|--------|------|
| uint32, int32, float32 | Big Endian | 标准工业设备 |
| uint32le, int32le, float32le | Little Endian | 某些PLC |

## 一行命令示例

```bash
# 读取 Int16
curl -X POST http://localhost:8080/device/execute -H "Content-Type: application/json" -d '{"channel":1,"command":"read","params":{"addr":100,"type":"int16"}}'

# 写入 Float32
curl -X POST http://localhost:8080/device/execute -H "Content-Type: application/json" -d '{"channel":1,"command":"write","params":{"addr":200,"type":"float32","value":98.765}}'

# 读取 UInt32
curl -X POST http://localhost:8080/device/execute -H "Content-Type: application/json" -d '{"channel":1,"command":"read","params":{"addr":300,"type":"uint32"}}'

# 写入 Bool
curl -X POST http://localhost:8080/device/execute -H "Content-Type: application/json" -d '{"channel":1,"command":"write","params":{"addr":5,"type":"bool","value":true}}'
```

## 常见场景

### 温度传感器（0.1°C 精度）
```bash
# 写入 25.6°C (存储为 256)
curl ... -d '{"channel":1,"command":"write","params":{"addr":100,"type":"int16","value":256}}'

# 读取后除以10得到实际温度
```

### 压力传感器
```bash
# 直接使用 float32
curl ... -d '{"channel":1,"command":"read","params":{"addr":200,"type":"float32"}}'
```

### 流量累计
```bash
# 使用 uint32 存储总流量
curl ... -d '{"channel":1,"command":"read","params":{"addr":300,"type":"uint32"}}'
```

### 控制继电器
```bash
# 使用 bool 控制线圈
curl ... -d '{"channel":1,"command":"write","params":{"addr":0,"type":"bool","value":true}}'
```

## Python 快速示例

```python
import requests

def read_typed(channel, addr, data_type):
    response = requests.post("http://localhost:8080/device/execute", json={
        "channel": channel,
        "command": "read",
        "params": {"addr": addr, "type": data_type}
    })
    return response.json()["data"]["value"]

def write_typed(channel, addr, value, data_type):
    response = requests.post("http://localhost:8080/device/execute", json={
        "channel": channel,
        "command": "write",
        "params": {"addr": addr, "type": data_type, "value": value}
    })
    return response.json()["code"] == 0

# 使用
temp = read_typed(1, 100, "int16") / 10.0  # 温度
pressure = read_typed(1, 200, "float32")   # 压力
write_typed(1, 5, True, "bool")            # 控制开关
```

## 配置示例

```json
{
  "nodes": [
    {
      "id": 1,
      "name": "温度传感器",
      "channel": 1,
      "data": {
        "addr": 100,
        "type": "int16",
        "scale": 0.1,
        "unit": "°C"
      }
    },
    {
      "id": 2,
      "name": "压力传感器",
      "channel": 1,
      "data": {
        "addr": 200,
        "type": "float32",
        "unit": "Pa"
      }
    }
  ]
}
```

## 故障排查

### 值不正确
- 检查字节序（Big Endian vs Little Endian）
- 尝试 `uint32` → `uint32le` 或 `float32` → `float32le`

### 浮点数精度损失
- Float32: 约7位有效数字
- Float64: 约15位有效数字
- 考虑使用更高精度类型

### Bool 类型错误
- Bool 使用线圈操作（FC01/FC05）
- 地址范围与寄存器不同

## 兼容性

原有的 `read`/`write` 接口仍然有效（默认 uint16）：

```bash
# 旧方式（默认 uint16）
curl -X POST http://localhost:8080/device/read -d '{"channel":1,"id":100}'

# 新方式（指定类型）
curl -X POST http://localhost:8080/device/execute -d '{"channel":1,"command":"read","params":{"addr":100,"type":"float32"}}'
```
