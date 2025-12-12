# Windows 服务管理指南

## 概述

DM-Rust 支持在 Windows 系统上作为系统服务运行。服务安装后会自动启动，并在系统重启后自动运行。

## 前置要求

- Windows 操作系统
- **管理员权限**（安装、卸载、启动、停止服务都需要管理员权限）

## 使用方法

### 1. 安装服务

以管理员身份运行 PowerShell 或 CMD，执行：

```powershell
.\dm-rust.exe --install
```

成功后会显示：
```
✓ 服务安装成功: DmRustService
  显示名称: DM-Rust Device Control Service
  描述: 工业设备统一控制系统服务
  执行文件: C:\path\to\dm-rust.exe
```

### 2. 启动服务

```powershell
# 方式1：使用程序命令
.\dm-rust.exe -s start

# 方式2：使用 Windows sc 命令
sc start DmRustService

# 方式3：使用服务管理器
services.msc
```

### 3. 停止服务

```powershell
# 方式1：使用程序命令
.\dm-rust.exe -s stop

# 方式2：使用 Windows sc 命令
sc stop DmRustService
```

### 4. 重启服务

```powershell
.\dm-rust.exe -s restart
```

### 5. 卸载服务

```powershell
.\dm-rust.exe --uninstall
```

**注意**：卸载前会自动停止正在运行的服务。

## 查看服务状态

```powershell
# 查询服务状态
sc query DmRustService

# 查看服务详细信息
sc qc DmRustService

# 或使用 PowerShell
Get-Service DmRustService
```

## 服务配置

### 服务信息

- **服务名称**: `DmRustService`
- **显示名称**: `DM-Rust Device Control Service`
- **启动类型**: 自动（Auto Start）
- **运行账户**: LocalSystem

### 修改启动类型

如果需要修改服务启动类型为手动或禁用：

```powershell
# 手动启动
sc config DmRustService start= demand

# 自动启动
sc config DmRustService start= auto

# 禁用
sc config DmRustService start= disabled
```

## 日志查看

服务运行时的日志可以通过以下方式查看：

1. **Windows 事件查看器**
   - 打开 `eventvwr.msc`
   - 导航到：Windows 日志 > 应用程序
   - 查找来源为 `DmRustService` 的事件

2. **应用程序日志文件**（如果配置了文件日志）
   - 查看配置文件中指定的日志路径

## 故障排除

### 问题：安装失败，提示"访问被拒绝"

**解决方案**：确保以管理员身份运行命令提示符或 PowerShell。

### 问题：服务无法启动

**解决方案**：
1. 检查配置文件路径是否正确
2. 检查数据库连接配置
3. 查看 Windows 事件查看器中的错误日志
4. 确保所需的端口未被占用

### 问题：卸载后服务仍在列表中

**解决方案**：等待几秒钟后刷新服务列表，或重启计算机。

## 命令参考

| 命令 | 描述 | 示例 |
|------|------|------|
| `--install` | 安装服务 | `.\dm-rust.exe --install` |
| `--uninstall` | 卸载服务 | `.\dm-rust.exe --uninstall` |
| `-s start` | 启动服务 | `.\dm-rust.exe -s start` |
| `-s stop` | 停止服务 | `.\dm-rust.exe -s stop` |
| `-s restart` | 重启服务 | `.\dm-rust.exe -s restart` |
| `-c <文件>` | 指定配置文件 | `.\dm-rust.exe -c custom.json` |
| `-l <级别>` | 设置日志级别 | `.\dm-rust.exe -l debug` |

## 最佳实践

1. **测试配置**：在安装服务前，先以普通模式运行程序测试配置是否正确：
   ```powershell
   .\dm-rust.exe -c config.json
   ```

2. **备份配置**：安装服务前备份配置文件。

3. **监控服务**：定期检查服务状态，可以设置监控脚本。

4. **更新程序**：更新程序时需要先停止服务，替换可执行文件后再启动服务。

## 示例：完整的安装流程

```powershell
# 1. 以管理员身份打开 PowerShell
# 2. 导航到程序目录
cd C:\path\to\dm-rust

# 3. 测试配置
.\dm-rust.exe -c config.json

# 4. 安装服务
.\dm-rust.exe --install

# 5. 启动服务
.\dm-rust.exe -s start

# 6. 验证服务状态
sc query DmRustService
```

## 注意事项

- 所有服务管理操作都需要**管理员权限**
- 配置文件路径建议使用绝对路径
- 服务运行时使用的是 LocalSystem 账户，确保该账户有权限访问所需资源
- 修改配置后需要重启服务才能生效
