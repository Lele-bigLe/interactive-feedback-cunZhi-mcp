use anyhow::Result;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use super::types::{clear_daemon_state, write_daemon_state, IpcRequest, IpcResponse};
use crate::config::AppState;
use crate::log_important;

/// 启动 IPC 服务端（在守护进程模式下运行）
///
/// 绑定随机端口，写入状态文件，监听 TCP 连接
pub async fn start_ipc_server(app_handle: AppHandle) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    write_daemon_state(port)?;
    log_important!(info, "IPC 服务端启动，监听 127.0.0.1:{}", port);

    // 注册退出时清理状态文件
    let app_handle_cleanup = app_handle.clone();
    tokio::spawn(async move {
        // 等待应用退出信号
        let _ = app_handle_cleanup.listen("tauri://close-requested", |_| {});
    });

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                log_important!(info, "IPC 收到连接: {}", addr);
                let app = app_handle.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_ipc_connection(stream, app).await {
                        log_important!(warn, "IPC 连接处理失败: {}", e);
                    }
                });
            }
            Err(e) => {
                log_important!(warn, "IPC accept 失败: {}", e);
            }
        }
    }
}

/// 处理单个 IPC 连接
async fn handle_ipc_connection(stream: tokio::net::TcpStream, app: AppHandle) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    buf_reader.read_line(&mut line).await?;
    let line = line.trim();

    if line.is_empty() {
        return Ok(());
    }

    let request: IpcRequest =
        serde_json::from_str(line).map_err(|e| anyhow::anyhow!("解析 IPC 请求失败: {}", e))?;

    let response = match request {
        IpcRequest::Ping => IpcResponse::Pong,

        IpcRequest::Shutdown => {
            log_important!(info, "收到 IPC 关闭请求");
            let resp = IpcResponse::ShutdownAck;
            let resp_json = serde_json::to_string(&resp)?;
            writer.write_all(resp_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;

            // 清理状态文件并退出
            let _ = clear_daemon_state();
            app.exit(0);
            return Ok(());
        }

        IpcRequest::Popup { request } => handle_popup_request(&app, request).await,
    };

    let resp_json = serde_json::to_string(&response)?;
    writer.write_all(resp_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}

/// 处理弹窗请求：通过 Tauri 事件触发前端弹窗，等待 response_channel 返回
async fn handle_popup_request(
    app: &AppHandle,
    request: crate::mcp::types::PopupRequest,
) -> IpcResponse {
    // 创建 oneshot channel 用于接收前端响应
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();

    // 将 sender 放入 AppState
    {
        let state = app.state::<AppState>();
        let mut channel = match state.response_channel.lock() {
            Ok(ch) => ch,
            Err(e) => {
                return IpcResponse::Error {
                    message: format!("获取响应通道失败: {}", e),
                };
            }
        };
        *channel = Some(tx);
    }

    // 显示窗口并发送弹窗请求事件
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }

    // 将 PopupRequest 转为 JSON Value 以匹配前端期望的格式
    let request_value = match serde_json::to_value(&request) {
        Ok(v) => v,
        Err(e) => {
            return IpcResponse::Error {
                message: format!("序列化请求失败: {}", e),
            };
        }
    };

    if let Err(e) = app.emit("mcp-request", request_value) {
        return IpcResponse::Error {
            message: format!("发送弹窗事件失败: {}", e),
        };
    }

    // 等待前端响应（通过 response_channel）
    match rx.await {
        Ok(response) => {
            IpcResponse::PopupResponse { response }
        }
        Err(_) => {
            IpcResponse::Error {
                message: "等待用户响应超时或通道关闭".to_string(),
            }
        }
    }
}

/// 清理守护进程（在应用退出时调用）
pub fn cleanup_daemon() {
    if let Err(e) = clear_daemon_state() {
        log_important!(warn, "清理守护进程状态失败: {}", e);
    }
}
