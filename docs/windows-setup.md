# Windows 环境配置指南

本文档介绍在 Windows 系统上开发万遥客户端 (Wanyao Client) 所需的环境配置步骤。

## 目录

1. [Rust 安装](#1-rust-安装)
2. [Visual Studio Build Tools 安装](#2-visual-studio-build-tools-安装)
3. [Node.js 安装](#3-nodejs-安装)
4. [WebView2 运行时说明](#4-webview2-运行时说明)
5. [常见问题排查](#5-常见问题排查)

---

## 1. Rust 安装

Tauri 应用需要 Rust 编译环境。

### 1.1 下载 rustup

访问 Rust 官网下载页面：https://rustup.rs/

或直接下载安装程序：https://win.rustup.rs/x86_64

### 1.2 运行安装程序

```powershell
# 方式一：运行下载的 rustup-init.exe
# 方式二：使用命令行安装
winget install Rustlang.Rustup
```

### 1.3 验证安装

```powershell
# 查看 Rust 版本
rustc --version

# 查看 Cargo 版本
cargo --version

# 查看 rustup 版本
rustup --version
```

### 1.4 配置镜像源（可选，国内用户推荐）

创建或编辑 `%USERPROFILE%\.cargo\config` 文件：

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
```

---

## 2. Visual Studio Build Tools 安装

Rust 在 Windows 上编译需要 MSVC (Microsoft Visual C++) 构建工具。

### 2.1 下载 Visual Studio Build Tools

访问：https://visualstudio.microsoft.com/visual-cpp-build-tools/

或直接下载：https://aka.ms/vs/17/release/vs_BuildTools.exe

### 2.2 安装必要组件

运行安装程序后，选择以下工作负载：

- **"使用 C++ 的桌面开发" (Desktop development with C++)**

确保包含以下组件：
- MSVC v143 - VS 2022 C++ x64/x86 生成工具
- Windows 10 SDK 或 Windows 11 SDK

### 2.3 验证安装

```powershell
# 检查 MSVC 编译器
where cl.exe

# 或查看已安装的 Visual Studio
"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe" -latest
```

---

## 3. Node.js 安装

前端开发需要 Node.js 环境。

### 3.1 下载 Node.js

访问官网：https://nodejs.org/

推荐下载 **LTS (长期支持版)** 版本。

### 3.2 使用包管理器安装（可选）

```powershell
# 使用 winget 安装
winget install OpenJS.NodeJS.LTS

# 或使用 Chocolatey 安装
choco install nodejs-lts
```

### 3.3 验证安装

```powershell
# 查看 Node.js 版本
node --version

# 查看 npm 版本
npm --version
```

### 3.4 配置 npm 镜像源（可选，国内用户推荐）

```powershell
# 设置淘宝镜像
npm config set registry https://registry.npmmirror.com

# 验证配置
npm config get registry

# 恢复官方源
npm config set registry https://registry.npmjs.org
```

---

## 4. WebView2 运行时说明

### 4.1 WebView2 简介

WebView2 是 Microsoft Edge 浏览器的嵌入式组件，Tauri 应用使用它来渲染前端界面。

### 4.2 系统要求

- Windows 10 (版本 1809 或更高)
- Windows 11

### 4.3 检查是否已安装

```powershell
# 查看注册表
Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue
```

### 4.4 下载安装

如果系统未安装 WebView2，可从以下地址下载：

- **Evergreen 引导程序**（推荐）：https://go.microsoft.com/fwlink/p/?LinkId=2124703
- **Evergreen 独立安装程序**：https://go.microsoft.com/fwlink/p/?LinkId=2124701

```powershell
# 使用 winget 安装
winget install Microsoft.EdgeWebView2Runtime
```

### 4.5 开发说明

- Windows 10 1809+ 和 Windows 11 通常已预装 WebView2
- 开发时如果遇到 WebView2 相关错误，请确保运行时已正确安装
- 生产环境打包时，可选择将 WebView2 运行时打包进安装程序

---

## 5. 常见问题排查

### 5.1 Rust 编译错误

**问题：`linker 'link.exe' not found`**

```
error: linker `link.exe` not found
```

**解决方案：**
1. 确认已安装 Visual Studio Build Tools
2. 重启终端或重新打开命令行窗口
3. 如果问题仍存在，重启电脑

---

**问题：`cargo build` 失败**

**解决方案：**
```powershell
# 更新 Rust 工具链
rustup update stable

# 清理并重新构建
cargo clean
cargo build
```

---

### 5.2 Node.js 相关问题

**问题：`npm install` 失败或速度慢**

**解决方案：**
```powershell
# 清理 npm 缓存
npm cache clean --force

# 删除 node_modules 重新安装
Remove-Item -Recurse -Force node_modules
Remove-Item package-lock.json
npm install
```

---

**问题：Node 版本不兼容**

**解决方案：**
```powershell
# 检查项目要求的 Node 版本
# 查看 package.json 中的 engines 字段

# 使用 nvm-windows 管理 Node 版本
# 下载地址：https://github.com/coreybutler/nvm-windows/releases
```

---

### 5.3 Tauri 构建问题

**问题：`cargo tauri build` 失败**

**解决方案：**
```powershell
# 确认 Rust 目标平台
rustup target list --installed

# 添加 Windows 目标
rustup target add x86_64-pc-windows-msvc

# 检查 Tauri CLI
cargo install tauri-cli --version "^2"
```

---

**问题：WebView2 相关错误**

**解决方案：**
1. 确认 WebView2 运行时已安装
2. 尝试重新安装 WebView2
3. 检查 Windows 更新是否完整

---

### 5.4 权限问题

**问题：脚本执行策略限制**

```
无法加载文件，因为在此系统上禁止运行脚本
```

**解决方案：**
```powershell
# 以管理员身份运行 PowerShell，执行：
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

### 5.5 环境变量问题

**问题：命令找不到**

```
'cargo' 不是内部或外部命令
'node' 不是内部或外部命令
```

**解决方案：**
```powershell
# 检查 PATH 环境变量
$env:PATH -split ';'

# 手动添加路径（临时）
$env:PATH += ";C:\Users\你的用户名\.cargo\bin"

# 永久添加（需要管理员权限）
[Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";C:\Users\你的用户名\.cargo\bin", "User")
```

---

## 快速验证清单

完成所有安装后，运行以下命令验证环境：

```powershell
# 验证 Rust
rustc --version
cargo --version

# 验证 Node.js
node --version
npm --version

# 验证 MSVC（应该输出 cl.exe 路径）
where cl.exe

# 验证 WebView2
Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue | Select-Object pv
```

---

## 参考链接

- [Rust 官网](https://www.rust-lang.org/)
- [Rustup 文档](https://rust-lang.github.io/rustup/)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [Node.js 官网](https://nodejs.org/)
- [WebView2 文档](https://docs.microsoft.com/en-us/microsoft-edge/webview2/)
- [Tauri 官网](https://tauri.app/)
