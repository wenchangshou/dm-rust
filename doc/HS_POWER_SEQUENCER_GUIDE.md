# HS-08R-16R 多功能电源时序器协议使用指南

## 协议概述

HS-08R-16R 是一个支持 12 路通道控制的多功能电源时序器,通过 **RS485/RS232 串口**通信。

### 通信参数
- **接口类型**: RS485/RS232 串口
- **波特率**: 9600
- **数据位**: 8
- **校验位**: 无
- **停止位**: 1
- **协议版本**: V1.1 (支持9-12路控制)

### 协议特点
- 485 只能写数据,不能读数据
- 读数据需要使用 232 接口
- 数据帧以 `5B B5` 作为帧头
- 支持延时控制(开/关延时参数,单位ms)
- 支持电压保护设置

### 串口设备命名
- **Linux**: `/dev/ttyUSB0`, `/dev/ttyS0`, `/dev/ttyAMA0` 等
- **Windows**: `COM1`, `COM2`, `COM3` 等
- **macOS**: `/dev/cu.usbserial-*`, `/dev/tty.usbserial-*` 等

## 配置说明

### 通道配置

```json
{
  "channel_id": 1,
  "enable": true,
  "statute": "hs-power-sequencer",
  "alias": "HS电源时序器",
  "arguments": {
    "port_name": "/dev/ttyUSB0",
    "baud_rate": 9600,
    "device_address": 1
  }
}
```

**参数说明**:
- `port_name`: 串口设备路径
  - Linux: `/dev/ttyUSB0`, `/dev/ttyS0` 等
  - Windows: `COM1`, `COM2` 等
  - macOS: `/dev/cu.usbserial-*` 等
- `baud_rate`: 波特率 (默认: 9600)
- `device_address`: 设备地址 (出厂默认: 1, 范围: 1-255)

### 节点配置

每个通道对应一个节点 (通道1-12):

```json
{
  "global_id": 1,
  "channel_id": 1,
  "id": 1,
  "category": "power",
  "alias": "通道1"
}
```

## API使用

### 1. 基础控制命令

#### 通道开
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "channel_on",
    "args": {"channel": 1}
  }'
```

**参数**:
- `channel`: 通道号 (1-12)

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {"success": true}
}
```

#### 通道关
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "channel_off",
    "args": {"channel": 1}
  }'
```

### 2. 批量控制

#### 一键开
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "all_on",
    "args": {}
  }'
```

#### 一键关
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "all_off",
    "args": {}
  }'
```

#### 延时开
按照预设的延时参数依次开启所有通道:
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "delayed_on",
    "args": {}
  }'
```

#### 延时关
按照预设的延时参数依次关闭所有通道:
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "delayed_off",
    "args": {}
  }'
```

### 3. 延时参数设置

#### 设置通道开延时
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "set_delay",
    "args": {
      "channel": 1,
      "delay_ms": 1000,
      "is_on": true
    }
  }'
```

**参数**:
- `channel`: 通道号 (1-12)
- `delay_ms`: 延时时间,单位毫秒 (如 1000ms = 1秒)
- `is_on`: true=设置开延时, false=设置关延时

**示例**: 设置通道1开延时为1秒 (1000ms)

#### 设置通道关延时
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "set_delay",
    "args": {
      "channel": 1,
      "delay_ms": 500,
      "is_on": false
    }
  }'
```

### 4. 状态查询

#### 读取设备状态
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "read_status",
    "args": {}
  }'
```

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "channels": [true, false, true, false, false, false, false, false]
  }
}
```

`channels` 数组表示各通道状态: `true`=开启, `false`=关闭

### 5. 时间设置

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "set_time",
    "args": {
      "year": 25,
      "month": 11,
      "day": 11,
      "hour": 14,
      "minute": 30,
      "second": 0
    }
  }'
```

**参数** (均为十进制):
- `year`: 年份的后两位 (如 2025年 → 25)
- `month`: 月份 (1-12)
- `day`: 日期 (1-31)
- `hour`: 小时 (0-23)
- `minute`: 分钟 (0-59)
- `second`: 秒 (0-59, 可选,默认0)

### 6. 设备地址管理

#### 读取设备地址
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "read_address",
    "args": {}
  }'
```

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {"address": 1}
}
```

#### 修改设备地址
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "write_address",
    "args": {"address": 5}
  }'
```

**注意**: 修改设备地址后,需要同步更新配置文件中的 `device_address` 参数

### 7. 电压保护设置

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "set_voltage_protection",
    "args": {
      "over_voltage": 250,
      "under_voltage": 200,
      "hysteresis": 20,
      "over_enable": true,
      "under_enable": true
    }
  }'
```

**参数**:
- `over_voltage`: 过压保护值 (V,十进制)
- `under_voltage`: 欠压保护值 (V,十进制)
- `hysteresis`: 回差值 (V,十进制,如20V)
- `over_enable`: 是否启用过压保护
- `under_enable`: 是否启用欠压保护

### 8. 恢复出厂设置

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "factory_reset",
    "args": {}
  }'
```

**警告**: 此操作会将所有参数恢复到出厂设置!

## 节点读写接口

除了自定义方法,也可以使用标准的节点读写接口:

### 写入节点 (通道控制)

```bash
# 开启通道1
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"global_id": 1, "value": 1}'

# 关闭通道1
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"global_id": 1, "value": 0}'
```

### 读取节点状态

```bash
curl -X GET "http://localhost:8080/device/read?global_id=1"
```

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "value": 1
  }
}
```

`value`: 1=开启, 0=关闭

### 批量操作

#### 批量写入
```bash
curl -X POST http://localhost:8080/device/writeMany \
  -H 'Content-Type: application/json' \
  -d '{
    "items": [
      {"id": 1, "value": 1},
      {"id": 2, "value": 1},
      {"id": 3, "value": 0}
    ]
  }'
```

#### 批量读取
```bash
curl -X POST http://localhost:8080/device/readMany \
  -H 'Content-Type: application/json' \
  -d '{"ids": [1, 2, 3, 4, 5]}'
```

## 使用场景示例

### 场景1: 教室设备顺序开机

设置各通道开启延时,避免瞬间电流过大:

```bash
# 设置通道1-8的开延时,每个通道间隔1秒
for i in {1..8}; do
  delay=$((i * 1000))
  curl -s -X POST http://localhost:8080/device/customMethod \
    -H 'Content-Type: application/json' \
    -d "{
      \"channel_id\": 1,
      \"method\": \"set_delay\",
      \"args\": {
        \"channel\": $i,
        \"delay_ms\": $delay,
        \"is_on\": true
      }
    }"
done

# 执行延时开启
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "delayed_on",
    "args": {}
  }'
```

### 场景2: 紧急关闭所有设备

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "all_off",
    "args": {}
  }'
```

### 场景3: 检查设备状态

```bash
# 读取所有通道状态
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "read_status",
    "args": {}
  }' | jq .
```

## 注意事项

1. **设备地址**: 出厂默认为 `01`,可通过 `write_address` 修改
2. **面板开关**: 使用串口控制时,设备面板上的开关应置于 `OFF` 位置
3. **延时单位**: 所有延时参数单位均为毫秒 (ms)
4. **电压保护**: 设置电压保护参数时,注意回差值的计算 (十六进制BCD码)
5. **485限制**: RS485接口只能写数据,读取状态需使用RS232接口

## 支持的方法列表

- `channel_on` - 通道开
- `channel_off` - 通道关
- `all_on` - 一键开
- `all_off` - 一键关
- `delayed_on` - 延时开
- `delayed_off` - 延时关
- `set_delay` - 设置延时参数
- `read_status` - 读取设备状态
- `set_time` - 设置设备时间
- `read_address` - 读取设备地址
- `write_address` - 修改设备地址
- `factory_reset` - 恢复出厂设置
- `set_voltage_protection` - 设置电压保护

## 故障排查

### 1. 无法打开串口
- 检查串口设备路径是否正确
  ```bash
  # Linux 查看串口设备
  ls -l /dev/ttyUSB* /dev/ttyS*
  
  # Windows 查看串口
  # 在设备管理器中查看端口(COM和LPT)
  ```
- 确认串口设备存在且有访问权限
  ```bash
  # Linux 添加用户到 dialout 组
  sudo usermod -a -G dialout $USER
  # 需要重新登录才能生效
  ```
- 检查串口是否被其他程序占用

### 2. 命令执行失败
- 确认设备地址 `device_address` 配置正确
- 检查通道号范围 (1-12)
- 确认波特率设置正确 (9600)
- 检查串口线连接是否正常
- 查看服务器日志获取详细错误信息

### 3. 状态读取为空
- RS485 接口不支持读取,需使用 RS232
- 确认设备支持状态查询功能
- 检查串口类型配置 (485/232)

### 4. 通信超时
- 检查波特率是否匹配
- 确认设备供电正常
- 检查串口线质量和连接
- 尝试增加超时时间

## 技术支持

如有问题,请查看服务器日志:
```bash
./target/release/dm-rust -c config.hs_power_sequencer.json -l debug
```
