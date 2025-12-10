# 批量读取接口快速参考

## 一句话说明
在单个 HTTP 请求中同时读取多个不同通道、不同点位的数据，并统一返回结果。

## API 端点
```
POST http://localhost:8080/device/batchRead
```

## 最小示例

### cURL
```bash
curl -X POST http://localhost:8080/device/batchRead \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
      {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"}
    ]
  }'
```

### Python
```python
import requests

response = requests.post("http://localhost:8080/device/batchRead", json={
    "items": [
        {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
        {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"}
    ]
})

data = {item["name"]: item["value"] 
        for item in response.json()["data"] 
        if item["success"]}

print(f"温度: {data['温度']}")
print(f"压力: {data['压力']}")
```

### JavaScript
```javascript
const response = await fetch('http://localhost:8080/device/batchRead', {
  method: 'POST',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({
    items: [
      {name: '温度', channel_id: 3, addr: 100, type: 'int16'},
      {name: '压力', channel_id: 3, addr: 200, type: 'float32'}
    ]
  })
});

const result = await response.json();
result.data.forEach(item => {
  if (item.success) {
    console.log(`${item.name}: ${item.value}`);
  }
});
```

## 请求格式

```json
{
  "items": [
    {
      "name": "数据点名称",
      "channel_id": 通道ID,
      // ... 其他参数（根据协议不同）
    }
  ]
}
```

### Modbus 参数
```json
{
  "name": "温度",
  "channel_id": 3,
  "addr": 100,
  "type": "int16"
}
```

## 响应格式

```json
{
  "state": 0,
  "message": "批量读取完成: 成功 2, 失败 0",
  "data": [
    {
      "name": "温度",
      "success": true,
      "value": 256
    },
    {
      "name": "压力",
      "success": true,
      "value": 101325.5
    }
  ]
}
```

## 使用场景

| 场景 | 说明 | 示例 |
|------|------|------|
| **单设备多点** | 同一设备读取多个数据点 | 温度、压力、流量 |
| **多设备监控** | 跨多个设备读取同类数据 | 1-10号机温度 |
| **混合协议** | 不同协议设备数据汇总 | Modbus + PJLink |
| **实时监控** | 定时循环读取显示 | 监控面板 |
| **数据记录** | 批量采集存储 | 数据库记录 |

## 常见读取模式

### 1. 单设备完整监控
```json
{
  "items": [
    {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
    {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
    {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"},
    {"name": "液位", "channel_id": 3, "addr": 400, "type": "uint16"},
    {"name": "泵状态", "channel_id": 3, "addr": 0, "type": "bool"}
  ]
}
```

### 2. 多设备同类数据
```json
{
  "items": [
    {"name": "1号机温度", "channel_id": 3, "addr": 100, "type": "int16"},
    {"name": "2号机温度", "channel_id": 4, "addr": 100, "type": "int16"},
    {"name": "3号机温度", "channel_id": 5, "addr": 100, "type": "int16"}
  ]
}
```

### 3. 连续地址批量读取
```json
{
  "items": [
    {"name": "点位1", "channel_id": 3, "addr": 100, "type": "int16"},
    {"name": "点位2", "channel_id": 3, "addr": 101, "type": "int16"},
    {"name": "点位3", "channel_id": 3, "addr": 102, "type": "int16"},
    {"name": "点位4", "channel_id": 3, "addr": 103, "type": "int16"}
  ]
}
```

## Python 快速工具类

```python
import requests
from typing import List, Dict, Any

class BatchReader:
    def __init__(self, base_url="http://localhost:8080"):
        self.url = f"{base_url}/device/batchRead"
    
    def read(self, items: List[Dict]) -> Dict[str, Any]:
        """返回 {name: value} 字典"""
        response = requests.post(self.url, json={"items": items})
        result = response.json()
        return {
            item["name"]: item["value"] 
            for item in result["data"] 
            if item["success"]
        }

# 使用
reader = BatchReader()
data = reader.read([
    {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
    {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"}
])

print(f"温度: {data['温度'] / 10}°C")
print(f"压力: {data['压力']} Pa")
```

## JavaScript 快速工具类

```javascript
class BatchReader {
  constructor(baseUrl = 'http://localhost:8080') {
    this.url = `${baseUrl}/device/batchRead`;
  }

  async read(items) {
    const response = await fetch(this.url, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({items})
    });
    const result = await response.json();
    
    const dict = {};
    result.data.forEach(item => {
      if (item.success) dict[item.name] = item.value;
    });
    return dict;
  }
}

// 使用
const reader = new BatchReader();
const data = await reader.read([
  {name: '温度', channel_id: 3, addr: 100, type: 'int16'},
  {name: '压力', channel_id: 3, addr: 200, type: 'float32'}
]);

console.log(`温度: ${data['温度'] / 10}°C`);
console.log(`压力: ${data['压力']} Pa`);
```

## 性能建议

| 项目 | 建议 | 原因 |
|------|------|------|
| 单次读取数量 | < 20 个点位 | 避免请求超时 |
| 更新频率 | 快变量 1-2秒，慢变量 5-10秒 | 减少网络负载 |
| 错误处理 | 必须检查 success 字段 | 部分失败不影响其他 |
| 网络优化 | 使用持久连接 | 提高效率 |

## 测试脚本

### Bash 测试
```bash
./test_batch_read.sh
```

### Python 演示
```bash
python3 examples/batch_read_demo.py
```

### JavaScript 演示
```bash
node examples/batch_read_example.js
```

## 错误处理模式

```python
result = requests.post(url, json={"items": items}).json()

# 分离成功和失败
success_data = {item["name"]: item["value"] 
                for item in result["data"] if item["success"]}
failed_items = [(item["name"], item["error"]) 
                for item in result["data"] if not item["success"]]

# 处理成功数据
for name, value in success_data.items():
    print(f"{name}: {value}")

# 报告失败项
if failed_items:
    print(f"\n警告: {len(failed_items)} 项读取失败")
    for name, error in failed_items:
        print(f"  - {name}: {error}")
```

## 实时监控模板

```python
import time
from datetime import datetime

def monitor_loop(items, interval=2.0):
    while True:
        timestamp = datetime.now().strftime("%H:%M:%S")
        response = requests.post(
            "http://localhost:8080/device/batchRead",
            json={"items": items}
        )
        result = response.json()
        
        print(f"\n[{timestamp}] {result['message']}")
        for item in result["data"]:
            if item["success"]:
                print(f"  ✓ {item['name']}: {item['value']}")
            else:
                print(f"  ✗ {item['name']}: {item['error']}")
        
        time.sleep(interval)

# 使用
items = [
    {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
    {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"}
]
monitor_loop(items, interval=2.0)
```

## 数据类型参考

| 类型 | 寄存器数 | 范围 | 应用 |
|------|---------|------|------|
| int16 | 1 | -32768 ~ 32767 | 温度（×10） |
| uint16 | 1 | 0 ~ 65535 | 转速、计数 |
| int32 | 2 | -2.1亿 ~ 2.1亿 | 位置、坐标 |
| uint32 | 2 | 0 ~ 42亿 | 流量累计 |
| float32 | 2 | ±3.4×10³⁸ | 压力、温度 |
| float64 | 4 | ±1.7×10³⁰⁸ | 高精度 |
| bool | - | true/false | 开关状态 |

对于 Little Endian 设备，使用 `int32le`, `uint32le`, `float32le`

## 故障排查

| 问题 | 可能原因 | 解决方法 |
|------|---------|---------|
| 部分失败 | 设备离线、地址错误 | 检查 error 字段 |
| 全部失败 | 通道不存在、配置错误 | 检查 channel_id |
| 值异常 | 字节序错误、类型错误 | 尝试 LE 类型 |
| 响应慢 | 读取项过多、设备慢 | 减少点位数量 |

## 文档链接

- 完整文档: `dm-rust/BATCH_READ_API.md`
- Modbus 数据类型: `dm-rust/MODBUS_DATA_TYPES.md`
- 配置示例: `dm-rust/config.example.json`

## 优势总结

✅ **一次请求多点数据** - 减少网络往返  
✅ **跨通道读取** - 不限单一设备  
✅ **混合协议** - Modbus、PJLink 等统一接口  
✅ **部分失败容错** - 单点失败不影响其他  
✅ **命名标识** - 通过 name 字段清晰标识  
✅ **灵活参数** - 支持各协议特定参数
