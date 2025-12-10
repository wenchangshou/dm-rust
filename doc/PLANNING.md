# Rust重构项目规划

## 设计原则

### 1. 类型安全
将JavaScript的动态类型转换为Rust的静态类型：
- 所有配置项使用强类型结构体定义
- 使用枚举表示有限的状态集（如协议类型、设备状态等）
- 避免使用 `any` 类型或过度使用 `Value`

### 2. 错误处理
- 使用 `Result<T, E>` 替代 JavaScript 的 try-catch
- 定义清晰的错误类型层次（`DeviceError`）
- 避免 `unwrap()`，优先使用 `?` 操作符
- 对外API返回友好的错误消息

### 3. 并发模型
- 使用 Tokio 异步运行时
- `DashMap` 用于线程安全的共享状态
- `Arc` + `RwLock` 用于共享可变状态
- 避免 `Mutex` 死锁（优先使用消息传递）

### 4. 模块化
```
protocols/     - 协议实现（每个协议一个文件）
device/        - 设备管理和调度
config/        - 配置定义和加载
web/           - HTTP/WebSocket服务
utils/         - 通用工具（错误、日志等）
```

## 关键重构点

### JavaScript → Rust 映射

| JavaScript | Rust |
|-----------|------|
| `Promise` | `async fn -> Result<T>` |
| `EventEmitter` | `async_channel` 或 `tokio::sync::broadcast` |
| `setTimeout` | `tokio::time::sleep` |
| `setInterval` | `loop { sleep(...) }` |
| `Map` | `HashMap` 或 `DashMap` |
| `this.xxx` | `self.xxx` |
| 原型继承 | Trait 实现 |

### 协议层设计

```rust
#[async_trait]
pub trait Protocol: Send + Sync {
    async fn execute(&mut self, cmd: &str, params: Value) -> Result<Value>;
    async fn get_status(&self) -> Result<Value>;
    async fn write(&mut self, id: u32, value: i32) -> Result<()>;
    async fn read(&self, id: u32) -> Result<i32>;
    fn name(&self) -> &str;
}
```

所有协议必须实现此 trait，确保接口一致性。

### 任务队列改进

**JavaScript版本问题：**
- 使用数组 + 定时器轮询
- 可能存在并发访问问题
- 超时检查不够精确

**Rust改进方案：**
```rust
// 使用优先队列
use std::collections::BinaryHeap;
use tokio::time::Instant;

struct Task {
    deadline: Instant,
    priority: u32,
    // ...
}

// 使用 tokio::select! 处理超时
tokio::select! {
    _ = tokio::time::sleep_until(next_deadline) => {
        // 处理超时任务
    }
    new_task = task_rx.recv() => {
        // 添加新任务
    }
}
```

### 依赖检查系统

**改进点：**
1. 类型安全的依赖定义
2. 避免循环依赖（编译期检查）
3. 支持复杂的依赖策略（AND/OR逻辑）

```rust
pub enum DependStrategy {
    Auto,   // 自动满足时执行
    Manual, // 手动触发
    All,    // 所有依赖必须满足
    Any,    // 任意依赖满足即可
}

pub struct Dependency {
    pub channel_id: Option<u32>,
    pub node_id: u32,
    pub expected_value: Option<i32>,
    pub expected_status: Option<bool>,
}
```

## 实施步骤

### Phase 1: 基础框架 ✅
- [x] 项目结构搭建
- [x] 配置类型定义
- [x] 协议 trait 定义
- [x] 基础错误处理
- [x] Web 服务器框架
- [x] WebSocket 客户端框架

### Phase 2: 协议实现
- [ ] PJLink 完整实现
- [ ] Modbus 完整实现
- [ ] ModbusSlave 实现
- [ ] XinkeQ1 实现
- [ ] ComputerControl (WOL) 实现
- [ ] Custom 协议实现

### Phase 3: 核心功能
- [ ] 配置文件加载（JSON/TOML）
- [ ] 设备初始化流程
- [ ] 依赖检查完整实现
- [ ] 任务队列优化
- [ ] 场景切换功能

### Phase 4: 通信层
- [ ] HTTP API 完整实现
- [ ] WebSocket 消息处理
- [ ] 错误响应规范化
- [ ] 心跳和重连机制

### Phase 5: 测试和优化
- [ ] 单元测试覆盖
- [ ] 集成测试
- [ ] 性能基准测试
- [ ] 内存泄漏检查
- [ ] 并发压力测试

### Phase 6: 生产就绪
- [ ] 日志系统完善
- [ ] 监控指标收集
- [ ] 配置热加载
- [ ] 优雅关闭
- [ ] Docker 镜像
- [ ] 部署文档

## 性能目标

| 指标 | 目标 | JavaScript版本 |
|-----|------|---------------|
| 启动时间 | < 100ms | ~500ms |
| 内存占用 | < 50MB | ~150MB |
| 响应延迟 | < 10ms | ~50ms |
| 并发连接 | > 1000 | ~100 |

## 兼容性考虑

### 配置格式
- 支持读取原 JavaScript 配置格式（通过 serde 转换）
- 提供配置迁移工具（JS → JSON/TOML）

### API 兼容
- HTTP API 完全兼容原版本
- WebSocket 协议保持一致
- 错误代码保持不变

### 协议兼容
- 所有硬件协议实现必须与原版本完全一致
- 保持命令格式和响应格式不变

## 风险和挑战

1. **串口通信**：Rust 生态的串口库可能与 Node.js 行为不完全一致
2. **异步模型差异**：需要仔细处理异步任务的取消和超时
3. **动态配置**：Rust 的静态类型可能限制配置的灵活性
4. **团队学习曲线**：需要时间适应 Rust 的所有权和生命周期

## 决策记录

### 为什么选择 Tokio？
- 成熟的异步运行时
- 丰富的生态（axum, tungstenite 等都基于 Tokio）
- 良好的性能和稳定性

### 为什么选择 Axum？
- 类型安全的路由
- 良好的中间件支持
- Tower 生态集成

### 为什么使用 DashMap？
- 比 Arc<RwLock<HashMap>> 更高效
- 提供细粒度锁
- API 友好

### 配置格式：JSON vs TOML
- 初期使用 JSON（与原版本兼容）
- 后续可迁移到 TOML（更人性化）
- serde 支持两种格式无缝切换
