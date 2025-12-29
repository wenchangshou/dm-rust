# Swagger / OpenAPI 使用指南

## 概述

本项目使用 `utoipa` 为 REST API 生成 Swagger/OpenAPI 文档，提供交互式 API 测试界面。

**重要**: Swagger 功能**仅用于开发环境**，生产环境不应启用以避免暴露 API 细节。

## 启用 Swagger

### 开发环境

在开发时使用 `--features swagger` 启用 Swagger:

```bash
# 运行开发服务器（启用 Swagger）
cargo run --features swagger

# 编译带 Swagger 的版本
cargo build --features swagger
```

### 生产环境

生产环境编译**不要**添加 `--features swagger`:

```bash
# 生产环境编译（不包含 Swagger）
cargo build --release

# 不会包含 Swagger UI 和文档
```

## 访问 Swagger UI

启动服务后，访问：

```
http://localhost:8080/swagger-ui/
```

## 可用 API 分组

### 1. Device（设备控制 API）

- `/lspcapi/device/write` - 写入节点
- `/lspcapi/device/read` - 读取节点
- `/lspcapi/device/getAllStatus` - 获取所有状态
- `/lspcapi/device/scene` - 执行场景
- 等等...

### 2. Screen（屏幕管理 API）

需要启用数据库功能：

- `/lspcapi/screens/` - 屏幕 CRUD 操作
- `/lspcapi/screens/:id/materials` - 获取屏幕素材

### 3. Material（素材管理 API）

需要启用数据库和资源配置：

- `/lspcapi/materials/` - 素材 CRUD 操作
- `/lspcapi/materials/upload` - 上传素材

### 4. TCP Simulator（TCP 模拟器 API）

**开发环境专用**，用于测试和开发：

- `/lspcapi/tcp-simulator/protocols` - 获取支持的协议
- `/lspcapi/tcp-simulator/create` - 创建模拟器
- `/lspcapi/tcp-simulator/list` - 列出所有模拟器
- `/lspcapi/tcp-simulator/:id/start` - 启动模拟器
- `/lspcapi/tcp-simulator/:id/modbus/slave` - Modbus Slave 管理
- `/lspcapi/tcp-simulator/:id/packets` - 报文监控

## 特性

### 1. 交互式测试

直接在 Swagger UI 中：

1. 展开任意 API
2. 点击 "Try it out"
3. 填写参数
4. 点击 "Execute"
5. 查看响应结果

### 2. 请求示例

每个 API 都包含：

- 请求参数示例
- 请求体示例（JSON）
- 响应体示例
- 错误响应示例

### 3. Schema 定义

在 "Schemas" 部分查看所有数据结构定义。

## 代码实现

### 添加新的 API 到文档

#### 1. 为请求/响应结构添加 Schema

```rust
use serde::Deserialize;

#[cfg(feature = "swagger")]
use utoipa::ToSchema;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct MyRequest {
    #[cfg_attr(feature = "swagger", schema(example = "示例值"))]
    pub field: String,
}
```

#### 2. 为 API 函数添加注解

```rust
#[cfg_attr(feature = "swagger", utoipa::path(
    post,
    path = "/lspcapi/my-api/action",
    tag = "My API",
    request_body = MyRequest,
    responses(
        (status = 200, description = "成功", body = MyResponse),
        (status = 400, description = "参数错误", body = Value)
    )
))]
pub async fn my_action(
    Json(req): Json<MyRequest>,
) -> Json<Value> {
    // 实现
}
```

#### 3. 在 swagger.rs 中注册

```rust
#[openapi(
    paths(
        // ...
        crate::web::my_api::my_action,
    ),
    components(
        schemas(
            // ...
            MyRequest,
            MyResponse,
        )
    ),
    tags(
        // ...
        (name = "My API", description = "我的 API 描述")
    )
)]
pub struct ApiDoc;
```

## 条件编译说明

### Feature Flag 机制

在 `Cargo.toml` 中定义：

```toml
[features]
default = []
swagger = []
```

### 代码中的条件编译

使用 `#[cfg(feature = "swagger")]` 和 `#[cfg_attr]`:

```rust
// 仅在 swagger feature 启用时导入
#[cfg(feature = "swagger")]
use utoipa::ToSchema;

// 条件性地 derive ToSchema
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct MyStruct {
    // ...
}

// 条件性地添加注解
#[cfg_attr(feature = "swagger", utoipa::path(...))]
pub async fn my_handler() {
    // ...
}

// 条件性地启用整个代码块
#[cfg(feature = "swagger")]
{
    app = app.merge(swagger_routes());
}
```

### 为什么使用条件编译？

1. **安全性**: 生产环境不暴露 API 文档
2. **减小二进制体积**: Swagger UI 相关代码不会编译进生产版本
3. **性能**: 减少运行时开销
4. **灵活性**: 开发和生产环境使用同一套代码

## 最佳实践

### 1. 文档注释

始终为 API 添加清晰的文档注释：

```rust
/// 创建模拟器
///
/// 创建一个新的 TCP 协议模拟器实例，可选择自动启动
#[cfg_attr(feature = "swagger", utoipa::path(...))]
pub async fn create_simulator() { }
```

### 2. 示例值

为重要字段提供示例值：

```rust
#[cfg_attr(feature = "swagger", schema(example = "PLC 模拟器"))]
pub name: String,

#[cfg_attr(feature = "swagger", schema(example = 502))]
pub port: u16,
```

### 3. 响应状态码

明确列出所有可能的响应状态：

```rust
responses(
    (status = 200, description = "成功"),
    (status = 400, description = "参数错误"),
    (status = 404, description = "资源不存在"),
    (status = 500, description = "服务器错误")
)
```

### 4. 标签分组

使用合理的标签组织 API：

```rust
tag = "TCP Simulator"  // 按功能模块分组
```

## 故障排查

### 1. Swagger UI 无法访问

检查是否使用 `--features swagger` 编译：

```bash
cargo run --features swagger
```

查看日志确认 Swagger 已启用：

```
[INFO] Swagger UI 已启用: /swagger-ui/ (开发环境)
```

### 2. API 未显示在文档中

确认：

1. 函数添加了 `#[cfg_attr(feature = "swagger", utoipa::path(...))]`
2. 在 `swagger.rs` 的 `paths()` 中注册
3. 相关结构体在 `schemas()` 中注册

### 3. 编译错误

常见错误：

- `summary` 和 `description` 不是 `utoipa::path` 的有效属性
  - 解决：使用文档注释 `///` 代替

- `ToSchema` 未导入
  - 解决：添加 `#[cfg(feature = "swagger")] use utoipa::ToSchema;`

## 参考资料

- [utoipa 官方文档](https://docs.rs/utoipa/)
- [Swagger/OpenAPI 规范](https://swagger.io/specification/)
- [本项目文档](./HTTP_API.md)
- [TCP 模拟器指南](./TCP_SIMULATOR_GUIDE.md)

## 总结

- ✅ **开发**: 使用 `--features swagger` 启用完整 API 文档
- ❌ **生产**: 不添加 feature flag，保持安全和精简
- 📚 **文档**: 通过 Swagger UI 交互式测试和学习 API
- 🔒 **安全**: 条件编译确保生产环境不暴露文档
