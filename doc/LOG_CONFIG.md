# 日志配置说明

## 概述

DM-Rust 支持灵活的日志配置，可以将日志输出到控制台、文件或同时输出到两者。这对于服务运行时的调试和监控非常重要。

## 配置项

在 `config.json` 中添加 `log` 配置块：

```json
{
  "log": {
    "level": "info",
    "target": "both",
    "file": "logs/dm-rust.log",
    "append": true
  }
}
```

### 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `level` | string | `"info"` | 日志级别：`trace`, `debug`, `info`, `warn`, `error` |
| `target` | string | `"console"` | 输出目标：`console`（控制台）, `file`（文件）, `both`（两者） |
| `file` | string | `"logs/dm-rust.log"` | 日志文件路径（当 target 为 `file` 或 `both` 时使用） |
| `append` | boolean | `true` | 是否追加到现有文件（false 则覆盖） |

## 日志级别说明

从详细到简略：

- **trace**: 最详细的日志，包含所有调试信息
- **debug**: 调试信息，用于开发和故障排查
- **info**: 一般信息，记录重要的运行状态（推荐用于生产环境）
- **warn**: 警告信息，可能的问题但不影响运行
- **error**: 错误信息，记录错误和异常

## 输出目标说明

### console（控制台）

日志仅输出到标准输出（控制台）。

**适用场景**：
- 开发调试
- 直接运行程序（非服务模式）
- 使用systemd等工具管理日志

**配置示例**：
```json
{
  "log": {
    "level": "debug",
    "target": "console"
  }
}
```

### file（文件）

日志仅输出到文件。

**适用场景**：
- Windows服务运行
- 需要长期保存日志
- 生产环境部署

**配置示例**：
```json
{
  "log": {
    "level": "info",
    "target": "file",
    "file": "C:/logs/device-manage.log",
    "append": true
  }
}
```

**注意**：
- 日志文件所在目录会自动创建
- 建议使用绝对路径以避免路径问题
- Windows路径使用正斜杠 `/` 或转义的反斜杠 `\\`

### both（控制台和文件）

日志同时输出到控制台和文件。

**适用场景**：
- 开发和测试环境
- 需要实时查看又要保存日志
- 调试Windows服务

**配置示例**：
```json
{
  "log": {
    "level": "debug",
    "target": "both",
    "file": "logs/dm-rust.log",
    "append": true
  }
}
```

## Windows服务运行建议

当作为Windows服务运行时，**强烈建议**使用文件日志：

```json
{
  "log": {
    "level": "info",
    "target": "file",
    "file": "C:/ProgramData/device-manage/logs/service.log",
    "append": true
  }
}
```

原因：
- Windows服务没有控制台窗口
- 文件日志便于故障排查
- 可以使用日志查看工具实时监控

## 常见配置示例

### 开发环境

```json
{
  "log": {
    "level": "debug",
    "target": "console"
  }
}
```

### 生产环境（普通运行）

```json
{
  "log": {
    "level": "info",
    "target": "both",
    "file": "logs/dm-rust.log",
    "append": true
  }
}
```

### 生产环境（Windows服务）

```json
{
  "log": {
    "level": "info",
    "target": "file",
    "file": "C:/ProgramData/device-manage/logs/service.log",
    "append": true
  }
}
```

### 故障排查

```json
{
  "log": {
    "level": "trace",
    "target": "both",
    "file": "logs/debug.log",
    "append": false
  }
}
```

## 日志文件管理

### 日志文件位置建议

**Windows服务**：
- `C:/ProgramData/device-manage/logs/` - 推荐
- `C:/logs/device-manage/` - 备选

**普通运行**：
- `./logs/` - 程序目录下
- 用户有权限的任意目录

### 日志轮转

当前版本不支持自动日志轮转。建议：

1. **手动管理**：定期清理或归档旧日志
2. **使用外部工具**：如 Windows 任务计划程序配合脚本
3. **设置 append=false**：每次启动覆盖（不推荐生产环境）

示例清理脚本（PowerShell）：
```powershell
# 删除7天前的日志
$logPath = "C:\ProgramData\device-manage\logs"
Get-ChildItem -Path $logPath -Filter "*.log" | 
    Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } | 
    Remove-Item
```

## 查看日志

### 实时查看（PowerShell）

```powershell
# 类似 tail -f
Get-Content -Path "C:\ProgramData\device-manage\logs\service.log" -Wait -Tail 50
```

### 搜索错误

```powershell
# 查找错误日志
Select-String -Path "logs\*.log" -Pattern "ERROR|WARN"
```

### 使用工具

推荐的日志查看工具：
- **Notepad++** - 支持大文件，语法高亮
- **BareTail** - 实时日志监控
- **Log Expert** - 功能丰富的日志分析工具

## 命令行参数覆盖

命令行参数 `-l` 或 `--log-level` 可以覆盖配置文件中的日志级别：

```powershell
# 使用 debug 级别运行，即使配置文件中是 info
.\dm-rust.exe -l debug

# 指定配置文件和日志级别
.\dm-rust.exe -c config.json -l trace
```

**注意**：命令行参数只影响日志级别，不影响输出目标和文件路径。

## 故障排除

### 问题：日志文件未创建

**可能原因**：
1. 目录权限不足
2. 路径错误

**解决方案**：
1. 检查日志文件路径是否正确
2. 确保程序有权限创建目录和文件
3. 使用绝对路径
4. 如果是服务，确保 LocalSystem 账户有权限

### 问题：日志文件过大

**解决方案**：
1. 设置 `append: false`（每次启动覆盖）
2. 定期手动清理
3. 使用脚本自动归档
4. 降低日志级别（如从 debug 改为 info）

### 问题：看不到服务日志

**解决方案**：
1. 确认配置文件中 target 设置为 `file` 或 `both`
2. 检查日志文件路径
3. 查看 Windows 事件查看器中的应用程序日志
4. 尝试手动运行程序查看错误

## 最佳实践

1. **开发环境**：使用 `console` 或 `both`，级别设为 `debug`
2. **生产环境**：使用 `file` 或 `both`，级别设为 `info`
3. **Windows服务**：必须使用 `file` 或 `both`
4. **故障排查**：临时调整为 `trace` 级别
5. **定期维护**：清理或归档旧日志文件
6. **使用绝对路径**：特别是Windows服务
