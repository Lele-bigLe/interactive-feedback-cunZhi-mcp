// 工作流数据模型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工作流节点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// 节点标识符（如 "memory_gate", "read_context"）
    pub id: String,
    /// 节点中文名称
    pub name: String,
    /// 是否为必需节点（不可跳过）
    #[serde(default)]
    pub required: bool,
    /// 跳过条件列表（如 "complexity:simple", "no_task_intent"）
    #[serde(default)]
    pub skip_when: Vec<String>,
    /// 节点执行建议（AI 可参考的操作说明）
    #[serde(default)]
    pub action: String,
}

/// 任务复杂度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    /// 简单任务（1-2文件，单点修改）
    Simple,
    /// 中等任务（3+文件，业务逻辑）
    Medium,
    /// 复杂任务（架构变更，破坏性操作）
    Complex,
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexity::Simple => write!(f, "simple"),
            Complexity::Medium => write!(f, "medium"),
            Complexity::Complex => write!(f, "complex"),
        }
    }
}

impl Complexity {
    /// 从字符串解析复杂度
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "simple" => Some(Complexity::Simple),
            "medium" => Some(Complexity::Medium),
            "complex" => Some(Complexity::Complex),
            _ => None,
        }
    }
}

/// 复杂度判断规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRule {
    /// 最大文件数（超过则不匹配此级别）
    pub max_files: Option<u32>,
    /// 触发该复杂度的任务性质关键词
    #[serde(default)]
    pub nature: Vec<String>,
}

/// 工作流定义（从 YAML 或默认配置加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 工作流节点列表（有序）
    pub nodes: Vec<WorkflowNode>,
    /// 复杂度判断规则
    #[serde(default)]
    pub complexity_rules: HashMap<String, ComplexityRule>,
}

/// hint 工具的返回结果
#[derive(Debug, Clone, Serialize)]
pub struct WorkflowHintResult {
    /// 评估的任务复杂度
    pub complexity: String,
    /// 建议执行的步骤列表
    pub suggested_steps: Vec<SuggestedStep>,
    /// 被跳过的步骤列表（含原因）
    pub skipped_steps: Vec<SkippedStep>,
    /// 提醒信息
    pub reminder: String,
}

/// 建议的执行步骤
#[derive(Debug, Clone, Serialize)]
pub struct SuggestedStep {
    /// 节点标识符
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 执行建议
    pub action: String,
}

/// 被跳过的步骤
#[derive(Debug, Clone, Serialize)]
pub struct SkippedStep {
    /// 节点标识符
    pub id: String,
    /// 跳过原因
    pub reason: String,
}
