// 循（Xun）MCP Server 实现
// 工作流感知 MCP Server，提供 hint 工具和 workflow_rules 资源

use anyhow::Result;
use rmcp::{
    Error as McpError, ServerHandler, ServiceExt, RoleServer,
    model::*,
    transport::stdio,
    service::RequestContext,
};
use std::sync::Arc;
use std::borrow::Cow;

use crate::workflow::{
    WorkflowDefinition, WorkflowHintResult,
    evaluate_workflow, load_workflow_definition,
    loader::generate_workflow_rules_text,
};

/// hint 工具的请求参数
#[derive(Debug, serde::Deserialize)]
struct HintRequest {
    /// 一句话描述当前任务
    task_description: String,
    /// AI 自主判断的任务复杂度（可选）
    #[serde(default)]
    complexity: Option<String>,
}

/// 循 MCP Server
#[derive(Clone)]
pub struct XunServer {
    /// 工作流定义
    workflow_def: WorkflowDefinition,
    /// 工作流规则文本（缓存，用于 instructions 和 rules_text 工具）
    rules_text: String,
}

impl XunServer {
    pub fn new() -> Self {
        let workflow_def = load_workflow_definition();
        let rules_text = generate_workflow_rules_text(&workflow_def);
        Self {
            workflow_def,
            rules_text,
        }
    }
}

impl Default for XunServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for XunServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "Xun-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            // 将工作流规则直接放入 instructions，这样 AI 连接时就能看到
            instructions: Some(format!(
                "工作流引导工具。任务开始时调用 hint 获取工作流建议，按建议执行，完成后调用寸止(zhi)确认。\n\n{}",
                self.rules_text
            )),
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
        // hint 工具 schema
        let hint_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "task_description": {
                    "type": "string",
                    "description": "一句话描述当前任务"
                },
                "complexity": {
                    "type": "string",
                    "enum": ["simple", "medium", "complex"],
                    "description": "AI 自主判断的任务复杂度（可选，MCP 会根据任务描述自动判断）"
                }
            },
            "required": ["task_description"]
        });

        let mut tools = Vec::new();

        if let serde_json::Value::Object(schema_map) = hint_schema {
            tools.push(Tool {
                name: Cow::Borrowed("hint"),
                description: Some(Cow::Borrowed(
                    "工作流引导工具。任务开始时调用，获取工作流建议（建议的步骤、可跳过的步骤、复杂度评估）"
                )),
                input_schema: Arc::new(schema_map),
                annotations: None,
            });
        }

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
        match request.name.as_ref() {
            "hint" => {
                // 解析请求参数
                let arguments_value = request
                    .arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                let hint_request: HintRequest = serde_json::from_value(arguments_value)
                    .map_err(|e| {
                        McpError::invalid_params(format!("参数解析失败: {}", e), None)
                    })?;

                // 评估工作流
                let result: WorkflowHintResult = evaluate_workflow(
                    &self.workflow_def,
                    &hint_request.task_description,
                    hint_request.complexity.as_deref(),
                );

                // 序列化结果
                let result_json = serde_json::to_string_pretty(&result)
                    .map_err(|e| {
                        McpError::internal_error(format!("结果序列化失败: {}", e), None)
                    })?;

                log::info!(
                    "hint 工具调用: task=\"{}\" → complexity={}",
                    hint_request.task_description,
                    result.complexity
                );

                Ok(CallToolResult::success(vec![Content::text(result_json)]))
            }
            _ => Err(McpError::invalid_request(
                format!("未知的工具: {}", request.name),
                None,
            )),
        }
    }
}

/// 启动工作流 MCP 服务器
pub async fn run_workflow_server() -> Result<(), Box<dyn std::error::Error>> {
    let service = XunServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            log::error!("启动循(Xun) MCP 服务器失败: {}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
