# 通道配置与自定义方法指南

## 概述

系统支持两种配置方式来定义通道参数，并支持为每个通道定义自定义方法。

## 通道参数配置

### 方式1: 使用 `arguments` 字段（推荐）

这是新的推荐配置方式，参数更加清晰和结构化。

```json
{
  "channel_id": 1,
  "enable": true,
  "statute": "pjlink",
  "arguments": {
    "addr": "192.168.20.59",
    "port": 4352,
    "password": null,
    "timeout": 5000
  }
}
```

**优点**:
- 参数集中在 `arguments` 字段中，清晰明了
- 与自定义方法的 `arguments` 概念一致
- 便于参数验证和文档生成

### 方式2: 扁平化参数（兼容旧配置）

为了兼容旧配置，系统仍然支持直接将参数写在通道配置的顶层。

```json
{
  "channel_id": 1,
  "enable": true,
  "statute": "pjlink",
  "addr": "192.168.20.59",
  "port": 4352,
  "password": null
}
```

**说明**:
- 系统优先读取 `arguments` 字段
- 如果没有 `arguments`，则读取扁平化的参数
- 两种方式可以共存，但推荐只使用一种

## 自定义方法配置

### 基本结构

在通道配置中添加 `methods` 数组，定义该通道支持的自定义方法：

```json
{
  "channel_id": 1,
  "enable": true,
  "statute": "pjlink",
  "arguments": {
    "addr": "192.168.20.59",
    "port": 4352
  },
  "methods": [
    {
      "name": "set_input",
      "description": "切换输入源",
      "arguments": [
        {
          "name": "source",
          "type": "string",
          "required": true,
          "description": "输入源名称：hdmi1, hdmi2, vga 等"
        }
      ]
    }
  ]
}
```

### 方法定义字段

#### `name` (必需)
- 类型: `string`
- 说明: 方法的唯一标识名称
- 示例: `"set_input"`, `"get_lamp_hours"`, `"adjust_brightness"`

#### `description` (可选)
- 类型: `string`
- 说明: 方法的功能描述
- 示例: `"切换投影仪输入源"`

#### `arguments` (可选)
- 类型: `array`
- 说明: 方法参数定义列表
- 默认: `[]`（无参数）

### 参数定义字段

每个方法参数可以包含以下字段：

#### `name` (必需)
- 类型: `string`
- 说明: 参数名称
- 示例: `"source"`, `"brightness"`, `"device_id"`

#### `type` (必需)
- 类型: `string`
- 可选值: `"string"`, `"number"`, `"boolean"`, `"object"`, `"array"`
- 说明: 参数的数据类型

#### `required` (可选)
- 类型: `boolean`
- 默认: `false`
- 说明: 参数是否必需

#### `default` (可选)
- 类型: `any`
- 说明: 参数的默认值（当参数不是必需且未提供时使用）
- 示例: `0`, `"auto"`, `true`

#### `description` (可选)
- 类型: `string`
- 说明: 参数的详细描述

## 完整示例

### 示例1: PJLink 投影仪控制

```json
{
  "channel_id": 1,
  "enable": true,
  "statute": "pjlink",
  "arguments": {
    "addr": "192.168.20.59",
    "port": 4352,
    "password": "secret"
  },
  "methods": [
    {
      "name": "set_input",
      "description": "切换输入源",
      "arguments": [
        {
          "name": "source",
          "type": "string",
          "required": true,
          "description": "输入源：hdmi1, hdmi2, vga1, vga2"
        }
      ]
    },
    {
      "name": "get_lamp_hours",
      "description": "获取灯泡使用时长",
      "arguments": []
    },
    {
      "name": "set_brightness",
      "description": "设置亮度",
      "arguments": [
        {
          "name": "level",
          "type": "number",
          "required": true,
          "description": "亮度等级 (0-100)"
        },
        {
          "name": "transition",
          "type": "boolean",
          "required": false,
          "default": true,
          "description": "是否渐变过渡"
        }
      ]
    }
  ]
}
```

### 示例2: 自定义协议

```json
{
  "channel_id": 7,
  "enable": true,
  "statute": "custom",
  "arguments": {
    "addr": "192.168.20.200",
    "port": 8888,
    "protocol": "tcp",
    "timeout": 3000
  },
  "methods": [
    {
      "name": "query_status",
      "description": "查询设备状态",
      "arguments": [
        {
          "name": "device_id",
          "type": "number",
          "required": true,
          "description": "设备ID"
        },
        {
          "name": "detail",
          "type": "boolean",
          "required": false,
          "default": false,
          "description": "是否返回详细信息"
        }
      ]
    },
    {
      "name": "batch_control",
      "description": "批量控制多个设备",
      "arguments": [
        {
          "name": "devices",
          "type": "array",
          "required": true,
          "description": "设备ID数组"
        },
        {
          "name": "action",
          "type": "string",
          "required": true,
          "description": "执行的动作"
        },
        {
          "name": "params",
          "type": "object",
          "required": false,
          "default": {},
          "description": "动作参数"
        }
      ]
    }
  ]
}
```

## HTTP API 调用

### 获取通道支持的方法

```bash
curl -X POST http://localhost:8080/device/getMethods \
  -H 'Content-Type: application/json' \
  -d '{"channel_id": 1}'
```

**响应**:
```json
{
  "state": 0,
  "message": "获取方法列表成功",
  "data": [
    "set_input",
    "get_lamp_hours",
    "set_brightness"
  ]
}
```

### 调用自定义方法

#### 单参数方法

```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "set_input",
    "arguments": {
      "source": "hdmi1"
    }
  }'
```

#### 多参数方法

```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "set_brightness",
    "arguments": {
      "level": 80,
      "transition": true
    }
  }'
```

#### 无参数方法

```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 1,
    "method_name": "get_lamp_hours",
    "arguments": {}
  }'
```

#### 复杂参数方法

```bash
curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{
    "channel_id": 7,
    "method_name": "batch_control",
    "arguments": {
      "devices": [1, 2, 3, 4],
      "action": "power_on",
      "params": {
        "delay": 1000,
        "retry": 3
      }
    }
  }'
```

## 协议实现指南

### 实现自定义方法支持

协议需要实现 `Protocol` trait 中的 `call_method` 方法：

```rust
use async_trait::async_trait;
use serde_json::Value;
use crate::protocols::Protocol;
use crate::utils::Result;

pub struct MyProtocol {
    // 协议字段
}

#[async_trait]
impl Protocol for MyProtocol {
    // ... 其他必需方法 ...
    
    async fn call_method(&mut self, method_name: &str, args: Value) -> Result<Value> {
        match method_name {
            "set_input" => {
                let source = args["source"].as_str()
                    .ok_or_else(|| DeviceError::Other("缺少 source 参数".into()))?;
                
                // 执行实际操作
                self.set_input_source(source).await?;
                
                Ok(serde_json::json!({
                    "result": "ok",
                    "current_input": source
                }))
            }
            
            "get_lamp_hours" => {
                let hours = self.query_lamp_hours().await?;
                
                Ok(serde_json::json!({
                    "lamp_hours": hours
                }))
            }
            
            _ => {
                Err(DeviceError::Other(
                    format!("不支持的方法: {}", method_name)
                ))
            }
        }
    }
    
    fn get_methods(&self) -> Vec<String> {
        vec![
            "set_input".to_string(),
            "get_lamp_hours".to_string(),
        ]
    }
}
```

### 参数验证示例

```rust
async fn call_method(&mut self, method_name: &str, args: Value) -> Result<Value> {
    match method_name {
        "set_brightness" => {
            // 获取必需参数
            let level = args["level"].as_i64()
                .ok_or_else(|| DeviceError::Other("缺少 level 参数".into()))?;
            
            // 验证参数范围
            if level < 0 || level > 100 {
                return Err(DeviceError::Other(
                    "level 必须在 0-100 之间".into()
                ));
            }
            
            // 获取可选参数（使用默认值）
            let transition = args["transition"].as_bool().unwrap_or(true);
            
            // 执行操作
            self.set_brightness(level as u8, transition).await?;
            
            Ok(serde_json::json!({
                "result": "ok",
                "brightness": level
            }))
        }
        _ => Err(DeviceError::Other(format!("不支持的方法: {}", method_name)))
    }
}
```

## 最佳实践

1. **参数配置**
   - 优先使用 `arguments` 字段
   - 保持参数命名一致性（如都用 `addr` 或都用 `address`）
   - 为复杂参数提供默认值

2. **方法定义**
   - 方法名使用下划线命名法（如 `set_input`）
   - 提供清晰的 `description`
   - 完整定义所有参数的类型和描述

3. **参数设计**
   - 尽量使用简单类型（string, number, boolean）
   - 复杂对象参数提供示例
   - 必需参数尽量少

4. **错误处理**
   - 参数验证失败返回清晰的错误信息
   - 包含参数名称和期望值范围
   - 使用标准错误码

5. **文档**
   - 在配置文件中使用 `_comment` 字段说明
   - 维护完整的方法列表文档
   - 提供调用示例

## 注意事项

1. **方法名唯一性**: 同一通道内的方法名必须唯一
2. **参数类型**: 调用时的参数类型必须与定义匹配
3. **必需参数**: 调用时必须提供所有标记为 `required: true` 的参数
4. **兼容性**: 旧配置（扁平化参数）仍然支持，但建议迁移到新格式
5. **性能**: 自定义方法调用与标准命令执行具有相同的性能特征

## 故障排查

### 方法未找到

```json
{
  "state": 30006,
  "message": "方法调用失败: 协议 Pjlink 不支持自定义方法: unknown_method",
  "data": null
}
```

**解决方案**:
1. 使用 `/device/getMethods` 获取支持的方法列表
2. 检查方法名拼写是否正确
3. 确认该协议实现了 `call_method` 方法

### 参数错误

```json
{
  "state": 30006,
  "message": "方法调用失败: 缺少 source 参数",
  "data": null
}
```

**解决方案**:
1. 检查配置文件中的参数定义
2. 确认调用时提供了所有必需参数
3. 验证参数类型是否正确

### 通道未找到

```json
{
  "state": 30002,
  "message": "通道未找到: 99",
  "data": null
}
```

**解决方案**:
1. 检查 `channel_id` 是否正确
2. 确认通道在配置文件中已定义且 `enable: true`
3. 查看启动日志确认通道初始化成功

## 更多示例

完整的配置示例请参考：
- [config.example.json](config.example.json) - 包含各种协议的配置示例
- [HTTP_API.md](HTTP_API.md) - HTTP API 详细文档
- [CONFIGURATION.md](CONFIGURATION.md) - 配置文件完整说明
