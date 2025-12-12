# 服务调试指南

## 问题：服务启动后没有响应，日志文件也没有创建

### 解决方案

#### 1. 检查日志文件位置

服务启动时会在**程序所在目录**创建日志文件：

```
C:\Users\wench\代码\dm-rust\target\release\logs\service-startup.log
```

查看此文件以获取启动详细信息。

#### 2. 手动测试程序

在安装服务前，先手动运行程序测试：

```powershell
cd C:\Users\wench\代码\dm-rust\target\release
.\dm-rust.exe
```

观察是否有错误输出。

#### 3. 检查配置文件

确保 `config.json` 在程序目录下：

```powershell
# 检查文件是否存在
Test-Path C:\Users\wench\代码\dm-rust\target\release\config.json

# 查看内容
Get-Content C:\Users\wench\代码\dm-rust\target\release\config.json
```

最简配置：
```json
{
  "channels": [],
  "nodes": [],
  "scenes": [],
  "web_server": {
    "port": 8080
  },
  "log": {
    "level": "debug",
    "target": "file",
    "file": "logs/service.log",
    "append": true
  }
}
```

#### 4. 查看Windows事件日志

打开事件查看器查看错误信息：

```powershell
# 使用PowerShell查看最近的应用程序日志
Get-EventLog -LogName Application -Newest 20 | 
    Where-Object {$_.Source -like "*device*" -or $_.Message -like "*dm-rust*"}
```

或手动打开：
- 按 `Win + R`
- 输入 `eventvwr.msc`
- 导航到：Windows日志 > 应用程序

#### 5. 重新安装服务

```powershell
# 以管理员身份运行
cd C:\Users\wench\代码\dm-rust\target\release

# 停止并卸载旧服务
.\dm-rust.exe -s stop
.\dm-rust.exe --uninstall

# 重新安装
.\dm-rust.exe --install

# 启动服务
.\dm-rust.exe -s start

# 查看服务状态
sc query device-manage
```

#### 6. 检查日志目录权限

确保程序有权限创建日志目录：

```powershell
# 检查目录是否存在
Test-Path C:\Users\wench\代码\dm-rust\target\release\logs

# 手动创建日志目录
New-Item -ItemType Directory -Path C:\Users\wench\代码\dm-rust\target\release\logs -Force

# 设置权限（如果需要）
icacls C:\Users\wench\代码\dm-rust\target\release\logs /grant "SYSTEM:(OI)(CI)F"
```

#### 7. 使用绝对路径配置

修改 `config.json` 使用绝对路径：

```json
{
  "log": {
    "level": "debug",
    "target": "file",
    "file": "C:/Users/wench/代码/dm-rust/target/release/logs/service.log",
    "append": true
  }
}
```

#### 8. 测试日志写入

手动测试是否能写入日志文件：

```powershell
# 创建测试文件
"Test" | Out-File -FilePath "C:\Users\wench\代码\dm-rust\target\release\logs\test.log"

# 检查是否创建成功
Get-Content "C:\Users\wench\代码\dm-rust\target\release\logs\test.log"
```

#### 9. 直接在控制台运行服务代码

临时修改配置使用控制台输出：

```json
{
  "log": {
    "level": "debug",
    "target": "console"
  }
}
```

然后手动运行：
```powershell
.\dm-rust.exe
```

#### 10. 检查端口占用

确保配置的端口（默认8080）未被占用：

```powershell
# 检查端口占用
netstat -ano | findstr :8080

# 如果被占用，修改配置文件中的端口
```

## 完整的调试流程

```powershell
# 1. 以管理员身份打开PowerShell
# 2. 进入程序目录
cd C:\Users\wench\代码\dm-rust\target\release

# 3. 创建日志目录
New-Item -ItemType Directory -Path logs -Force

# 4. 检查配置文件
Get-Content config.json

# 5. 手动运行测试
.\dm-rust.exe -l debug

# 6. 如果手动运行正常，停止程序并安装服务
# Ctrl+C 停止

# 7. 卸载旧服务（如果存在）
.\dm-rust.exe --uninstall

# 8. 重新安装
.\dm-rust.exe --install

# 9. 启动服务
.\dm-rust.exe -s start

# 10. 查看日志
Get-Content logs/service-startup.log -Wait

# 11. 查看服务状态
sc query device-manage

# 12. 如果失败，查看Windows事件日志
Get-EventLog -LogName Application -Newest 10
```

## 常见问题

### 问题：找不到配置文件

**解决**：将 `config.json` 放在可执行文件同目录下

### 问题：权限不足

**解决**：以管理员身份运行所有命令

### 问题：日志目录创建失败

**解决**：手动创建日志目录并设置权限

### 问题：服务启动超时

**解决**：
1. 检查配置文件语法是否正确
2. 检查数据库连接（如果启用）
3. 检查端口是否被占用
4. 查看 `service-startup.log` 文件

## 推荐的服务配置

```json
{
  "channels": [],
  "nodes": [],
  "scenes": [],
  "web_server": {
    "port": 8080
  },
  "log": {
    "level": "info",
    "target": "file",
    "file": "C:/ProgramData/device-manage/logs/service.log",
    "append": true
  },
  "database": {
    "enable": false
  }
}
```

## 获取帮助

如果问题仍然存在，请收集以下信息：

1. `logs/service-startup.log` 文件内容
2. Windows事件日志中的错误信息
3. 配置文件内容
4. 服务状态输出：`sc query device-manage`
