# HS-08R-16R 串口配置说明

## 快速开始

### 1. 查找串口设备

#### Linux
```bash
# 查看所有串口设备
ls -l /dev/ttyUSB* /dev/ttyS* /dev/ttyAMA*

# 查看串口设备信息
dmesg | grep tty

# 实时监控串口连接
sudo dmesg -w
# 然后插拔 USB 转串口设备,查看识别信息
```

常见串口设备:
- `/dev/ttyUSB0` - USB 转串口 (FTDI, CH340, CP2102 等)
- `/dev/ttyS0` - 主板串口 COM1
- `/dev/ttyAMA0` - 树莓派硬件串口

#### Windows
在设备管理器中查看:
1. 打开设备管理器
2. 展开 "端口(COM和LPT)"
3. 查看 COM 端口号,如 `COM1`, `COM3` 等

#### macOS
```bash
# 查看串口设备
ls -l /dev/cu.* /dev/tty.*

# 常见的 USB 转串口
/dev/cu.usbserial-*
/dev/tty.usbserial-*
```

### 2. 设置串口权限 (Linux)

```bash
# 添加当前用户到 dialout 组
sudo usermod -a -G dialout $USER

# 或者直接修改串口权限 (临时)
sudo chmod 666 /dev/ttyUSB0

# 需要重新登录才能生效
```

### 3. 测试串口连接

使用 minicom 或 screen 测试串口通信:

```bash
# 安装 minicom
sudo apt install minicom  # Ubuntu/Debian
sudo yum install minicom  # CentOS/RHEL

# 配置 minicom
sudo minicom -s
# 设置串口设备: /dev/ttyUSB0
# 设置波特率: 9600
# 设置数据位: 8
# 设置校验位: None
# 设置停止位: 1

# 或使用 screen
screen /dev/ttyUSB0 9600

# 退出 screen: Ctrl+A 然后按 K
```

## 配置示例

### 基本配置 (Linux)

```json
{
  "system": {
    "log_level": "debug",
    "http_port": 8080
  },
  "channels": [
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
  ],
  "nodes": [
    {
      "global_id": 1,
      "channel_id": 1,
      "id": 1,
      "category": "power",
      "alias": "通道1"
    }
  ]
}
```

### Windows 配置

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "hs-power-sequencer",
      "alias": "HS电源时序器",
      "arguments": {
        "port_name": "COM3",
        "baud_rate": 9600,
        "device_address": 1
      }
    }
  ]
}
```

### 树莓派配置

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "hs-power-sequencer",
      "alias": "HS电源时序器",
      "arguments": {
        "port_name": "/dev/ttyAMA0",
        "baud_rate": 9600,
        "device_address": 1
      }
    }
  ]
}
```

### 多设备配置

```json
{
  "channels": [
    {
      "channel_id": 1,
      "enable": true,
      "statute": "hs-power-sequencer",
      "alias": "时序器1",
      "arguments": {
        "port_name": "/dev/ttyUSB0",
        "baud_rate": 9600,
        "device_address": 1
      }
    },
    {
      "channel_id": 2,
      "enable": true,
      "statute": "hs-power-sequencer",
      "alias": "时序器2",
      "arguments": {
        "port_name": "/dev/ttyUSB1",
        "baud_rate": 9600,
        "device_address": 1
      }
    }
  ]
}
```

## 硬件连接说明

### RS485 接线

```
设备端          USB转485模块
-------         ------------
A/D+   <----->  A/D+
B/D-   <----->  B/D-
GND    <----->  GND
```

**注意事项**:
1. RS485 是差分信号,注意 A/B 极性
2. 如果通信异常,尝试交换 A/B 线
3. 长距离通信建议加终端电阻 (120Ω)
4. 多设备连接需要使用不同的设备地址

### RS232 接线

```
设备端          USB转RS232模块/DB9
-------         -----------------
TXD    <----->  RXD (Pin 2)
RXD    <----->  TXD (Pin 3)
GND    <----->  GND (Pin 5)
```

**注意事项**:
1. RS232 是单端信号,TX/RX 交叉连接
2. 电平标准: ±3V 到 ±15V
3. 传输距离较短 (≤15m)
4. 只支持点对点通信

## 常见问题

### 1. 找不到串口设备

**Linux**:
```bash
# 检查内核模块是否加载
lsmod | grep usbserial
lsmod | grep ftdi_sio
lsmod | grep ch341

# 手动加载模块
sudo modprobe usbserial
sudo modprobe ftdi_sio
```

**Windows**:
- 安装对应的 USB 转串口驱动 (CH340/CP2102/FTDI)
- 检查设备管理器中是否有黄色感叹号

### 2. 权限被拒绝 (Permission denied)

```bash
# 临时解决
sudo chmod 666 /dev/ttyUSB0

# 永久解决
sudo usermod -a -G dialout $USER
# 然后重新登录

# 或创建 udev 规则
sudo nano /etc/udev/rules.d/50-myusb.rules
# 添加:
SUBSYSTEM=="tty", ATTRS{idVendor}=="1a86", ATTRS{idProduct}=="7523", MODE="0666"
# 重新加载规则
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### 3. 设备忙 (Device busy)

```bash
# 查找占用串口的进程
sudo lsof /dev/ttyUSB0

# 或
sudo fuser /dev/ttyUSB0

# 结束占用进程
sudo killall minicom
sudo killall screen
```

### 4. 通信异常

- 检查波特率是否匹配 (9600)
- 检查数据位 (8), 校验位 (None), 停止位 (1)
- RS485 尝试交换 A/B 线
- 检查串口线质量
- 测量串口电平是否正常
- 查看系统日志: `journalctl -f`

## 调试技巧

### 1. 查看串口数据

使用 `strace` 监控串口通信:
```bash
sudo strace -e read,write -f ./target/release/dm-rust -c config.hs_power_sequencer.json 2>&1 | grep ttyUSB
```

### 2. 串口监听

使用 `interceptty` 监听串口数据:
```bash
# 安装
sudo apt install interceptty

# 使用虚拟串口监听
interceptty /dev/ttyUSB0 /tmp/interceptty
# 然后配置程序使用 /tmp/interceptty
```

### 3. 十六进制查看

使用 `hexdump` 或 `xxd` 查看原始数据:
```bash
# 捕获串口数据
cat /dev/ttyUSB0 | hexdump -C

# 或使用 xxd
cat /dev/ttyUSB0 | xxd
```

## 推荐硬件

### USB 转 RS485 模块
- CH340/CH341 芯片
- FTDI FT232 芯片
- CP2102 芯片

### USB 转 RS232 模块
- FTDI FT232R
- Prolific PL2303
- CH340G

购买时注意:
1. 选择知名品牌,避免山寨芯片
2. 确认支持的操作系统
3. RS485 模块最好支持自动收发切换
4. 带光电隔离更好,防止干扰

## 参考资料

- [tokio-serial 文档](https://docs.rs/tokio-serial/)
- [Linux 串口编程指南](https://tldp.org/HOWTO/Serial-Programming-HOWTO/)
- [RS485 通信协议详解](https://zh.wikipedia.org/wiki/RS-485)
