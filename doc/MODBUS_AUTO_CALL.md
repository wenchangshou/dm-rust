# Modbus 自动召唤功能文档

## 概述

Modbus 自动召唤功能允许系统定时自动读取 Modbus 设备的数据，并将数据缓存在内存中。当 HTTP API 读取节点数据时，将优先从缓存读取，避免频繁访问设备，提高响应速度。

## 功能特性

- ✅ 支持定时自动召唤 Modbus 数据
- ✅ 支持多个召唤任务配置（不同地址区间、不同间隔）
- ✅ 支持 4 种功能码：`holding`（保持寄存器）、`input`（输入寄存器）、`coil`（线圈）、`discrete`（离散输入）
- ✅ 数据自动缓存到内存
- ✅ HTTP read/write API 优先从缓存读取
- ✅ 节点支持数据点映射配置（地址、数据类型、缩放比例）

## 配置说明

### 1. 通道配置（Channel）

在 `channels` 数组中添加 `auto_call` 配置：

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "modbus",
      "arguments": {
        "type": "tcp",
        "addr": "192.168.200.23",
        "port": 502,
        "slave_id": 1
      },
      "auto_call": [
        {
          "function": "holding",
          "start_addr": 0,
          "count": 100,
          "interval_ms": 1000
        },
        {
          "function": "input",
          "start_addr": 200,
          "count": 50,
          "interval_ms": 2000
        }
      ]
    }
  ]
}
```

#### `auto_call` 字段说明

| 字段 | 类型 | 说明 | 示例 |
|------|------|------|------|
| `function` | string | 功能码类型 | `"holding"`, `"input"`, `"coil"`, `"discrete"` |
| `start_addr` | number | 起始地址 | `0`, `100`, `200` |
| `count` | number | 读取数量（寄存器/线圈数量） | `100`, `50` |
| `interval_ms` | number | 召唤间隔（毫秒） | `1000` (1秒), `5000` (5秒) |

### 2. 节点配置（Node）

在 `nodes` 数组中添加 `data_point` 配置：

```json
{
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 1,
      "category": "sensor",
      "alias": "温度传感器1",
      "data_point": {
        "type": "int16",
        "addr": 20,
        "scale": 0.1,
        "unit": "°C"
      }
    },
    {
      "global_id": 2,
      "channel_id": 1,
      "id": 2,
      "category": "sensor",
      "alias": "湿度传感器1",
      "data_point": {
        "type": "uint16",
        "addr": 21,
        "scale": 0.01,
        "unit": "%RH"
      }
    },
    {
      "global_id": 3,
      "channel_id": 1,
      "id": 3,
      "category": "sensor",
      "alias": "压力传感器1",
      "data_point": {
        "type": "float32",
        "addr": 30,
        "unit": "kPa"
      }
    }
  ]
}
```

#### `data_point` 字段说明

| 字段 | 类型 | 必填 | 说明 | 示例 |
|------|------|------|------|------|
| `type` | string | ✅ | 数据类型 | `"int16"`, `"uint16"`, `"float32"`, `"int32"` |
| `addr` | number | ✅ | 寄存器地址 | `20`, `30`, `100` |
| `scale` | number | ❌ | 缩放比例（原始值 × scale） | `0.1`, `0.01`, `10` |
| `unit` | string | ❌ | 数据单位（仅用于说明） | `"°C"`, `"%RH"`, `"kPa"` |

#### 支持的数据类型

| 类型 | 说明 | 寄存器数量 | 字节序 |
|------|------|------------|--------|
| `uint16` / `u16` | 无符号16位整数 | 1 | - |
| `int16` / `i16` | 有符号16位整数 | 1 | - |
| `uint32` / `u32` | 无符号32位整数 | 2 | Big Endian |
| `int32` / `i32` | 有符号32位整数 | 2 | Big Endian |
| `uint32le` / `u32le` | 无符号32位整数 | 2 | Little Endian |
| `int32le` / `i32le` | 有符号32位整数 | 2 | Little Endian |
| `float32` / `float` / `f32` | 32位浮点数 | 2 | Big Endian |
| `float32le` / `floatle` / `f32le` | 32位浮点数 | 2 | Little Endian |
| `float64` / `double` / `f64` | 64位浮点数 | 4 | Big Endian |
| `bool` / `boolean` / `bit` | 布尔值（线圈） | 1 | - |

## 使用示例

### 完整配置文件

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "modbus",
      "arguments": {
        "type": "tcp",
        "addr": "192.168.200.23",
        "port": 502,
        "slave_id": 1
      },
      "auto_call": [
        {
          "function": "holding",
          "start_addr": 0,
          "count": 100,
          "interval_ms": 1000
        }
      ]
    }
  ],
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 1,
      "category": "sensor",
      "alias": "温度传感器1",
      "data_point": {
        "type": "int16",
        "addr": 20,
        "scale": 0.1,
        "unit": "°C"
      }
    },
    {
      "global_id": 2,
      "channel_id": 1,
      "id": 2,
      "category": "sensor",
      "alias": "湿度传感器1",
      "data_point": {
        "type": "uint16",
        "addr": 21,
        "scale": 0.01,
        "unit": "%RH"
      }
    }
  ],
  "scenes": [],
  "web_server": {
    "port": 8080
  }
}
```

### HTTP API 读取节点数据

```bash
# 读取温度传感器（从缓存读取，速度快）
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{"id": 1}'

# 响应示例：
# {"status":"success","value":250}  # 实际温度 = 250 × 0.1 = 25.0°C
```

### HTTP API 读取原始 Modbus 数据

```bash
# 直接读取指定地址的数据（支持指定数据类型）
curl -X POST http://localhost:8080/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "read_typed",
    "params": {
      "addr": 20,
      "type": "int16",
      "use_cache": true
    }
  }'

# 响应示例：
# {
#   "status": "success",
#   "value": 250,
#   "type": "int16",
#   "from_cache": true
# }
```

## 工作原理

1. **启动时初始化**：系统启动时，根据 `auto_call` 配置为每个召唤任务创建一个后台任务
2. **定时召唤**：后台任务按照 `interval_ms` 定时读取指定地址区间的数据
3. **数据缓存**：读取到的数据存储在内存中的 `HashMap<u16, (Value, String, Instant)>`
4. **缓存读取**：当 HTTP API 调用 `read` 或 `read_typed` 时，优先从缓存读取
5. **缓存未命中**：如果缓存中没有数据，直接从设备读取并更新缓存

## 性能优势

- **减少设备访问**：避免频繁建立 Modbus 连接
- **快速响应**：从内存读取，响应时间从 30-50ms 降低到 <1ms
- **降低设备负载**：减少对 Modbus 设备的访问频率
- **数据一致性**：定时更新确保数据及时性

## 注意事项

1. **地址范围**：`auto_call` 的地址范围应包含所有节点的 `data_point.addr`
2. **召唤间隔**：根据实际需求设置合理的间隔，过短会增加网络负载
3. **内存占用**：缓存数据存储在内存中，大量数据点会增加内存占用
4. **数据时效性**：缓存数据的更新频率取决于 `interval_ms` 设置
5. **设备支持**：确保 Modbus 设备支持相应的功能码和地址范围

## 故障排查

### 日志查看

启动程序时使用 debug 日志级别：

```bash
./target/release/dm-rust -c config.modbus_types.json -l debug
```

关键日志信息：

```
INFO dm_rust::protocols::modbus: 启动 1 个自动召唤任务
DEBUG tokio_modbus::service::tcp: Call ReadHoldingRegisters(0, 100)
DEBUG dm_rust::protocols::modbus: 自动召唤成功: 保持寄存器 addr=0 count=100
```

### 常见错误

#### `IllegalDataAddress`

```
WARN dm_rust::protocols::modbus: 自动召唤失败: 协议错误: Modbus异常: IllegalDataAddress
```

**原因**：设备不支持该地址范围或地址不存在

**解决**：检查 `start_addr` 和 `count` 配置，确保在设备支持的地址范围内

#### 连接超时

**原因**：设备地址错误或设备离线

**解决**：检查 `addr` 和 `port` 配置，确认设备在线

## 扩展阅读

- [Modbus 协议文档](MODBUS_USAGE.md)
- [HTTP API 文档](HTTP_API.md)
- [配置文件说明](CONFIGURATION.md)
