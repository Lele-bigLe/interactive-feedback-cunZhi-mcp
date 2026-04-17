use anyhow::Result;
use rmcp::{
    model::*, service::RequestContext, transport::stdio, Error as McpError, RoleServer,
    ServerHandler, ServiceExt,
};

use super::tools::InteractionTool;
use super::types::ZhiRequest;
use crate::{log_debug, log_important};

#[derive(Clone, Default)]
pub struct ZhiServer;

impl ZhiServer {
    pub fn new() -> Self {
        Self
    }
}

impl ServerHandler for ZhiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Zhi-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some("Zhi 智能代码审查工具，支持交互式对话确认".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ServerInfo, McpError> {
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        use std::borrow::Cow;
        use std::sync::Arc;

        let mut tools = Vec::new();

        // 寸止工具始终可用（必需工具）
        let zhi_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "要显示给用户的消息"
                },
                "predefined_options": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "预定义的选项列表（可选）"
                },
                "is_markdown": {
                    "type": "boolean",
                    "description": "消息是否为Markdown格式，默认为true"
                },
                "project_path": {
                    "type": "string",
                    "description": "项目路径（可选，建议传入 git 根目录绝对路径，用于同项目防重复和超时重发）"
                }
            },
            "required": ["message"]
        });

        if let serde_json::Value::Object(schema_map) = zhi_schema {
            tools.push(Tool {
                name: Cow::Borrowed("zhi"),
                description: Some(Cow::Borrowed(
                    "智能代码审查交互工具，支持预定义选项、自由文本输入和图片上传",
                )),
                input_schema: Arc::new(schema_map),
                annotations: None,
            });
        }

        log_debug!(
            "返回给客户端的工具列表: {:?}",
            tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        log_debug!("收到工具调用请求: {}", request.name);

        match request.name.as_ref() {
            "zhi" => {
                // 解析请求参数
                let arguments_value = request
                    .arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let zhi_request: ZhiRequest = serde_json::from_value(arguments_value)
                    .map_err(|e| McpError::invalid_params(format!("参数解析失败: {}", e), None))?;

                // 调用寸止工具
                InteractionTool::zhi(zhi_request).await
            }
            _ => Err(McpError::invalid_request(
                format!("未知的工具: {}", request.name),
                None,
            )),
        }
    }
}

/// 启动MCP服务器
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // 创建并运行服务器
    let service = ZhiServer::new().serve(stdio()).await.inspect_err(|e| {
        log_important!(error, "启动服务器失败: {}", e);
    })?;

    // 等待服务器关闭
    service.waiting().await?;
    Ok(())
}
