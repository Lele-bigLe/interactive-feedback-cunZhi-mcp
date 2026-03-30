# 寸止 🛑

> **AI 对话"早泄"终结者，让交流持续到底**

还在为 AI 助手总是提前结束对话而抓狂吗？明明还有很多要聊，它却说"还有什么需要帮助的吗？"**寸止** 专治这个毛病！

当 AI 想要"草草了事"时，寸止会及时弹出对话框，让你能够继续深入交流，直到真正解决问题为止。

## 🌟 核心特性

- 🛑 **智能拦截**：AI 想结束时自动弹出继续选项
- 🧠 **记忆管理**：按项目存储开发规范和偏好
- 🎨 **优雅交互**：Markdown 支持、多种输入方式
- ⚡ **即装即用**：3 秒安装，跨平台支持

## 📸 看看效果

### 🛑 智能拦截弹窗
![寸止弹窗演示](./screenshots/popup.png)

*当 AI 想要结束对话时，寸止智能弹窗及时出现，提供预定义选项快速选择，让交流持续深入*

### ⚙️ 设置管理界面
![寸止设置界面](./screenshots/settings.png)

*优雅的设置界面，支持记忆管理、功能开关、主题切换和智能提示词生成*

## 🚀 开始使用

### 方式一：下载安装包（推荐）

1. 访问 [Releases 页面](https://github.com/Lele-bigLe/interactive-feedback-cunZhi-mcp/releases)
2. 下载适合你系统的版本：

   | 平台 | 文件 | 说明 |
   |------|------|------|
   | 🪟 **Windows** | `cunzhi_*_x64-setup.exe` | 安装包（推荐） |
   | 🪟 **Windows** | `cunzhi-cli-v*-windows-x86_64.zip` | 便携版 CLI |
   | 🐧 **Linux** | `cunzhi-cli-v*-linux-x86_64.tar.gz` | CLI 二进制 |
   | 🍎 **macOS (Intel)** | `cunzhi-cli-v*-macos-x86_64.tar.gz` | CLI 二进制 |
   | 🍎 **macOS (Apple Silicon)** | `cunzhi-cli-v*-macos-aarch64.tar.gz` | CLI 二进制 |

3. **Windows**：运行 `.exe` 安装包，按提示完成安装
4. **macOS/Linux**：解压后将 `zhi` 和 `等一下` 添加到系统 PATH

### 方式二：从源码构建

```bash
git clone https://github.com/Lele-bigLe/interactive-feedback-cunZhi-mcp.git
cd interactive-feedback-cunZhi-mcp
pnpm install
pnpm tauri:build
```

## ⚡ 快速上手

### 第一步：配置 MCP 客户端

在你的 MCP 客户端（如 Claude Desktop）配置文件中添加：

```json
{
  "mcpServers": {
    "cunzhi": {
      "command": "zhi"
    }
  }
}
```

> 💡 如果 `zhi` 未加入 PATH，请使用绝对路径，例如：
> ```json
> { "command": "C:/Users/<你的用户名>/AppData/Local/cunzhi/zhi.exe" }
> ```

### 第二步：打开设置界面

```bash
# 运行设置命令
等一下
```

### 第三步：配置提示词

在设置界面的"参考提示词"标签页：
1. 查看自动生成的提示词
2. 点击复制按钮
3. 将提示词添加到你的 AI 助手中

### 第四步：开始使用

现在你的 AI 助手就拥有了智能拦截、记忆管理和弹窗交互功能！

> 💡 **小贴士**：你可以参考生成的提示词进行个性化修改，打造专属的 AI 交互体验。

## 🔧 工具说明

寸止提供了多个 MCP 工具来增强 AI 助手的能力：

| 工具 | 命令 | 功能 |
|------|------|------|
| **智能交互** | `zhi` | 弹窗收集用户反馈，支持 Markdown、图片上传 |
| **记忆管理** | `ji` | 按项目存储开发规范和偏好 |
| **代码搜索** | `sou` | 基于 ACE 的语义代码搜索 |

- 📖 [代码搜索工具详细使用说明](./ACEMCP.md)
- 📖 [MCP 配置说明](./MCP_CONFIG.md)

### 🙏 致谢

感谢以下开源项目及其贡献者：

- **[acemcp](https://github.com/qy527145/acemcp)** - 由 [@qy527145](https://github.com/qy527145) 开发，提供了强大的代码库语义搜索能力。本项目在保留原有功能的基础上，使用 Rust 重写了核心逻辑并集成到寸止的 MCP 工具生态中。

## 🤝 参与贡献

寸止是开源项目，我们欢迎所有形式的贡献！

### 🛠️ 本地开发
```bash
git clone https://github.com/Lele-bigLe/interactive-feedback-cunZhi-mcp.git
cd interactive-feedback-cunZhi-mcp
pnpm install
pnpm tauri:dev
```

## 📄 开源协议

MIT License - 自由使用，欢迎贡献！
