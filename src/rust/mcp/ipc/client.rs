use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use super::types::{read_daemon_state, IpcRequest, IpcResponse};
use crate::mcp::types::PopupRequest;

/// 通过 IPC 发送弹窗请求到守护进程
///
/// 成功返回响应字符串，失败返回错误（调用方可回退到直接启动进程）
pub fn send_popup_via_ipc(request: &PopupRequest) -> Result<String> {
    // 读取守护进程状态
    let state = read_daemon_state()?.ok_or_else(|| anyhow::anyhow!("守护进程未运行"))?;

    // 检查守护进程是否存活
    if !is_process_alive(state.pid) {
        super::types::clear_daemon_state()?;
        anyhow::bail!("守护进程已退出（PID: {}）", state.pid);
    }

    // 在独立线程创建 runtime，避免 "runtime within runtime" panic
    let request = request.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(send_ipc_request(state.port, &request))
    })
    .join()
    .map_err(|_| anyhow::anyhow!("IPC 线程执行失败"))?
}

/// 通过 IPC 发送关闭命令到守护进程
pub fn send_shutdown_via_ipc() -> Result<()> {
    let state = read_daemon_state()?.ok_or_else(|| anyhow::anyhow!("守护进程未运行"))?;

    if !is_process_alive(state.pid) {
        super::types::clear_daemon_state()?;
        return Ok(());
    }

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", state.port)).await?;
            let request = IpcRequest::Shutdown;
            let request_json = serde_json::to_string(&request)?;

            stream.write_all(request_json.as_bytes()).await?;
            stream.write_all(b"\n").await?;
            stream.flush().await?;

            // 等待确认
            let mut buf_reader = BufReader::new(stream);
            let mut line = String::new();
            buf_reader.read_line(&mut line).await?;

            Ok::<(), anyhow::Error>(())
        })
    })
    .join()
    .map_err(|_| anyhow::anyhow!("IPC 线程执行失败"))?
}

/// 检查守护进程是否存活（ping）
pub fn is_daemon_alive() -> bool {
    let state = match read_daemon_state() {
        Ok(Some(s)) => s,
        _ => return false,
    };

    if !is_process_alive(state.pid) {
        let _ = super::types::clear_daemon_state();
        return false;
    }

    // 在独立线程 ping，避免嵌套 runtime
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return false,
        };

        rt.block_on(async {
            let result =
                tokio::time::timeout(std::time::Duration::from_secs(2), ping_daemon(state.port))
                    .await;
            matches!(result, Ok(Ok(())))
        })
    })
    .join()
    .unwrap_or(false)
}

/// 发送弹窗请求
async fn send_ipc_request(port: u16, request: &PopupRequest) -> Result<String> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    let (reader, mut writer) = stream.into_split();

    let ipc_request = IpcRequest::Popup {
        request: request.clone(),
    };
    let request_json = serde_json::to_string(&ipc_request)?;

    writer.write_all(request_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // 等待响应（无超时限制，用户交互时间不确定）
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?;

    let response: IpcResponse = serde_json::from_str(line.trim())?;

    match response {
        IpcResponse::PopupResponse { response } => Ok(response),
        IpcResponse::Error { message } => anyhow::bail!("守护进程错误: {}", message),
        _ => anyhow::bail!("收到意外的 IPC 响应"),
    }
}

/// 对守护进程发送 ping
async fn ping_daemon(port: u16) -> Result<()> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    let (reader, mut writer) = stream.into_split();

    let request = IpcRequest::Ping;
    let request_json = serde_json::to_string(&request)?;

    writer.write_all(request_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?;

    let response: IpcResponse = serde_json::from_str(line.trim())?;
    match response {
        IpcResponse::Pong => Ok(()),
        _ => anyhow::bail!("ping 响应异常"),
    }
}

/// 检查进程是否存活
fn is_process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }

    #[cfg(unix)]
    {
        // 通过 /proc/{pid} 检查进程是否存在（Linux），或 ps 命令（macOS）
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new(&format!("/proc/{}", pid)).exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            use std::process::Command;
            Command::new("ps")
                .args(["-p", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }
}
