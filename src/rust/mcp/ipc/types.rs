use serde::{Deserialize, Serialize};

use crate::mcp::types::PopupRequest;

/// IPC 请求消息
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcRequest {
    /// 弹窗请求
    #[serde(rename = "popup")]
    Popup { request: PopupRequest },
    /// 关闭守护进程
    #[serde(rename = "shutdown")]
    Shutdown,
    /// 心跳检测
    #[serde(rename = "ping")]
    Ping,
}

/// IPC 响应消息
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcResponse {
    /// 弹窗响应
    #[serde(rename = "popup_response")]
    PopupResponse { response: String },
    /// 错误
    #[serde(rename = "error")]
    Error { message: String },
    /// 心跳回复
    #[serde(rename = "pong")]
    Pong,
    /// 关闭确认
    #[serde(rename = "shutdown_ack")]
    ShutdownAck,
}

/// 守护进程状态文件内容
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonState {
    pub pid: u32,
    pub port: u16,
    pub started_at: String,
}

/// 获取守护进程状态文件路径
pub fn daemon_state_path() -> anyhow::Result<std::path::PathBuf> {
    let runtime_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
        .join("cunzhi")
        .join("runtime");
    std::fs::create_dir_all(&runtime_dir)?;
    Ok(runtime_dir.join("daemon.json"))
}

/// 写入守护进程状态
pub fn write_daemon_state(port: u16) -> anyhow::Result<()> {
    let state = DaemonState {
        pid: std::process::id(),
        port,
        started_at: chrono::Utc::now().to_rfc3339(),
    };
    let path = daemon_state_path()?;
    std::fs::write(&path, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

/// 读取守护进程状态
pub fn read_daemon_state() -> anyhow::Result<Option<DaemonState>> {
    let path = daemon_state_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let state: DaemonState = serde_json::from_str(&content)?;
    Ok(Some(state))
}

/// 清除守护进程状态
pub fn clear_daemon_state() -> anyhow::Result<()> {
    let path = daemon_state_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}
