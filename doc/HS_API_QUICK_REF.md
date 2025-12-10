# HS-08R-16R API 快速参考

## ⚠️ 重要提示

**正确的 API 端点**: `/device/callMethod`  
**正确的字段名**: `method_name` 和 `arguments`

## 🔑 API 格式

```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "方法名",
    "arguments": {参数对象}
  }'
```

## 📋 所有方法列表

### 1. channel_on - 开启通道
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"channel_on","arguments":{"channel":1}}'
```

### 2. channel_off - 关闭通道
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"channel_off","arguments":{"channel":1}}'
```

### 3. all_on - 一键开启
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"all_on","arguments":{}}'
```

### 4. all_off - 一键关闭
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"all_off","arguments":{}}'
```

### 5. delayed_on - 延时开启
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"delayed_on","arguments":{"channel":1}}'
```

### 6. delayed_off - 延时关闭
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"delayed_off","arguments":{"channel":1}}'
```

### 7. set_delay - 设置延时参数
```bash
# 设置通道1: 开延时2000ms, 关延时1000ms
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id":1,
    "method_name":"set_delay",
    "arguments":{
      "channel":1,
      "delay_ms":2000,
      "is_on":true
    }
  }'
```

**参数说明**:
- `channel`: 通道号 (1-12)
- `delay_ms`: 延时毫秒数
- `is_on`: `true`=开延时, `false`=关延时

### 8. read_status - 读取设备状态
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"read_status","arguments":{}}'
```

**响应示例**:
```json
{
  "state": 10000,
  "message": "方法调用成功",
  "data": {
    "success": true,
    "channels": [true, false, false, false, false, false, false, false, false, false, false, false]
  }
}
```

### 9. set_time - 设置设备时间
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id":1,
    "method_name":"set_time",
    "arguments":{
      "year":25,
      "month":11,
      "day":11,
      "hour":14,
      "minute":30,
      "second":0
    }
  }'
```

**参数说明**: year 是两位数 (2025年 = 25)

### 10. read_address - 读取设备地址
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"read_address","arguments":{}}'
```

### 11. write_address - 修改设备地址
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"write_address","arguments":{"address":5}}'
```

### 12. factory_reset - 恢复出厂设置
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"factory_reset","arguments":{}}'
```

### 13. set_voltage_protection - 设置电压保护
```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id":1,
    "method_name":"set_voltage_protection",
    "arguments":{
      "over_voltage":250,
      "under_voltage":180,
      "hysteresis":5,
      "over_enable":true,
      "under_enable":true
    }
  }'
```

## 📖 标准节点接口

### 读取节点状态
```bash
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{"global_id":1}'
```

### 写入节点状态
```bash
# 开启
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"global_id":1,"value":1}'

# 关闭
curl -X POST http://localhost:8080/device/write \
  -H 'Content-Type: application/json' \
  -d '{"global_id":1,"value":0}'
```

### 批量读取
```bash
curl -X POST http://localhost:8080/device/readMany \
  -H 'Content-Type: application/json' \
  -d '{"global_ids":[1,2,3,4,5,6]}'
```

### 批量写入
```bash
curl -X POST http://localhost:8080/device/writeMany \
  -H 'Content-Type: application/json' \
  -d '{
    "writes":[
      {"global_id":1,"value":1},
      {"global_id":2,"value":1},
      {"global_id":3,"value":0}
    ]
  }'
```

## 🔍 获取方法列表
```bash
curl -X POST http://localhost:8080/device/getMethods \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1}'
```

## 📦 响应格式

所有API返回统一格式:
```json
{
  "state": 10000,          // 10000=成功, 其他=错误码
  "message": "描述信息",
  "data": {}               // 返回数据
}
```

## 🚀 快速测试

```bash
# 1. 启动服务
./target/release/dm-rust -c config.hs_power_sequencer.json -l info

# 2. 运行测试脚本
chmod +x test_hs_http.sh
./test_hs_http.sh

# 3. 单个命令测试
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"read_status","arguments":{}}'
```

## 📝 常见错误

### 错误: 没有响应
**原因**: 使用了错误的端点 `/device/customMethod`  
**解决**: 使用正确端点 `/device/callMethod`

### 错误: 参数解析失败
**原因**: 字段名错误 (`method`/`args`)  
**解决**: 使用正确字段名 (`method_name`/`arguments`)

### 错误: 串口打开失败
**原因**: 串口设备不存在或权限不足  
**解决**: 检查 `/dev/ttyUSB0` 是否存在，添加用户到 `dialout` 组

## 📚 完整文档

- **HS_POWER_SEQUENCER_GUIDE.md** - 协议完整说明
- **HS_SERIAL_CONFIG.md** - 串口配置指南
- **HS_HTTP_EXAMPLES.md** - 详细示例 (需要更新)
- **test_hs_http.sh** - 自动化测试脚本

---

**提示**: 所有示例已使用正确的 API 格式更新！
