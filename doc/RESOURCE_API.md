# 资源管理 API 文档

资源管理 API 提供文件上传、查询、更新、删除等功能，支持图片、视频、音频、文档等多种资源类型的统一管理。

## 目录

- [配置说明](#配置说明)
- [数据模型](#数据模型)
- [API 端点](#api-端点)
  - [上传资源](#上传资源)
  - [获取资源列表](#获取资源列表)
  - [获取单个资源](#获取单个资源)
  - [创建资源记录](#创建资源记录)
  - [更新资源](#更新资源)
  - [删除资源](#删除资源)
  - [批量覆盖资源](#批量覆盖资源)
  - [访问静态资源](#访问静态资源)
- [错误码](#错误码)

---

## 配置说明

在 `config.json` 中添加资源管理配置：

```json
{
  "database": {
    "enable": true,
    "url": "mysql://user:password@localhost:3306/database_name"
  },
  "resource": {
    "enable": true,
    "path": "/data/dm-rust/resources",
    "url_prefix": "/static"
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `enable` | boolean | 是 | 是否启用资源管理功能 |
| `path` | string | 是 | 静态文件存储根路径 |
| `url_prefix` | string | 否 | URL 访问前缀，默认 `/static` |

> **注意**：资源管理功能依赖数据库，需同时启用 `database.enable`。

---

## 数据模型

### Resource（资源）

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "示例图片.png",
  "resource_type": "image",
  "path": "202512/550e8400-e29b-41d4-a716-446655440000.png",
  "size": 102400,
  "mime_type": "image/png",
  "original_name": "示例图片.png",
  "created_at": "2025-12-03T10:30:00Z",
  "updated_at": "2025-12-03T10:30:00Z"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 资源唯一标识（UUID） |
| `name` | string | 资源名称 |
| `resource_type` | string | 资源类型：`image`, `video`, `audio`, `document`, `other` |
| `path` | string | 文件相对路径（相对于配置的 `path`） |
| `size` | number | 文件大小（字节） |
| `mime_type` | string | MIME 类型 |
| `original_name` | string | 原始文件名 |
| `created_at` | string | 创建时间（ISO 8601） |
| `updated_at` | string | 更新时间（ISO 8601） |

### 资源类型映射

| 类型 | 文件扩展名 |
|------|-----------|
| `image` | png, jpg, jpeg, gif, webp, svg, bmp, ico |
| `video` | mp4, webm, ogg, avi, mkv, mov, flv, wmv |
| `audio` | mp3, wav, flac, aac, m4a, wma |
| `document` | pdf, doc, docx, xls, xlsx, ppt, pptx, txt, md |
| `other` | 其他类型 |

---

## API 端点

### 上传资源

上传单个文件，自动创建数据库记录并返回访问 URL。

**请求**

```
POST /api/resources/upload
Content-Type: multipart/form-data
```

**表单字段**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `file` | file | 是 | 要上传的文件（单文件） |

**示例**

```bash
curl -X POST http://localhost:8080/api/resources/upload \
  -F "file=@/path/to/image.png"
```

**响应**

```json
{
  "state": 0,
  "message": "上传成功",
  "data": {
    "resource": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "image.png",
      "resource_type": "image",
      "path": "202512/550e8400-e29b-41d4-a716-446655440000.png",
      "size": 102400,
      "mime_type": "image/png",
      "original_name": "image.png",
      "created_at": "2025-12-03T10:30:00Z",
      "updated_at": "2025-12-03T10:30:00Z"
    },
    "url": "/static/202512/550e8400-e29b-41d4-a716-446655440000.png"
  }
}
```

**文件存储结构**

文件按日期（YYYYMM）组织存储：

```
/data/dm-rust/resources/
├── 202512/
│   ├── uuid1.png
│   ├── uuid2.mp4
│   └── uuid3.pdf
├── 202601/
│   └── uuid4.jpg
```

---

### 获取资源列表

获取所有资源或按条件筛选。

**请求**

```
GET /api/resources
GET /api/resources?type={resource_type}
GET /api/resources?name={keyword}
```

**查询参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 否 | 按资源类型筛选：`image`, `video`, `audio`, `document`, `other` |
| `name` | string | 否 | 按名称模糊搜索 |

**示例**

```bash
# 获取所有资源
curl http://localhost:8080/api/resources

# 按类型筛选
curl http://localhost:8080/api/resources?type=image

# 按名称搜索
curl http://localhost:8080/api/resources?name=logo
```

**响应**

```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "logo.png",
      "resource_type": "image",
      "path": "202512/550e8400-e29b-41d4-a716-446655440000.png",
      "size": 102400,
      "mime_type": "image/png",
      "original_name": "logo.png",
      "created_at": "2025-12-03T10:30:00Z",
      "updated_at": "2025-12-03T10:30:00Z"
    }
  ]
}
```

---

### 获取单个资源

根据 ID 获取资源详情。

**请求**

```
GET /api/resources/:id
```

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 资源 ID |

**示例**

```bash
curl http://localhost:8080/api/resources/550e8400-e29b-41d4-a716-446655440000
```

**响应**

```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "logo.png",
    "resource_type": "image",
    "path": "202512/550e8400-e29b-41d4-a716-446655440000.png",
    "size": 102400,
    "mime_type": "image/png",
    "original_name": "logo.png",
    "created_at": "2025-12-03T10:30:00Z",
    "updated_at": "2025-12-03T10:30:00Z"
  }
}
```

---

### 创建资源记录

手动创建资源记录（不上传文件）。

**请求**

```
POST /api/resources
Content-Type: application/json
```

**请求体**

```json
{
  "id": "可选，不填则自动生成UUID",
  "name": "资源名称",
  "resource_type": "image",
  "path": "202512/custom-file.png",
  "size": 102400,
  "mime_type": "image/png",
  "original_name": "原始文件名.png"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 否 | 资源 ID，不填则自动生成 |
| `name` | string | 是 | 资源名称 |
| `resource_type` | string | 是 | 资源类型 |
| `path` | string | 是 | 文件相对路径 |
| `size` | number | 否 | 文件大小，默认 0 |
| `mime_type` | string | 否 | MIME 类型，默认空 |
| `original_name` | string | 否 | 原始文件名，默认空 |

**示例**

```bash
curl -X POST http://localhost:8080/api/resources \
  -H "Content-Type: application/json" \
  -d '{
    "name": "手动添加的资源",
    "resource_type": "document",
    "path": "202512/manual-doc.pdf"
  }'
```

**响应**

```json
{
  "state": 0,
  "message": "创建成功",
  "data": {
    "id": "新生成的UUID",
    "name": "手动添加的资源",
    "resource_type": "document",
    "path": "202512/manual-doc.pdf",
    "size": 0,
    "mime_type": "",
    "original_name": "",
    "created_at": "2025-12-03T10:30:00Z",
    "updated_at": "2025-12-03T10:30:00Z"
  }
}
```

---

### 更新资源

更新资源的名称或类型。

**请求**

```
PUT /api/resources/:id
Content-Type: application/json
```

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 资源 ID |

**请求体**

```json
{
  "name": "新名称",
  "resource_type": "新类型"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `name` | string | 否 | 新的资源名称 |
| `resource_type` | string | 否 | 新的资源类型 |

**示例**

```bash
curl -X PUT http://localhost:8080/api/resources/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{"name": "更新后的名称"}'
```

**响应**

```json
{
  "state": 0,
  "message": "更新成功",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "更新后的名称",
    "resource_type": "image",
    "path": "202512/550e8400-e29b-41d4-a716-446655440000.png",
    "size": 102400,
    "mime_type": "image/png",
    "original_name": "logo.png",
    "created_at": "2025-12-03T10:30:00Z",
    "updated_at": "2025-12-03T10:35:00Z"
  }
}
```

---

### 删除资源

删除资源记录及对应的物理文件。

**请求**

```
DELETE /api/resources/:id
```

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 资源 ID |

**示例**

```bash
curl -X DELETE http://localhost:8080/api/resources/550e8400-e29b-41d4-a716-446655440000
```

**响应**

```json
{
  "state": 0,
  "message": "删除成功",
  "data": null
}
```

> **注意**：删除操作会同时删除数据库记录和物理文件。

---

### 批量覆盖资源

删除所有现有资源记录并重新创建（事务操作）。

**请求**

```
POST /api/resources/replace
Content-Type: application/json
```

**请求体**

```json
{
  "resources": [
    {
      "name": "资源1",
      "resource_type": "image",
      "path": "202512/file1.png"
    },
    {
      "name": "资源2",
      "resource_type": "video",
      "path": "202512/file2.mp4"
    }
  ]
}
```

**示例**

```bash
curl -X POST http://localhost:8080/api/resources/replace \
  -H "Content-Type: application/json" \
  -d '{
    "resources": [
      {"name": "新资源1", "resource_type": "image", "path": "202512/new1.png"},
      {"name": "新资源2", "resource_type": "image", "path": "202512/new2.png"}
    ]
  }'
```

**响应**

```json
{
  "state": 0,
  "message": "覆盖成功，共 2 条记录",
  "data": [
    {
      "id": "uuid1",
      "name": "新资源1",
      "resource_type": "image",
      "path": "202512/new1.png",
      "...": "..."
    },
    {
      "id": "uuid2",
      "name": "新资源2",
      "resource_type": "image",
      "path": "202512/new2.png",
      "...": "..."
    }
  ]
}
```

> **警告**：此操作会删除所有现有记录，请谨慎使用。

---

### 访问静态资源

通过 URL 直接访问上传的静态文件。

**请求**

```
GET /static/{path}
```

**路径参数**

| 参数 | 类型 | 说明 |
|------|------|------|
| `path` | string | 文件相对路径（与数据库中的 `path` 字段对应） |

**示例**

```bash
# 直接在浏览器访问
http://localhost:8080/static/202512/550e8400-e29b-41d4-a716-446655440000.png

# 或使用 curl
curl http://localhost:8080/static/202512/550e8400-e29b-41d4-a716-446655440000.png \
  --output downloaded.png
```

**响应**

- 成功：返回文件内容，`Content-Type` 根据文件扩展名自动设置
- 失败：
  - `400 Bad Request` - 无效的路径
  - `404 Not Found` - 文件不存在

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 400 | 参数错误 |
| 404 | 资源不存在 |

**错误响应示例**

```json
{
  "state": 404,
  "message": "资源不存在",
  "data": null
}
```

---

## 数据库表结构

```sql
CREATE TABLE IF NOT EXISTS `lspc_resource` (
    `id` VARCHAR(64) NOT NULL PRIMARY KEY COMMENT '主键ID（UUID）',
    `name` VARCHAR(255) NOT NULL COMMENT '资源名称',
    `resource_type` VARCHAR(32) NOT NULL COMMENT '资源类型：image, video, audio, document, other',
    `path` VARCHAR(1024) NOT NULL COMMENT '文件路径（相对于静态目录）',
    `size` BIGINT NOT NULL DEFAULT 0 COMMENT '文件大小（字节）',
    `mime_type` VARCHAR(128) NOT NULL DEFAULT '' COMMENT 'MIME类型',
    `original_name` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '原始文件名',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX `idx_resource_type` (`resource_type`),
    INDEX `idx_name` (`name`),
    INDEX `idx_path` (`path`(255))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='资源管理表';
```

---

## 使用场景示例

### 前端图片上传组件

```javascript
async function uploadImage(file) {
  const formData = new FormData();
  formData.append('file', file);

  const response = await fetch('/api/resources/upload', {
    method: 'POST',
    body: formData
  });

  const result = await response.json();
  if (result.state === 0 && result.data) {
    // 获取上传后的访问 URL
    const imageUrl = result.data.url;
    console.log('图片地址:', imageUrl);
    return imageUrl;
  }
  throw new Error(result.message);
}
```

### 资源管理页面

```javascript
// 获取图片列表
const images = await fetch('/api/resources?type=image').then(r => r.json());

// 删除资源
await fetch(`/api/resources/${resourceId}`, { method: 'DELETE' });

// 更新资源名称
await fetch(`/api/resources/${resourceId}`, {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ name: '新名称' })
});
```
