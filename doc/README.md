# 设备控制系统 - Rust重构版

## 项目概述

这是一个设备控制系统的Rust重构版本，原项目使用Node.js开发。该系统用于统一管理和控制各种硬件设备，包括投影仪、LED显示屏、电脑、灯光等。

### 原项目 vs Rust重构

**原项目特点：**
- 使用Node.js + Babel (ES6)
- 动态类型，配置灵活但类型不安全
- 通过修改配置文件适配不同现场环境

**Rust重构优势：**
- 强类型系统，编译期发现错误
- 更好的性能和内存安全
- 并发安全（无需担心数据竞争）
- 严格的错误处理

## 架构设计

### 核心模块

```
dm-rust/
├── src/
│   ├── main.rs              # 程序入口
│   ├── config/              # 配置管理
│   │   └── mod.rs          # 配置结构定义和加载
│   ├── device/              # 设备控制核心
│   │   └── mod.rs          # DeviceController实现
│   ├── protocols/           # 协议实现
│   │   ├── mod.rs          # Protocol trait定义
│   │   ├── pjlink.rs       # PJLink投影仪协议
│   │   ├── modbus.rs       # Modbus标准协议
│   │   ├── modbus_slave.rs # Modbus网关
│   │   ├── xinke_q1.rs     # 新科Q1开关机模块
│   │   ├── computer_control.rs # WOL协议
│   │   └── custom.rs       # 自定义协议
│   ├── web/                 # Web服务
│   │   ├── mod.rs
│   │   └── server.rs       # HTTP API服务器
│   └── utils/               # 工具模块
│       ├── error.rs        # 错误类型定义
│       └── logger.rs       # 日志工具
```

### 数据流

1. **配置加载** → 从配置文件读取通道、节点、场景配置
2. **设备初始化** → 根据配置创建Protocol实例
3. **命令接收** → 通过 HTTP API 接收控制命令
4. **依赖检查** → 检查节点依赖条件是否满足
5. **任务调度** → 未满足条件的任务进入队列，定期重试
6. **协议执行** → 调用对应Protocol实现发送控制指令

## 支持的协议

| 协议名称 | 用途 | 实现状态 |
|---------|------|---------|
| PJLink | 投影仪标准控制协议 | ✅ 基础框架 |
| Modbus | 工业标准协议（遥信/遥控） | ✅ 基础框架 |
| ModbusSlave | Modbus网关（管理多组Modbus） | ✅ 基础框架 |
| XinkeQ1 | 新科开关机模块 | ✅ 基础框架 |
| ComputerControl | WOL开机/UDP关机/UDP控制/心跳/Ping检测 | ✅ 已完善 |
| Custom | 自定义协议 | ✅ 基础框架 |
| BFHD1 | 内蒙电脑开关机 | 🚧 待实现 |
| NmDk | 内蒙灯光控制 | 🚧 待实现 |
| Vivitek | Vivitek设备 | 🚧 待实现 |
| HikvisionLed | 海康威视LED | 🚧 待实现 |

## 开发指南

### 环境要求

- Rust 1.70+
- Cargo

### 构建和运行

```bash
# 进入项目目录
cd dm-rust

# 构建项目
cargo build

# 开发模式运行（带详细日志）
cargo run

# 生产模式构建
cargo build --release

# 运行测试
cargo test
```

### 添加新协议

1. 在 `src/protocols/` 创建新文件，如 `my_protocol.rs`
2. 实现 `Protocol` trait：
   ```rust
   use async_trait::async_trait;
   use crate::protocols::Protocol;
   
   pub struct MyProtocol {
       // 字段
   }
   
   #[async_trait]
   impl Protocol for MyProtocol {
       async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
           // 实现
       }
       // ... 其他方法
   }
   ```
3. 在 `src/protocols/mod.rs` 中导出
4. 在 `src/device/mod.rs` 的 `create_protocol` 中添加创建逻辑
5. 在 `src/config/mod.rs` 的 `StatuteType` 枚举中添加新类型

## 快速开始

### 前置要求

- Rust 1.70+ (推荐使用 1.91+)
- Cargo (随Rust安装)

### 安装运行

```bash
# 克隆项目
git clone <repository-url>
cd devicecontrol/dm-rust

# 编译
cargo build --release

# 运行（使用默认配置）
cargo run

# 或直接运行编译后的二进制文件
./target/release/dm-rust
```

### 配置文件

系统提供了多个配置示例：

1. **config.minimal.json** - 最小配置示例
   - 仅包含必要字段
   - 适合快速测试

2. **config.example.json** - 完整配置示例
   - 包含所有支持的协议
   - 详细的参数说明
   - 包含注释帮助理解

3. **config.classroom.json** - 智慧教室场景
   - 真实应用场景配置
   - 多种场景模式（上课、投影、自习等）
   - 展示依赖关系和场景编排

使用自定义配置：

```bash
# 复制示例配置
cp config.example.json config.json

# 编辑配置文件
vim config.json

# 运行（默认读取 config.json）
cargo run
```

详细配置说明请参考 [CONFIGURATION.md](CONFIGURATION.md)

## HTTP API 控制

系统通过 HTTP REST API 提供设备控制功能，默认端口 8080。

### 主要接口

```bash
# 写入设备（控制设备开关）
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"value":1}'

# 读取设备状态
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{"id":1}'

# 获取所有节点状态
curl -X POST http://localhost:8080/device/getAllNodeStates

# 执行场景
curl -X POST http://localhost:8080/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"开机场景"}'

# 调用自定义方法（新功能）
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"set_input","arguments":{"source":"hdmi1"}}'

# 获取通道支持的方法列表
curl -X POST http://localhost:8080/device/getMethods \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1}'

# 批量读取多个数据点（新功能）
curl -X POST http://localhost:8080/device/batchRead \
  -H 'Content-Type: application/json' \
  -d '{
    "items": [
      {"name":"温度","channel_id":3,"addr":100,"type":"int16"},
      {"name":"压力","channel_id":3,"addr":200,"type":"float32"},
      {"name":"流量","channel_id":3,"addr":300,"type":"uint32"}
    ]
  }'
```

完整 API 文档请参考 [HTTP_API.md](HTTP_API.md)

### 批量读取接口 ⭐ 新功能

批量读取接口允许在单个请求中同时读取多个不同通道的数据点，支持：
- **跨通道读取** - 同时读取多个设备的数据
- **混合协议** - Modbus、PJLink 等统一接口
- **自定义命名** - 为每个数据点指定易识别的名称
- **部分失败容错** - 单点失败不影响其他数据读取

快速参考：[BATCH_READ_QUICK_REF.md](BATCH_READ_QUICK_REF.md)  
完整文档：[BATCH_READ_API.md](BATCH_READ_API.md)  
实现总结：[BATCH_READ_SUMMARY.md](BATCH_READ_SUMMARY.md)  
电脑控制：[COMPUTER_CONTROL_USAGE.md](COMPUTER_CONTROL_USAGE.md)

### 自定义方法支持

系统支持为每个通道定义自定义方法，实现灵活的设备控制。详细说明请参考 [CUSTOM_METHODS_GUIDE.md](CUSTOM_METHODS_GUIDE.md)

### 配置文件结构

## API接口

所有控制接口统一使用 HTTP REST API，详细文档请参考 [HTTP_API.md](HTTP_API.md)

### 快速示例

```bash
# 1. 开启投影仪
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"id":1,"value":1}'

# 2. 执行开机场景
curl -X POST http://localhost:8080/device/executeScene \
  -H 'Content-Type: application/json' \
  -d '{"name":"开机场景"}'

# 3. 查询所有设备状态
curl -X POST http://localhost:8080/device/getAllNodeStates
```

## 关键差异点（JS vs Rust）

### 类型安全
**JS:** 配置对象可以随意添加字段
**Rust:** 所有字段必须在结构体中定义，使用 `#[serde(flatten)]` 处理动态配置

### 错误处理
**JS:** 使用 try-catch 和 Promise rejection
**Rust:** 使用 `Result<T, E>` 类型，强制处理错误

### 并发模型
**JS:** Event Loop + async/await
**Rust:** Tokio异步运行时，编译期保证线程安全

### 依赖管理
**JS:** npm/package.json
**Rust:** Cargo.toml

## 待办事项

- [ ] 完善各协议的具体实现（PJLink、Modbus等）
- [ ] 实现配置文件加载（从JSON/TOML）
- [ ] 完善依赖检查逻辑
- [ ] 添加场景切换功能
- [ ] 实现串口通信
- [ ] 添加单元测试和集成测试
- [ ] 性能优化和错误处理完善
- [ ] 添加监控和指标收集
- [ ] 文档完善

## 参考资料

- 原项目文档：`../doc/配置文件说明.md`
- 原项目API：`../doc/控制指令.md`
- Tokio文档：https://tokio.rs
- Axum文档：https://docs.rs/axum
