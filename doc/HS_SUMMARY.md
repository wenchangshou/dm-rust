# HS-08R-16R 电源时序器协议 - 实现总结

## ✅ 已完成

### 1. 协议实现 (`src/protocols/hs_power_sequencer.rs`)
- ✅ **串口通信**: 使用 tokio-serial 实现 RS485/RS232 通信
- ✅ **波特率**: 9600, 8数据位, 无校验, 1停止位
- ✅ **协议帧格式**: 帧头 `5B B5` + 8字节数据
- ✅ **12路通道控制**: 支持单通道开/关
- ✅ **批量控制**: 一键开/关所有通道
- ✅ **延时控制**: 延时开/关(按预设参数)
- ✅ **参数设置**: 开/关延时参数(ms单位)
- ✅ **状态查询**: 读取所有通道状态
- ✅ **时间管理**: 设置设备时间(BCD码转换)
- ✅ **地址管理**: 读取/修改设备地址(1-255)
- ✅ **电压保护**: 过压/欠压保护参数设置
- ✅ **系统管理**: 恢复出厂设置

### 2. 系统集成
- ✅ 在 `mod.rs` 中注册协议
- ✅ 在 `config/mod.rs` 中添加协议类型
- ✅ 在 `channel_manager.rs` 中添加协议创建
- ✅ 实现 Protocol trait 标准接口
- ✅ 支持自定义方法调用

### 3. 配置文件
- ✅ `config.hs_power_sequencer.json` - 完整配置示例
- ✅ 串口设备配置 (port_name, baud_rate, device_address)
- ✅ 12路通道节点配置

### 4. 文档
- ✅ `HS_POWER_SEQUENCER_GUIDE.md` - 详细使用指南
- ✅ `HS_SERIAL_CONFIG.md` - 串口配置专题
- ✅ 所有API的使用示例
- ✅ 实际应用场景
- ✅ 故障排查指南

### 5. 测试工具
- ✅ `test_hs_power_sequencer.sh` - 15个测试用例

## 📋 核心特性

### 串口配置
```json
{
  "port_name": "/dev/ttyUSB0",  // 串口设备路径
  "baud_rate": 9600,            // 波特率
  "device_address": 1           // 设备地址
}
```

### 支持的命令 (13个)
1. `channel_on` - 通道开
2. `channel_off` - 通道关
3. `all_on` - 一键开
4. `all_off` - 一键关
5. `delayed_on` - 延时开
6. `delayed_off` - 延时关
7. `set_delay` - 设置延时参数
8. `read_status` - 读取设备状态
9. `set_time` - 设置设备时间
10. `read_address` - 读取设备地址
11. `write_address` - 修改设备地址
12. `factory_reset` - 恢复出厂设置
13. `set_voltage_protection` - 设置电压保护

### API接口
- 自定义方法: `/device/customMethod`
- 标准读写: `/device/read`, `/device/write`
- 批量操作: `/device/readMany`, `/device/writeMany`

## 🚀 快速开始

### 1. 查找串口设备
```bash
# Linux
ls -l /dev/ttyUSB* /dev/ttyS*

# Windows: 设备管理器查看 COM 端口
```

### 2. 配置权限 (Linux)
```bash
sudo usermod -a -G dialout $USER
# 重新登录生效
```

### 3. 修改配置文件
```json
{
  "channels": [{
    "channel_id": 1,
    "statute": "hs-power-sequencer",
    "arguments": {
      "port_name": "/dev/ttyUSB0",
      "baud_rate": 9600,
      "device_address": 1
    }
  }]
}
```

### 4. 启动服务
```bash
./target/release/dm-rust -c config.hs_power_sequencer.json -l info
```

### 5. 测试命令
```bash
# 开启通道1
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method":"channel_on","args":{"channel":1}}'

# 读取状态
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method":"read_status","args":{}}'
```

## 📝 配置示例

### Linux (USB转串口)
```json
{
  "port_name": "/dev/ttyUSB0",
  "baud_rate": 9600,
  "device_address": 1
}
```

### Windows
```json
{
  "port_name": "COM3",
  "baud_rate": 9600,
  "device_address": 1
}
```

### 树莓派 (硬件串口)
```json
{
  "port_name": "/dev/ttyAMA0",
  "baud_rate": 9600,
  "device_address": 1
}
```

## 🔧 硬件连接

### RS485
```
设备    USB转485
A/D+ <-> A/D+
B/D- <-> B/D-
GND  <-> GND
```

### RS232
```
设备    USB转232
TXD <-> RXD
RXD <-> TXD
GND <-> GND
```

## ⚠️ 注意事项

1. **串口设备**: 确认串口设备路径正确
2. **权限问题**: Linux 需要添加用户到 dialout 组
3. **485接口**: 只能写不能读,读取需使用232接口
4. **设备地址**: 出厂默认为1,可通过命令修改
5. **面板开关**: 使用串口控制时置于OFF位置
6. **极性接线**: RS485注意A/B极性,如通信异常可尝试交换

## 📚 文档目录

1. **HS_POWER_SEQUENCER_GUIDE.md** - 完整使用指南
   - API使用示例
   - 应用场景
   - 故障排查

2. **HS_SERIAL_CONFIG.md** - 串口配置专题
   - 查找串口设备
   - 权限设置
   - 硬件连接
   - 调试技巧

3. **config.hs_power_sequencer.json** - 配置示例

4. **test_hs_power_sequencer.sh** - 测试脚本

## 🐛 故障排查

### 1. 找不到串口
```bash
# 查看串口设备
ls -l /dev/ttyUSB* /dev/ttyS*

# 查看内核日志
dmesg | grep tty
```

### 2. 权限拒绝
```bash
# 添加用户组
sudo usermod -a -G dialout $USER

# 或临时授权
sudo chmod 666 /dev/ttyUSB0
```

### 3. 设备忙
```bash
# 查找占用进程
sudo lsof /dev/ttyUSB0

# 结束进程
sudo killall minicom
```

### 4. 通信失败
- 检查波特率是否为9600
- RS485尝试交换A/B线
- 检查串口线质量
- 确认设备供电正常

## 📊 测试覆盖

测试脚本包含15个测试用例:
1. ✅ 单通道开/关
2. ✅ 设备状态查询
3. ✅ 延时参数设置
4. ✅ 延时开启
5. ✅ 一键开/关
6. ✅ 设备地址读取
7. ✅ 时间设置
8. ✅ 标准节点接口
9. ✅ 批量读写操作

## 🎯 下一步

1. **实际硬件测试**: 使用真实设备测试所有功能
2. **性能优化**: 串口通信超时和重试机制
3. **错误处理**: 增强异常情况处理
4. **日志记录**: 完善调试日志输出
5. **文档完善**: 根据实际测试补充文档

## 技术栈

- **Rust**: 主要编程语言
- **tokio-serial**: 异步串口通信库
- **tokio**: 异步运行时
- **serde**: JSON序列化/反序列化
- **tracing**: 日志记录

## 版本信息

- **协议版本**: V1.1
- **实现日期**: 2025-11-11
- **编译状态**: ✅ 通过
- **测试状态**: 待实际硬件测试

---

**提示**: 首次使用前请仔细阅读 `HS_SERIAL_CONFIG.md` 了解串口配置!
