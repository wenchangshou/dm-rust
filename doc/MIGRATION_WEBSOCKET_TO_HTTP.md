# WebSocket 到 HTTP 迁移指南

## 概述

系统已从 WebSocket 客户端模式改为纯 HTTP REST API 控制模式。这一变更简化了架构，提高了兼容性和易用性。

## 主要变更

### 1. 配置文件变更

**之前（WebSocket 模式）:**
```json
{
  "channels": [...],
  "nodes": [...],
  "scenes": [...],
  "websocket": {
    "type": "client",
    "ip": "192.168.20.100",
    "port": 9000,
    "socket_name": "/dm",
    "socket_type": "Controller"
  },
  "web_server": {
    "port": 8080
  }
}
```

**现在（HTTP 模式）:**
```json
{
  "channels": [...],
  "nodes": [...],
  "scenes": [...],
  "web_server": {
    "port": 8080
  }
}
```

### 2. 控制方式变更

#### 之前：WebSocket 协议

需要建立 WebSocket 连接，发送注册消息，然后通过 WebSocket 发送控制命令：

```javascript
// WebSocket 客户端代码
const ws = new WebSocket('ws://192.168.20.100:9000/dm');

ws.on('open', () => {
  // 发送注册消息
  ws.send(JSON.stringify({
    messageType: "RegisterToDaemon",
    SocketName: "/dm",
    SocketType: "Controller"
  }));
  
  // 发送控制命令
  ws.send(JSON.stringify({
    messageType: "WriteDevice",
    id: 1,
    value: 1
  }));
});
```

#### 现在：HTTP REST API

直接发送 HTTP POST 请求：

```bash
# 控制设备
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"value":1}'

# 执行场景
curl -X POST http://localhost:8080/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"开机场景"}'
```

### 3. 代码变更

#### 移除的文件/模块
- `src/web/websocket_client.rs` - WebSocket 客户端实现
- `Config.websocket` - WebSocket 配置结构

#### 增强的模块
- `src/web/server.rs` - 新增完整的 HTTP API 接口

#### 新增的文档
- `HTTP_API.md` - 完整的 HTTP API 文档
- `test_http_api.sh` - HTTP API 测试脚本

## 迁移步骤

### 对于部署者

1. **更新配置文件**
   ```bash
   # 编辑配置文件，移除 websocket 配置块
   vim config.json
   ```

2. **重新编译**
   ```bash
   cd dm-rust
   cargo build --release
   ```

3. **重启服务**
   ```bash
   ./target/release/dm-rust
   ```

### 对于控制端开发者

1. **改用 HTTP 客户端**
   
   将 WebSocket 客户端代码改为 HTTP 请求：

   **Python 示例:**
   ```python
   import requests
   
   BASE_URL = "http://localhost:8080"
   
   # 控制设备
   response = requests.post(
       f"{BASE_URL}/device/write",
       json={"id": 1, "value": 1}
   )
   print(response.json())
   
   # 执行场景
   response = requests.post(
       f"{BASE_URL}/device/executeScene",
       json={"name": "开机场景"}
   )
   print(response.json())
   ```

   **JavaScript/Node.js 示例:**
   ```javascript
   const axios = require('axios');
   
   const BASE_URL = 'http://localhost:8080';
   
   // 控制设备
   async function writeDevice(id, value) {
     const response = await axios.post(`${BASE_URL}/device/write`, {
       id: id,
       value: value
     });
     return response.data;
   }
   
   // 执行场景
   async function executeScene(name) {
     const response = await axios.post(`${BASE_URL}/device/executeScene`, {
       name: name
     });
     return response.data;
   }
   ```

2. **更新命令映射**

   | WebSocket 消息类型 | HTTP 接口 |
   |-------------------|-----------|
   | WriteDevice | POST /device/write |
   | ReadDevice | POST /device/read |
   | GetAllStatus | POST /device/getAllStatus |
   | GetNodeStates | POST /device/getAllNodeStates |
   | ExecuteScene | POST /device/executeScene |
   | ExecuteCommand | POST /device/executeCommand |

## 优势

### 1. 更简单的架构
- 无需维护长连接
- 无需处理连接断开重连逻辑
- 减少了系统复杂度

### 2. 更好的兼容性
- 任何支持 HTTP 的客户端都可以控制
- 易于与现有系统集成
- 可以直接使用 curl 命令行测试

### 3. 更容易调试
- 使用标准 HTTP 工具（如 Postman、curl）
- 请求响应模式更直观
- 日志更清晰

### 4. 更容易扩展
- 可以轻松添加新的接口
- 支持标准 HTTP 中间件（CORS、认证等）
- 易于实现 API 版本控制

## 注意事项

1. **实时状态推送**
   
   HTTP 模式是请求-响应模式，不支持服务器主动推送。如需实时状态更新，可以：
   - 客户端轮询 `/device/getAllNodeStates`
   - 未来可添加 Server-Sent Events (SSE) 或 WebSocket 作为可选功能

2. **并发控制**
   
   HTTP 请求是无状态的，系统内部已实现并发安全，但建议：
   - 避免对同一设备短时间内发送大量请求
   - 使用场景功能批量控制多个设备

3. **安全性**
   
   当前版本未实现认证，建议：
   - 仅在内网环境使用
   - 通过防火墙限制访问
   - 后续可添加 token 认证或基于角色的访问控制

## 测试验证

使用提供的测试脚本验证系统工作正常：

```bash
# 启动系统
cd dm-rust
cargo run --release &

# 等待启动完成
sleep 3

# 运行测试
./test_http_api.sh
```

或手动测试关键接口：

```bash
# 健康检查
curl http://localhost:8080

# 获取所有节点状态
curl -X POST http://localhost:8080/device/getAllNodeStates

# 控制设备
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"value":1}'
```

## 问题排查

### 端口被占用
```bash
# 检查端口占用
lsof -i :8080

# 修改配置文件中的端口
vim config.json
# 将 web_server.port 改为其他端口
```

### 连接被拒绝
```bash
# 确认服务正在运行
ps aux | grep dm-rust

# 检查日志
# 在运行窗口查看日志输出
```

### API 返回错误
- 检查请求参数是否正确
- 查看服务端日志输出
- 确认设备配置是否正确

## 回滚方案

如需回退到 WebSocket 模式，可以：

1. 使用 git 回退到迁移前的提交
2. 或保留之前的 `websocket_client.rs` 文件作为备份

## 进一步阅读

- [HTTP_API.md](HTTP_API.md) - 完整的 API 文档
- [CONFIGURATION.md](CONFIGURATION.md) - 配置文件说明
- [README.md](README.md) - 项目概述

## 支持

如有问题或建议，请提交 Issue 或联系维护团队。
