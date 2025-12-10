# 设备控制 API 文档

## 概述

设备控制 API 提供了完整的设备管理和控制功能，包括读写操作、状态查询、场景执行、通道命令执行等。

**API 路径前缀**: `/device`

**Swagger 文档**: `http://localhost:18080/swagger-ui/`

---

## API 列表

### 1. 状态查询 API

#### 1.1 获取所有通道状态

获取所有通道的连接状态和详细信息。

**请求**:
```
POST /device/getAllStatus
Content-Type: application/json

{}
```

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "channels": {
      "1": {
        "connected": true,
        "channel_id": 1,
        "protocol": "mock",
        ...
      }
    }
  }
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/getAllStatus \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

#### 1.2 获取所有节点状态

获取所有配置的节点状态信息。

**请求**:
```
POST /device/getAllNodeStates
Content-Type: application/json

{}
```

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "global_id": 1,
      "channel_id": 1,
      "device_id": 1,
      "category": "light",
      "alias": "灯光1",
      "current_value": 100,
      "online": true
    }
  ]
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/getAllNodeStates \
  -H "Content-Type: application/json" \
  -d '{}'
```

---

#### 1.3 获取单个节点状态

根据全局 ID 获取指定节点的状态。

**请求**:
```
POST /device/getNodeState
Content-Type: application/json

{
  "id": 1
}
```

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "global_id": 1,
    "channel_id": 1,
    "device_id": 1,
    "category": "light",
    "alias": "灯光1",
    "current_value": 100,
    "online": true
  }
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/getNodeState \
  -H "Content-Type: application/json" \
  -d '{"id": 1}'
```

---

### 2. 读写操作 API

#### 2.1 读取设备值

读取指定节点的当前值。

**请求**:
```
POST /device/read
Content-Type: application/json

{
  "id": 1
}
```

**参数说明**:
- `id`: 节点全局 ID（global_id）

**响应**:
```json
{
  "state": 0,
  "message": "读取成功",
  "data": 100.0
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{"id": 1}'
```

---

#### 2.2 批量读取设备值

批量读取多个节点的值。

**请求**:
```
POST /device/readMany
Content-Type: application/json

{
  "ids": [1, 2, 10]
}
```

**参数说明**:
- `ids`: 节点全局 ID 列表

**响应**:
```json
{
  "state": 0,
  "message": "批量读取完成: 成功 3, 失败 0",
  "data": [
    {
      "id": 1,
      "success": true,
      "value": 100.0,
      "error": null
    },
    {
      "id": 2,
      "success": true,
      "value": 200.0,
      "error": null
    },
    {
      "id": 10,
      "success": true,
      "value": 1.0,
      "error": null
    }
  ]
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/readMany \
  -H "Content-Type: application/json" \
  -d '{"ids": [1, 2, 10]}'
```

---

#### 2.3 写入设备值

向指定节点写入值。

**请求**:
```
POST /device/write
Content-Type: application/json

{
  "id": 1,
  "value": 100
}
```

**参数说明**:
- `id`: 节点全局 ID（global_id）
- `value`: 要写入的值（整数）

**响应**:
```json
{
  "state": 0,
  "message": "操作成功",
  "data": null
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/write \
  -H "Content-Type: application/json" \
  -d '{"id": 1, "value": 100}'
```

---

#### 2.4 批量写入设备值

批量向多个节点写入值。

**请求**:
```
POST /device/writeMany
Content-Type: application/json

{
  "items": [
    {"id": 1, "value": 100},
    {"id": 2, "value": 200}
  ]
}
```

**参数说明**:
- `items`: 写入项列表
  - `id`: 节点全局 ID
  - `value`: 要写入的值

**响应**:
```json
{
  "state": 0,
  "message": "批量写入完成: 成功 2, 失败 0",
  "data": [
    {
      "id": 1,
      "success": true,
      "error": null
    },
    {
      "id": 2,
      "success": true,
      "error": null
    }
  ]
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/writeMany \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"id": 1, "value": 100},
      {"id": 2, "value": 200}
    ]
  }'
```

---

### 3. 场景控制 API

#### 3.1 执行场景

执行预定义的场景。

**请求**:
```
POST /device/executeScene
Content-Type: application/json

{
  "name": "打开所有灯光"
}
```

**参数说明**:
- `name`: 场景名称（在配置文件中定义）

**响应**:
```json
{
  "state": 0,
  "message": "场景 '打开所有灯光' 执行成功",
  "data": null
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/executeScene \
  -H "Content-Type: application/json" \
  -d '{"name": "打开所有灯光"}'
```

---

### 4. 通道命令 API

#### 4.1 执行通道命令

直接向指定通道发送命令。

**请求**:
```
POST /device/executeCommand
Content-Type: application/json

{
  "channel_id": 1,
  "command": "ping",
  "params": {}
}
```

**参数说明**:
- `channel_id`: 通道 ID
- `command`: 命令名称（取决于协议）
- `params`: 命令参数（JSON 对象）

**响应**:
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

**Mock 协议支持的命令**:
- `ping`: 测试连接
- `reset`: 重置所有值
- `set_error_rate`: 设置错误率
- `get_all_values`: 获取所有存储的值
- `batch_write`: 批量写入
- `batch_read`: 批量读取

**curl 示例**:
```bash
# Ping 命令
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "ping",
    "params": {}
  }'

# 获取所有值
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "get_all_values",
    "params": {}
  }'

# 批量写入
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "batch_write",
    "params": {
      "writes": [
        {"addr": 1, "value": 100},
        {"addr": 2, "value": 200}
      ]
    }
  }'
```

---

#### 4.2 调用通道方法

调用通道协议提供的自定义方法。

**请求**:
```
POST /device/callMethod
Content-Type: application/json

{
  "channel_id": 1,
  "method_name": "get_statistics",
  "arguments": {}
}
```

**参数说明**:
- `channel_id`: 通道 ID
- `method_name`: 方法名称
- `arguments`: 方法参数（JSON 对象）

**响应**:
```json
{
  "state": 0,
  "message": "方法调用成功",
  "data": {
    "read_count": 15,
    "write_count": 8,
    "error_count": 0,
    "stored_values": 3,
    "total_operations": 23
  }
}
```

**Mock 协议支持的方法**:
- `simulate_fault`: 模拟设备故障
- `clear_fault`: 清除故障状态
- `get_statistics`: 获取统计信息
- `set_delay`: 设置响应延迟
- `get_value`: 读取指定地址值
- `set_value`: 设置指定地址值

**curl 示例**:
```bash
# 获取统计信息
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method_name": "get_statistics",
    "arguments": {}
  }'

# 设置延迟
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method_name": "set_delay",
    "arguments": {
      "delay_ms": 100
    }
  }'

# 模拟故障
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method_name": "simulate_fault",
    "arguments": {}
  }'
```

---

#### 4.3 获取方法列表

获取指定通道支持的所有自定义方法。

**请求**:
```
POST /device/getMethods
Content-Type: application/json

{
  "channel_id": 1
}
```

**参数说明**:
- `channel_id`: 通道 ID

**响应**:
```json
{
  "state": 0,
  "message": "获取方法列表成功",
  "data": [
    "simulate_fault",
    "clear_fault",
    "get_statistics",
    "set_delay",
    "get_value",
    "set_value"
  ]
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/getMethods \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1}'
```

---

### 5. 批量操作 API

#### 5.1 批量读取（通过通道命令）

通过通道命令批量读取多个数据点。

**请求**:
```
POST /device/batchRead
Content-Type: application/json

{
  "items": [
    {
      "name": "温度",
      "channel_id": 1,
      "addr": 100
    },
    {
      "name": "湿度",
      "channel_id": 1,
      "addr": 101
    }
  ]
}
```

**参数说明**:
- `items`: 读取项列表
  - `name`: 项目名称（用于标识）
  - `channel_id`: 通道 ID
  - 其他参数根据协议而定

**响应**:
```json
{
  "state": 0,
  "message": "批量读取完成: 成功 2, 失败 0",
  "data": [
    {
      "name": "温度",
      "success": true,
      "value": 25.5,
      "error": null
    },
    {
      "name": "湿度",
      "success": true,
      "value": 60.0,
      "error": null
    }
  ]
}
```

**curl 示例**:
```bash
curl -X POST http://localhost:18080/device/batchRead \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {
        "name": "温度",
        "channel_id": 1,
        "addr": 100
      },
      {
        "name": "湿度",
        "channel_id": 1,
        "addr": 101
      }
    ]
  }'
```

---

## 错误码说明

| 状态码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1 | 通用错误 |
| 400 | 参数无效 |
| 404 | 设备或节点不存在 |

---

## 使用示例

### 场景 1: 基础设备控制

```bash
# 1. 查看所有节点
curl -X POST http://localhost:18080/device/getAllNodeStates \
  -H "Content-Type: application/json" \
  -d '{}'

# 2. 打开灯光（写入值 100）
curl -X POST http://localhost:18080/device/write \
  -H "Content-Type: application/json" \
  -d '{"id": 1, "value": 100}'

# 3. 读取当前状态
curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{"id": 1}'

# 4. 关闭灯光
curl -X POST http://localhost:18080/device/write \
  -H "Content-Type: application/json" \
  -d '{"id": 1, "value": 0}'
```

### 场景 2: 批量操作

```bash
# 批量打开多个灯光
curl -X POST http://localhost:18080/device/writeMany \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"id": 1, "value": 100},
      {"id": 2, "value": 100}
    ]
  }'

# 批量读取状态
curl -X POST http://localhost:18080/device/readMany \
  -H "Content-Type: application/json" \
  -d '{"ids": [1, 2]}'
```

### 场景 3: 场景控制

```bash
# 执行预定义场景
curl -X POST http://localhost:18080/device/executeScene \
  -H "Content-Type: application/json" \
  -d '{"name": "打开所有灯光"}'
```

### 场景 4: 调试和测试（使用 Mock 协议）

```bash
# 1. 测试连接
curl -X POST http://localhost:18080/device/executeCommand \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "command": "ping",
    "params": {}
  }'

# 2. 获取统计信息
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method_name": "get_statistics",
    "arguments": {}
  }'

# 3. 模拟设备故障
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method_name": "simulate_fault",
    "arguments": {}
  }'

# 4. 尝试操作（应该失败）
curl -X POST http://localhost:18080/device/write \
  -H "Content-Type: application/json" \
  -d '{"id": 1, "value": 100}'

# 5. 清除故障
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{
    "channel_id": 1,
    "method_name": "clear_fault",
    "arguments": {}
  }'
```

---

## Swagger UI

访问 `http://localhost:18080/swagger-ui/` 可以使用交互式 API 文档：

1. 查看所有 API 端点
2. 查看请求/响应模型
3. 直接在浏览器中测试 API
4. 查看详细的参数说明

---

## 注意事项

1. **节点 ID**: 所有读写操作使用的是节点的 `global_id`（全局 ID），不是设备内部的 ID
2. **通道 ID**: 通道命令和方法调用使用的是 `channel_id`，在配置文件中定义
3. **异步操作**: 所有 API 调用都是异步的，复杂场景可能需要等待执行完成
4. **错误处理**: 批量操作时，部分失败不会影响其他项的执行
5. **协议差异**: 不同协议支持的命令和方法不同，使用前请查看协议文档

---

## 相关文档

- [Mock 协议使用指南](MOCK_PROTOCOL_GUIDE.md)
- [配置文件说明](CONFIGURATION.md)
- [HTTP API 文档](HTTP_API.md)
