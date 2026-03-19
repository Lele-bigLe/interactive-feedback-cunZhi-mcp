// 工作流感知 MCP Server（循 Xun-mcp）
// 独立的 MCP Server binary，提供工作流引导能力

pub mod server;

pub use server::run_workflow_server;
