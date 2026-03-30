# 寸止 MCP 工具安装指南

## 快速安装

### 方式一：下载预编译版本（推荐）

从 [Releases](https://github.com/Lele-bigLe/interactive-feedback-cunZhi-mcp/releases) 页面下载对应平台的预编译版本：

| 平台 | 文件 | 说明 |
|------|------|------|
| **Windows** | `cunzhi_*_x64-setup.exe` | 安装包（推荐，自动配置 PATH） |
| **Windows** | `cunzhi-cli-v*-windows-x86_64.zip` | 便携版 CLI |
| **Linux** | `cunzhi-cli-v*-linux-x86_64.tar.gz` | CLI 二进制 |
| **macOS (Intel)** | `cunzhi-cli-v*-macos-x86_64.tar.gz` | CLI 二进制 |
| **macOS (Apple Silicon)** | `cunzhi-cli-v*-macos-aarch64.tar.gz` | CLI 二进制 |

#### Windows 安装步骤：

1. 下载 `cunzhi_*_x64-setup.exe`
2. 双击运行安装包
3. 按提示完成安装（默认安装到 `%LOCALAPPDATA%\cunzhi`）

#### macOS / Linux 安装步骤：

```bash
# 下载并解压（以 Linux 为例）
tar -xzf cunzhi-cli-v*-linux-x86_64.tar.gz

# 将二进制文件复制到系统路径
sudo cp zhi 等一下 /usr/local/bin/
```

### 方式二：使用安装脚本

```bash
# 克隆仓库
git clone https://github.com/Lele-bigLe/interactive-feedback-cunZhi-mcp.git
cd interactive-feedback-cunZhi-mcp

# 运行安装脚本
chmod +x install.sh
./install.sh
```

## 验证安装

```bash
# 检查工具是否正确安装
zhi --help
等一下 --help
```

## MCP 客户端配置

将以下配置添加到您的 MCP 客户端配置文件中：

```json
{
  "mcpServers": {
    "cunzhi": {
      "command": "zhi"
    }
  }
}
```

> 如果未加入 PATH，请使用绝对路径：
> ```json
> { "command": "C:/Users/<你的用户名>/AppData/Local/cunzhi/zhi.exe" }
> ```

## 使用方法

### MCP 服务器模式
```bash
zhi  # 启动 MCP 服务器
```

### 弹窗界面模式
```bash
等一下                          # 启动设置界面
等一下 --mcp-request file       # MCP 弹窗模式
```

## 工具说明

- **zhi**: MCP 服务器，提供记忆管理和智能交互功能
- **等一下**: 弹窗界面，用于用户交互和设置

## 系统要求

- **Linux**: x86_64 架构
- **macOS**: 10.15+ (支持 Intel 和 Apple Silicon)
- **Windows**: Windows 10+ x86_64

## 故障排除

### 权限问题
```bash
# Linux/macOS
chmod +x zhi 等一下
```

### PATH 问题
确保安装目录已添加到 PATH 环境变量中。

### 依赖问题
两个 CLI 工具必须在同一目录下才能正常工作。

## 开发者安装

如果您想从源码构建：

```bash
# 安装依赖
cargo --version  # 需要 Rust 1.70+
pnpm --version   # 需要 pnpm

# 构建
pnpm install
pnpm build
cargo build --release

# 安装
cp target/release/zhi target/release/等一下 ~/.local/bin/
```

## 更新

### 使用预编译版本
重新下载最新版本并替换旧文件。

### 使用源码
```bash
git pull
./install.sh
```
