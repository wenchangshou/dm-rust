# 快速开始指南

## 前置要求

- Rust 1.72+ (推荐 1.83+)
- Cargo (Rust包管理器)

## 安装Rust

如果还没有安装Rust，请访问 https://rustup.rs/ 或运行：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 项目设置

### 1. 克隆项目

```bash
cd /path/to/devicecontrol
```

### 2. 进入Rust项目目录

```bash
cd dm-rust
```

### 3. 创建配置文件

复制示例配置文件：

```bash
cp config.example.json config.json
```

编辑 `config.json` 根据你的环境调整配置。

### 4. 构建项目

```bash
# 开发构建（包含调试信息）
cargo build

# 生产构建（优化性能）
cargo build --release
```

### 5. 运行项目

```bash
# 开发模式（带日志输出）
cargo run

# 或运行已构建的二进制文件
./target/debug/dm-rust

# 生产模式
./target/release/dm-rust
```

## 开发工作流

### 实时检查代码

```bash
# 检查代码是否能编译（不生成二进制文件，速度快）
cargo check

# 运行代码格式化
cargo fmt

# 运行代码检查工具
cargo clippy
```

### 自动重载开发

安装 cargo-watch：

```bash
cargo install cargo-watch
```

运行自动重载：

```bash
cargo watch -x run
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 显示测试输出
cargo test -- --nocapture
```

## 配置说明

### 基本配置结构

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "pjlink",
      "addr": "192.168.1.100",
      "port": 4352
    }
  ],
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 1,
      "category": "screen",
      "alias": "主投影仪"
    }
  ],
  "web_server": {
    "port": 8080
  }
}
```

### 支持的协议类型

- `pjlink` - PJLink投影仪协议
- `modbus` - Modbus标准协议
- `modbus-slave` - Modbus网关
- `xinkeQ1` - 新科Q1开关机模块
- `computerControl` - Wake-on-LAN
- `custom` - 自定义协议

## 测试API

### 使用curl测试

```bash
# 健康检查
curl http://localhost:8080/device

# 获取所有设备状态
curl -X POST http://localhost:8080/device/getAllStatus \
  -H "Content-Type: application/json"

# 写入设备
curl -X POST http://localhost:8080/device/write \
  -H "Content-Type: application/json" \
  -d '{"id": 1, "value": 1}'
```

### 使用WebSocket测试

可以使用 `websocat` 工具：

```bash
# 安装 websocat
cargo install websocat

# 连接到WebSocket服务器
websocat ws://192.168.20.100:9000
```

## 常见问题

### 编译错误：rustc版本过低

```bash
# 更新Rust到最新稳定版
rustup update stable
```

### 端口被占用

修改 `config.json` 中的 `web_server.port` 配置。

### 依赖下载慢

配置国内镜像源，在 `~/.cargo/config` 添加：

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
```

### 无法连接设备

1. 检查设备IP和端口是否正确
2. 确认网络连通性
3. 查看日志输出确认具体错误

## 日志查看

默认日志级别为 INFO，可以通过环境变量调整：

```bash
# 显示调试日志
RUST_LOG=debug cargo run

# 只显示错误日志
RUST_LOG=error cargo run

# 指定模块日志级别
RUST_LOG=dm_rust::protocols=debug cargo run
```

## 下一步

- 阅读 [README.md](README.md) 了解项目架构
- 查看 [PLANNING.md](PLANNING.md) 了解开发规划
- 参考 [PROTOCOL_MAPPING.md](PROTOCOL_MAPPING.md) 了解JS到Rust的迁移细节
- 阅读原项目文档 `../doc/配置文件说明.md` 和 `../doc/控制指令.md`
