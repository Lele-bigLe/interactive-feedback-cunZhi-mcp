// 循（Xun）MCP 服务器入口点
// 工作流感知 MCP Server

use cunzhi::{workflow_mcp::run_workflow_server, utils::auto_init_logger, log_important};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自动初始化日志系统
    auto_init_logger()?;

    log_important!(info, "启动 循(Xun) 工作流 MCP 服务器");
    run_workflow_server().await
}
