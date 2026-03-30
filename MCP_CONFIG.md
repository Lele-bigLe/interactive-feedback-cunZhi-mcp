# cunzhi MCP 配置说明

本文档说明如何把 `cunzhi` 接入支持 MCP 的客户端，并给出当前推荐的配置方式。

## 一、核心说明

`cunzhi` 目前包含两个主要可执行文件：

- `等一下.exe`：GUI 界面，用于展示弹窗和设置页面
- `zhi.exe`：MCP 服务端主命令

兼容说明：

- `寸止.exe` 仍可作为兼容别名存在
- **推荐配置始终使用 `zhi`**

## 二、Windows 安装后的默认位置

如果使用安装包安装，默认目录通常为：

```text
C:\Users\<你的用户名>\AppData\Local\cunzhi
```

常见文件：

```text
C:\Users\<你的用户名>\AppData\Local\cunzhi\等一下.exe
C:\Users\<你的用户名>\AppData\Local\cunzhi\zhi.exe
C:\Users\<你的用户名>\AppData\Local\cunzhi\uninstall.exe
```

## 三、MCP 客户端推荐配置

### 1. 推荐：使用绝对路径

```json
{
  "mcpServers": {
    "cunzhi": {
      "command": "C:/Users/<你的用户名>/AppData/Local/cunzhi/zhi.exe"
    }
  }
}
```

### 2. 如果已经加入 PATH

```json
{
  "mcpServers": {
    "cunzhi": {
      "command": "zhi"
    }
  }
}
```

### 3. 兼容旧名称（不推荐，仅兼容）

```json
{
  "mcpServers": {
    "cunzhi": {
      "command": "寸止"
    }
  }
}
```

> 建议统一改为 `zhi`，避免后续文档、打包和排障时命令名不一致。

## 四、当前功能说明

当前 `cunzhi` MCP 弹窗已支持：

- 项目级防重复发起（依赖 `project_path`）
- 倒计时显示在底部
- 暂停计时
- 继续计时
- 重新计时
- 超时无响应自动重新发起
- 在设置页中修改默认倒计时

## 五、超时设置入口

设置路径：

```text
MCP 工具 -> cunzhi -> 右侧齿轮
```

当前默认值：

- **10 分钟**

可配置范围：

- **1 - 60 分钟**

## 六、测试建议

如果要验证项目锁，请务必：

1. 通过 `zhi` 发起请求
2. 传入 `project_path`
3. 使用两个会话/两个窗口并发测试

示例参数：

```json
{
  "message": "项目锁测试",
  "predefined_options": ["保持不动"],
  "is_markdown": true,
  "project_path": "D:/sorftwer/cunzhi"
}
```

## 七、常见问题

### 1. 只双击 `等一下.exe` 可以测项目锁吗？

不可以。

因为：

- `等一下.exe` 只是 GUI
- 项目锁、防重复、超时自动重发在 `zhi` 服务端逻辑里

### 2. 安装后调 MCP 提示 `Transport closed`

通常说明：

- MCP 客户端还没重启
- 还在连旧的可执行文件路径
- 没有切到最新安装目录里的 `zhi.exe`

建议：

1. 检查 MCP 配置中的命令路径
2. 重启 MCP 客户端
3. 优先使用绝对路径验证

---

如需发布给其他人，建议优先分发：

- `cunzhi_*_x64-setup.exe`（NSIS 安装包，推荐）
- `cunzhi_*_x64_zh-CN.msi`（MSI 安装包）
- `cunzhi-cli-v*-windows-x86_64.zip`（便携版 CLI）

下载地址：[Releases 页面](https://github.com/Lele-bigLe/interactive-feedback-cunZhi-mcp/releases)

并在文档里统一使用 `zhi` 作为 MCP 命令名。
