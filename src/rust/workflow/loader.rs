// 工作流配置加载器
// 支持从 ~/.cunzhi/workflow.yaml 加载自定义配置

use anyhow::Result;
use std::path::PathBuf;

use super::definition::WorkflowDefinition;
use super::default::default_workflow;

/// 加载工作流定义
///
/// 优先从 ~/.cunzhi/workflow.yaml 加载用户自定义配置，
/// 如果文件不存在或解析失败则使用内置默认配置
pub fn load_workflow_definition() -> WorkflowDefinition {
    match try_load_custom_workflow() {
        Ok(Some(def)) => {
            log::info!("已加载自定义工作流配置");
            def
        }
        Ok(None) => {
            log::debug!("未找到自定义工作流配置，使用内置默认值");
            default_workflow()
        }
        Err(e) => {
            log::warn!("加载自定义工作流配置失败: {}，使用内置默认值", e);
            default_workflow()
        }
    }
}

/// 尝试加载用户自定义的 workflow.yaml
fn try_load_custom_workflow() -> Result<Option<WorkflowDefinition>> {
    let config_path = get_workflow_config_path();

    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path)?;
    let definition: WorkflowDefinition = serde_yaml::from_str(&content)?;

    // 基础校验：至少要有一个节点
    if definition.nodes.is_empty() {
        anyhow::bail!("workflow.yaml 中节点列表为空");
    }

    Ok(Some(definition))
}

/// 获取 workflow.yaml 的配置路径
fn get_workflow_config_path() -> PathBuf {
    // 优先使用 ~/.cunzhi/workflow.yaml
    if let Some(home) = dirs::home_dir() {
        return home.join(".cunzhi").join("workflow.yaml");
    }

    // 回退到当前目录
    PathBuf::from("workflow.yaml")
}

/// 生成工作流规则的文本描述（用于 Resource 返回）
pub fn generate_workflow_rules_text(definition: &WorkflowDefinition) -> String {
    let mut text = String::new();
    text.push_str("# 工作流规则\n\n");
    text.push_str("## 执行节点\n\n");

    for node in &definition.nodes {
        let required_tag = if node.required { " [必需]" } else { "" };
        text.push_str(&format!("- **{}**{}: {}\n", node.name, required_tag, node.action));

        if !node.skip_when.is_empty() {
            text.push_str(&format!("  跳过条件: {}\n", node.skip_when.join(", ")));
        }
    }

    text.push_str("\n## 复杂度规则\n\n");

    for (level, rule) in &definition.complexity_rules {
        text.push_str(&format!("### {}\n", level));
        if let Some(max) = rule.max_files {
            text.push_str(&format!("- 最大文件数: {}\n", max));
        }
        if !rule.nature.is_empty() {
            text.push_str(&format!("- 触发关键词: {}\n", rule.nature.join(", ")));
        }
        text.push('\n');
    }

    text.push_str("## 核心要求\n\n");
    text.push_str("1. 任务开始时调用 `hint` 工具获取工作流建议\n");
    text.push_str("2. 按建议的步骤执行（可根据实际情况灵活调整）\n");
    text.push_str("3. 完成后**必须**调用寸止(zhi)工具获取用户反馈\n");
    text.push_str("4. 未收到\"结束\"指令前，保持循环交互\n");

    text
}
