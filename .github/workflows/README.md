# GitHub Actions 工作流说明

本项目包含三个 GitHub Actions 工作流，用于自动化构建、测试和发布。

## 工作流列表

### 1. CI 工作流 (`.github/workflows/ci.yml`)

**触发条件**:
- Push 到 `master`, `main`, `develop` 分支
- Pull Request 到 `master`, `main` 分支

**功能**:
- ✅ 代码检查 (cargo check)
- ✅ 单元测试 (cargo test) - Ubuntu, Windows, macOS
- ✅ 代码格式检查 (cargo fmt)
- ✅ Clippy 静态分析
- ✅ 构建测试 - 多平台

**用途**: 确保代码质量，每次提交都会自动运行测试。

---

### 2. Windows 构建工作流 (`.github/workflows/build-windows.yml`)

**触发条件**:
- Push 到 `master`, `main` 分支
- Push tag (格式: `v*`)
- Pull Request 到 `master`, `main` 分支
- 手动触发 (workflow_dispatch)

**功能**:
- ✅ Windows MSVC 构建 (原生)
- ✅ Windows GNU 构建 (交叉编译)
- ✅ 自动上传构建产物
- ✅ Tag 时自动创建 Release

**产物**:
- `dm-rust-windows-x64.zip` (MSVC 版本)
- `dm-rust-windows-x64-gnu.zip` (GNU 版本)

---

### 3. 多平台发布工作流 (`.github/workflows/release.yml`)

**触发条件**:
- Push tag (格式: `v*`)
- 手动触发 (workflow_dispatch)

**支持平台**:
- ✅ Windows (x64, MSVC)
- ✅ Linux (x64, GNU)
- ✅ macOS (x64, Intel)
- ✅ macOS (arm64, Apple Silicon)

**功能**:
- 多平台并行构建
- 自动创建 GitHub Release
- 生成 Release Notes
- 上传所有平台的构建产物

**产物**:
- `dm-rust-windows-x64-msvc.zip`
- `dm-rust-linux-x64.tar.gz`
- `dm-rust-macos-x64.tar.gz`
- `dm-rust-macos-arm64.tar.gz`

每个产物包含:
- 可执行文件
- 示例配置文件 (`config.example.json`, `config.mock.json`)
- 文档目录 (`doc/`)
- README 文件

---

## 使用方法

### 创建发布版本

1. **打标签**:
```bash
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0
```

2. **自动化流程**:
   - GitHub Actions 自动触发
   - 编译所有平台版本
   - 创建 GitHub Release
   - 上传所有构建产物

3. **下载产物**:
   - 访问项目的 [Releases 页面](../../releases)
   - 选择对应平台的文件下载

### 手动触发构建

1. 访问 [Actions 页面](../../actions)
2. 选择 "Build Windows Release" 或 "Multi-Platform Release Build"
3. 点击 "Run workflow"
4. 选择分支并点击 "Run workflow"

### 查看构建状态

- 访问 [Actions 页面](../../actions)
- 查看各个工作流的运行状态
- 点击具体的运行记录查看详细日志

---

## 本地测试构建

在提交之前，可以本地测试构建：

```bash
# 检查代码
cargo check --all-features

# 运行测试
cargo test --verbose

# 格式检查
cargo fmt --all -- --check

# Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 构建 Release 版本
cargo build --release

# 交叉编译到 Windows (在 Linux 上)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## 配置要求

### GitHub 仓库设置

1. **Secrets**: 不需要额外配置，使用默认的 `GITHUB_TOKEN`

2. **Actions 权限**:
   - 确保 Actions 有读写权限
   - Settings → Actions → General → Workflow permissions
   - 选择 "Read and write permissions"

3. **启用 Actions**:
   - 确保 GitHub Actions 已启用
   - Settings → Actions → General → Actions permissions

---

## 产物说明

### Windows 版本

- **MSVC 版本** (`windows-x64-msvc`):
  - 推荐使用
  - 依赖 Windows 系统库
  - 更好的性能和兼容性

- **GNU 版本** (`windows-x64-gnu`):
  - 使用 MinGW 编译
  - 可能需要额外的 DLL 文件

### Linux 版本

- 使用 GNU 工具链编译
- 适用于大多数 Linux 发行版
- 需要 GLIBC 2.31+

### macOS 版本

- **x64**: Intel 处理器
- **arm64**: Apple Silicon (M1/M2/M3)
- Universal Binary 可以考虑后续添加

---

## 故障排除

### 构建失败

1. 检查依赖是否正确安装
2. 查看 Actions 日志中的详细错误信息
3. 确保 `Cargo.lock` 文件已提交

### Release 创建失败

1. 确保 tag 格式正确 (以 `v` 开头)
2. 检查 Actions 权限设置
3. 确保没有同名的 Release 存在

### 缓存问题

如果遇到缓存相关问题，可以：
1. 在 Actions 页面手动清除缓存
2. 或者在代码中更新缓存 key 的版本号

---

## 自定义配置

### 修改构建目标

编辑 `.github/workflows/release.yml`:

```yaml
matrix:
  platform:
    - os: windows-latest
      target: x86_64-pc-windows-msvc
      name: windows-x64-msvc
```

### 修改产物内容

在工作流文件中修改 "Create release directory" 步骤，添加或删除要包含的文件。

### 修改 Release Notes

编辑 `.github/workflows/release.yml` 中的 "Create Release Notes" 步骤。

---

## 相关链接

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Rust 交叉编译指南](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo-release](https://github.com/crate-ci/cargo-release)

---

## 状态徽章

在项目 README 中添加状态徽章：

```markdown
![CI](https://github.com/YOUR_USERNAME/dm-rust/workflows/CI/badge.svg)
![Build Windows](https://github.com/YOUR_USERNAME/dm-rust/workflows/Build%20Windows%20Release/badge.svg)
![Release](https://github.com/YOUR_USERNAME/dm-rust/workflows/Multi-Platform%20Release%20Build/badge.svg)
```

替换 `YOUR_USERNAME` 为实际的 GitHub 用户名或组织名。
