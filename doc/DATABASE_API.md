# 数据库 API 快速参考

## 配置

在 `config.json` 中添加数据库配置：

```json
{
  "database": {
    "enable": true,
    "url": "mysql://username:password@localhost:3306/database_name"
  }
}
```

## 数据表结构

### lspc_screen 表

```sql
CREATE TABLE `lspc_screen` (
  `id` varchar(36) NOT NULL,
  `type` enum('Clean','Close','Normal','Pause','Register','Vote') NOT NULL,
  `name` varchar(255) NOT NULL,
  `content` mediumtext NOT NULL,
  `active` bit(1) NOT NULL,
  `created_at` datetime(6) NOT NULL,
  `updated_at` datetime(6) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
```

### lspc_material 表

```sql
CREATE TABLE `lspc_material` (
  `id` varchar(36) NOT NULL,
  `name` varchar(255) NOT NULL,
  `path` varchar(255) NOT NULL,
  `created_at` datetime(6) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
```

---

## Screen API

### 获取所有 Screen
```bash
GET /api/screens
```

### 按类型查询
```bash
GET /api/screens?type=Normal
```

支持的类型: `Clean`, `Close`, `Normal`, `Pause`, `Register`, `Vote`

### 获取激活的 Screen
```bash
GET /api/screens?active=true
```

### 获取单个 Screen
```bash
GET /api/screens/:id
```

### 创建 Screen
```bash
POST /api/screens
Content-Type: application/json

{
  "type": "Normal",
  "name": "屏幕名称",
  "content": "屏幕内容",
  "active": true
}
```

注：`id` 可选，不提供则自动生成 UUID

### 更新 Screen
```bash
PUT /api/screens/:id
Content-Type: application/json

{
  "name": "新名称",
  "content": "新内容",
  "active": false
}
```

注：所有字段都是可选的，只更新提供的字段

### 删除 Screen
```bash
DELETE /api/screens/:id
```

### 批量覆盖 Screen（重要！）
删除所有现有数据，然后创建新数据：

```bash
POST /api/screens/replace
Content-Type: application/json

{
  "screens": [
    {
      "id": "uuid-1",
      "type": "Normal",
      "name": "屏幕1",
      "content": "内容1",
      "active": true
    },
    {
      "id": "uuid-2",
      "type": "Vote",
      "name": "屏幕2",
      "content": "内容2",
      "active": false
    }
  ]
}
```

---

## Material API

### 获取所有 Material
```bash
GET /api/materials
```

### 按名称搜索
```bash
GET /api/materials?name=关键词
```

### 获取单个 Material
```bash
GET /api/materials/:id
```

### 创建 Material
```bash
POST /api/materials
Content-Type: application/json

{
  "name": "素材名称",
  "path": "/path/to/file.png"
}
```

### 更新 Material
```bash
PUT /api/materials/:id
Content-Type: application/json

{
  "name": "新名称",
  "path": "/new/path/to/file.png"
}
```

### 删除 Material
```bash
DELETE /api/materials/:id
```

### 批量覆盖 Material（重要！）
删除所有现有数据，然后创建新数据：

```bash
POST /api/materials/replace
Content-Type: application/json

{
  "materials": [
    {
      "id": "uuid-1",
      "name": "素材1",
      "path": "/path/1.png"
    },
    {
      "id": "uuid-2",
      "name": "素材2",
      "path": "/path/2.jpg"
    }
  ]
}
```

---

## 响应格式

所有 API 返回统一的 JSON 格式：

### 成功响应
```json
{
  "state": 0,
  "message": "成功",
  "data": { ... }
}
```

### 错误响应
```json
{
  "state": 1,
  "message": "错误信息",
  "data": null
}
```

### 错误码
- `0`: 成功
- `1`: 通用错误
- `400`: 参数错误
- `404`: 资源不存在

---

## 使用示例 (curl)

### 创建 Screen
```bash
curl -X POST http://localhost:8080/api/screens \
  -H "Content-Type: application/json" \
  -d '{"type":"Normal","name":"测试屏幕","content":"Hello World","active":true}'
```

### 批量覆盖 Screen
```bash
curl -X POST http://localhost:8080/api/screens/replace \
  -H "Content-Type: application/json" \
  -d '{
    "screens": [
      {"type":"Normal","name":"屏幕1","content":"内容1","active":true},
      {"type":"Vote","name":"屏幕2","content":"内容2","active":false}
    ]
  }'
```

### 查询所有激活的 Screen
```bash
curl "http://localhost:8080/api/screens?active=true"
```

### 更新 Screen
```bash
curl -X PUT http://localhost:8080/api/screens/your-uuid-here \
  -H "Content-Type: application/json" \
  -d '{"name":"更新后的名称"}'
```

### 删除 Screen
```bash
curl -X DELETE http://localhost:8080/api/screens/your-uuid-here
```
