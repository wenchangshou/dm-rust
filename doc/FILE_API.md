# 文件管理 API 文档

本文档描述设备控制系统的文件管理 HTTP API 接口。

## 配置说明

在 `config.json` 中添加 `file` 配置项以启用文件管理功能：

```json
{
  "file": {
    "enable": true,
    "path": "/tmp/device_files"
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `enable` | boolean | 是 | 是否启用文件管理功能 |
| `path` | string | 是 | 文件存储根目录路径 |

## API 响应格式

所有 API 响应均采用统一的 JSON 格式：

```json
{
  "state": 0,
  "message": "成功",
  "data": {}
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `state` | number | 状态码，0 表示成功，非 0 表示失败 |
| `message` | string | 状态描述信息 |
| `data` | any | 响应数据，失败时可能为 null |

---

## API 接口列表

### 1. 列出目录内容

列出指定目录下的所有文件和子目录。

**请求**

```
GET /file/list?path=<相对路径>
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | string | 否 | 相对于根目录的路径，默认为根目录 |

**响应**

```json
{
  "state": 0,
  "message": "成功",
  "data": [
    {
      "name": "documents",
      "path": "documents",
      "is_dir": true,
      "size": 4096,
      "modified": "2025-12-03 10:30:00"
    },
    {
      "name": "readme.txt",
      "path": "readme.txt",
      "is_dir": false,
      "size": 1024,
      "modified": "2025-12-03 09:15:00"
    }
  ]
}
```

**示例**

```bash
# 列出根目录
curl "http://localhost:8080/file/list"

# 列出子目录
curl "http://localhost:8080/file/list?path=documents"
```

---

### 2. 上传文件

上传一个或多个文件到指定目录。

**请求**

```
POST /file/upload?path=<目标目录>
Content-Type: multipart/form-data
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | string | 否 | 上传目标目录，默认为根目录 |
| `file` | file | 是 | 上传的文件（multipart/form-data） |

**响应**

```json
{
  "state": 0,
  "message": "上传完成，成功 2 个文件",
  "data": ["file1.txt", "file2.pdf"]
}
```

**示例**

```bash
# 上传单个文件到根目录
curl -X POST "http://localhost:8080/file/upload" \
  -F "file=@/path/to/local/file.txt"

# 上传文件到指定目录
curl -X POST "http://localhost:8080/file/upload?path=documents" \
  -F "file=@/path/to/local/file.txt"

# 上传多个文件
curl -X POST "http://localhost:8080/file/upload" \
  -F "file1=@/path/to/file1.txt" \
  -F "file2=@/path/to/file2.pdf"
```

---

### 3. 下载文件

下载指定的文件。

**请求**

```
GET /file/download?path=<文件路径>
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | string | 是 | 要下载的文件相对路径 |

**响应**

成功时返回文件内容（二进制流），响应头包含：
- `Content-Type: application/octet-stream`
- `Content-Disposition: attachment; filename="文件名"`

失败时返回错误信息。

**示例**

```bash
# 下载文件
curl -O "http://localhost:8080/file/download?path=documents/report.pdf"

# 下载并保存为指定文件名
curl -o my_report.pdf "http://localhost:8080/file/download?path=documents/report.pdf"
```

---

### 4. 删除文件或目录

删除指定的文件或目录（递归删除）。

**请求**

```
POST /file/delete
Content-Type: application/json
```

**请求体**

```json
{
  "path": "documents/old_file.txt"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | string | 是 | 要删除的文件或目录路径 |

**响应**

```json
{
  "state": 0,
  "message": "删除成功",
  "data": null
}
```

**示例**

```bash
# 删除文件
curl -X POST "http://localhost:8080/file/delete" \
  -H "Content-Type: application/json" \
  -d '{"path": "documents/old_file.txt"}'

# 删除目录（递归删除所有内容）
curl -X POST "http://localhost:8080/file/delete" \
  -H "Content-Type: application/json" \
  -d '{"path": "old_directory"}'
```

---

### 5. 创建目录

创建新目录（支持递归创建多级目录）。

**请求**

```
POST /file/mkdir
Content-Type: application/json
```

**请求体**

```json
{
  "path": "documents/2025/reports"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | string | 是 | 要创建的目录路径 |

**响应**

```json
{
  "state": 0,
  "message": "目录创建成功",
  "data": null
}
```

**示例**

```bash
# 创建单级目录
curl -X POST "http://localhost:8080/file/mkdir" \
  -H "Content-Type: application/json" \
  -d '{"path": "new_folder"}'

# 创建多级目录
curl -X POST "http://localhost:8080/file/mkdir" \
  -H "Content-Type: application/json" \
  -d '{"path": "documents/2025/Q1/reports"}'
```

---

### 6. 重命名文件或目录

重命名或移动文件/目录。

**请求**

```
POST /file/rename
Content-Type: application/json
```

**请求体**

```json
{
  "old_path": "documents/old_name.txt",
  "new_path": "documents/new_name.txt"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `old_path` | string | 是 | 原路径 |
| `new_path` | string | 是 | 新路径 |

**响应**

```json
{
  "state": 0,
  "message": "重命名成功",
  "data": null
}
```

**示例**

```bash
# 重命名文件
curl -X POST "http://localhost:8080/file/rename" \
  -H "Content-Type: application/json" \
  -d '{"old_path": "file.txt", "new_path": "renamed_file.txt"}'

# 移动文件到其他目录
curl -X POST "http://localhost:8080/file/rename" \
  -H "Content-Type: application/json" \
  -d '{"old_path": "file.txt", "new_path": "documents/file.txt"}'
```

---

### 7. 获取文件信息

获取指定文件或目录的详细信息。

**请求**

```
GET /file/info?path=<文件路径>
```

**参数**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `path` | string | 是 | 文件或目录的相对路径 |

**响应**

```json
{
  "state": 0,
  "message": "成功",
  "data": {
    "name": "report.pdf",
    "path": "documents/report.pdf",
    "is_dir": false,
    "size": 102400,
    "modified": "2025-12-03 14:30:00"
  }
}
```

**示例**

```bash
curl "http://localhost:8080/file/info?path=documents/report.pdf"
```

---

## 错误码

| 状态码 | 说明 |
|--------|------|
| 0 | 成功 |
| 30001 | 无效的参数 |
| 30000 | 一般错误（包含详细错误信息） |

## 安全说明

1. **路径遍历防护**：所有路径参数都会进行安全校验，防止访问配置目录之外的文件
2. **根目录保护**：不允许删除配置的根目录
3. **功能开关**：可通过配置 `file.enable` 控制是否启用文件管理功能

## 使用示例

### Python 示例

```python
import requests

BASE_URL = "http://localhost:8080"

# 列出目录
response = requests.get(f"{BASE_URL}/file/list")
print(response.json())

# 上传文件
with open("local_file.txt", "rb") as f:
    response = requests.post(
        f"{BASE_URL}/file/upload",
        files={"file": f}
    )
print(response.json())

# 下载文件
response = requests.get(f"{BASE_URL}/file/download?path=uploaded_file.txt")
with open("downloaded_file.txt", "wb") as f:
    f.write(response.content)

# 创建目录
response = requests.post(
    f"{BASE_URL}/file/mkdir",
    json={"path": "new_directory"}
)
print(response.json())

# 删除文件
response = requests.post(
    f"{BASE_URL}/file/delete",
    json={"path": "old_file.txt"}
)
print(response.json())
```

### JavaScript/Node.js 示例

```javascript
const axios = require('axios');
const FormData = require('form-data');
const fs = require('fs');

const BASE_URL = 'http://localhost:8080';

// 列出目录
async function listFiles(path = '') {
  const response = await axios.get(`${BASE_URL}/file/list`, {
    params: { path }
  });
  return response.data;
}

// 上传文件
async function uploadFile(filePath, targetPath = '') {
  const form = new FormData();
  form.append('file', fs.createReadStream(filePath));

  const response = await axios.post(`${BASE_URL}/file/upload`, form, {
    params: { path: targetPath },
    headers: form.getHeaders()
  });
  return response.data;
}

// 下载文件
async function downloadFile(remotePath, localPath) {
  const response = await axios.get(`${BASE_URL}/file/download`, {
    params: { path: remotePath },
    responseType: 'stream'
  });

  const writer = fs.createWriteStream(localPath);
  response.data.pipe(writer);

  return new Promise((resolve, reject) => {
    writer.on('finish', resolve);
    writer.on('error', reject);
  });
}

// 创建目录
async function createDirectory(path) {
  const response = await axios.post(`${BASE_URL}/file/mkdir`, { path });
  return response.data;
}

// 删除文件/目录
async function deleteFile(path) {
  const response = await axios.post(`${BASE_URL}/file/delete`, { path });
  return response.data;
}
```
