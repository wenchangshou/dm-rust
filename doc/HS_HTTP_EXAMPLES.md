# HS-08R-16R HTTP API 调用示例

本文档提供 HS-08R-16R 电源时序器的完整 HTTP API 调用示例。

## 目录

- [基础配置](#基础配置)
- [1. 单通道控制](#1-单通道控制)
- [2. 批量控制](#2-批量控制)
- [3. 延时控制](#3-延时控制)
- [4. 状态查询](#4-状态查询)
- [5. 参数设置](#5-参数设置)
- [6. 时间管理](#6-时间管理)
- [7. 地址管理](#7-地址管理)
- [8. 系统管理](#8-系统管理)
- [9. 标准节点接口](#9-标准节点接口)
- [10. 批量读写](#10-批量读写)
- [完整测试脚本](#完整测试脚本)

---

## 基础配置

确保服务已启动：
```bash
./target/release/dm-rust -c config.hs_power_sequencer.json -l info
```

默认配置：
- **HTTP端口**: 8080
- **channel_id**: 1
- **设备地址**: 1
- **串口**: /dev/pts/31 (或 /dev/ttyUSB0)

---

## 1. 单通道控制

### 1.1 开启通道1
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "channel_on",
    "args": {
      "channel": 1
    }
  }'
```

**响应示例**:
```json
{
  "success": true,
  "result": "通道 1 已开启"
}
```

### 1.2 关闭通道1
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "channel_off",
    "args": {
      "channel": 1
    }
  }'
```

### 1.3 控制多个通道（依次调用）
```bash
# 开启通道1-3
for i in 1 2 3; do
  curl -X POST http://localhost:8080/device/customMethod \
    -H 'Content-Type: application/json' \
    -d "{\"channel_id\":1,\"method\":\"channel_on\",\"args\":{\"channel\":$i}}"
  echo ""
  sleep 0.5
done
```

---

## 2. 批量控制

### 2.1 一键开启所有通道
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "all_on",
    "args": {}
  }'
```

**响应示例**:
```json
{
  "success": true,
  "result": "所有通道已开启"
}
```

### 2.2 一键关闭所有通道
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "all_off",
    "args": {}
  }'
```

---

## 3. 延时控制

### 3.1 设置延时参数
设置通道1的开启延时为2000ms，关闭延时为1000ms：

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "set_delay",
    "args": {
      "channel": 1,
      "on_delay_ms": 2000,
      "off_delay_ms": 1000
    }
  }'
```

**响应示例**:
```json
{
  "success": true,
  "result": "通道 1 延时参数已设置: 开延时=2000ms, 关延时=1000ms"
}
```

### 3.2 延时开启
按预设参数延时开启通道1：

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "delayed_on",
    "args": {
      "channel": 1
    }
  }'
```

### 3.3 延时关闭
按预设参数延时关闭通道1：

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "delayed_off",
    "args": {
      "channel": 1
    }
  }'
```

### 3.4 设置所有通道延时
```bash
# 设置所有12路通道的延时参数
for i in {1..12}; do
  curl -X POST http://localhost:8080/device/customMethod \
    -H 'Content-Type: application/json' \
    -d "{
      \"channel_id\": 1,
      \"method\": \"set_delay\",
      \"args\": {
        \"channel\": $i,
        \"on_delay_ms\": $((i * 500)),
        \"off_delay_ms\": $((i * 300))
      }
    }"
  echo ""
done
```

---

## 4. 状态查询

### 4.1 读取设备状态
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "read_status",
    "args": {}
  }'
```

**响应示例**:
```json
{
  "success": true,
  "result": {
    "device_address": 1,
    "channel_states": [
      {"channel": 1, "state": "on"},
      {"channel": 2, "state": "off"},
      {"channel": 3, "state": "on"},
      {"channel": 4, "state": "off"},
      {"channel": 5, "state": "off"},
      {"channel": 6, "state": "off"},
      {"channel": 7, "state": "off"},
      {"channel": 8, "state": "off"},
      {"channel": 9, "state": "off"},
      {"channel": 10, "state": "off"},
      {"channel": 11, "state": "off"},
      {"channel": 12, "state": "off"}
    ],
    "on_count": 2,
    "off_count": 10
  }
}
```

### 4.2 解析状态（使用 jq）
```bash
# 查看开启的通道数量
curl -s -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method":"read_status","args":{}}' | jq '.result.on_count'

# 列出所有开启的通道
curl -s -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method":"read_status","args":{}}' | \
  jq '.result.channel_states[] | select(.state == "on") | .channel'
```

---

## 5. 参数设置

### 5.1 设置电压保护参数
设置过压保护为250V，欠压保护为180V：

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "set_voltage_protection",
    "args": {
      "over_voltage": 250,
      "under_voltage": 180
    }
  }'
```

**响应示例**:
```json
{
  "success": true,
  "result": "电压保护已设置: 过压=250V, 欠压=180V"
}
```

---

## 6. 时间管理

### 6.1 设置设备时间
设置为 2025年11月11日 14:30:00：

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "set_time",
    "args": {
      "year": 2025,
      "month": 11,
      "day": 11,
      "hour": 14,
      "minute": 30,
      "second": 0
    }
  }'
```

### 6.2 使用当前系统时间
```bash
# Linux/macOS
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{
    \"channel_id\": 1,
    \"method\": \"set_time\",
    \"args\": {
      \"year\": $(date +%Y),
      \"month\": $(date +%-m),
      \"day\": $(date +%-d),
      \"hour\": $(date +%-H),
      \"minute\": $(date +%-M),
      \"second\": $(date +%-S)
    }
  }"
```

---

## 7. 地址管理

### 7.1 读取设备地址
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "read_address",
    "args": {}
  }'
```

**响应示例**:
```json
{
  "success": true,
  "result": {
    "address": 1
  }
}
```

### 7.2 修改设备地址
将设备地址修改为5：

```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "write_address",
    "args": {
      "new_address": 5
    }
  }'
```

**注意**: 修改地址后需要更新配置文件中的 `device_address` 参数！

---

## 8. 系统管理

### 8.1 恢复出厂设置
```bash
curl -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method": "factory_reset",
    "args": {}
  }'
```

**警告**: 此操作将清除所有延时参数和设置，谨慎使用！

---

## 9. 标准节点接口

### 9.1 读取节点状态（通过 global_id）
```bash
# 读取 global_id=1 的节点（对应通道1）
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{
    "global_id": 1
  }'
```

**响应示例**:
```json
{
  "success": true,
  "value": 1.0  // 1=开启, 0=关闭
}
```

### 9.2 写入节点状态
```bash
# 开启 global_id=1 的节点
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{
    "global_id": 1,
    "value": 1
  }'

# 关闭 global_id=1 的节点
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{
    "global_id": 1,
    "value": 0
  }'
```

---

## 10. 批量读写

### 10.1 批量读取多个节点
```bash
curl -X POST http://localhost:8080/device/readMany \
  -H 'Content-Type: application/json' \
  -d '{
    "global_ids": [1, 2, 3, 4, 5, 6]
  }'
```

**响应示例**:
```json
{
  "success": true,
  "results": [
    {"global_id": 1, "success": true, "value": 1.0},
    {"global_id": 2, "success": true, "value": 0.0},
    {"global_id": 3, "success": true, "value": 1.0},
    {"global_id": 4, "success": true, "value": 0.0},
    {"global_id": 5, "success": true, "value": 0.0},
    {"global_id": 6, "success": true, "value": 0.0}
  ]
}
```

### 10.2 批量写入多个节点
```bash
# 开启通道1-4，关闭通道5-6
curl -X POST http://localhost:8080/device/writeMany \
  -H 'Content-Type: application/json' \
  -d '{
    "writes": [
      {"global_id": 1, "value": 1},
      {"global_id": 2, "value": 1},
      {"global_id": 3, "value": 1},
      {"global_id": 4, "value": 1},
      {"global_id": 5, "value": 0},
      {"global_id": 6, "value": 0}
    ]
  }'
```

### 10.3 读取所有12路通道
```bash
curl -X POST http://localhost:8080/device/readMany \
  -H 'Content-Type: application/json' \
  -d '{
    "global_ids": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
  }'
```

---

## 完整测试脚本

### test_hs_http.sh
```bash
#!/bin/bash

BASE_URL="http://localhost:8080"
CHANNEL_ID=1

echo "=========================================="
echo "HS-08R-16R HTTP API 测试脚本"
echo "=========================================="
echo ""

# 测试1: 单通道控制
echo "【测试1】单通道控制"
echo "开启通道1..."
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"channel_on\",\"args\":{\"channel\":1}}" | jq
sleep 1

echo "关闭通道1..."
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"channel_off\",\"args\":{\"channel\":1}}" | jq
sleep 1
echo ""

# 测试2: 读取状态
echo "【测试2】读取设备状态"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"read_status\",\"args\":{}}" | jq
echo ""

# 测试3: 设置延时参数
echo "【测试3】设置延时参数"
echo "设置通道1延时: 开2s, 关1s"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"set_delay\",\"args\":{\"channel\":1,\"on_delay_ms\":2000,\"off_delay_ms\":1000}}" | jq
sleep 1
echo ""

# 测试4: 延时开启
echo "【测试4】延时开启通道1（等待2秒...）"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"delayed_on\",\"args\":{\"channel\":1}}" | jq
sleep 3
echo ""

# 测试5: 一键关闭
echo "【测试5】一键关闭所有通道"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"all_off\",\"args\":{}}" | jq
sleep 1
echo ""

# 测试6: 一键开启
echo "【测试6】一键开启所有通道"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"all_on\",\"args\":{}}" | jq
sleep 1
echo ""

# 测试7: 批量读取
echo "【测试7】批量读取前6路通道状态"
curl -s -X POST $BASE_URL/device/readMany \
  -H 'Content-Type: application/json' \
  -d '{"global_ids":[1,2,3,4,5,6]}' | jq
echo ""

# 测试8: 批量写入
echo "【测试8】批量写入: 开启1-3, 关闭4-6"
curl -s -X POST $BASE_URL/device/writeMany \
  -H 'Content-Type: application/json' \
  -d '{"writes":[{"global_id":1,"value":1},{"global_id":2,"value":1},{"global_id":3,"value":1},{"global_id":4,"value":0},{"global_id":5,"value":0},{"global_id":6,"value":0}]}' | jq
sleep 1
echo ""

# 测试9: 读取设备地址
echo "【测试9】读取设备地址"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"read_address\",\"args\":{}}" | jq
echo ""

# 测试10: 设置电压保护
echo "【测试10】设置电压保护参数"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"set_voltage_protection\",\"args\":{\"over_voltage\":250,\"under_voltage\":180}}" | jq
echo ""

# 测试11: 设置设备时间
echo "【测试11】设置设备时间为当前系统时间"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{
    \"channel_id\": $CHANNEL_ID,
    \"method\": \"set_time\",
    \"args\": {
      \"year\": $(date +%Y),
      \"month\": $(date +%-m),
      \"day\": $(date +%-d),
      \"hour\": $(date +%-H),
      \"minute\": $(date +%-M),
      \"second\": $(date +%-S)
    }
  }" | jq
echo ""

# 最终状态
echo "【最终状态】读取所有通道状态"
curl -s -X POST $BASE_URL/device/customMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method\":\"read_status\",\"args\":{}}" | jq

echo ""
echo "=========================================="
echo "测试完成!"
echo "=========================================="
```

### 使用方法
```bash
# 添加执行权限
chmod +x test_hs_http.sh

# 运行测试
./test_hs_http.sh
```

---

## Python 示例

### hs_control.py
```python
#!/usr/bin/env python3
import requests
import json
import time
from typing import Dict, List, Any

class HSPowerSequencer:
    """HS-08R-16R 电源时序器 HTTP 客户端"""
    
    def __init__(self, base_url: str = "http://localhost:8080", channel_id: int = 1):
        self.base_url = base_url
        self.channel_id = channel_id
    
    def custom_method(self, method: str, args: Dict[str, Any]) -> Dict:
        """调用自定义方法"""
        url = f"{self.base_url}/device/customMethod"
        payload = {
            "channel_id": self.channel_id,
            "method": method,
            "args": args
        }
        response = requests.post(url, json=payload)
        return response.json()
    
    def channel_on(self, channel: int) -> Dict:
        """开启指定通道"""
        return self.custom_method("channel_on", {"channel": channel})
    
    def channel_off(self, channel: int) -> Dict:
        """关闭指定通道"""
        return self.custom_method("channel_off", {"channel": channel})
    
    def all_on(self) -> Dict:
        """一键开启所有通道"""
        return self.custom_method("all_on", {})
    
    def all_off(self) -> Dict:
        """一键关闭所有通道"""
        return self.custom_method("all_off", {})
    
    def delayed_on(self, channel: int) -> Dict:
        """延时开启指定通道"""
        return self.custom_method("delayed_on", {"channel": channel})
    
    def delayed_off(self, channel: int) -> Dict:
        """延时关闭指定通道"""
        return self.custom_method("delayed_off", {"channel": channel})
    
    def set_delay(self, channel: int, on_delay_ms: int, off_delay_ms: int) -> Dict:
        """设置通道延时参数"""
        return self.custom_method("set_delay", {
            "channel": channel,
            "on_delay_ms": on_delay_ms,
            "off_delay_ms": off_delay_ms
        })
    
    def read_status(self) -> Dict:
        """读取设备状态"""
        return self.custom_method("read_status", {})
    
    def set_time(self, year: int, month: int, day: int, 
                 hour: int, minute: int, second: int) -> Dict:
        """设置设备时间"""
        return self.custom_method("set_time", {
            "year": year, "month": month, "day": day,
            "hour": hour, "minute": minute, "second": second
        })
    
    def read_address(self) -> Dict:
        """读取设备地址"""
        return self.custom_method("read_address", {})
    
    def write_address(self, new_address: int) -> Dict:
        """修改设备地址"""
        return self.custom_method("write_address", {"new_address": new_address})
    
    def set_voltage_protection(self, over_voltage: int, under_voltage: int) -> Dict:
        """设置电压保护参数"""
        return self.custom_method("set_voltage_protection", {
            "over_voltage": over_voltage,
            "under_voltage": under_voltage
        })
    
    def factory_reset(self) -> Dict:
        """恢复出厂设置"""
        return self.custom_method("factory_reset", {})
    
    def read_many(self, global_ids: List[int]) -> Dict:
        """批量读取节点"""
        url = f"{self.base_url}/device/readMany"
        response = requests.post(url, json={"global_ids": global_ids})
        return response.json()
    
    def write_many(self, writes: List[Dict[str, Any]]) -> Dict:
        """批量写入节点"""
        url = f"{self.base_url}/device/writeMany"
        response = requests.post(url, json={"writes": writes})
        return response.json()


# 使用示例
if __name__ == "__main__":
    hs = HSPowerSequencer()
    
    print("1. 开启通道1")
    result = hs.channel_on(1)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    time.sleep(1)
    
    print("\n2. 读取设备状态")
    result = hs.read_status()
    print(json.dumps(result, indent=2, ensure_ascii=False))
    
    print("\n3. 设置延时参数")
    result = hs.set_delay(1, 2000, 1000)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    
    print("\n4. 批量读取前6路通道")
    result = hs.read_many([1, 2, 3, 4, 5, 6])
    print(json.dumps(result, indent=2, ensure_ascii=False))
    
    print("\n5. 一键关闭所有通道")
    result = hs.all_off()
    print(json.dumps(result, indent=2, ensure_ascii=False))
```

---

## JavaScript 示例

### hs_control.js
```javascript
const axios = require('axios');

class HSPowerSequencer {
    constructor(baseUrl = 'http://localhost:8080', channelId = 1) {
        this.baseUrl = baseUrl;
        this.channelId = channelId;
    }

    async customMethod(method, args) {
        const url = `${this.baseUrl}/device/customMethod`;
        const payload = {
            channel_id: this.channelId,
            method: method,
            args: args
        };
        const response = await axios.post(url, payload);
        return response.data;
    }

    async channelOn(channel) {
        return await this.customMethod('channel_on', { channel });
    }

    async channelOff(channel) {
        return await this.customMethod('channel_off', { channel });
    }

    async allOn() {
        return await this.customMethod('all_on', {});
    }

    async allOff() {
        return await this.customMethod('all_off', {});
    }

    async setDelay(channel, onDelayMs, offDelayMs) {
        return await this.customMethod('set_delay', {
            channel,
            on_delay_ms: onDelayMs,
            off_delay_ms: offDelayMs
        });
    }

    async readStatus() {
        return await this.customMethod('read_status', {});
    }

    async readMany(globalIds) {
        const url = `${this.baseUrl}/device/readMany`;
        const response = await axios.post(url, { global_ids: globalIds });
        return response.data;
    }

    async writeMany(writes) {
        const url = `${this.baseUrl}/device/writeMany`;
        const response = await axios.post(url, { writes });
        return response.data;
    }
}

// 使用示例
(async () => {
    const hs = new HSPowerSequencer();

    console.log('1. 开启通道1');
    let result = await hs.channelOn(1);
    console.log(JSON.stringify(result, null, 2));

    console.log('\n2. 读取设备状态');
    result = await hs.readStatus();
    console.log(JSON.stringify(result, null, 2));

    console.log('\n3. 批量读取前6路通道');
    result = await hs.readMany([1, 2, 3, 4, 5, 6]);
    console.log(JSON.stringify(result, null, 2));

    console.log('\n4. 一键关闭所有通道');
    result = await hs.allOff();
    console.log(JSON.stringify(result, null, 2));
})();
```

---

## 常见场景

### 场景1: 顺序开启所有通道（每个间隔1秒）
```bash
for i in {1..12}; do
  echo "开启通道 $i..."
  curl -s -X POST http://localhost:8080/device/customMethod \
    -H 'Content-Type: application/json' \
    -d "{\"channel_id\":1,\"method\":\"channel_on\",\"args\":{\"channel\":$i}}"
  echo ""
  sleep 1
done
```

### 场景2: 反向关闭所有通道
```bash
for i in {12..1}; do
  echo "关闭通道 $i..."
  curl -s -X POST http://localhost:8080/device/customMethod \
    -H 'Content-Type: application/json' \
    -d "{\"channel_id\":1,\"method\":\"channel_off\",\"args\":{\"channel\":$i}}"
  echo ""
  sleep 1
done
```

### 场景3: 监控所有通道状态（每5秒刷新）
```bash
while true; do
  clear
  echo "=== HS-08R-16R 状态监控 ==="
  echo "时间: $(date '+%Y-%m-%d %H:%M:%S')"
  echo ""
  curl -s -X POST http://localhost:8080/device/customMethod \
    -H 'Content-Type: application/json' \
    -d '{"channel_id":1,"method":"read_status","args":{}}' | jq
  sleep 5
done
```

---

## 故障排查

### 连接失败
```bash
# 检查服务是否运行
ps aux | grep dm-rust

# 检查端口是否监听
netstat -tuln | grep 8080

# 测试连接
curl http://localhost:8080/health
```

### 超时问题
```bash
# 增加超时时间
curl --max-time 30 -X POST http://localhost:8080/device/customMethod ...
```

### 查看详细错误
```bash
# 使用 -v 参数查看详细信息
curl -v -X POST http://localhost:8080/device/customMethod \
  -H 'Content-Type: application/json' \
  -d '...'
```

---

## 参考文档

- **HS_POWER_SEQUENCER_GUIDE.md** - 完整协议文档
- **HS_SERIAL_CONFIG.md** - 串口配置指南
- **config.hs_power_sequencer.json** - 配置示例

---

**更新日期**: 2025-11-11
