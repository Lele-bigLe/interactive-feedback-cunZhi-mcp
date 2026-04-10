use crate::app::cli::is_daemon_mode;
use crate::config::{load_config_and_apply_window_settings, AppState};
use crate::log_important;
use crate::mcp::ipc::server::{cleanup_daemon, start_ipc_server};
use crate::ui::exit_handler::setup_exit_handlers;
use crate::ui::{initialize_audio_asset_manager, setup_window_event_listeners};
use tauri::{AppHandle, Manager};

/// 应用设置和初始化
pub async fn setup_application(app_handle: &AppHandle) -> Result<(), String> {
    let state = app_handle.state::<AppState>();

    // 加载配置并应用窗口设置
    if let Err(e) = load_config_and_apply_window_settings(&state, app_handle).await {
        log_important!(warn, "加载配置失败: {}", e);
    }

    // 守护进程模式：隐藏主窗口，启动 IPC 服务端
    if is_daemon_mode() {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.hide();
        }

        let app_for_ipc = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = start_ipc_server(app_for_ipc).await {
                log_important!(error, "IPC 服务端异常退出: {}", e);
            }
        });

        // 注册退出时清理
        if let Some(window) = app_handle.get_webview_window("main") {
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Destroyed = event {
                    cleanup_daemon();
                }
            });
        }

        log_important!(info, "守护进程模式初始化完成");
        return Ok(());
    }

    // 非守护进程模式：正常初始化
    // 初始化音频资源管理器
    if let Err(e) = initialize_audio_asset_manager(app_handle) {
        log_important!(warn, "初始化音频资源管理器失败: {}", e);
    }

    // 设置窗口事件监听器
    setup_window_event_listeners(app_handle);

    // 设置退出处理器
    if let Err(e) = setup_exit_handlers(app_handle) {
        log_important!(warn, "设置退出处理器失败: {}", e);
    }

    Ok(())
}
