# Mock 协议使用指南

## 概述

Mock 协议是一个模拟设备协议，专门用于接口调试和测试。它提供了完整的设备模拟功能，包括读写操作、延迟模拟、错误模拟等。

## 特性

- ✅ **内存状态存储** - 所有读写操作在内存中进行
- ✅ **延迟模拟** - 可配置响应延迟，模拟真实设备的响应时间
- ✅ **错误率模拟** - 可配置错误概率，测试错误处理逻辑
- ✅ **故障模拟** - 支持模拟设备故障状态
- ✅ **统计信息** - 记录读写次数、错误次数等统计数据
- ✅ **批量操作** - 支持批量读写操作
- ✅ **自定义方法** - 提供多个自定义方法用于调试

## 配置示例

### 基础配置

```json
{
  "enable": true,
  "channel_id": 1,
  "statute": "mock",
  "description": "模拟设备",
  "arguments": {
    "delay_ms": 50,
    "error_rate": 0.0,
    "initial_values": {
      "1": 100,
      "2": 200
    }
  }
}
```

### 配置参数说明

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `delay_ms` | number | 否 | 0 | 响应延迟（毫秒），用于模拟真实设备响应时间 |
| `error_rate` | number | 否 | 0.0 | 错误率（0.0-1.0），随机产生错误的概率 |
| `initial_values` | object | 否 | {} | 初始值对象，键为地址（字符串），值为数值 |

### 完整配置示例

参见项目根目录下的 `config.mock.json` 文件。

## API 使用示例

### 1. 基础读写操作

#### 写入数据

```bash
curl -X POST http://localhost:18080/device/write \
  -H "Content-Type: application/json" \
  -d '{
    "global_id": 1,
    "value": 100
  }'
```

响应：
```json
{
  "state": 0,
  "message": "写入成功",
  "data": null
}
```

#### 读取数据

```bash
curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{
    "global_id": 1
  }'
```

响应：
```json
{
  "state": 0,
  "message": "读取成功",
  "data": {
    "value": 100
  }
}
```

### 2. 批量操作

#### 批量写入

```bash
curl -X POST http://localhost:18080/device/writeMany \
  -H "Content-Type: application/json" \
  -d '{
    "writes": [
      {"global_id": 1, "value": 100},
      {"global_id": 2, "value": 200}
    ]
  }'
```

#### 批量读取

```bash
curl -X POST http://localhost:18080/device/readMany \
  -H "Content-Type: application/json" \
  -d '{
    "reads": [1, 2, 10]
  }'
```

### 3. 执行命令

#### Ping 命令

```bash
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "ping",
    "params": {}
  }'
```

响应：
```json
{
  "state": 0,
  "message": "命令执行成功",
  "data": {
    "status": "ok",
    "message": "pong",
    "channel_id": 1
  }
}
```

#### 获取所有存储的值

```bash
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "get_all_values",
    "params": {}
  }'
```

响应：
```json
{
  "state": 0,
  "message": "命令执行成功",
  "data": {
    "status": "ok",
    "values": {
      "1": 100,
      "2": 200,
      "10": 1
    }
  }
}
```

#### 设置错误率

```bash
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "set_error_rate",
    "params": {
      "rate": 0.1
    }
  }'
```

#### 重置所有值

```bash
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "reset",
    "params": {}
  }'
```

#### 批量写入（通过命令）

```bash
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "batch_write",
    "params": {
      "writes": [
        {"addr": 1, "value": 100},
        {"addr": 2, "value": 200},
        {"addr": 10, "value": 1}
      ]
    }
  }'
```

#### 批量读取（通过命令）

```bash
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "batch_read",
    "params": {
      "addrs": [1, 2, 10]
    }
  }'
```

### 4. 自定义方法

#### 获取支持的方法列表

```bash
curl -X POST http://localhost:18080/device/getMethods \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1
  }'
```

响应：
```json
{
  "state": 0,
  "message": "获取方法列表成功",
  "data": {
    "methods": [
      "simulate_fault",
      "clear_fault",
      "get_statistics",
      "set_delay",
      "get_value",
      "set_value"
    ]
  }
}
```

#### 模拟设备故障

```bash
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method": "simulate_fault",
    "args": {}
  }'
```

**注意**：模拟故障后，所有读写操作都会失败，直到调用 `clear_fault` 清除故障。

#### 清除设备故障

```bash
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method": "clear_fault",
    "args": {}
  }'
```

#### 获取统计信息

```bash
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method": "get_statistics",
    "args": {}
  }'
```

响应：
```json
{
  "state": 0,
  "message": "调用方法成功",
  "data": {
    "read_count": 15,
    "write_count": 8,
    "error_count": 2,
    "stored_values": 3,
    "total_operations": 23
  }
}
```

#### 设置延迟

```bash
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method": "set_delay",
    "args": {
      "delay_ms": 500
    }
  }'
```

#### 读取指定地址的值

```bash
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method": "get_value",
    "args": {
      "addr": 1
    }
  }'
```

#### 设置指定地址的值

```bash
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method": "set_value",
    "args": {
      "addr": 1,
      "value": 999
    }
  }'
```

### 5. 查询状态

#### 获取通道状态

```bash
curl -X POST http://localhost:18080/device/getAllStatus \
  -H "Content-Type: application/json" \
  -d '{}'
```

响应：
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "channels": {
      "1": {
        "connected": true,
        "channel_id": 1,
        "fault": false,
        "delay_ms": 50,
        "error_rate": 0.0,
        "statistics": {
          "read_count": 15,
          "write_count": 8,
          "error_count": 0,
          "stored_values": 3
        }
      }
    }
  }
}
```

#### 获取节点状态

```bash
curl -X POST http://localhost:18080/device/getAllNodeStates \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 6. 场景执行

```bash
curl -X POST http://localhost:18080/device/executeScene \
  -H "Content-Type: application/json" \
  -d '{
    "scene_id": 1
  }'
```

## 支持的命令列表

| 命令 | 说明 | 参数 |
|------|------|------|
| `ping` | 测试连接 | 无 |
| `reset` | 重置所有值为0 | 无 |
| `set_error_rate` | 设置错误率 | `rate`: 0.0-1.0 |
| `get_all_values` | 获取所有存储的值 | 无 |
| `batch_write` | 批量写入 | `writes`: [{addr, value}...] |
| `batch_read` | 批量读取 | `addrs`: [addr...] |

## 支持的自定义方法列表

| 方法 | 说明 | 参数 |
|------|------|------|
| `simulate_fault` | 模拟设备故障 | 无 |
| `clear_fault` | 清除故障状态 | 无 |
| `get_statistics` | 获取统计信息 | 无 |
| `set_delay` | 设置响应延迟 | `delay_ms`: 延迟毫秒数 |
| `get_value` | 读取指定地址值 | `addr`: 地址 |
| `set_value` | 设置指定地址值 | `addr`: 地址, `value`: 值 |

## 测试场景

### 1. 基础功能测试

```bash
# 1. 写入数据
curl -X POST http://localhost:18080/device/write \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1, "value": 100}'

# 2. 读取数据
curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1}'

# 3. 查看统计
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1, "method": "get_statistics", "args": {}}'
```

### 2. 错误处理测试

```bash
# 1. 设置10%错误率
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1, "command": "set_error_rate", "params": {"rate": 0.1}}'

# 2. 多次读写，观察错误
for i in {1..20}; do
  curl -X POST http://localhost:18080/device/write \
    -H "Content-Type: application/json" \
    -d "{\"global_id\": 1, \"value\": $i}"
done

# 3. 查看错误统计
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1, "method": "get_statistics", "args": {}}'
```

### 3. 故障模拟测试

```bash
# 1. 模拟故障
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1, "method": "simulate_fault", "args": {}}'

# 2. 尝试读写（应该失败）
curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1}'

# 3. 清除故障
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1, "method": "clear_fault", "args": {}}'

# 4. 再次尝试读写（应该成功）
curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1}'
```

### 4. 性能测试

```bash
# 1. 设置不同延迟
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1, "method": "set_delay", "args": {"delay_ms": 100}}'

# 2. 测试响应时间
time curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1}'
```

## 运行服务

```bash
# 使用 Mock 配置启动服务
cargo run -- -c config.mock.json
```

## 注意事项

1. **内存存储** - 所有数据存储在内存中，重启服务后会丢失
2. **线程安全** - 使用 Arc<Mutex> 保证并发安全
3. **地址映射** - global_id 会映射到通道内的 id（节点配置中定义）
4. **错误率** - 错误率为 0 时不会产生任何错误
5. **延迟模拟** - 延迟会应用于所有操作（读、写、命令、方法调用）
6. **故障状态** - 故障状态下所有操作都会失败，用于测试错误处理

## 调试技巧

1. **使用统计信息** - 定期调用 `get_statistics` 查看操作统计
2. **逐步测试** - 先测试基础读写，再测试复杂场景
3. **错误注入** - 使用错误率模拟不稳定的网络或设备
4. **性能调优** - 使用延迟参数找到最佳超时设置
5. **批量操作** - 使用批量命令测试高并发场景

## 与其他协议的对比

| 特性 | Mock | Modbus | PJLink |
|------|------|--------|--------|
| 真实设备 | ❌ | ✅ | ✅ |
| 网络通信 | ❌ | ✅ | ✅ |
| 配置简单 | ✅ | ❌ | ❌ |
| 支持调试 | ✅ | ❌ | ❌ |
| 错误模拟 | ✅ | ❌ | ❌ |
| 延迟控制 | ✅ | ❌ | ❌ |

Mock 协议专为开发和测试设计，不依赖真实硬件，非常适合：
- 接口开发和调试
- 自动化测试
- 压力测试
- 错误处理测试
- 演示和培训
