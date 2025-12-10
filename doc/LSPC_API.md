# LSPC API 文档

## 1. 概述

LSPC (Large Screen Presentation Controller) API 提供屏幕设计和素材管理功能。

**基础路径**: `/lspcapi`

**服务端口**: `18080`

**通用响应格式**:
```json
{
  "state": 0,
  "message": "成功",
  "data": { ... }
}
```

**状态码说明**:

| state | 说明 |
|-------|------|
| 0 | 成功 |
| 1 | 通用错误 |
| 400 | 参数无效 |
| 404 | 资源不存在 |

---

## 2. 数据库设计

### 2.1 表结构

#### 2.1.1 屏幕设计表：lspc_screen

```sql
CREATE TABLE IF NOT EXISTS `lspc_screen` (
    `id` VARCHAR(64) NOT NULL PRIMARY KEY COMMENT '主键ID（UUID）',
    `type` VARCHAR(32) NOT NULL COMMENT '类型：Clean, Close, Normal, Pause, Register, Vote',
    `name` VARCHAR(255) NOT NULL COMMENT '名称',
    `content` TEXT NOT NULL COMMENT '内容（JSON或文本）',
    `active` TINYINT(1) NOT NULL DEFAULT 0 COMMENT '是否激活',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX `idx_type` (`type`),
    INDEX `idx_active` (`active`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='屏幕配置表';
```

| 字段 | 类型 | 说明 |
|------|------|------|
| id | VARCHAR(64) | 主键，UUID |
| type | VARCHAR(32) | 屏幕类型：Clean, Close, Normal, Pause, Register, Vote |
| name | VARCHAR(255) | 屏幕设计名称 |
| content | TEXT | 屏幕内容（JSON格式） |
| active | TINYINT(1) | 是否为当前激活的设计 |
| created_at | DATETIME | 创建时间 |
| updated_at | DATETIME | 更新时间 |

#### 2.1.2 素材表：lspc_material

```sql
CREATE TABLE IF NOT EXISTS `lspc_material` (
    `id` VARCHAR(64) NOT NULL PRIMARY KEY COMMENT '主键ID（UUID）',
    `name` VARCHAR(255) NOT NULL COMMENT '素材名称',
    `screen_id` VARCHAR(64) NOT NULL DEFAULT '' COMMENT '关联的屏幕ID',
    `preset` TINYINT(1) NOT NULL DEFAULT 0 COMMENT '是否为预设素材',
    `path` VARCHAR(1024) NOT NULL DEFAULT '' COMMENT '文件路径（相对于静态目录）',
    `resource_type` VARCHAR(32) NOT NULL DEFAULT '' COMMENT '资源类型：image, video, audio, document, other',
    `size` BIGINT NOT NULL DEFAULT 0 COMMENT '文件大小（字节）',
    `mime_type` VARCHAR(128) NOT NULL DEFAULT '' COMMENT 'MIME类型',
    `original_name` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '原始文件名',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    INDEX `idx_name` (`name`),
    INDEX `idx_screen_id` (`screen_id`),
    INDEX `idx_preset` (`preset`),
    INDEX `idx_resource_type` (`resource_type`),
    INDEX `idx_path` (`path`(255))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='素材管理表';
```

| 字段 | 类型 | 说明 |
|------|------|------|
| id | VARCHAR(64) | 主键，UUID |
| name | VARCHAR(255) | 素材名称 |
| screen_id | VARCHAR(64) | 关联的屏幕设计ID |
| preset | TINYINT(1) | 是否为预设素材（true=预设，false=上传） |
| path | VARCHAR(1024) | 文件路径（相对于静态目录） |
| resource_type | VARCHAR(32) | 资源类型：image, video, audio, document, other |
| size | BIGINT | 文件大小（字节） |
| mime_type | VARCHAR(128) | MIME类型 |
| original_name | VARCHAR(255) | 原始文件名 |
| created_at | DATETIME | 创建时间 |

### 2.2 表关系图

```
┌─────────────────┐       ┌─────────────────┐
│   lspc_screen   │       │  lspc_material  │
├─────────────────┤       ├─────────────────┤
│ id (PK)         │◄──────│ screen_id       │
│ type            │       │ id (PK)         │
│ name            │       │ name            │
│ content         │       │ preset          │
│ active          │       │ path            │
│ created_at      │       │ resource_type   │
│ updated_at      │       │ size            │
└─────────────────┘       │ mime_type       │
                          │ original_name   │
                          │ created_at      │
                          └─────────────────┘
```

---

## 3. 屏幕设计 API

### 3.1 获取屏幕设计列表

**请求**:
```
GET /lspcapi/screens
```

**请求参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 否 | 按屏幕类型过滤，可选值：Register, Vote, Normal, Clean, Pause, Close |
| active | boolean | 否 | 按激活状态过滤，true 只获取激活的设计 |

> **参数组合说明**：
> - 同时指定 `type` 和 `active` 时，执行 AND 查询（例如：获取激活的 Vote 类型屏幕）
> - 只指定 `type` 时，返回该类型的所有屏幕（不论激活状态）
> - 只指定 `active=true` 时，返回所有激活的屏幕（不论类型）
> - 都不指定时，返回所有屏幕

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "screen_type": "Vote",
      "name": "投票屏幕设计",
      "content": "{\"title\": \"请投票\"}",
      "active": true,
      "created_at": "2025-12-04T11:39:25.551152Z",
      "updated_at": "2025-12-04T11:39:25.551152Z"
    }
  ]
}
```

**curl 示例**:
```bash
# 获取所有屏幕设计
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens" \
  -H "Accept: application/json"

# 按类型过滤
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens?type=Vote" \
  -H "Accept: application/json"

# 只获取激活的设计
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens?active=true" \
  -H "Accept: application/json"

# 获取激活的 Vote 类型屏幕（AND 查询）
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens?type=Vote&active=true" \
  -H "Accept: application/json"
```

---

### 3.2 获取单个屏幕设计

**请求**:
```
GET /lspcapi/screens/{id}
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 屏幕设计 ID |

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "screen_type": "Vote",
    "name": "投票屏幕设计",
    "content": "{\"title\": \"请投票\"}",
    "active": true,
    "created_at": "2025-12-04T11:39:25.551152Z",
    "updated_at": "2025-12-04T11:39:25.551152Z"
  }
}
```

**curl 示例**:
```bash
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens/550e8400-e29b-41d4-a716-446655440000" \
  -H "Accept: application/json"
```

---

### 3.3 新建屏幕设计

**请求**:
```
POST /lspcapi/screens
```

**请求体**:
```json
{
  "type": "Vote",
  "name": "投票屏幕设计",
  "content": "{\"title\": \"请投票\"}",
  "active": false
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 否 | 屏幕ID（可选，不提供则自动生成UUID） |
| type | string | 是 | 屏幕类型 |
| name | string | 是 | 屏幕设计名称 |
| content | string | 是 | 屏幕内容（JSON字符串） |
| active | boolean | 否 | 是否激活，默认 false |

**响应**:
```json
{
  "state": 0,
  "message": "创建成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "screen_type": "Vote",
    "name": "投票屏幕设计",
    "content": "{\"title\": \"请投票\"}",
    "active": false,
    "created_at": "2025-12-04T11:39:25.551152Z",
    "updated_at": "2025-12-04T11:39:25.551152Z"
  }
}
```

**curl 示例**:
```bash
curl -X POST "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "Vote",
    "name": "投票屏幕设计",
    "content": "{\"title\": \"请投票\"}"
  }'
```

---

### 3.4 更新屏幕设计

**请求**:
```
PUT /lspcapi/screens/{id}
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 屏幕设计 ID |

**请求体**:
```json
{
  "type": "Vote",
  "name": "更新后的名称",
  "content": "{\"title\": \"新内容\"}",
  "active": true
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| type | string | 否 | 屏幕类型 |
| name | string | 否 | 屏幕设计名称 |
| content | string | 否 | 屏幕内容（JSON字符串） |
| active | boolean | 否 | 是否激活 |

> 注意：所有字段都是可选的，只更新传入的字段

**响应**:
```json
{
  "state": 0,
  "message": "更新成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "screen_type": "Vote",
    "name": "更新后的名称",
    "content": "{\"title\": \"新内容\"}",
    "active": true,
    "created_at": "2025-12-04T11:39:25.551152Z",
    "updated_at": "2025-12-04T11:47:10.808280Z"
  }
}
```

**curl 示例**:
```bash
curl -X PUT "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens/550e8400-e29b-41d4-a716-446655440000" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "更新后的名称",
    "content": "{\"title\": \"新内容\"}"
  }'
```

---

### 3.5 删除屏幕设计

**请求**:
```
DELETE /lspcapi/screens/{id}
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 屏幕设计 ID |

**响应**:
```json
{
  "state": 0,
  "message": "删除成功"
}
```

**curl 示例**:
```bash
curl -X DELETE "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens/550e8400-e29b-41d4-a716-446655440000" \
  -H "Accept: application/json"
```

---

### 3.6 设置屏幕为激活状态

将指定屏幕设计设为激活状态，同时将**同一类型**的其他屏幕设为非激活。

> **重要说明**：此操作只影响相同类型（type）的屏幕。例如：
> - 激活一个 `Vote` 类型的屏幕时，只有其他 `Vote` 类型的屏幕会被设为非激活
> - `Normal`、`Register`、`Clean` 等其他类型的屏幕不受影响
> - 这意味着每种类型可以有各自的激活屏幕

**请求**:
```
PUT /lspcapi/screens/{id}/active
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 屏幕设计 ID |

**响应**:
```json
{
  "state": 0,
  "message": "设置成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "screen_type": "Vote",
    "name": "投票屏幕设计",
    "content": "{\"title\": \"请投票\"}",
    "active": true,
    "created_at": "2025-12-04T11:39:25.551152Z",
    "updated_at": "2025-12-04T14:03:04.693670Z"
  }
}
```

**curl 示例**:
```bash
curl -X PUT "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens/550e8400-e29b-41d4-a716-446655440000/active" \
  -H "Accept: application/json"
```

---

### 3.7 批量覆盖屏幕设计

删除所有屏幕设计后重新创建。

**请求**:
```
POST /lspcapi/screens/replace
```

**请求体**:
```json
{
  "screens": [
    {
      "type": "Vote",
      "name": "投票屏幕1",
      "content": "{}",
      "active": true
    },
    {
      "type": "Normal",
      "name": "普通屏幕1",
      "content": "{}",
      "active": false
    }
  ]
}
```

**响应**:
```json
{
  "state": 0,
  "message": "覆盖成功，共 2 条记录",
  "data": [...]
}
```

**curl 示例**:
```bash
curl -X POST "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens/replace" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
    "screens": [
      {"type": "Vote", "name": "投票屏幕", "content": "{}", "active": true}
    ]
  }'
```

---

## 4. 素材 API

### 4.1 获取素材列表

**请求**:
```
GET /lspcapi/materials
```

**请求参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| preset | boolean | 否 | 按是否为预设素材过滤（true=预设，false=上传） |
| name | string | 否 | 按素材名称过滤（模糊匹配） |

> 注意：preset 优先于 name 参数

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "背景图片",
      "screen_id": "550e8400-e29b-41d4-a716-446655440000",
      "preset": true,
      "path": "https://127.0.0.1/lspc/upload/202512/bg.png",
      "resource_type": "image",
      "size": 102400,
      "mime_type": "image/png",
      "original_name": "bg.png",
      "created_at": "2025-12-04T12:00:08Z"
    }
  ]
}
```

**curl 示例**:
```bash
# 获取所有素材
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials" \
  -H "Accept: application/json"

# 获取预设素材
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials?preset=true" \
  -H "Accept: application/json"

# 按名称模糊搜索
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials?name=背景" \
  -H "Accept: application/json"
```

---

### 4.2 获取单个素材

**请求**:
```
GET /lspcapi/materials/{id}
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 素材 ID |

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "背景图片",
    "screen_id": "550e8400-e29b-41d4-a716-446655440000",
    "preset": false,
    "path": "https://127.0.0.1/lspc/upload/202512/bg.png",
    "resource_type": "image",
    "size": 102400,
    "mime_type": "image/png",
    "original_name": "bg.png",
    "created_at": "2025-12-04T12:00:08Z"
  }
}
```

**curl 示例**:
```bash
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials/550e8400-e29b-41d4-a716-446655440001" \
  -H "Accept: application/json"
```

---

### 4.3 根据屏幕ID获取素材列表

**请求**:
```
GET /lspcapi/screens/{id}/materials
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 屏幕设计 ID |

**响应**:
```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "背景图片",
      "screen_id": "550e8400-e29b-41d4-a716-446655440000",
      "preset": false,
      "path": "https://127.0.0.1/lspc/upload/202512/bg.png",
      "resource_type": "image",
      "size": 102400,
      "mime_type": "image/png",
      "original_name": "bg.png",
      "created_at": "2025-12-04T12:00:08Z"
    }
  ]
}
```

**curl 示例**:
```bash
curl -X GET "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/screens/550e8400-e29b-41d4-a716-446655440000/materials" \
  -H "Accept: application/json"
```

---

### 4.4 上传素材

上传文件并创建素材记录。

**请求**:
```
POST /lspcapi/materials
Content-Type: multipart/form-data
```

**Form 参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| screenId | string | 是 | 关联的屏幕设计 ID |
| file | file | 是 | 上传的文件 |

**响应**:
```json
{
  "state": 0,
  "message": "上传成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440003",
    "name": "test-upload.jpg",
    "screen_id": "550e8400-e29b-41d4-a716-446655440000",
    "preset": false,
    "path": "https://127.0.0.1/lspc/upload/202512/550e8400-e29b-41d4-a716-446655440003.jpg",
    "createdAt": "2025-12-04T14:25:43.753408Z"
  }
}
```

**curl 示例**:
```bash
curl -X POST "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials" \
  -H "Accept: application/json" \
  -F "screenId=550e8400-e29b-41d4-a716-446655440000" \
  -F "file=@/path/to/image.jpg"
```

---

### 4.5 更新素材

**请求**:
```
PUT /lspcapi/materials/{id}
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 素材 ID |

**请求体**:
```json
{
  "name": "新名称",
  "screen_id": "新屏幕ID",
  "preset": true
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| name | string | 否 | 素材名称 |
| screen_id | string | 否 | 关联的屏幕ID |
| preset | boolean | 否 | 是否为预设素材 |

> 注意：所有字段都是可选的，只更新传入的字段

**响应**:
```json
{
  "state": 0,
  "message": "更新成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "新名称",
    "screen_id": "新屏幕ID",
    "preset": true,
    "path": "https://127.0.0.1/lspc/upload/202512/bg.png",
    "resource_type": "image",
    "size": 102400,
    "mime_type": "image/png",
    "original_name": "bg.png",
    "created_at": "2025-12-04T12:00:08Z"
  }
}
```

**curl 示例**:
```bash
curl -X PUT "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials/550e8400-e29b-41d4-a716-446655440001" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "新名称"
  }'
```

---

### 4.6 删除素材

删除素材记录，同时删除关联的物理文件。

**请求**:
```
DELETE /lspcapi/materials/{id}
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| id | string | 是 | 素材 ID |

**响应**:
```json
{
  "state": 0,
  "message": "删除成功"
}
```

**curl 示例**:
```bash
curl -X DELETE "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials/550e8400-e29b-41d4-a716-446655440001" \
  -H "Accept: application/json"
```

---

### 4.7 批量覆盖素材

删除所有素材后重新创建。

**请求**:
```
POST /lspcapi/materials/replace
```

**请求体**:
```json
{
  "materials": [
    {
      "name": "背景1",
      "screen_id": "scr-001",
      "preset": true,
      "path": "202512/bg1.png",
      "resource_type": "image",
      "size": 102400,
      "mime_type": "image/png"
    }
  ]
}
```

**响应**:
```json
{
  "state": 0,
  "message": "覆盖成功，共 1 条记录",
  "data": [...]
}
```

**curl 示例**:
```bash
curl -X POST "https://lspc.zoolon.com.cn:8443/lspc/lspcapi/materials/replace" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{
    "materials": [
      {"name": "背景", "screen_id": "scr-001", "path": "202512/bg.png", "resource_type": "image"}
    ]
  }'
```

---

## 5. 静态资源访问

访问已上传的静态资源文件。

**请求**:
```
GET /static/{path}
```

**路径参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| path | string | 是 | 资源文件路径（如：202512/550e8400.jpg） |

**响应**:
- 成功：返回文件内容，Content-Type 根据文件类型自动设置
- 失败：返回 404 或 400 状态码

**curl 示例**:
```bash
curl -X GET "https://lspc.zoolon.com.cn:8443/static/202512/550e8400.jpg" \
  -o downloaded.jpg
```

---

## 6. 错误响应

当请求失败时，响应格式如下：

```json
{
  "state": 404,
  "message": "Screen 不存在"
}
```

**常见错误**:

| state | 说明 |
|-------|------|
| 1 | 通用错误（数据库操作失败等） |
| 400 | 请求参数无效 |
| 404 | 资源不存在 |

---

## 7. 屏幕类型说明

| 类型 | 说明 |
|------|------|
| Register | 报到进程 |
| Vote | 表决进程 |
| Normal | 普通进程 |
| Clean | 清空大屏 |
| Pause | 暂停会议 |
| Close | 结束会议 |

---

## 8. 资源类型说明

| 类型 | 说明 | 文件扩展名 |
|------|------|-----------|
| image | 图片 | png, jpg, jpeg, gif, webp, svg, bmp, ico |
| video | 视频 | mp4, webm, ogg, avi, mkv, mov, flv, wmv |
| audio | 音频 | mp3, wav, flac, aac, m4a, wma |
| document | 文档 | pdf, doc, docx, xls, xlsx, ppt, pptx, txt, md |
| other | 其他 | 其他未识别的文件类型 |
