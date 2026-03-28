# 万遥客户端 (Wanyao Client)

一个现代化的桌面端一体化开发工具，基于 Tauri 2.0 构建。

## 项目简介

万遥客户端是一个开源的桌面应用程序，集成了系统监控、串口调试、网络调试、固件烧录、数据可视化和自动化测试等功能，旨在为嵌入式开发者提供一站式的开发工具链。

## 功能特性

### 现有模块

| 模块 | 功能 | 状态 |
|------|------|------|
| 系统监控 | CPU/GPU/内存实时监控 | ✅ 已实现 |
| 窗口控制 | 自定义标题栏、最小化/最大化/关闭 | ✅ 已实现 |
| Python 嵌入式 | 执行 Python 代码和脚本 | ⚠️ 可选依赖 |
| 串口助手 | 串口通信调试 | ✅ 已实现 |
| 网络助手 | TCP/UDP 网络调试 | 🚧 规划中 |
| 固件烧录 | MCU 固件烧录工具 | 🚧 规划中 |
| 数据可视化 | 实时波形显示 | 🚧 规划中 |
| 自动化测试 | 可编程测试框架 | 🚧 规划中 |

### 技术特性

- 🖥️ **跨平台**: 支持 Windows、Linux、macOS
- 🌐 **网页优先开发**: 先在浏览器预览，再编译 Tauri
- 🔄 **CI/CD 集成**: GitHub Actions 自动构建多平台安装包
- 🎨 **现代化 UI**: 深色主题、响应式设计

## 快速开始

### 环境要求

| 工具 | 版本要求 |
|------|----------|
| Node.js | >= 18.x |
| Rust | >= 1.70 |
| Visual Studio Build Tools | (Windows) |

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
# 网页预览模式（推荐先使用此模式开发）
npm run dev

# Tauri 开发模式
npm run tauri:dev
```

### 构建

```bash
# 生产构建
npm run tauri:build

# 调试构建
npm run tauri:build:debug
```

## 项目结构

```
wanyao-client/
├── index.html              # 主页面
├── css/                    # 样式目录
│   ├── common.css         # 公共样式
│   └── theme.css          # 主题样式
├── js/                     # 脚本目录
│   ├── common.js          # 公共函数
│   ├── system-monitor.js  # 系统监控模块
│   ├── window-controls.js # 窗口控制模块
│   ├── python-module.js   # Python 模块
│   └── serial.js          # 串口模块
├── docs/                   # 文档目录
│   ├── wsl2-setup.md      # WSL2 环境配置
│   └── windows-setup.md   # Windows 环境配置
├── .github/workflows/      # CI/CD 配置
│   └── build.yml          # 构建工作流
├── src-tauri/              # Tauri 后端
│   ├── src/
│   │   ├── main.rs        # 入口
│   │   └── lib.rs         # 主库
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── server.js               # 开发服务器
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

- **触发条件**: push 到 main/dev 分支、PR、Release
- **构建平台**: Windows (.msi/.exe)、Linux (.deb/.AppImage)、macOS (.dmg)
- **自动发布**: 创建 v 开头标签时自动发布 Release

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
