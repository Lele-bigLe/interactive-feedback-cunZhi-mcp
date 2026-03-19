// 工作流引擎
// 根据任务描述评估复杂度，生成工作流建议路径

use super::definition::*;

/// 评估任务并生成工作流建议
///
/// # 参数
/// - `definition`: 工作流定义
/// - `task_description`: 任务描述文本
/// - `hint_complexity`: AI 自主判断的复杂度（可选，优先采纳）
pub fn evaluate_workflow(
    definition: &WorkflowDefinition,
    task_description: &str,
    hint_complexity: Option<&str>,
) -> WorkflowHintResult {
    // 1. 确定复杂度
    let complexity = determine_complexity(definition, task_description, hint_complexity);
    let complexity_tag = format!("complexity:{}", complexity);

    // 2. 根据复杂度过滤节点
    let mut suggested_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    for node in &definition.nodes {
        // 检查是否应该跳过此节点
        let skip_reason = should_skip_node(node, &complexity_tag, task_description);

        if let Some(reason) = skip_reason {
            skipped_steps.push(SkippedStep {
                id: node.id.clone(),
                reason,
            });
        } else {
            suggested_steps.push(SuggestedStep {
                id: node.id.clone(),
                name: node.name.clone(),
                action: node.action.clone(),
            });
        }
    }

    // 3. 构建提醒信息
    let reminder = build_reminder(&complexity);

    WorkflowHintResult {
        complexity: complexity.to_string(),
        suggested_steps,
        skipped_steps,
        reminder,
    }
}

/// 确定任务复杂度
fn determine_complexity(
    definition: &WorkflowDefinition,
    task_description: &str,
    hint_complexity: Option<&str>,
) -> Complexity {
    // AI 提供的复杂度建议优先
    if let Some(hint) = hint_complexity {
        if let Some(c) = Complexity::from_str_opt(hint) {
            return c;
        }
    }

    // 通过关键词匹配判断复杂度
    let desc_lower = task_description.to_lowercase();

    // 从高到低匹配（复杂 → 中等 → 简单）
    if matches_complexity_rule(definition, "complex", &desc_lower) {
        return Complexity::Complex;
    }
    if matches_complexity_rule(definition, "medium", &desc_lower) {
        return Complexity::Medium;
    }

    // 默认为简单
    Complexity::Simple
}

/// 检查任务描述是否匹配某个复杂度规则
fn matches_complexity_rule(
    definition: &WorkflowDefinition,
    level: &str,
    desc_lower: &str,
) -> bool {
    if let Some(rule) = definition.complexity_rules.get(level) {
        for keyword in &rule.nature {
            if desc_lower.contains(&keyword.to_lowercase()) {
                return true;
            }
        }
    }
    false
}

/// 判断是否应跳过某个节点，返回跳过原因
fn should_skip_node(
    node: &WorkflowNode,
    complexity_tag: &str,
    task_description: &str,
) -> Option<String> {
    // 必需节点永远不跳过
    if node.required {
        return None;
    }

    let desc_lower = task_description.to_lowercase();

    for condition in &node.skip_when {
        // 复杂度条件匹配（如 "complexity:simple"）
        if condition.starts_with("complexity:") && condition == complexity_tag {
            let level = condition.strip_prefix("complexity:").unwrap_or("");
            return Some(format!("{}任务可跳过「{}」", level, node.name));
        }

        // 通用条件匹配（如 "no_task_intent", "config_only"）
        match condition.as_str() {
            "no_task_intent" => {
                // 简单的意图检测：无明确任务关键词
                let task_keywords = ["修改", "添加", "删除", "修复", "实现", "重构", "创建",
                                     "fix", "add", "delete", "implement", "refactor", "create",
                                     "update", "change", "move", "remove", "build"];
                if !task_keywords.iter().any(|k| desc_lower.contains(k)) {
                    return Some(format!("无明确任务意图，跳过「{}」", node.name));
                }
            }
            "simple_greeting" => {
                let greetings = ["你好", "hello", "hi", "hey", "嗨"];
                if greetings.iter().any(|g| desc_lower.starts_with(g)) && desc_lower.len() < 20 {
                    return Some(format!("简单问候，跳过「{}」", node.name));
                }
            }
            "config_only" => {
                let config_keywords = ["配置", "config", "设置", "env", ".env", "yaml", "json配置"];
                if config_keywords.iter().any(|k| desc_lower.contains(k))
                    && !desc_lower.contains("重构")
                    && !desc_lower.contains("架构")
                {
                    return Some(format!("仅配置变更，跳过「{}」", node.name));
                }
            }
            _ => {}
        }
    }

    None
}

/// 构建提醒信息
fn build_reminder(complexity: &Complexity) -> String {
    match complexity {
        Complexity::Simple => {
            "完成后请调用寸止(zhi)工具获取用户反馈".to_string()
        }
        Complexity::Medium => {
            "建议每步完成后调用寸止(zhi)确认，确保方向正确".to_string()
        }
        Complexity::Complex => {
            "请先确认方案再实施，每个关键节点调用寸止(zhi)确认".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::default::default_workflow;

    #[test]
    fn test_simple_task() {
        let def = default_workflow();
        let result = evaluate_workflow(&def, "修改按钮颜色", None);
        assert_eq!(result.complexity, "simple");
        // 简单任务应跳过 plan_design 和 verify
        assert!(result.skipped_steps.iter().any(|s| s.id == "plan_design"));
    }

    #[test]
    fn test_complex_task() {
        let def = default_workflow();
        let result = evaluate_workflow(&def, "重构整体 architecture 架构", None);
        assert_eq!(result.complexity, "complex");
        // 复杂任务不应跳过 plan_design
        assert!(!result.skipped_steps.iter().any(|s| s.id == "plan_design"));
    }

    #[test]
    fn test_hint_override() {
        let def = default_workflow();
        let result = evaluate_workflow(&def, "简单修改", Some("complex"));
        // AI 提供的复杂度应优先
        assert_eq!(result.complexity, "complex");
    }

    #[test]
    fn test_gate_always_included() {
        let def = default_workflow();
        let result = evaluate_workflow(&def, "随便什么任务", None);
        // gate(cunzhi) 节点永远包含在建议步骤中
        assert!(result.suggested_steps.iter().any(|s| s.id == "gate"));
    }
}
