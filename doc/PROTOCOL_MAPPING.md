# JavaScript与Rust协议对照表

本文档说明原JavaScript版本的协议实现如何映射到Rust版本。

## 协议实现对照

### 1. PJLink 投影仪协议

**JavaScript版本** (`src/statute/pjlink.js`)
```javascript
function pjlink(options) {
  this.beamer = new Pjlink(options.addr, options.port, 'JBMIAProjectorLink')
  this.id = options.channelId
}

pjlink.prototype.execute = function(body) {
  const { cmd } = body.options
  if (cmd === 'powerOn') {
    // 开机逻辑
  }
}
```

**Rust版本** (`src/protocols/pjlink.rs`)
```rust
pub struct PjlinkProtocol {
    addr: String,
    port: u16,
    password: Option<String>,
}

#[async_trait]
impl Protocol for PjlinkProtocol {
    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        match command {
            "powerOn" => { /* 开机逻辑 */ }
            _ => Err(DeviceError::ProtocolError("未知命令".into()))
        }
    }
}
```

**关键差异：**
- Rust使用枚举匹配替代字符串比较
- 异步函数返回 `Result` 而非 Promise
- 使用 trait 替代原型继承

---

### 2. Modbus 标准协议

**JavaScript版本** (`src/statute/modbus.js`)
```javascript
function modbus(options) {
  this.type = options.type  // 'tcp' or 'serialPort'
  this.buffer = []
  this.callInterval = options.callInterval || 5000
  
  if (type === 'tcp') {
    this.tcpConnect(options)
  } else {
    this.serialPortConnect(options)
  }
}

modbus.prototype.callDi = function(data) {
  this.client.readCoils(startAddr, number).then(...)
}
```

**Rust版本** (`src/protocols/modbus.rs`)
```rust
pub struct ModbusProtocol {
    connection_type: ConnectionType,
    buffer: Arc<RwLock<Vec<ModbusData>>>,
    call_interval: Duration,
}

enum ConnectionType {
    Tcp { addr: String, port: u16 },
    Serial { port: String, baud_rate: u32 },
}

impl ModbusProtocol {
    async fn read_coils(&self, start: u16, count: u16) -> Result<Vec<bool>> {
        // 使用 tokio-modbus crate
    }
}
```

**关键差异：**
- 使用枚举区分连接类型
- buffer 使用 `Arc<RwLock>` 保证线程安全
- 使用 `tokio-modbus` 替代 `zoolon-modbus`

---

### 3. ModbusSlave 网关模式

**JavaScript版本** (`src/statute/modbus-slave/`)
```
modbus-slave/
├── modbus-slave.js    # 主控制器
├── modbus-device.js   # 单个设备
├── cmdStack.js        # 命令队列
└── callCmdStack.js    # 定时调用
```

**Rust版本设计**
```rust
pub struct ModbusSlaveProtocol {
    devices: DashMap<u8, ModbusDevice>,
    command_queue: Arc<RwLock<VecDeque<Command>>>,
    address_map: HashMap<String, u16>,
}

struct ModbusDevice {
    device_id: u8,
    connection: ModbusConnection,
    call_list: Vec<CallConfig>,
}

struct Command {
    device_id: u8,
    operation: Operation,
    retry_count: u32,
    timeout: Instant,
}
```

**关键改进：**
- 使用 `DashMap` 管理多设备（无锁并发）
- `VecDeque` 替代 JS 数组实现队列
- 类型安全的命令结构

---

### 4. XinkeQ1 开关机模块

**JavaScript版本** (`src/statute/xinkeQ1.js`)
```javascript
xinkeQ1.prototype.single = function(body) {
  const { id, status } = body.options
  const buffer = Buffer.from([0x55, 0xaa, ...])
  this.client.write(buffer)
}
```

**Rust版本** (`src/protocols/xinke_q1.rs`)
```rust
impl XinkeQ1Protocol {
    async fn send_single(&mut self, id: u8, status: bool) -> Result<()> {
        let command = self.build_command(id, status);
        
        let mut stream = TcpStream::connect(&self.addr).await?;
        stream.write_all(&command).await?;
        
        let mut response = [0u8; 64];
        stream.read(&mut response).await?;
        
        self.parse_response(&response)
    }
    
    fn build_command(&self, id: u8, status: bool) -> Vec<u8> {
        vec![0x55, 0xaa, /* ... */]
    }
}
```

**关键差异：**
- 使用 `Vec<u8>` 替代 Node.js Buffer
- 显式错误处理（无隐式异常）
- 类型安全的参数

---

### 5. ComputerControl (WOL)

**JavaScript版本** (`src/statute/computerControl.js`)
```javascript
const wol = require('wake_on_lan')

computerControl.prototype.boot = function(mac) {
  wol.wake(mac, (error) => {
    // ...
  })
}
```

**Rust版本** (`src/protocols/computer_control.rs`)
```rust
use wake_on_lan::MagicPacket;

impl ComputerControlProtocol {
    async fn boot(&self, mac: &str) -> Result<()> {
        let mac_addr = mac.parse::<MacAddr>()?;
        let packet = MagicPacket::new(&mac_addr);
        
        packet.send().await?;
        Ok(())
    }
}
```

---

## 事件系统映射

### JavaScript Event Bus
```javascript
// utils/Event.js
Event.create('device').trigger('execute', message)
Event.create('device').listen('execute', handler)
```

### Rust Channel System
```rust
use tokio::sync::broadcast;

pub struct EventBus {
    tx: broadcast::Sender<DeviceEvent>,
}

pub enum DeviceEvent {
    Execute { channel_id: u32, command: String },
    StatusChanged { node_id: u32, value: i32 },
}

// 发送事件
event_bus.send(DeviceEvent::Execute { ... });

// 订阅事件
let mut rx = event_bus.subscribe();
while let Ok(event) = rx.recv().await {
    match event {
        DeviceEvent::Execute { .. } => { /* 处理 */ }
        _ => {}
    }
}
```

---

## 配置加载对照

### JavaScript动态配置
```javascript
// src/config/config.js
const channel = [{ channelId: 1, statute: 'pjlink', addr: '...', port: 4352 }]
const Nodes = [{ globalId: 1, channelId: 1, ... }]

module.exports = { channel, Nodes, Scenes }
```

### Rust强类型配置
```rust
// config.json
{
  "channels": [...],
  "nodes": [...],
  "scenes": [...]
}

// Rust加载
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    channels: Vec<ChannelConfig>,
    nodes: Vec<NodeConfig>,
    scenes: Vec<SceneConfig>,
}

let config: Config = serde_json::from_str(&content)?;
```

---

## 依赖系统重构

### JavaScript版本
```javascript
// 检查依赖
checkFrontState(taskItem) {
  const node = this.getOneNode(taskItem.id)
  if (!node.depend) return true
  
  for (let dep of node.depend) {
    const depNode = this.getOneNode(dep.id)
    if (depNode.value !== dep.value) return false
  }
  return true
}
```

### Rust版本
```rust
async fn check_dependencies(&self, node: &NodeConfig) -> Result<bool> {
    let Some(deps) = &node.depend else {
        return Ok(true);
    };
    
    for dep in deps {
        let dep_node = self.nodes.get(&dep.id)
            .ok_or(DeviceError::DeviceNotFound(dep.id))?;
        
        // 使用 match 处理不同类型的依赖条件
        match (&dep.value, &dep.status) {
            (Some(expected_val), _) => {
                if dep_node.current_value != *expected_val {
                    return Ok(false);
                }
            }
            (_, Some(expected_status)) => {
                if dep_node.online != *expected_status {
                    return Ok(false);
                }
            }
            _ => return Err(DeviceError::ConfigError("无效的依赖配置".into())),
        }
    }
    
    Ok(true)
}
```

---

## 性能优化点

| 方面 | JavaScript | Rust |
|-----|-----------|------|
| 内存管理 | GC（可能停顿） | 无GC（零成本抽象） |
| 并发模型 | 单线程事件循环 | 多线程 + 异步 |
| 类型检查 | 运行时 | 编译时 |
| 错误处理 | try-catch开销 | Result零成本 |
| 数据结构 | 数组、对象 | 特化容器（DashMap等） |

---

## 迁移检查清单

- [ ] 所有协议实现功能对等
- [ ] HTTP API接口兼容
- [ ] WebSocket消息格式一致
- [ ] 配置文件可互换
- [ ] 错误代码保持不变
- [ ] 日志格式相似
- [ ] 性能指标达标
- [ ] 通过集成测试
