// 内置默认工作流定义
// 无需外部 YAML 文件即可工作

use std::collections::HashMap;
use super::definition::{WorkflowDefinition, WorkflowNode, ComplexityRule};

/// 返回内置的默认工作流定义
pub fn default_workflow() -> WorkflowDefinition {
    WorkflowDefinition {
        nodes: vec![
            WorkflowNode {
                id: "memory_gate".to_string(),
                name: "记忆搜索".to_string(),
                required: false,
                skip_when: vec![
                    "no_task_intent".to_string(),
                    "simple_greeting".to_string(),
                ],
                action: "调用 smart_search 搜索相关记忆，命中则 memory_read".to_string(),
            },
            WorkflowNode {
                id: "read_context".to_string(),
                name: "上下文读取".to_string(),
                required: true,
                skip_when: vec![],
                action: "读取相关代码文件，理解现有实现".to_string(),
            },
            WorkflowNode {
                id: "plan_design".to_string(),
                name: "方案设计".to_string(),
                required: false,
                skip_when: vec![
                    "complexity:simple".to_string(),
                    "complexity:medium".to_string(),
                ],
                action: "使用 sequential-thinking 分析方案（≤3步）".to_string(),
            },
            WorkflowNode {
                id: "execute".to_string(),
                name: "代码实现".to_string(),
                required: true,
                skip_when: vec![],
                action: "执行代码修改，遵循项目代码规范".to_string(),
            },
            WorkflowNode {
                id: "verify".to_string(),
                name: "验证".to_string(),
                required: false,
                skip_when: vec![
                    "complexity:simple".to_string(),
                    "config_only".to_string(),
                ],
                action: "运行 lint/build/typecheck 验证变更".to_string(),
            },
            WorkflowNode {
                id: "gate".to_string(),
                name: "确认(cunzhi)".to_string(),
                required: true,
                skip_when: vec![],
                action: "调用寸止(zhi)工具获取用户反馈，循环直到用户说\"结束\"".to_string(),
            },
        ],
        complexity_rules: {
            let mut rules = HashMap::new();
            rules.insert(
                "simple".to_string(),
                ComplexityRule {
                    max_files: Some(2),
                    nature: vec![
                        "register".to_string(),
                        "config".to_string(),
                        "single_edit".to_string(),
                        "fix_typo".to_string(),
                        "add_field".to_string(),
                    ],
                },
            );
            rules.insert(
                "medium".to_string(),
                ComplexityRule {
                    max_files: Some(5),
                    nature: vec![
                        "new_feature".to_string(),
                        "multi_file".to_string(),
                        "refactor".to_string(),
                        "bug_fix".to_string(),
                    ],
                },
            );
            rules.insert(
                "complex".to_string(),
                ComplexityRule {
                    max_files: None,
                    nature: vec![
                        "architecture".to_string(),
                        "breaking_change".to_string(),
                        "migration".to_string(),
                        "redesign".to_string(),
                    ],
                },
            );
            rules
        },
    }
}
