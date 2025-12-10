# 批量读取接口文档

## 概述

批量读取接口允许在单个 HTTP 请求中同时读取多个不同通道的数据点，支持跨协议、跨设备的数据采集，并统一返回结果。

## API 端点

```
POST http://localhost:8080/device/batchRead
Content-Type: application/json
```

## 请求格式

```json
{
  "items": [
    {
      "name": "数据点名称",
      "channel_id": 通道ID,
      // ... 其他参数（取决于通道协议）
    }
  ]
}
```

### 请求参数说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| items | Array | 是 | 批量读取项列表 |
| items[].name | String | 是 | 数据点名称（用于标识返回结果） |
| items[].channel_id | Number | 是 | 通道ID（对应配置文件中的 channel_id） |
| items[]... | Any | - | 其他参数根据通道协议而定 |

### 不同协议的参数格式

#### Modbus 协议
```json
{
  "name": "温度传感器",
  "channel_id": 3,
  "addr": 100,
  "type": "int16"
}
```

参数：
- `addr`: 寄存器地址
- `type`: 数据类型（uint16, int16, uint32, int32, float32, float64, bool 等）

#### PJLink 协议
```json
{
  "name": "投影仪状态",
  "channel_id": 1,
  "command": "get_power"
}
```

#### 自定义协议
```json
{
  "name": "设备A状态",
  "channel_id": 7,
  "device_id": 1,
  "register": 100
}
```

## 响应格式

### 成功响应

```json
{
  "state": 0,
  "message": "批量读取完成: 成功 3, 失败 1",
  "data": [
    {
      "name": "温度传感器",
      "success": true,
      "value": 256
    },
    {
      "name": "压力传感器",
      "success": true,
      "value": 101325.5
    },
    {
      "name": "流量计",
      "success": false,
      "error": "连接超时"
    }
  ]
}
```

### 响应字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| state | Number | 状态码（0=成功） |
| message | String | 汇总信息 |
| data | Array | 结果列表 |
| data[].name | String | 数据点名称（对应请求中的 name） |
| data[].success | Boolean | 是否读取成功 |
| data[].value | Any | 读取到的值（success=true 时存在） |
| data[].error | String | 错误信息（success=false 时存在） |

## 使用示例

### 示例 1: 读取单个 Modbus 设备的多个点位

**场景**: 从同一个 Modbus 设备读取温度、压力、流量三个数据

```bash
curl -X POST http://localhost:8080/device/batchRead \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {
        "name": "温度",
        "channel_id": 3,
        "addr": 100,
        "type": "int16"
      },
      {
        "name": "压力",
        "channel_id": 3,
        "addr": 200,
        "type": "float32"
      },
      {
        "name": "流量",
        "channel_id": 3,
        "addr": 300,
        "type": "uint32"
      }
    ]
  }'
```

**响应示例**:
```json
{
  "state": 0,
  "message": "批量读取完成: 成功 3, 失败 0",
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
    },
    {
      "name": "流量",
      "success": true,
      "value": 12345678
    }
  ]
}
```

**数据处理**:
```javascript
// 温度需要除以10
const temperature = response.data.find(d => d.name === "温度").value / 10; // 25.6°C

// 压力直接使用
const pressure = response.data.find(d => d.name === "压力").value; // 101325.5 Pa

// 流量直接使用
const flow = response.data.find(d => d.name === "流量").value; // 12345678
```

### 示例 2: 跨多个 Modbus 设备读取

**场景**: 从不同的 Modbus 通道读取数据

```json
{
  "items": [
    {
      "name": "1号机组温度",
      "channel_id": 3,
      "addr": 100,
      "type": "int16"
    },
    {
      "name": "2号机组温度",
      "channel_id": 4,
      "addr": 100,
      "type": "int16"
    },
    {
      "name": "3号机组温度",
      "channel_id": 5,
      "addr": 100,
      "type": "int16"
    }
  ]
}
```

### 示例 3: 跨协议混合读取

**场景**: 同时读取 Modbus 设备数据和投影仪状态

```json
{
  "items": [
    {
      "name": "环境温度",
      "channel_id": 3,
      "addr": 100,
      "type": "int16"
    },
    {
      "name": "投影仪电源状态",
      "channel_id": 1,
      "command": "get_power"
    },
    {
      "name": "空调控制状态",
      "channel_id": 6,
      "addr": 0,
      "type": "bool"
    }
  ]
}
```

### 示例 4: 完整的监控数据采集

**场景**: 工业现场实时监控面板，一次性读取所有关键数据

```bash
curl -X POST http://localhost:8080/device/batchRead \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"name": "主机温度", "channel_id": 3, "addr": 100, "type": "int16"},
      {"name": "副机温度", "channel_id": 3, "addr": 101, "type": "int16"},
      {"name": "压力1", "channel_id": 3, "addr": 200, "type": "float32"},
      {"name": "压力2", "channel_id": 3, "addr": 202, "type": "float32"},
      {"name": "总流量", "channel_id": 3, "addr": 300, "type": "uint32"},
      {"name": "瞬时流量", "channel_id": 3, "addr": 302, "type": "float32"},
      {"name": "泵1状态", "channel_id": 3, "addr": 0, "type": "bool"},
      {"name": "泵2状态", "channel_id": 3, "addr": 1, "type": "bool"},
      {"name": "阀门位置", "channel_id": 3, "addr": 400, "type": "uint16"},
      {"name": "报警状态", "channel_id": 3, "addr": 2, "type": "bool"}
    ]
  }'
```

## Python 客户端示例

### 基础封装

```python
import requests
from typing import List, Dict, Any

class DeviceClient:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
    
    def batch_read(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """批量读取数据"""
        url = f"{self.base_url}/device/batchRead"
        response = requests.post(url, json={"items": items})
        return response.json()
    
    def batch_read_dict(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """批量读取并返回字典格式 {name: value}"""
        result = self.batch_read(items)
        return {
            item["name"]: item.get("value") 
            for item in result["data"] 
            if item["success"]
        }

# 使用示例
client = DeviceClient()

# 定义读取列表
read_items = [
    {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
    {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
    {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"},
]

# 批量读取
result = client.batch_read_dict(read_items)

# 使用数据
print(f"温度: {result['温度'] / 10}°C")
print(f"压力: {result['压力']} Pa")
print(f"流量: {result['流量']} m³/h")
```

### 高级应用：实时监控

```python
import time
from datetime import datetime

class RealtimeMonitor:
    def __init__(self, client: DeviceClient, items: List[Dict], interval=1.0):
        self.client = client
        self.items = items
        self.interval = interval
    
    def start(self):
        """开始实时监控"""
        print("实时监控开始...")
        try:
            while True:
                timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                result = self.client.batch_read(self.items)
                
                print(f"\n[{timestamp}] 数据采集:")
                for item in result["data"]:
                    if item["success"]:
                        print(f"  {item['name']}: {item['value']}")
                    else:
                        print(f"  {item['name']}: 错误 - {item['error']}")
                
                time.sleep(self.interval)
        except KeyboardInterrupt:
            print("\n监控停止")

# 使用
client = DeviceClient()
items = [
    {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
    {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
    {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"},
]

monitor = RealtimeMonitor(client, items, interval=2.0)
monitor.start()
```

## JavaScript/Node.js 示例

```javascript
const axios = require('axios');

class DeviceClient {
  constructor(baseUrl = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }

  async batchRead(items) {
    const response = await axios.post(`${this.baseUrl}/device/batchRead`, {
      items: items
    });
    return response.data;
  }

  async batchReadDict(items) {
    const result = await this.batchRead(items);
    const dict = {};
    result.data.forEach(item => {
      if (item.success) {
        dict[item.name] = item.value;
      }
    });
    return dict;
  }
}

// 使用示例
async function main() {
  const client = new DeviceClient();
  
  const items = [
    { name: '温度', channel_id: 3, addr: 100, type: 'int16' },
    { name: '压力', channel_id: 3, addr: 200, type: 'float32' },
    { name: '流量', channel_id: 3, addr: 300, type: 'uint32' },
  ];
  
  const data = await client.batchReadDict(items);
  
  console.log(`温度: ${data['温度'] / 10}°C`);
  console.log(`压力: ${data['压力']} Pa`);
  console.log(`流量: ${data['流量']} m³/h`);
}

main().catch(console.error);
```

## 前端应用示例 (Vue.js)

```vue
<template>
  <div class="monitor-panel">
    <h2>设备监控面板</h2>
    <div class="data-grid">
      <div v-for="item in displayData" :key="item.name" class="data-card">
        <div class="label">{{ item.name }}</div>
        <div class="value" :class="{ error: !item.success }">
          {{ item.success ? formatValue(item) : '错误' }}
        </div>
        <div class="unit">{{ item.unit }}</div>
      </div>
    </div>
    <div class="status">
      最后更新: {{ lastUpdate }}
    </div>
  </div>
</template>

<script>
export default {
  data() {
    return {
      displayData: [],
      lastUpdate: null,
      timer: null,
      readItems: [
        { name: '温度', channel_id: 3, addr: 100, type: 'int16', unit: '°C', scale: 0.1 },
        { name: '压力', channel_id: 3, addr: 200, type: 'float32', unit: 'Pa', scale: 1 },
        { name: '流量', channel_id: 3, addr: 300, type: 'uint32', unit: 'm³/h', scale: 1 },
        { name: '泵状态', channel_id: 3, addr: 0, type: 'bool', unit: '', scale: 1 },
      ]
    }
  },
  mounted() {
    this.startMonitoring();
  },
  beforeUnmount() {
    if (this.timer) clearInterval(this.timer);
  },
  methods: {
    async fetchData() {
      try {
        const response = await fetch('http://localhost:8080/device/batchRead', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ items: this.readItems })
        });
        const result = await response.json();
        
        this.displayData = result.data.map((item, index) => ({
          ...item,
          ...this.readItems[index]
        }));
        
        this.lastUpdate = new Date().toLocaleTimeString();
      } catch (error) {
        console.error('读取失败:', error);
      }
    },
    formatValue(item) {
      if (!item.success) return '-';
      if (item.type === 'bool') return item.value ? '开' : '关';
      return (item.value * item.scale).toFixed(2);
    },
    startMonitoring() {
      this.fetchData();
      this.timer = setInterval(() => {
        this.fetchData();
      }, 2000);
    }
  }
}
</script>

<style scoped>
.monitor-panel {
  padding: 20px;
}
.data-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 15px;
  margin: 20px 0;
}
.data-card {
  border: 1px solid #ddd;
  padding: 15px;
  border-radius: 8px;
  text-align: center;
}
.label {
  font-size: 14px;
  color: #666;
  margin-bottom: 10px;
}
.value {
  font-size: 32px;
  font-weight: bold;
  color: #333;
}
.value.error {
  color: #f56c6c;
}
.unit {
  font-size: 14px;
  color: #999;
  margin-top: 5px;
}
.status {
  text-align: center;
  color: #999;
  margin-top: 20px;
}
</style>
```

## 性能优化建议

### 1. 合理分组
将读取频率相同的数据点分组，避免频繁读取不常变化的数据：

```python
# 快速变化数据（每1秒读取）
fast_items = [
    {"name": "瞬时流量", "channel_id": 3, "addr": 302, "type": "float32"},
    {"name": "瞬时压力", "channel_id": 3, "addr": 200, "type": "float32"},
]

# 慢速变化数据（每10秒读取）
slow_items = [
    {"name": "总流量", "channel_id": 3, "addr": 300, "type": "uint32"},
    {"name": "运行时间", "channel_id": 3, "addr": 500, "type": "uint32"},
]
```

### 2. 错误处理
对失败的读取进行重试或告警：

```python
def batch_read_with_retry(client, items, max_retries=3):
    for attempt in range(max_retries):
        result = client.batch_read(items)
        
        # 检查是否有失败的项
        failed = [item for item in result["data"] if not item["success"]]
        
        if not failed:
            return result
        
        if attempt < max_retries - 1:
            print(f"重试 {len(failed)} 个失败项...")
            time.sleep(1)
    
    return result
```

### 3. 数据缓存
对于高频访问的数据，使用本地缓存：

```python
from functools import lru_cache
import time

class CachedDeviceClient:
    def __init__(self, client, cache_time=1.0):
        self.client = client
        self.cache_time = cache_time
        self.cache = {}
    
    def batch_read(self, items):
        # 生成缓存键
        cache_key = str(items)
        now = time.time()
        
        if cache_key in self.cache:
            cached_data, cached_time = self.cache[cache_key]
            if now - cached_time < self.cache_time:
                return cached_data
        
        # 读取新数据
        result = self.client.batch_read(items)
        self.cache[cache_key] = (result, now)
        return result
```

## 故障排查

### 问题 1: 部分数据读取失败

**现象**: 返回结果中部分项的 `success` 为 `false`

**可能原因**:
1. 设备离线或网络不通
2. 寄存器地址不存在
3. 数据类型不匹配
4. 通道 ID 错误

**解决方法**:
- 检查 `error` 字段的详细错误信息
- 验证设备连接状态
- 确认寄存器地址和数据类型
- 检查配置文件中的 channel_id

### 问题 2: 响应时间过长

**现象**: 批量读取耗时过长

**可能原因**:
1. 读取项过多
2. 设备响应慢
3. 网络延迟高

**解决方法**:
- 减少单次读取的项数
- 将读取项分批处理
- 优化网络配置
- 考虑使用异步并发读取

### 问题 3: 数据值异常

**现象**: 读取到的值明显不合理

**可能原因**:
1. 字节序不正确（Big Endian vs Little Endian）
2. 数据类型选择错误
3. 需要额外的缩放计算

**解决方法**:
- 尝试 Little Endian 类型（如 `float32le`）
- 检查设备文档确认数据类型
- 在应用层添加缩放处理

## 最佳实践

1. **命名规范**: 使用清晰的名称标识数据点，便于后续处理
2. **分批读取**: 单次不要读取过多数据点（建议 < 20 个）
3. **错误处理**: 始终检查 `success` 字段，对失败项进行处理
4. **数据验证**: 对读取到的值进行合理性验证
5. **性能监控**: 记录读取耗时，及时发现性能问题
6. **容错设计**: 对单个点位的读取失败不应影响整体业务

## 与其他接口对比

| 接口 | 用途 | 优势 | 适用场景 |
|------|------|------|----------|
| `/device/read` | 单点读取 | 简单直接 | 读取单个节点 |
| `/device/getAllNodeStates` | 获取所有节点状态 | 包含节点元信息 | 查看系统整体状态 |
| `/device/batchRead` | 批量读取 | 跨通道、可自定义、高效 | 数据采集、监控面板 |
| `/device/executeCommand` | 执行通道命令 | 灵活通用 | 单个通道的复杂操作 |

## 总结

批量读取接口是一个高效的数据采集工具，特别适合：
- 实时监控面板
- 数据记录与分析
- 跨设备数据汇总
- 高频数据采集

通过合理设计读取列表和错误处理，可以构建稳定高效的工业数据采集系统。
