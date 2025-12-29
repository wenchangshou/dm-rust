# TCP 协议模拟器使用指南

## 概述

TCP 协议模拟器是一个强大的开发测试工具，用于模拟各种工业设备协议的 TCP 服务器。它支持创建多个模拟器实例，每个实例可以模拟不同的协议和设备行为。

### 主要特性

- **多协议支持**: 支持 Modbus TCP、Scene Loader 等协议
- **多实例管理**: 同时运行多个模拟器实例
- **持久化存储**: 模拟器配置自动保存，重启后恢复
- **报文监控**: 实时查看收发的原始报文数据
- **值生成器**: 支持多种动态值生成模式，模拟真实设备行为
- **Web UI**: 提供友好的 Web 界面进行管理

## 快速开始

### 1. 创建模拟器

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "测试 Modbus 设备",
    "protocol": "modbus",
    "port": 502,
    "auto_start": true
  }'
```

### 2. 查看模拟器列表

```bash
curl http://localhost:8080/lspcapi/tcp-simulator/list
```

### 3. 连接测试

使用 Modbus 客户端工具连接到 `localhost:502` 进行测试。

---

## API 参考

### 基础 API

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/lspcapi/tcp-simulator/protocols` | 获取支持的协议列表 |
| POST | `/lspcapi/tcp-simulator/create` | 创建模拟器 |
| GET | `/lspcapi/tcp-simulator/list` | 列出所有模拟器 |
| GET | `/lspcapi/tcp-simulator/:id` | 获取模拟器详情 |
| DELETE | `/lspcapi/tcp-simulator/:id` | 删除模拟器 |
| POST | `/lspcapi/tcp-simulator/:id/start` | 启动模拟器 |
| POST | `/lspcapi/tcp-simulator/:id/stop` | 停止模拟器 |
| POST | `/lspcapi/tcp-simulator/:id/state` | 更新状态 |
| POST | `/lspcapi/tcp-simulator/:id/fault` | 触发故障 |
| POST | `/lspcapi/tcp-simulator/:id/clear-fault` | 清除故障 |
| POST | `/lspcapi/tcp-simulator/:id/online` | 设置在线状态 |

### Modbus API

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/lspcapi/tcp-simulator/:id/modbus/slaves` | 获取 Slave 列表 |
| POST | `/lspcapi/tcp-simulator/:id/modbus/slave` | 添加 Slave |
| DELETE | `/lspcapi/tcp-simulator/:id/modbus/slave/:slaveId` | 删除 Slave |
| POST | `/lspcapi/tcp-simulator/:id/modbus/register` | 添加/更新寄存器 |
| POST | `/lspcapi/tcp-simulator/:id/modbus/register/delete` | 删除寄存器 |
| POST | `/lspcapi/tcp-simulator/:id/modbus/register/value` | 更新寄存器值 |
| POST | `/lspcapi/tcp-simulator/:id/modbus/registers/batch` | 批量更新寄存器值 |

### 报文监控 API

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/lspcapi/tcp-simulator/:id/packets` | 获取报文列表 |
| DELETE | `/lspcapi/tcp-simulator/:id/packets` | 清空报文记录 |
| POST | `/lspcapi/tcp-simulator/:id/packets/settings` | 设置监控选项 |

---

## 支持的协议

### 1. Modbus TCP

Modbus TCP 是工业领域最广泛使用的通信协议之一。模拟器完整支持以下功能码：

| 功能码 | 名称 | 说明 |
|--------|------|------|
| 0x01 | Read Coils | 读线圈 |
| 0x02 | Read Discrete Inputs | 读离散输入 |
| 0x03 | Read Holding Registers | 读保持寄存器 |
| 0x04 | Read Input Registers | 读输入寄存器 |
| 0x05 | Write Single Coil | 写单个线圈 |
| 0x06 | Write Single Register | 写单个保持寄存器 |
| 0x0F | Write Multiple Coils | 写多个线圈 |
| 0x10 | Write Multiple Registers | 写多个保持寄存器 |

#### 寄存器类型

| 类型 | Modbus 地址 | 数据类型 | 读写 |
|------|-------------|----------|------|
| Coil (线圈) | 0xxxx | bit | 读写 |
| Discrete Input (离散输入) | 1xxxx | bit | 只读 |
| Input Register (输入寄存器) | 3xxxx | 16-bit | 只读 |
| Holding Register (保持寄存器) | 4xxxx | 16-bit | 读写 |

#### 数据类型

| 类型 | 说明 | 占用寄存器数 |
|------|------|------------|
| bit | 布尔值 | 1 (线圈/离散输入) |
| uint16 | 无符号 16 位整数 | 1 |
| int16 | 有符号 16 位整数 | 1 |
| uint32 | 无符号 32 位整数 | 2 |
| int32 | 有符号 32 位整数 | 2 |
| float32 | 32 位浮点数 | 2 |

### 2. Scene Loader

场景加载器协议，用于控制场景切换。

---

## 值生成器

值生成器允许寄存器值按照预设规则自动变化，用于模拟真实设备的动态数据。

### 生成模式

| 模式 | 说明 | 适用场景 |
|------|------|---------|
| **fixed** | 固定值（默认） | 静态配置 |
| **random** | 范围内随机值 | 模拟波动的传感器数据 |
| **increment** | 循环递增 | 模拟计数器、累积值 |
| **decrement** | 循环递减 | 模拟倒计时、消耗值 |
| **sine** | 正弦波变化 | 模拟周期性变化（温度、压力等） |
| **toggle** | 开关切换 | 模拟开关状态变化 |
| **sequence** | 序列循环 | 模拟状态机、档位切换 |

### 生成器参数

```typescript
interface GeneratorConfig {
  mode: GeneratorMode       // 生成模式
  min?: number              // 最小值
  max?: number              // 最大值
  step?: number             // 步长（increment/decrement）
  period?: number           // 周期（毫秒，sine/toggle）
  sequence?: number[]       // 序列值（sequence）
  interval: number          // 更新间隔（毫秒）
}
```

### 示例

#### 1. 随机温度值 (20-30°C)

```json
{
  "address": 0,
  "type": "holding_register",
  "dataType": "float32",
  "name": "温度传感器",
  "value": 25.0,
  "generator": {
    "mode": "random",
    "min": 20.0,
    "max": 30.0,
    "interval": 1000
  }
}
```

#### 2. 递增计数器 (0-1000)

```json
{
  "address": 10,
  "type": "holding_register",
  "dataType": "uint16",
  "name": "生产计数",
  "value": 0,
  "generator": {
    "mode": "increment",
    "min": 0,
    "max": 1000,
    "step": 1,
    "interval": 500
  }
}
```

#### 3. 正弦波压力值

```json
{
  "address": 20,
  "type": "input_register",
  "dataType": "float32",
  "name": "压力传感器",
  "value": 100.0,
  "generator": {
    "mode": "sine",
    "min": 80.0,
    "max": 120.0,
    "period": 10000,
    "interval": 100
  }
}
```

#### 4. 状态序列循环

```json
{
  "address": 30,
  "type": "holding_register",
  "dataType": "uint16",
  "name": "运行状态",
  "value": 0,
  "generator": {
    "mode": "sequence",
    "sequence": [0, 1, 2, 3, 2, 1],
    "interval": 2000
  }
}
```

#### 5. 开关切换

```json
{
  "address": 0,
  "type": "coil",
  "dataType": "bit",
  "name": "报警灯",
  "value": false,
  "generator": {
    "mode": "toggle",
    "period": 1000,
    "interval": 1000
  }
}
```

---

## 报文监控

报文监控功能可以记录所有收发的原始 TCP 数据，便于调试和分析。

### 报文记录格式

```typescript
interface PacketRecord {
  id: number              // 唯一 ID
  timestamp: string       // 时间戳 (ISO 8601)
  direction: 'received' | 'sent'  // 方向
  peer_addr: string       // 客户端地址
  hex_data: string        // 十六进制数据
  size: number            // 数据大小（字节）
  parsed?: object         // 协议解析信息（可选）
}
```

### 使用示例

#### 获取最近 100 条报文

```bash
curl "http://localhost:8080/lspcapi/tcp-simulator/sim_xxx/packets?limit=100"
```

#### 增量获取（获取 ID > 50 的报文）

```bash
curl "http://localhost:8080/lspcapi/tcp-simulator/sim_xxx/packets?afterId=50"
```

#### 清空报文

```bash
curl -X DELETE "http://localhost:8080/lspcapi/tcp-simulator/sim_xxx/packets"
```

---

## 完整 API 示例

### 创建 Modbus 模拟器

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "PLC 模拟器",
    "protocol": "modbus",
    "bind_addr": "0.0.0.0",
    "port": 502,
    "auto_start": true
  }'
```

响应：

```json
{
  "state": 0,
  "message": "模拟器创建成功",
  "data": {
    "id": "sim_abc123",
    "name": "PLC 模拟器",
    "protocol": "modbus",
    "bind_addr": "0.0.0.0",
    "port": 502,
    "status": "running",
    "state": {
      "online": true,
      "fault": null,
      "values": {},
      "stats": {
        "total_connections": 0,
        "active_connections": 0,
        "bytes_received": 0,
        "bytes_sent": 0,
        "last_activity": null
      }
    }
  }
}
```

### 添加 Modbus Slave

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/sim_abc123/modbus/slave \
  -H "Content-Type: application/json" \
  -d '{
    "slaveId": 1,
    "registers": []
  }'
```

### 添加带生成器的寄存器

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/sim_abc123/modbus/register \
  -H "Content-Type: application/json" \
  -d '{
    "slaveId": 1,
    "register": {
      "address": 0,
      "type": "holding_register",
      "dataType": "uint16",
      "name": "温度",
      "value": 250,
      "generator": {
        "mode": "random",
        "min": 200,
        "max": 300,
        "interval": 1000
      }
    }
  }'
```

### 更新寄存器值

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/sim_abc123/modbus/register/value \
  -H "Content-Type: application/json" \
  -d '{
    "slaveId": 1,
    "registerType": "holding_register",
    "address": 0,
    "value": 300
  }'
```

### 批量更新寄存器值

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/sim_abc123/modbus/registers/batch \
  -H "Content-Type: application/json" \
  -d '{
    "updates": [
      {"slaveId": 1, "registerType": "holding_register", "address": 0, "value": 100},
      {"slaveId": 1, "registerType": "holding_register", "address": 1, "value": 200},
      {"slaveId": 1, "registerType": "coil", "address": 0, "value": true}
    ]
  }'
```

---

## 持久化

模拟器配置自动保存到 `simulators.json` 文件，包括：

- 模拟器基本配置（名称、协议、端口等）
- 所有 Slave 和寄存器配置
- 值生成器配置
- 是否自动启动

服务重启后会自动恢复所有模拟器。

---

## Web UI 使用

访问 `http://localhost:3000`（开发模式）或部署后的地址，可以使用 Web 界面管理模拟器。

### 功能

1. **模拟器管理**
   - 创建/删除模拟器
   - 启动/停止模拟器
   - 查看连接统计

2. **Modbus 配置**
   - 添加/删除 Slave
   - 添加/编辑/删除寄存器
   - 配置值生成器
   - 实时值显示（自动刷新）

3. **报文监控**
   - 实时查看收发报文
   - 十六进制数据显示
   - 清空报文记录

---

## 故障模拟

可以通过 API 模拟设备故障：

### 触发故障

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/sim_xxx/fault \
  -H "Content-Type: application/json" \
  -d '{"fault_type": "communication_error"}'
```

### 清除故障

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/sim_xxx/clear-fault
```

### 设置离线

```bash
curl -X POST http://localhost:8080/lspcapi/tcp-simulator/sim_xxx/online \
  -H "Content-Type: application/json" \
  -d '{"online": false}'
```

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 30001 | 模拟器不存在 |
| 30003 | 参数错误 |
| 30006 | 操作失败 |

---

## 最佳实践

### 1. 模拟真实设备场景

使用值生成器组合模拟真实设备：

```json
{
  "registers": [
    {
      "address": 0,
      "name": "运行状态",
      "dataType": "uint16",
      "value": 1,
      "generator": { "mode": "fixed" }
    },
    {
      "address": 1,
      "name": "当前温度",
      "dataType": "float32",
      "value": 25.0,
      "generator": {
        "mode": "sine",
        "min": 20.0,
        "max": 30.0,
        "period": 60000,
        "interval": 1000
      }
    },
    {
      "address": 3,
      "name": "累计产量",
      "dataType": "uint32",
      "value": 0,
      "generator": {
        "mode": "increment",
        "min": 0,
        "max": 999999,
        "step": 1,
        "interval": 5000
      }
    }
  ]
}
```

### 2. 开发调试流程

1. 创建模拟器并配置所需寄存器
2. 启用报文监控
3. 使用实际客户端连接测试
4. 通过报文监控分析通信问题
5. 调整寄存器值和生成器配置

### 3. 自动化测试

在 CI/CD 流程中使用模拟器进行自动化测试：

```bash
# 启动模拟器
curl -X POST .../create -d '...'

# 运行测试
pytest tests/modbus_tests.py

# 清理
curl -X DELETE .../sim_xxx
```

---

## 技术实现

### 架构

```
TcpSimulatorManager
    │
    ├── TcpSimulatorServer (实例1)
    │       ├── ProtocolHandler (modbus)
    │       ├── SimulatorState
    │       └── PacketMonitor
    │
    └── TcpSimulatorServer (实例2)
            └── ProtocolHandler (scene_loader)
```

### 值生成器工作原理

- 每 100ms 执行一次 `tick_generators()` 检查
- 根据每个寄存器的 `interval` 配置决定是否更新
- 更新后自动持久化保存

### 线程安全

- 使用 `DashMap` 管理模拟器实例
- 使用 `RwLock` 保护服务器状态
- 支持多客户端并发连接

---

## 更新日志

### v1.0.0

- 初始版本
- 支持 Modbus TCP 协议
- 支持值生成器
- 支持报文监控
- 支持持久化存储
- Web UI 管理界面
