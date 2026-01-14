# xFusion 服务器控制协议使用指南

本文档介绍如何使用 xFusion 协议通过 iBMC Redfish API 控制 xFusion 服务器的电源状态。

## 概述

xFusion 协议专为 xFusion（原华为）服务器设计，通过 iBMC（智能基板管理控制器）的 Redfish API 实现服务器电源控制。

### 主要特性

- **iBMC Redfish API 控制** - 通过 HTTPS 调用 iBMC 接口进行开关机
- **会话令牌机制** - 每次操作前自动获取新的会话令牌
- **多种电源操作** - 支持开机、正常关机、强制关机、重启等
- **状态监控** - 通过 UDP 心跳监控服务器在线状态
- **SSL 证书兼容** - 自动忽略自签名证书验证

---

## 配置说明

### 通道配置

```json
{
    "channel_id": 10,
    "enable": true,
    "statute": "xFusion",
    "arguments": {
        "nodes": [
            {
                "id": 101,
                "mac": "00:11:22:33:44:55",
                "ip": "192.168.11.101",
                "port": 8888,
                "ibmc_url": "https://10.103.69.212",
                "ibmc_username": "Administrator",
                "ibmc_password": "Admin@9000",
                "system_id": "1"
            }
        ],
        "broadcast_addr": "255.255.255.255",
        "wol_port": 9,
        "shutdown_port": 4001
    }
}
```

### 节点参数说明

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | number | ✓ | 节点唯一标识符 |
| `mac` | string | ✓ | 服务器 MAC 地址，格式如 `00:11:22:33:44:55` |
| `ip` | string | - | 服务器 IP 地址（用于 UDP 状态监控） |
| `port` | number | - | UDP 监控端口 |
| `ibmc_url` | string | ✓ | iBMC 管理地址，如 `https://10.103.69.212` |
| `ibmc_username` | string | ✓ | iBMC 登录用户名 |
| `ibmc_password` | string | ✓ | iBMC 登录密码 |
| `system_id` | string | - | 系统 ID，默认为 `"1"` |

### 通道级参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `broadcast_addr` | string | `255.255.255.255` | 广播地址（保留字段） |
| `wol_port` | number | `9` | WOL 端口（保留字段） |
| `shutdown_port` | number | 同 wol_port | 关机广播端口（保留字段） |

---

## 控制命令

### 1. 开机 (powerOn)

通过 iBMC Redfish API 发送 `On` 指令开机。

**HTTP 请求：**
```http
POST /api/channel/execute
Content-Type: application/json

{
    "channel_id": 10,
    "command": "powerOn",
    "params": {
        "id": 101
    }
}
```

**或使用 MAC 地址：**
```json
{
    "channel_id": 10,
    "command": "powerOn",
    "params": {
        "mac": "00:11:22:33:44:55"
    }
}
```

**响应：**
```json
{
    "status": "ok",
    "action": "powerOn"
}
```

### 2. 正常关机 (powerOff)

发送 `GracefulShutdown` 指令，让操作系统正常关机。

**HTTP 请求：**
```http
POST /api/channel/execute
Content-Type: application/json

{
    "channel_id": 10,
    "command": "powerOff",
    "params": {
        "id": 101
    }
}
```

### 3. 强制关机 (forceOff)

发送 `ForceOff` 指令，立即强制断电。

**HTTP 请求：**
```json
{
    "channel_id": 10,
    "command": "forceOff",
    "params": {
        "id": 101
    }
}
```

### 4. 强制重启 (forceRestart)

发送 `ForceRestart` 指令，强制重启服务器。

**HTTP 请求：**
```json
{
    "channel_id": 10,
    "command": "forceRestart",
    "params": {
        "id": 101
    }
}
```

### 5. 强制下电再上电 (forcePowerCycle)

发送 `ForcePowerCycle` 指令，先断电再上电。

**HTTP 请求：**
```json
{
    "channel_id": 10,
    "command": "forcePowerCycle",
    "params": {
        "id": 101
    }
}
```

### 6. 通用重置命令 (reset)

支持所有 Redfish ResetType 值。

**HTTP 请求：**
```json
{
    "channel_id": 10,
    "command": "reset",
    "params": {
        "id": 101,
        "resetType": "Nmi"
    }
}
```

**支持的 ResetType 值：**

| 值 | 说明 |
|----|------|
| `On` | 上电 |
| `ForceOff` | 强制下电 |
| `GracefulShutdown` | 正常下电 |
| `ForceRestart` | 强制重启 |
| `Nmi` | 触发不可屏蔽中断 |
| `ForcePowerCycle` | 强制下电再上电 |

### 7. 获取节点状态 (get)

获取指定节点的在线状态和音频信息（如果支持）。

**HTTP 请求：**
```json
{
    "channel_id": 10,
    "command": "get",
    "params": {
        "id": 101
    }
}
```

**响应：**
```json
{
    "id": 101,
    "online": true,
    "volume": 50,
    "mute": false
}
```

### 8. 获取所有节点状态 (getAllStatus)

获取通道下所有节点的状态。

**HTTP 请求：**
```json
{
    "channel_id": 10,
    "command": "getAllStatus",
    "params": {}
}
```

**响应：**
```json
{
    "channel_id": 10,
    "list": [
        {
            "id": 101,
            "mac": "00:11:22:33:44:55",
            "ip": "192.168.11.101",
            "port": 8888,
            "online": true,
            "ibmc_url": "https://10.103.69.212"
        }
    ]
}
```

### 9. 心跳更新 (heartbeat)

更新节点的心跳时间戳。

**HTTP 请求：**
```json
{
    "channel_id": 10,
    "command": "heartbeat",
    "params": {
        "mac": "00:11:22:33:44:55"
    }
}
```

---

## 简化写入接口

可以通过 `/api/device/write` 接口使用简化的开关机控制：

**开机：**
```http
POST /api/device/write
Content-Type: application/json

{
    "global_id": 1001,
    "value": 1
}
```

**关机：**
```json
{
    "global_id": 1001,
    "value": 0
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
            "statute": "xFusion",
            "arguments": {
                "nodes": [
                    {
                        "id": 101,
                        "mac": "00:11:22:33:44:55",
                        "ip": "192.168.11.101",
                        "port": 8888,
                        "ibmc_url": "https://10.103.69.212",
                        "ibmc_username": "Administrator",
                        "ibmc_password": "Admin@9000",
                        "system_id": "1"
                    },
                    {
                        "id": 102,
                        "mac": "00:11:22:33:44:56",
                        "ip": "192.168.11.102",
                        "port": 8888,
                        "ibmc_url": "https://10.103.69.213",
                        "ibmc_username": "Administrator",
                        "ibmc_password": "Admin@9000",
                        "system_id": "1"
                    }
                ]
            }
        }
    ],
    "nodes": [
        {
            "global_id": 1001,
            "channel_id": 10,
            "id": 101,
            "alias": "xFusion服务器1"
        },
        {
            "global_id": 1002,
            "channel_id": 10,
            "id": 102,
            "alias": "xFusion服务器2"
        }
    ],
    "scenes": [
        {
            "name": "开启所有服务器",
            "nodes": [
                { "id": 1001, "value": 1 },
                { "id": 1002, "value": 1, "delay": 5000 }
            ]
        },
        {
            "name": "关闭所有服务器",
            "nodes": [
                { "id": 1001, "value": 0 },
                { "id": 1002, "value": 0, "delay": 5000 }
            ]
        }
    ],
    "task_settings": {
        "timeout_ms": 30000,
        "check_interval_ms": 500,
        "max_retries": 3
    },
    "web_server": {
        "port": 8080
    }
}
```

---

## iBMC API 工作原理

### 1. 获取会话令牌

每次执行电源操作前，协议会自动向 iBMC 发送认证请求：

```bash
curl --location --request POST 'https://10.103.69.212/redfish/v1/SessionService/Sessions' \
--header 'Content-Type: application/json' \
--data-raw '{ 
    "UserName": "Administrator", 
    "Password": "Admin@9000" 
}'
```

从响应中提取 `Oem.xFusion.X-Auth-Token` 字段作为会话令牌。

### 2. 执行电源操作

使用获取的令牌调用电源控制 API：

```bash
curl --location --request POST 'https://10.103.69.212/redfish/v1/Systems/1/Actions/ComputerSystem.Reset' \
--header 'X-Auth-Token: <token>' \
--header 'Content-Type: application/json' \
--data-raw '{
  "ResetType": "On"
}'
```

---

## 注意事项

1. **HTTPS 证书** - iBMC 通常使用自签名证书，协议会自动忽略证书验证
2. **会话管理** - 每次操作前都会重新获取会话令牌，无需担心令牌过期
3. **超时设置** - HTTP 请求超时默认为 30 秒
4. **错误处理** - 如果 iBMC 返回错误状态码，将记录详细错误信息到日志

---

## 故障排查

### 常见错误

| 错误信息 | 可能原因 | 解决方案 |
|----------|----------|----------|
| `iBMC 会话请求失败` | 网络不通或 URL 错误 | 检查 `ibmc_url` 配置和网络连通性 |
| `会话响应中未找到 X-Auth-Token` | 用户名或密码错误 | 检查 `ibmc_username` 和 `ibmc_password` |
| `iBMC 电源操作返回错误` | 服务器状态不允许该操作 | 检查服务器当前状态 |

### 日志级别

设置日志级别为 `debug` 可以查看详细的请求和响应信息：

```json
{
    "log": {
        "level": "debug"
    }
}
```

---

## 会话持久化

xFusion 协议支持会话 Token 的持久化存储，即使服务重启也能恢复之前的 Token：

### 存储位置

Token 数据存储在：`data/protocol_storage/channel_{channel_id}.json`

### 工作机制

1. **首次请求** - 创建新 Token，同时保存到内存和持久化存储
2. **后续请求** - 优先从内存获取，内存没有则从持久化存储恢复
3. **Token 失效** - 当收到 401 或 NoValidSession 错误时，自动刷新 Token
4. **服务重启** - 从持久化存储恢复上次的 Token

### 手动清除缓存

如需手动清除缓存的 Token，可以删除对应的存储文件或调用 API 清除。

---

## 全局存储 API

所有协议都可以使用统一的存储接口保存数据：

```rust
use crate::protocols::storage::get_or_init_storage;

// 获取存储实例
let storage = get_or_init_storage().await;

// 设置值
storage.set(channel_id, "key", serde_json::json!("value")).await;

// 获取值
let value = storage.get_string(channel_id, "key").await;

// 删除值
storage.remove(channel_id, "key").await;

// 批量操作
storage.set_many(channel_id, values).await;
storage.get_all(channel_id).await;
```

