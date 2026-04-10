use crate::config::AppState;
use crate::log_important;
use tauri::{AppHandle, Manager, WindowEvent};

/// 保存当前窗口位置到配置
fn save_window_position(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if let Ok(position) = window.outer_position() {
            let state = app_handle.state::<AppState>();
            let x = position.x;
            let y = position.y;
            if crate::constants::validation::is_valid_window_position(x, y) {
                if let Ok(mut config) = state.config.lock() {
                    config.ui_config.window_config.position_x = Some(x);
                    config.ui_config.window_config.position_y = Some(y);
                };
            }
        }
    }
}

/// 设置窗口事件监听器
pub fn setup_window_event_listeners(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let app_handle_clone = app_handle.clone();

        window.on_window_event(move |event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    // 阻止默认的关闭行为
                    api.prevent_close();

                    // 关闭前保存窗口位置
                    save_window_position(&app_handle_clone);

                    let app_handle = app_handle_clone.clone();

                    // 异步处理退出请求
                    tauri::async_runtime::spawn(async move {
                        let state = app_handle.state::<AppState>();

                        log_important!(info, "🖱️ 窗口关闭按钮被点击");

                        // 保存配置（含位置信息）
                        if let Err(e) = crate::config::save_config(&state, &app_handle).await {
                            log::warn!("关闭时保存配置失败: {}", e);
                        }

                        // 窗口关闭按钮点击应该直接退出，不需要双重确认
                        match crate::ui::exit::handle_system_exit_request(
                            state,
                            &app_handle,
                            true, // 手动点击关闭按钮
                        )
                        .await
                        {
                            Ok(exited) => {
                                if !exited {
                                    log_important!(info, "退出被阻止，等待二次确认");
                                } else {
                                    log_important!(info, "应用已退出");
                                }
                            }
                            Err(e) => {
                                log_important!(error, "处理退出请求失败: {}", e);
                            }
                        }
                    });
                }
                _ => {}
            }
        });
    }
}
