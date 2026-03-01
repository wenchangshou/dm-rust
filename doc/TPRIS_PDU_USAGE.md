# 特普瑞斯 PDU 协议使用指南

## 概述

**厂商**: 深圳特普瑞斯科技有限公司
**设备**: PDU (Power Distribution Unit) 智能电源分配单元
**协议**: 基于 Modbus TCP，支持 8 路开关控制
**协议标识**: `tpris-pdu`

### 功能列表

| 功能 | 命令 | 说明 |
|------|------|------|
| 读取开关状态 | `read_switch_status` | 读取8路开关状态 |
| 批量控制开关 | `write_switch_all` | 同时设置8路开关 |
| 单独控制开关 | `write_switch_single` | 控制指定开关 |

---

## 通道配置

### 基本配置

```json
{
  "channels": [
    {
      "channel_id": 10,
      "enable": true,
      "statute": "tpris-pdu",
      "arguments": {
        "addr": "192.168.1.100",
        "port": 502,
        "slave_id": 2
      }
    }
  ]
}
```

### 参数说明

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `addr` | string | ✅ | — | 设备 IP 地址，也可使用 `ip` 作为参数名 |
| `port` | number | — | `502` | Modbus TCP 端口 |
| `slave_id` | number | — | `2` | 从站地址（协议默认值为 2） |

### 节点配置示例

```json
{
  "nodes": [
    {
      "global_id": 1001,
      "channel_id": 10,
      "id": 1,
      "alias": "开关1"
    },
    {
      "global_id": 1002,
      "channel_id": 10,
      "id": 2,
      "alias": "开关2"
    }
  ]
}
```

> **说明**: 节点的 `id` 对应开关编号（1-8），通过 `read`/`write` 简化接口可直接按节点读写。

---

## 协议原理

### Modbus 寄存器映射

| 寄存器地址 | 功能码 | 用途 | 说明 |
|-----------|--------|------|------|
| `0x0030` | FC03 (读) | 读取8位开关状态 | 返回值低字节为位掩码 |
| `0x0030` | FC06 (写) | 批量设置8位开关 | 值为位掩码 |
| `0x0034` | FC06 (写) | 单独控制开关 | 高字节=编号, 低字节=动作 |

### 位掩码解析 (寄存器 0x0030)

返回值的低字节按位对应 8 路开关，低位在前：

```
位:     bit7  bit6  bit5  bit4  bit3  bit2  bit1  bit0
开关:   8号   7号   6号   5号   4号   3号   2号   1号
值:     0=关  0=关  0=关  0=关  0=关  0=关  0=关  0=关
```

**示例**：

| 十六进制 | 二进制 | 含义 |
|---------|--------|------|
| `0xFF` | `11111111` | 全部开启 |
| `0x00` | `00000000` | 全部关闭 |
| `0x0F` | `00001111` | 开关 1-4 开启, 5-8 关闭 |
| `0x55` | `01010101` | 开关 1,3,5,7 开启; 2,4,6,8 关闭 |
| `0xAA` | `10101010` | 开关 2,4,6,8 开启; 1,3,5,7 关闭 |

### 单独控制编码 (寄存器 0x0034)

写入的 16 位值编码方式：

```
高字节 (bits 15-8): 开关编号 (0x01 ~ 0x08)
低字节 (bits 7-0):  动作码 (0x01=关闭, 0x02=开启)
```

**示例**：

| 开关 | 关闭 | 开启 |
|------|------|------|
| 第1路 | `0x0101` | `0x0102` |
| 第2路 | `0x0201` | `0x0202` |
| 第3路 | `0x0301` | `0x0302` |
| 第4路 | `0x0401` | `0x0402` |
| 第5路 | `0x0501` | `0x0502` |
| 第6路 | `0x0601` | `0x0602` |
| 第7路 | `0x0701` | `0x0702` |
| 第8路 | `0x0801` | `0x0802` |

---

## HTTP API 使用

### 1. 简化接口 (read/write)

通过节点 ID 直接读写单个开关，ID 对应开关编号（1-8）。

#### 读取开关状态

```bash
curl -X POST http://localhost:8080/device/read \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "id": 1
  }'
```

**响应**：
```json
{
  "code": 0,
  "data": 1,
  "msg": "success"
}
```
> `data` 为 `1` 表示开启，`0` 表示关闭。

#### 写入开关状态

```bash
# 开启第1路
curl -X POST http://localhost:8080/device/write \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "id": 1,
    "value": 1
  }'

# 关闭第3路
curl -X POST http://localhost:8080/device/write \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "id": 3,
    "value": 0
  }'
```

**响应**：
```json
{
  "code": 0,
  "msg": "success"
}
```

---

### 2. 扩展命令接口 (execute)

#### read_switch_status — 读取全部开关状态

读取 8 路开关的完整状态。

```bash
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "command": "read_switch_status",
    "params": {}
  }'
```

**响应**：
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "raw_value": 255,
    "switches": {
      "1": true,
      "2": true,
      "3": true,
      "4": true,
      "5": true,
      "6": true,
      "7": true,
      "8": true
    }
  },
  "msg": "success"
}
```

**参数**：

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `addr` | number | — | `0x0030` | 开关状态寄存器地址 |

---

#### write_switch_all — 批量控制全部开关

同时设置 8 路开关的状态，支持两种输入方式。

**方式1：位掩码数值**

```bash
# 1-4开启, 5-8关闭 (0x0F = 二进制 00001111)
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "command": "write_switch_all",
    "params": {
      "value": 15
    }
  }'
```

**方式2：开关对象**

```bash
# 1,3,5,7开启, 2,4,6,8关闭
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "command": "write_switch_all",
    "params": {
      "switches": {
        "1": true,
        "2": false,
        "3": true,
        "4": false,
        "5": true,
        "6": false,
        "7": true,
        "8": false
      }
    }
  }'
```

**响应**：
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "value": 85,
    "binary": "01010101"
  },
  "msg": "success"
}
```

**参数**：

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `addr` | number | — | `0x0030` | 开关控制寄存器地址 |
| `value` | number | 二选一 | — | 位掩码数值 (0-255) |
| `switches` | object | 二选一 | — | 开关状态对象 `{"1": true, ...}` |

**常用 value 值速查表**：

| 场景 | value (十进制) | value (十六进制) | 二进制 |
|------|---------------|-----------------|--------|
| 全部开启 | 255 | 0xFF | 11111111 |
| 全部关闭 | 0 | 0x00 | 00000000 |
| 1-4开启, 5-8关闭 | 15 | 0x0F | 00001111 |
| 5-8开启, 1-4关闭 | 240 | 0xF0 | 11110000 |
| 奇数路开启 | 85 | 0x55 | 01010101 |
| 偶数路开启 | 170 | 0xAA | 10101010 |
| 仅第1路开启 | 1 | 0x01 | 00000001 |
| 仅第8路开启 | 128 | 0x80 | 10000000 |

---

#### write_switch_single — 单独控制开关

控制指定的某一路开关，不影响其他路的状态。

```bash
# 开启第3路
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "command": "write_switch_single",
    "params": {
      "switch_id": 3,
      "action": "on"
    }
  }'

# 关闭第5路
curl -X POST http://localhost:8080/device/execute \
  -H "Content-Type: application/json" \
  -d '{
    "channel": 10,
    "command": "write_switch_single",
    "params": {
      "switch_id": 5,
      "action": "off"
    }
  }'
```

**响应**：
```json
{
  "code": 0,
  "data": {
    "status": "success",
    "switch_id": 3,
    "action": "on"
  },
  "msg": "success"
}
```

**参数**：

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|------|------|------|--------|------|
| `addr` | number | — | `0x0034` | 单独控制寄存器地址 |
| `switch_id` | number | ✅ | — | 开关编号 (1-8) |
| `action` | string/bool | ✅ | — | 动作：`"on"`/`"off"` 或 `true`/`false` |

> **提示**：`action` 也支持 `"open"`/`"close"` 和 `"1"`/`"0"` 格式。也可以使用 `on` 参数 (bool) 替代 `action`。

---

## 场景配置

可以在场景（scene）中组合使用开关控制：

```json
{
  "scenes": [
    {
      "name": "全部开启",
      "nodes": [
        { "id": 1001, "value": 1 },
        { "id": 1002, "value": 1, "delay": 500 },
        { "id": 1003, "value": 1, "delay": 1000 },
        { "id": 1004, "value": 1, "delay": 1500 },
        { "id": 1005, "value": 1, "delay": 2000 },
        { "id": 1006, "value": 1, "delay": 2500 },
        { "id": 1007, "value": 1, "delay": 3000 },
        { "id": 1008, "value": 1, "delay": 3500 }
      ]
    },
    {
      "name": "全部关闭",
      "nodes": [
        { "id": 1008, "value": 0 },
        { "id": 1007, "value": 0, "delay": 500 },
        { "id": 1006, "value": 0, "delay": 1000 },
        { "id": 1005, "value": 0, "delay": 1500 },
        { "id": 1004, "value": 0, "delay": 2000 },
        { "id": 1003, "value": 0, "delay": 2500 },
        { "id": 1002, "value": 0, "delay": 3000 },
        { "id": 1001, "value": 0, "delay": 3500 }
      ]
    }
  ]
}
```

---

## 完整配置示例

```json
{
  "channels": [
    {
      "channel_id": 10,
      "enable": true,
      "statute": "tpris-pdu",
      "arguments": {
        "addr": "192.168.1.100",
        "port": 502,
        "slave_id": 2
      }
    }
  ],
  "nodes": [
    { "global_id": 1001, "channel_id": 10, "id": 1, "alias": "PDU-开关1" },
    { "global_id": 1002, "channel_id": 10, "id": 2, "alias": "PDU-开关2" },
    { "global_id": 1003, "channel_id": 10, "id": 3, "alias": "PDU-开关3" },
    { "global_id": 1004, "channel_id": 10, "id": 4, "alias": "PDU-开关4" },
    { "global_id": 1005, "channel_id": 10, "id": 5, "alias": "PDU-开关5" },
    { "global_id": 1006, "channel_id": 10, "id": 6, "alias": "PDU-开关6" },
    { "global_id": 1007, "channel_id": 10, "id": 7, "alias": "PDU-开关7" },
    { "global_id": 1008, "channel_id": 10, "id": 8, "alias": "PDU-开关8" }
  ],
  "scenes": [
    {
      "name": "全部开启",
      "nodes": [
        { "id": 1001, "value": 1 },
        { "id": 1002, "value": 1 },
        { "id": 1003, "value": 1 },
        { "id": 1004, "value": 1 },
        { "id": 1005, "value": 1 },
        { "id": 1006, "value": 1 },
        { "id": 1007, "value": 1 },
        { "id": 1008, "value": 1 }
      ]
    }
  ]
}
```

---

## 原始报文参考（Modbus RTU）

以下是协议原始 RTU 报文（CRC 校验），仅作为协议参考。实际通信使用 Modbus TCP，不需要 CRC。

### 读取开关状态

```
发送: 02 03 00 30 00 01 84 36
返回: 02 03 02 00 FF BC 04
      └─ FF = 11111111 → 全部开启
```

### 批量控制

```
全部开启:          02 06 00 30 00 FF 49 F0
全部关闭:          02 06 00 30 00 00 88 34
1-4开启 5-8关闭:   02 06 00 30 00 0F C9 F2
1357开启 2468关闭:  02 06 00 30 00 55 49 C9
```

### 单独控制

```
第1路关闭: 02 06 00 34 01 01 08 67    开启: 02 06 00 34 01 02 48 66
第2路关闭: 02 06 00 34 02 01 08 97    开启: 02 06 00 34 02 02 48 96
第3路关闭: 02 06 00 34 03 01 09 07    开启: 02 06 00 34 03 02 49 06
第4路关闭: 02 06 00 34 04 01 0B 37    开启: 02 06 00 34 04 02 4B 36
第5路关闭: 02 06 00 34 05 01 0A A7    开启: 02 06 00 34 05 02 4A A6
第6路关闭: 02 06 00 34 06 01 0A 57    开启: 02 06 00 34 06 02 4A 56
第7路关闭: 02 06 00 34 07 01 0B C7    开启: 02 06 00 34 07 02 4B C6
第8路关闭: 02 06 00 34 08 01 0E 37    开启: 02 06 00 34 08 02 4E 36
```

---

## 错误处理

| 错误类型 | 错误码 | 说明 |
|---------|--------|------|
| 参数错误 | `400` | 缺少必填参数或参数值无效 |
| 连接错误 | `30002` | Modbus TCP 连接失败 |
| 协议异常 | `30003` | Modbus 协议异常响应 |

**常见错误及处理**：

```json
// 开关编号超范围
{ "code": 400, "msg": "开关编号必须在1-8之间" }

// 连接失败
{ "code": 30002, "msg": "连接错误: Modbus TCP 连接失败: Connection refused" }

// 缺少参数
{ "code": 400, "msg": "缺少 switch_id 参数" }
```

---

## 测试脚本

### Bash 测试脚本

```bash
#!/bin/bash
BASE_URL="http://localhost:8080"
CHANNEL=10

echo "=== 特普瑞斯 PDU 测试 ==="

# 1. 读取全部开关状态
echo -e "\n1. 读取开关状态"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"read_switch_status\",
    \"params\": {}
  }" | jq '.'

# 2. 全部开启
echo -e "\n2. 全部开启"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"write_switch_all\",
    \"params\": { \"value\": 255 }
  }" | jq '.'

# 3. 全部关闭
echo -e "\n3. 全部关闭"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"write_switch_all\",
    \"params\": { \"value\": 0 }
  }" | jq '.'

# 4. 单独开启第1路
echo -e "\n4. 开启第1路"
curl -s -X POST "$BASE_URL/device/execute" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"command\": \"write_switch_single\",
    \"params\": { \"switch_id\": 1, \"action\": \"on\" }
  }" | jq '.'

# 5. 通过 write 接口控制第2路
echo -e "\n5. 通过 write 接口开启第2路"
curl -s -X POST "$BASE_URL/device/write" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"id\": 2,
    \"value\": 1
  }" | jq '.'

# 6. 通过 read 接口读取第1路
echo -e "\n6. 通过 read 接口读取第1路"
curl -s -X POST "$BASE_URL/device/read" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel\": $CHANNEL,
    \"id\": 1
  }" | jq '.'

echo -e "\n=== 测试完成 ==="
```

### Python 示例

```python
import requests

BASE_URL = "http://localhost:8080"
CHANNEL = 10

def read_switches():
    """读取全部开关状态"""
    resp = requests.post(f"{BASE_URL}/device/execute", json={
        "channel": CHANNEL,
        "command": "read_switch_status",
        "params": {}
    })
    result = resp.json()
    if result["code"] == 0:
        switches = result["data"]["switches"]
        for i in range(1, 9):
            status = "开启" if switches[str(i)] else "关闭"
            print(f"  开关{i}: {status}")
    return result

def set_all_switches(switch_states: dict):
    """批量设置开关 (使用 switches 对象方式)"""
    resp = requests.post(f"{BASE_URL}/device/execute", json={
        "channel": CHANNEL,
        "command": "write_switch_all",
        "params": {"switches": switch_states}
    })
    return resp.json()

def control_single(switch_id: int, on: bool):
    """单独控制某路开关"""
    resp = requests.post(f"{BASE_URL}/device/execute", json={
        "channel": CHANNEL,
        "command": "write_switch_single",
        "params": {
            "switch_id": switch_id,
            "action": "on" if on else "off"
        }
    })
    return resp.json()

# 使用示例
if __name__ == "__main__":
    # 读取状态
    print("当前开关状态:")
    read_switches()

    # 批量: 1-4开启, 5-8关闭
    print("\n设置 1-4 开启, 5-8 关闭:")
    set_all_switches({
        "1": True, "2": True, "3": True, "4": True,
        "5": False, "6": False, "7": False, "8": False
    })

    # 单独: 开启第5路
    print("\n单独开启第5路:")
    control_single(5, True)

    # 确认状态
    print("\n最终状态:")
    read_switches()
```

---

## 注意事项

1. **连接方式**: 每次命令都会创建新的 Modbus TCP 连接。对于高频操作场景，建议适当增大命令间隔。

2. **批量 vs 单独控制**:
   - `write_switch_all` 会同时设置全部 8 路状态，未指定的开关默认为关闭
   - `write_switch_single` 只影响指定的开关，不改变其他开关状态
   - 如需保持其他开关不变，建议先 `read_switch_status` 再组合后 `write_switch_all`

3. **从站地址**: 默认为 `2`（根据协议示例），如设备使用不同的从站地址请在配置中修改 `slave_id`。

4. **Modbus TCP vs RTU**: 本协议使用 Modbus TCP 通信，文档中的 RTU 报文仅作参考。TCP 模式下不需要 CRC 校验，由 TCP 协议保证数据完整性。
