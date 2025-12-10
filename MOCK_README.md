# Mock 协议 - 快速开始

## 简介

Mock 协议是一个模拟设备协议，专门用于接口调试和测试，无需连接真实硬件。

## 快速使用

### 1. 启动服务

```bash
# 使用 Mock 配置启动
cargo run -- -c config.mock.json
```

### 2. 运行测试脚本

```bash
# 自动化测试
./test/test_mock_protocol.sh
```

### 3. 手动测试

```bash
# 写入数据
curl -X POST http://localhost:18080/device/write \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1, "value": 100}'

# 读取数据
curl -X POST http://localhost:18080/device/read \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1}'

# 获取统计
curl -X POST http://localhost:18080/device/callMethod \
  -H "Content-Type: application/json" \
  -d '{"channel_id": 1, "method": "get_statistics", "args": {}}'
```

## 主要特性

✅ **零配置** - 开箱即用，无需硬件
✅ **内存存储** - 所有数据在内存中处理
✅ **延迟模拟** - 可配置响应延迟
✅ **错误注入** - 支持错误率配置
✅ **故障模拟** - 模拟设备故障场景
✅ **统计信息** - 记录操作统计

## 配置示例

```json
{
  "channel_id": 1,
  "statute": "mock",
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

## 支持的方法

- `simulate_fault` - 模拟设备故障
- `clear_fault` - 清除故障状态
- `get_statistics` - 获取统计信息
- `set_delay` - 设置响应延迟
- `get_value` - 读取指定地址值
- `set_value` - 设置指定地址值

## 完整文档

详细使用说明请参阅：[doc/MOCK_PROTOCOL_GUIDE.md](doc/MOCK_PROTOCOL_GUIDE.md)

## 文件清单

- `src/protocols/mock.rs` - Mock 协议实现
- `config.mock.json` - Mock 配置示例
- `doc/MOCK_PROTOCOL_GUIDE.md` - 完整使用指南
- `test/test_mock_protocol.sh` - 自动化测试脚本
