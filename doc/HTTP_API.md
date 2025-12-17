# HTTP API 文档

设备控制系统通过 HTTP REST API 提供设备控制功能，默认监听端口 8080。

## 基础信息

- **基础URL**: `http://localhost:8080`
- **请求方式**: 所有控制接口均为 `POST` 请求
- **Content-Type**: `application/json`

## 响应格式

所有接口返回统一的 JSON 格式：

```json
{
  "state": 0,          // 状态码，0 表示成功
  "message": "成功",   // 状态描述
  "data": {}           // 返回数据（可选）
}
```

### 错误码说明

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 400 | 参数错误 |
| 30001 | 设备未找到 |
| 30002 | 通道未找到 |
| 30003 | 超时 |
| 30004 | 依赖条件未满足 |
| 30006 | 一般错误 |

## API 接口

### 1. 系统信息

#### GET /

获取系统信息。

**响应示例**:
```
Device Control System (Rust Version)
```

---

### 2. 写入设备

#### POST /device/write

向指定节点写入值（控制设备）。

**请求参数**:
```json
{
  "id": 1,        // 节点 global_id
  "value": 1      // 要写入的值（0/1 表示关/开，其他值根据设备定义）
}
```

**响应示例**:
```json
{
  "state": 0,
  "message": "操作成功",
  "data": null
}
```

**使用示例**:
```bash
# 开启 global_id 为 1 的投影仪
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"value":1}'

# 关闭 global_id 为 2 的电脑
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":2,"value":0}'
```

---

### 3. 读取设备

#### POST /device/read

读取指定节点的当前值。

**请求参数**:
```json
{
  "id": 1        // 节点 global_id
}
```

**响应示例**:
```json
{
  "state": 0,
  "message": "读取成功",
  "data": 1      // 当前值
}
```

**使用示例**:
```bash
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{"id":1}'
```

---

### 4. 获取所有节点状态

#### POST /device/getAllNodeStates

获取系统中所有节点的状态信息。

**请求参数**: 无

**响应示例**:
```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "global_id": 1,
      "channel_id": 1,
      "device_id": 1,
      "category": "screen",
      "alias": "主投影仪",
      "current_value": 1,
      "online": true
    },
    {
      "global_id": 2,
      "channel_id": 2,
      "device_id": 1,
      "category": "pc",
      "alias": "1号电脑",
      "current_value": null,
      "online": false
    }
  ]
}
```

**使用示例**:
```bash
curl -X POST http://localhost:8080/device/getAllNodeStates
```

---

### 5. 获取单个节点状态

#### POST /device/getNodeState

获取指定节点的详细状态。

**请求参数**:
```json
{
  "id": 1        // 节点 global_id
}
```

**响应示例**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "global_id": 1,
    "channel_id": 1,
    "device_id": 1,
    "category": "screen",
    "alias": "主投影仪",
    "current_value": 1,
    "online": true
  }
}
```

**使用示例**:
```bash
curl -X POST http://localhost:8080/device/getNodeState \
  -H 'Content-Type: application/json' \
  -d '{"id":1}'
```

---

### 6. 获取所有通道状态

#### POST /device/getAllStatus

获取所有通道（物理设备连接）的状态。

**请求参数**: 无

**响应示例**:
```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "channel_id": 1,
      "statute": "Pjlink",
      "status": {
        "connected": true,
        "error": null
      }
    },
    {
      "channel_id": 2,
      "statute": "XinkeQ1",
      "status": {
        "connected": true,
        "error": null
      }
    }
  ]
}
```

**使用示例**:
```bash
curl -X POST http://localhost:8080/device/getAllStatus
```

---

### 7. 执行场景

#### POST /device/executeScene

执行预定义的场景（按配置顺序批量控制多个设备）。

**请求参数**:
```json
{
  "name": "开机场景"     // 场景名称（在配置文件中定义）
}
```

**响应示例**:
```json
{
  "state": 0,
  "message": "场景 '开机场景' 执行成功",
  "data": null
}
```

**使用示例**:
```bash
# 执行开机场景
curl -X POST http://localhost:8080/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"开机场景"}'

# 执行关机场景
curl -X POST http://localhost:8080/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"关机场景"}'
```

**重要说明**:
- 系统同一时间只允许执行一个场景
- 如果有场景正在执行中，再次执行其他场景会返回错误
- 可以通过 `/device/sceneStatus` 接口查询当前场景执行状态

---

### 8. 获取场景执行状态

#### GET /device/sceneStatus

获取当前场景执行状态，用于查询是否有场景正在执行及具体场景名称。

**请求参数**: 无

**响应示例**:
```json
// 没有场景正在执行
{
  "state": 0,
  "message": "获取场景执行状态成功",
  "data": {
    "is_executing": false,
    "current_scene": null
  }
}

// 有场景正在执行
{
  "state": 0,
  "message": "获取场景执行状态成功",
  "data": {
    "is_executing": true,
    "current_scene": "开机场景"
  }
}
```

**使用示例**:
```bash
# 获取场景执行状态
curl -X GET http://localhost:8080/device/sceneStatus
```

**应用场景**:
- 在执行场景前检查是否有场景正在运行
- 前端界面显示当前执行状态
- 防止场景并发执行导致的冲突

---

### 9. 执行通道命令

#### POST /device/executeCommand

直接向通道（协议层）发送命令。

**请求参数**:
```json
{
  "channel_id": 1,           // 通道 ID
  "command": "power_on",     // 命令名称
  "params": {                // 命令参数（根据协议不同）
    "input": "hdmi1"
  }
}
```

**响应示例**:
```json
{
  "state": 0,
  "message": "命令执行成功",
  "data": {
    "result": "ok"
  }
}
```

**使用示例**:
```bash
curl -X POST http://localhost:8080/device/executeCommand \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "command": "get_status",
    "params": {}
  }'
```

---

### 10. 调用自定义方法

#### POST /device/callMethod

调用通道定义的自定义方法。

**请求参数**:
```json
{
  "channel_id": 1,           // 通道 ID
  "method_name": "set_input", // 方法名称
  "arguments": {              // 方法参数（根据方法定义）
    "source": "hdmi1"
  }
}
```

**响应示例**:
```json
{
  "state": 0,
  "message": "方法调用成功",
  "data": {
    "result": "ok",
    "current_input": "hdmi1"
  }
}
```

**使用示例**:
```bash
# 切换投影仪输入源到 HDMI1
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "set_input",
    "arguments": {
      "source": "hdmi1"
    }
  }'

# 查询灯泡使用时长
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "get_lamp_hours",
    "arguments": {}
  }'
```

---

### 11. 获取通道支持的方法列表

#### POST /device/getMethods

获取指定通道支持的自定义方法列表。

**请求参数**:
```json
{
  "channel_id": 1           // 通道 ID
}
```

**响应示例**:
```json
{
  "state": 0,
  "message": "获取方法列表成功",
  "data": [
    "set_input",
    "get_lamp_hours",
    "set_brightness"
  ]
}
```

**使用示例**:
```bash
curl -X POST http://localhost:8080/device/getMethods \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1}'
```

---

## 常见使用场景

### 场景 1: 开机流程

```bash
# 方式1: 使用场景（推荐）
curl -X POST http://localhost:8080/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"开机场景"}'

# 方式2: 手动控制每个设备
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"value":1}'  # 投影仪

sleep 2

curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":2,"value":1}'  # 电脑1

curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":4,"value":1}'  # 灯光
```

### 场景 2: 查询系统状态

```bash
# 获取所有节点状态
curl -X POST http://localhost:8080/device/getAllNodeStates

# 获取特定设备状态
curl -X POST http://localhost:8080/device/getNodeState \
  -H 'Content-Type: application/json' \
  -d '{"id":1}'
```

### 场景 3: 读取设备值

```bash
# 读取投影仪状态
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{"id":1}'
```

### 场景 4: 使用自定义方法

```bash
# 1. 获取通道支持的方法
curl -X POST http://localhost:8080/device/getMethods \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1}'

# 2. 调用自定义方法
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "set_input",
    "arguments": {"source": "hdmi2"}
  }'
```

---

## 配置说明

HTTP 服务器端口在配置文件中设置：

```json
{
  "web_server": {
    "port": 8080
  }
}
```

修改端口后重启服务生效。

---

## 注意事项

1. **依赖管理**: 使用 `/device/write` 接口时，系统会自动检查节点依赖关系。如果依赖未满足且策略为 `auto`，会自动先满足依赖条件。

2. **场景执行**: 场景中定义的 `delay` 参数会在操作之间添加延迟，确保设备按正确顺序启动。

3. **错误处理**: 所有接口在出错时会返回非 0 的 `state` 值和错误描述信息。

4. **并发控制**: 系统内部使用异步机制处理并发请求，但建议避免对同一设备短时间内发送大量控制命令。

5. **节点 ID**: 使用 `global_id`（全局唯一 ID）而不是 `device_id`（设备内部 ID）来引用节点。

---

## 开发与调试

### 启动服务

```bash
cd dm-rust
cargo run --release
```

### 查看日志

日志会输出到控制台，包含：
- 通道初始化信息
- 设备控制操作
- 错误和警告信息

### 测试脚本示例

```bash
#!/bin/bash
BASE_URL="http://localhost:8080"

# 测试系统连接
echo "测试连接..."
curl $BASE_URL

# 获取所有节点状态
echo -e "\n\n获取所有节点状态..."
curl -X POST $BASE_URL/device/getAllNodeStates

# 执行开机场景
echo -e "\n\n执行开机场景..."
curl -X POST $BASE_URL/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"开机场景"}'

# 等待 10 秒
sleep 10

# 执行关机场景
echo -e "\n\n执行关机场景..."
curl -X POST $BASE_URL/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"关机场景"}'
```

---

## 更新历史

- **v1.0** (2025-11-11): 从 WebSocket 改为纯 HTTP API 控制方式
  - 移除 WebSocket 客户端依赖
  - 增强 HTTP API，提供完整的设备控制接口
  - 添加节点状态查询接口
  - 添加场景执行接口
