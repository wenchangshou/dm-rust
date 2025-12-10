# 通用配置系统设计

## 核心理念

**框架只定义标准，协议自己解析配置参数**

这个设计原则使得系统更加灵活和可扩展：
- 框架层面只关心通道ID、协议类型、是否启用等通用字段
- 每个协议有自己的配置需求，由协议自己解析
- 添加新协议不需要修改框架代码

## 配置结构

### 通道配置（Channel）

```json
{
  "channel_id": 1,        // 通道ID（必填）
  "enable": true,         // 是否启用（必填）
  "statute": "pjlink",    // 协议类型（必填）
  // ... 其他字段由具体协议自行定义和解析
}
```

框架只处理这三个标准字段：
- `channel_id`: 唯一标识通道
- `enable`: 控制是否启用该通道
- `statute`: 指定使用哪个协议

其余所有字段都会被收集到 `params` (JSON Value) 中，传递给协议的 `from_config` 方法。

## 协议实现指南

### 1. 定义配置结构

每个协议定义自己的配置结构：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct MyProtocolConfig {
    addr: String,
    port: u16,
    timeout: Option<u64>,
    // ... 其他协议特定参数
}
```

### 2. 实现 from_config 方法

```rust
impl Protocol for MyProtocol {
    fn from_config(channel_id: u32, params: Value) -> Result<Box<dyn Protocol>> {
        // 1. 将 JSON Value 反序列化为配置结构
        let config: MyProtocolConfig = serde_json::from_value(params)
            .map_err(|e| DeviceError::ConfigError(
                format!("配置解析失败: {}", e)
            ))?;
        
        // 2. 验证配置（可选）
        if config.port == 0 {
            return Err(DeviceError::ConfigError("端口号不能为0".into()));
        }
        
        // 3. 创建协议实例
        Ok(Box::new(Self {
            channel_id,
            addr: config.addr,
            port: config.port,
            timeout: config.timeout.unwrap_or(5000),
        }))
    }
}
```

## 内置协议配置示例

### PJLink 协议

```json
{
  "channel_id": 1,
  "enable": true,
  "statute": "pjlink",
  "addr": "192.168.20.59",
  "port": 4352,
  "password": null
}
```

### Modbus TCP

```json
{
  "channel_id": 2,
  "enable": true,
  "statute": "modbus",
  "type": "tcp",
  "addr": "192.168.20.130",
  "port": 502,
  "slave_id": 1,
  "call_interval": 5000,
  "auto_call": true
}
```

### Modbus 串口

```json
{
  "channel_id": 3,
  "enable": true,
  "statute": "modbus",
  "type": "serial",
  "serial_port": "/dev/ttyUSB0",
  "baud_rate": 9600,
  "slave_id": 1
}
```

### Modbus Slave (网关模式)

```json
{
  "channel_id": 4,
  "enable": true,
  "statute": "modbus-slave",
  "map": {
    "A": 1,
    "B": 62,
    "C": 123,
    "D": 184
  },
  "device_list": [
    {
      "type": "tcp",
      "ip": "192.168.20.59",
      "port": 2000,
      "call_interval": 5000,
      "call_list": [
        {
          "type": "di",
          "device_no": 1,
          "internal_addr": 1,
          "external_addr": 0,
          "count": 16
        }
      ]
    }
  ]
}
```

### XinkeQ1 (电脑控制模块)

```json
{
  "channel_id": 5,
  "enable": true,
  "statute": "xinkeQ1",
  "addr": "192.168.20.123",
  "port": 6000
}
```

### Computer Control (WOL)

```json
{
  "channel_id": 6,
  "enable": true,
  "statute": "computerControl",
  "mac_address": "00:11:22:33:44:55",
  "broadcast_addr": "192.168.20.255"
}
```

## 添加新协议

### 步骤 1: 定义协议类型

在 `src/config/mod.rs` 中添加新的协议类型：

```rust
pub enum StatuteType {
    // ... 现有协议
    #[serde(rename = "myProtocol")]
    MyProtocol,
}
```

### 步骤 2: 创建协议文件

在 `src/protocols/` 下创建 `my_protocol.rs`：

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::protocols::Protocol;
use crate::utils::{Result, DeviceError};

#[derive(Debug, Deserialize, Serialize)]
struct MyProtocolConfig {
    // 定义你的配置字段
    param1: String,
    param2: u16,
}

pub struct MyProtocol {
    channel_id: u32,
    param1: String,
    param2: u16,
}

#[async_trait]
impl Protocol for MyProtocol {
    fn from_config(channel_id: u32, params: Value) -> Result<Box<dyn Protocol>> {
        let config: MyProtocolConfig = serde_json::from_value(params)
            .map_err(|e| DeviceError::ConfigError(format!("配置解析失败: {}", e)))?;
        
        Ok(Box::new(Self {
            channel_id,
            param1: config.param1,
            param2: config.param2,
        }))
    }
    
    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        // 实现命令执行逻辑
        todo!()
    }
    
    async fn get_status(&self) -> Result<Value> {
        // 实现状态获取逻辑
        todo!()
    }
    
    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        // 实现写入逻辑
        todo!()
    }
    
    async fn read(&self, id: u32) -> Result<i32> {
        // 实现读取逻辑
        todo!()
    }
    
    fn name(&self) -> &str {
        "myProtocol"
    }
}
```

### 步骤 3: 注册协议

在 `src/protocols/mod.rs` 中导出：

```rust
pub mod my_protocol;
pub use my_protocol::MyProtocol;
```

在 `src/device/channel_manager.rs` 中添加创建逻辑：

```rust
StatuteType::MyProtocol => {
    MyProtocol::from_config(config.channel_id, config.params.clone())?
}
```

### 步骤 4: 使用配置

在 `config.json` 中添加通道：

```json
{
  "channel_id": 100,
  "enable": true,
  "statute": "myProtocol",
  "param1": "value1",
  "param2": 1234
}
```

## 优势

### 1. 解耦
- 框架不需要知道每个协议的具体参数
- 协议变更不影响框架代码

### 2. 灵活性
- 每个协议可以有完全不同的配置结构
- 支持可选参数、默认值、复杂嵌套结构

### 3. 类型安全
- 使用 serde 进行类型安全的反序列化
- 编译时检查配置结构定义
- 运行时验证配置数据

### 4. 可扩展性
- 添加新协议只需：
  1. 定义配置结构
  2. 实现 Protocol trait
  3. 注册到枚举
- 不需要修改框架核心代码

## 命令执行的通用性

类似配置，命令执行也采用通用方式：

```rust
async fn execute(&mut self, command: &str, params: Value) -> Result<Value>
```

- `command`: 命令名称（字符串），由协议定义
- `params`: 命令参数（JSON），由协议解析
- 返回值: JSON 格式的执行结果

示例：

```json
// PJLink 开机命令
{
  "command": "powerOn",
  "params": {}
}

// Modbus 写入命令
{
  "command": "writeCoil",
  "params": {
    "addr": 100,
    "value": 1
  }
}
```

## 最佳实践

1. **配置验证**: 在 `from_config` 中验证配置的合法性
2. **默认值**: 使用 `#[serde(default)]` 为可选字段提供默认值
3. **错误消息**: 提供清晰的错误消息帮助排查配置问题
4. **文档**: 为每个协议的配置编写示例和说明
5. **向后兼容**: 添加新字段时使用 `Option<T>` 保持向后兼容

## 总结

这个通用配置系统将"配置即数据"的理念发挥到极致：

- **框架定义标准接口**：Protocol trait
- **协议实现具体逻辑**：from_config + execute
- **配置驱动行为**：JSON 格式灵活配置

这种设计使得系统既有统一的管理方式，又具备高度的灵活性和可扩展性。
