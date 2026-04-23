# 万遥客户端 (Wanyao Client)

一个现代化的桌面端一体化开发工具，基于 Tauri 2.0 构建。

## 项目简介

万遥客户端是一个开源的桌面应用程序，集成了系统监控、串口调试、网络调试、固件烧录、数据可视化和自动化测试等功能，旨在为嵌入式开发者提供一站式的开发工具链。

## 功能特性

### 现有模块

| 模块 | 功能 | 状态 |
|------|------|------|
| 系统监控 | CPU/GPU/内存实时监控，支持 NVIDIA GPU 使用率检测 | ✅ 已实现 |
| 串口助手 | 串口通信调试，支持数据收发、波特率设置 | ✅ 已实现 |
| 网络助手 | TCP/UDP 网络调试，支持客户端和服务器模式 | ✅ 已实现 |
| 固件烧录 | MCU 固件烧录工具，支持 STC/STM32/ESP32 | ✅ 已实现 |
| 数据可视化 | 实时波形显示，支持多通道数据展示和导出 | ✅ 已实现 |
| 自动化测试 | 可编程测试框架，支持脚本编写和报告生成 | ✅ 已实现 |
| Python 嵌入式 | 执行 Python 代码和脚本 | ⚠️ 可选依赖（需要 Python 环境） |

### 功能详细介绍

#### 📊 仪表盘
- 实时显示 CPU、内存、GPU 使用率和趋势图
- GPU 检测支持 NVIDIA（通过 nvidia-smi）、AMD 和 Intel 显卡
- 功能模块快速导航卡片

#### 🔌 串口助手
- 自动扫描可用串口
- 支持自定义波特率（9600~921600）、数据位、停止位、校验位、流控制
- 支持十六进制/文本收发模式
- 支持时间戳显示
- 支持快捷发送（AT 命令等）

#### 🌐 网络助手
- 支持 TCP 客户端、TCP 服务器、UDP 三种模式
- 支持十六进制/文本收发模式
- 支持时间戳显示
- 支持快捷发送（AT、HTTP GET 等）

#### 💾 固件烧录
- 支持 STC、STM32、ESP32 等芯片类型
- 支持芯片自动检测
- 支持烧录、验证、擦除操作
- 实时输出烧录日志

#### 📈 数据可视化
- 支持自定义数据通道（名称、颜色）
- 支持手动输入和实时模拟数据
- 支持导出为 CSV/JSON 格式
- 基于 Chart.js 的实时波形图

#### 🧪 自动化测试
- 支持添加/编辑/删除测试用例
- 支持批量执行和单个执行
- 支持导出测试报告（文本/JSON/CSV）
- 可加载外部 JSON 测试脚本

#### ⚙️ 设置
- 深色/浅色/跟随系统主题切换
- Python 环境检测（需要 Python 3.x）
- 版本信息查看

### 技术特性

- 🖥️ **跨平台**: 支持 Windows、Linux、macOS
- 🌐 **网页优先开发**: 先在浏览器预览，再编译 Tauri
- 🔄 **CI/CD 集成**: GitHub Actions 自动构建多平台安装包
- 🎨 **现代化 UI**: 深色/浅色主题切换、响应式设计
- 🔌 **Web 模拟模式**: 串口/网络模块在浏览器中可模拟运行
- 🐛 **内置调试面板**: 页面底部调试日志，捕获全局错误
- 📜 **缩放支持**: 页面缩放时自动显示滚动条

## 快速开始

### 环境要求

| 工具 | 版本要求 | 说明 |
|------|----------|------|
| Node.js | >= 18.x | 必需，用于前端开发 |
| Rust | >= 1.70 | 必需，用于 Tauri 后端编译 |
| Visual Studio Build Tools | - | Windows 必需 |
| Python | >= 3.x | 可选，Python 嵌入式功能需要 |

> ⚠️ **Python 环境说明**: Python 嵌入式功能（在设置页面中）需要系统安装 Python 3.x。如果未安装 Python，相关功能将不可用，但不影响其他功能正常使用。

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
# 网页预览模式（推荐先使用此模式开发）
node server.js

# Tauri 开发模式
npm run dev
```

### 构建

```bash
# 生产构建
npm run build
```

## 项目结构

```
wanyao-client/
├── src/                    # 前端源文件
│   ├── index.html          # 主页面
│   ├── css/                # 样式目录
│   │   ├── common.css      # 公共样式
│   │   ├── theme.css       # 主题变量（深色/浅色）
│   │   └── pages.css       # 页面专属样式
│   └── js/                 # 脚本目录
│       ├── common.js       # 公共函数（导航、主题、工具）
│       ├── system-monitor.js # 系统监控模块
│       ├── python-module.js  # Python 模块
│       ├── serial.js       # 串口助手模块
│       ├── network.js      # 网络助手模块
│       ├── flasher.js      # 固件烧录模块
│       ├── visualization.js # 数据可视化模块
│       └── tester.js       # 自动化测试模块
├── docs/                   # 文档目录
│   ├── wsl2-setup.md       # WSL2 环境配置
│   └── windows-setup.md    # Windows 环境配置
├── .github/workflows/      # CI/CD 配置
│   └── build.yml           # 构建工作流
├── src-tauri/              # Tauri 后端
│   ├── src/
│   │   ├── main.rs         # 入口
│   │   ├── lib.rs          # 模块注册入口
│   │   └── modules/        # 功能模块目录
│   │       ├── mod.rs      # 模块导出
│   │       ├── system.rs   # 系统监控模块
│   │       ├── python.rs   # Python 嵌入式模块
│   │       ├── serial.rs   # 串口助手模块
│   │       ├── network.rs  # 网络助手模块
│   │       ├── flasher.rs  # 固件烧录模块
│   │       ├── visualization.rs # 数据可视化模块
│   │       └── tester.rs   # 自动化测试模块
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── scripts/                # 构建脚本
│   └── copy-assets.js      # 前端资源复制
├── server.js               # 开发服务器（端口 3001）
└── package.json            # 项目配置
```

## 开发环境配置

### WSL2 开发环境

详见 [WSL2 环境配置指南](docs/wsl2-setup.md)

### Windows 宿主构建环境

详见 [Windows 环境配置指南](docs/windows-setup.md)

### 交叉工作流

本项目支持 WSL2 + Windows 宿主交叉开发：

1. **WSL2**: 代码编辑、Git 操作、npm 操作
2. **Windows 宿主**: Tauri 构建、运行调试
3. **GitHub Actions**: CI/CD 自动构建验证

## CI/CD

项目配置了 GitHub Actions 自动构建：

- **触发条件**: 推送 `v*` 标签或手动触发
- **构建平台**: Windows (.msi/.exe)、Linux (.deb/.AppImage)、macOS (.dmg，双架构)
- **自动发布**: 创建 v 开头标签时自动发布 GitHub Release

## 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. Fork 本项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 开发原则

1. **网页优先**: 先在浏览器测试，再编译 Tauri
2. **保留现有功能**: 新功能不能破坏现有功能
3. **增量开发**: 在现有基础上添加新功能
4. **向后兼容**: 保持 API 兼容性

## 开源协议

本项目采用 MIT 协议 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 联系方式

- 项目主页: [GitHub Repository](https://github.com/goddeath23333/wanyao-client)
- 问题反馈: [Issues](https://github.com/goddeath23333/wanyao-client/issues)

## 更新日志

### v2.1.0 (2025-04-23)
- 移除自定义标题栏，使用原生窗口边框
- 改进 GPU 检测：使用 PowerShell Get-CimInstance 替代已弃用的 wmic
- 新增 NVIDIA GPU 使用率检测（通过 nvidia-smi）
- 修复页面缩放时无滚动条的问题
- 改进 Python 环境检测提示
- 新增功能引导介绍
- 固件烧录、数据可视化、自动化测试模块已实现
- 更新 README 文档

### v1.1.1 (2025-04-19)
- 新增网络助手功能（部分实现）
  - 支持 TCP 客户端/服务器模式
  - 支持 UDP 连接创建
  - 支持十六进制/文本收发模式
  - 支持快捷发送功能
  - 前端 UI 完整，Web 模拟模式可用
  - UDP 发送逻辑待完善
- 前端项目结构调整至 `src/` 目录
- 新增 `pages.css` 页面专属样式
- 新增 `scripts/copy-assets.js` 构建脚本

### v1.1.0 (2025-03-28)
- 新增串口助手功能
  - 支持串口扫描和选择
  - 支持自定义波特率、数据位、停止位、校验位、流控制
  - 支持十六进制/文本收发模式
  - 支持时间戳显示
  - 支持快捷发送功能

### v1.0.0 (2025-03-28)
- 初始版本发布
- 系统监控功能
- 窗口控制功能
- CI/CD 多平台构建
- WSL2 + Windows 交叉开发环境
