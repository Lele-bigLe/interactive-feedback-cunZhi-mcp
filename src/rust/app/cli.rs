use crate::config::load_standalone_telegram_config;
use crate::telegram::handle_telegram_only_mcp_request;
use crate::mcp::ipc::client::send_shutdown_via_ipc;
use crate::log_important;
use crate::app::builder::run_tauri_app;
use anyhow::Result;

/// 全局标记：是否为守护进程模式
static DAEMON_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 检查当前是否为守护进程模式
pub fn is_daemon_mode() -> bool {
    DAEMON_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// 处理命令行参数
pub fn handle_cli_args() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        // 无参数：正常启动GUI
        1 => {
            run_tauri_app();
        }
        // 单参数：帮助、版本、守护进程、关闭守护进程
        2 => {
            match args[1].as_str() {
                "--help" | "-h" => print_help(),
                "--version" | "-v" => print_version(),
                "--daemon" => {
                    DAEMON_MODE.store(true, std::sync::atomic::Ordering::Relaxed);
                    log_important!(info, "以守护进程模式启动");
                    run_tauri_app();
                }
                "--shutdown" => {
                    handle_shutdown();
                }
                _ => {
                    eprintln!("未知参数: {}", args[1]);
                    print_help();
                    std::process::exit(1);
                }
            }
        }
        // 多参数：MCP请求模式
        _ => {
            if args[1] == "--mcp-request" && args.len() >= 3 {
                handle_mcp_request(&args[2])?;
            } else {
                eprintln!("无效的命令行参数");
                print_help();
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// 处理MCP请求
fn handle_mcp_request(request_file: &str) -> Result<()> {
    // 检查Telegram配置，决定是否启用纯Telegram模式
    match load_standalone_telegram_config() {
        Ok(telegram_config) => {
            if telegram_config.enabled && telegram_config.hide_frontend_popup {
                // 纯Telegram模式：不启动GUI，直接处理
                if let Err(e) = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(handle_telegram_only_mcp_request(request_file))
                {
                    log_important!(error, "处理Telegram请求失败: {}", e);
                    std::process::exit(1);
                }
            } else {
                // 正常模式：启动GUI处理弹窗
                run_tauri_app();
            }
        }
        Err(e) => {
            log_important!(warn, "加载Telegram配置失败: {}，使用默认GUI模式", e);
            // 配置加载失败时，使用默认行为（启动GUI）
            run_tauri_app();
        }
    }
    Ok(())
}

/// 处理关闭守护进程命令
fn handle_shutdown() {
    match send_shutdown_via_ipc() {
        Ok(()) => {
            println!("守护进程已关闭");
        }
        Err(e) => {
            eprintln!("关闭守护进程失败: {}", e);
            std::process::exit(1);
        }
    }
}

/// 显示帮助信息
fn print_help() {
    println!("寸止 - 智能代码审查工具");
    println!();
    println!("用法:");
    println!("  等一下                      启动设置界面");
    println!("  等一下 --daemon             以守护进程模式启动（后台运行，通过 IPC 接收弹窗请求）");
    println!("  等一下 --shutdown           关闭守护进程");
    println!("  等一下 --mcp-request <文件>  处理 MCP 请求");
    println!("  等一下 --help               显示此帮助信息");
    println!("  等一下 --version            显示版本信息");
}

/// 显示版本信息
fn print_version() {
    println!("寸止 v{}", env!("CARGO_PKG_VERSION"));
}
