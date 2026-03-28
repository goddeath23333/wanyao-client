# WSL2 开发环境配置指南

本文档介绍如何在 WSL2 中配置 wanyao-client 项目的开发环境。

---

## 1. WSL2 安装步骤

### 1.1 启用 WSL 功能

以管理员身份打开 PowerShell，执行以下命令：

```powershell
# 启用 WSL
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart

# 启用虚拟机平台
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart
```

### 1.2 重启计算机

```powershell
Restart-Computer
```

### 1.3 下载并安装 WSL2 Linux 内核更新包

从微软官方下载并安装：
https://aka.ms/wsl2kernel

### 1.4 设置 WSL2 为默认版本

```powershell
wsl --set-default-version 2
```

### 1.5 安装 Linux 发行版

```powershell
# 查看可用发行版
wsl --list --online

# 安装 Ubuntu（推荐）
wsl --install -d Ubuntu

# 或安装其他发行版
wsl --install -d Ubuntu-22.04
```

### 1.6 初始化 Linux 发行版

首次启动会要求设置用户名和密码，按提示完成配置。

---

## 2. Node.js 安装（使用 NodeSource）

### 2.1 更新系统包

```bash
sudo apt update && sudo apt upgrade -y
```

### 2.2 安装 curl

```bash
sudo apt install -y curl
```

### 2.3 添加 NodeSource 仓库

```bash
# 安装 Node.js 20.x LTS（推荐）
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -

# 或安装 Node.js 22.x
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -

# 或安装 Node.js 18.x
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
```

### 2.4 安装 Node.js

```bash
sudo apt install -y nodejs
```

### 2.5 验证安装

```bash
node --version
npm --version
```

### 2.6 配置 npm 全局包目录（可选）

```bash
# 创建全局包目录
mkdir -p ~/.npm-global

# 配置 npm 使用新目录
npm config set prefix '~/.npm-global'

# 添加到 PATH
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

---

## 3. Git 配置

### 3.1 安装 Git

```bash
sudo apt install -y git
```

### 3.2 配置用户信息

```bash
git config --global user.name "你的用户名"
git config --global user.email "你的邮箱@example.com"
```

### 3.3 配置换行符处理

```bash
# Windows 环境，建议使用 CRLF
git config --global core.autocrlf true

# 或保持原样（跨平台协作推荐）
git config --global core.autocrlf input
```

### 3.4 配置 SSH 密钥（可选）

```bash
# 生成 SSH 密钥
ssh-keygen -t ed25519 -C "你的邮箱@example.com"

# 查看公钥
cat ~/.ssh/id_ed25519.pub

# 将公钥添加到 GitHub/GitLab
```

### 3.5 配置 Git 凭据管理器

```bash
# 配置使用 Windows 凭据管理器
git config --global credential.helper "/mnt/c/Program\ Files/Git/mingw64/bin/git-credential-manager.exe"
```

---

## 4. 文件系统互通说明

### 4.1 从 WSL2 访问 Windows 文件

Windows 磁盘挂载在 `/mnt/` 目录下：

```bash
# 访问 H 盘
cd /mnt/h/wanyao/wanyao-client

# 访问 C 盘
cd /mnt/c/Users/你的用户名

# 查看所有挂载的磁盘
ls /mnt/
```

### 4.2 从 Windows 访问 WSL2 文件

WSL2 文件系统可通过以下方式访问：

```
# 在文件资源管理器地址栏输入
\\wsl$\Ubuntu\home\你的用户名

# 或在 PowerShell 中
explorer.exe .
```

### 4.3 文件系统性能建议

| 操作位置 | 性能 | 说明 |
|---------|------|------|
| WSL2 内部 (`~/projects/`) | 快 | 推荐用于 git clone、npm install |
| Windows 磁盘 (`/mnt/h/`) | 较慢 | 适合编辑文件，不适合大量 I/O 操作 |

### 4.4 推荐的项目位置

```bash
# 方案一：放在 WSL2 内部（性能最佳）
mkdir -p ~/projects/wanyao-client

# 方案二：放在 Windows 磁盘（方便 Windows 编辑器访问）
cd /mnt/h/wanyao/wanyao-client
```

### 4.5 符号链接处理

```bash
# WSL2 中创建指向 Windows 文件的符号链接
ln -s /mnt/h/wanyao/wanyao-client ~/wanyao-client

# 这样可以快速切换
cd ~/wanyao-client
```

---

## 5. 开发工作流建议

### 5.1 推荐工作流

```bash
# 1. 进入项目目录
cd /mnt/h/wanyao/wanyao-client

# 2. 安装依赖
npm install

# 3. 启动开发服务器
node server.js

# 4. 在 Windows 浏览器中访问
# http://localhost:端口号
```

### 5.2 使用 Windows 编辑器

```bash
# 用 VS Code 打开项目
code .

# 或指定路径
code /mnt/h/wanyao/wanyao-client
```

### 5.3 常用命令别名

添加到 `~/.bashrc`：

```bash
# 项目快捷跳转
alias wanyao='cd /mnt/h/wanyao/wanyao-client'

# 开发命令
alias dev='node server.js'
alias install='npm install'
alias build='npm run build'

# Git 快捷命令
alias gs='git status'
alias ga='git add .'
alias gc='git commit -m'
alias gp='git push'
alias gl='git log --oneline -10'
```

应用配置：

```bash
source ~/.bashrc
```

### 5.4 端口转发说明

WSL2 使用虚拟网络，但端口会自动转发到 Windows：

```bash
# 在 WSL2 中启动服务
node server.js  # 假设监听 3000 端口

# 在 Windows 中可以直接访问
# http://localhost:3000
```

### 5.5 常见问题解决

#### 问题：npm install 速度慢

```bash
# 使用国内镜像
npm config set registry https://registry.npmmirror.com

# 或使用 nrm 管理镜像源
npm install -g nrm
nrm use taobao
```

#### 问题：文件权限问题

```bash
# 重置文件权限
sudo chmod -R 755 /mnt/h/wanyao/wanyao-client
```

#### 问题：WSL2 内存占用过高

创建 `C:\Users\你的用户名\.wslconfig` 文件：

```ini
[wsl2]
memory=4GB
processors=2
swap=2GB
```

重启 WSL2：

```powershell
wsl --shutdown
```

---

## 附录：常用命令速查

| 命令 | 说明 |
|------|------|
| `wsl --list --verbose` | 查看已安装的发行版 |
| `wsl --shutdown` | 关闭所有 WSL 实例 |
| `wsl -d Ubuntu` | 启动指定发行版 |
| `wsl --set-version Ubuntu 2` | 将发行版转换为 WSL2 |
| `explorer.exe .` | 用 Windows 资源管理器打开当前目录 |
| `code .` | 用 VS Code 打开当前目录 |
| `cmd.exe /c command` | 在 WSL 中执行 Windows 命令 |
| `powershell.exe -c command` | 在 WSL 中执行 PowerShell 命令 |

---

## 参考链接

- [WSL 官方文档](https://docs.microsoft.com/zh-cn/windows/wsl/)
- [NodeSource 官方仓库](https://github.com/nodesource/distributions)
- [Git 官方文档](https://git-scm.com/doc)
