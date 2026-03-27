use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use ring::digest::{Context as ShaContext, SHA256};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::constants::{
    mcp::{MAX_RETRY_COUNT, REQUEST_TIMEOUT_MS},
    validation::is_valid_popup_timeout,
};
use crate::mcp::{
    types::{McpResponse, PopupRequest},
    utils::decode_and_normalize_path,
};

#[derive(Debug, Clone)]
struct ProjectPopupScope {
    project_key: String,
    project_path: String,
    project_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectPopupState {
    request_id: String,
    project_path: String,
    project_name: String,
    timeout_ms: u64,
    retry_count: u32,
    expires_at: DateTime<Utc>,
    #[serde(default)]
    paused: bool,
    #[serde(default)]
    remaining_ms: u64,
    updated_at: DateTime<Utc>,
}

/// 创建 Tauri 弹窗
///
/// 优先调用与 MCP 服务器同目录的 UI 命令，找不到时使用全局版本
pub fn create_tauri_popup(request: &PopupRequest) -> Result<String> {
    let timeout_ms = if is_valid_popup_timeout(request.timeout_ms) {
        request.timeout_ms
    } else {
        REQUEST_TIMEOUT_MS
    };
    let project_scope = build_project_popup_scope(request.project_path.as_deref())?;
    let mut retry_count = 0;

    loop {
        let expires_at = Utc::now() + Duration::milliseconds(timeout_ms as i64);

        if let Some(scope) = project_scope.as_ref() {
            ensure_project_request_available(scope, &request.id)?;
            persist_project_popup_state(scope, &request.id, timeout_ms, retry_count, expires_at)?;
        }

        let popup_request = build_runtime_popup_request(request, project_scope.as_ref(), timeout_ms, retry_count);
        let popup_response = launch_tauri_popup(&popup_request);

        match popup_response {
            Ok(response) => {
                if is_timeout_response(&response) {
                    if retry_count >= MAX_RETRY_COUNT {
                        if let Some(scope) = project_scope.as_ref() {
                            clear_project_popup_state(scope, &request.id)?;
                        }
                        return Ok("用户长时间未响应，寸止已停止自动重发".to_string());
                    }

                    retry_count += 1;
                    continue;
                }

                if let Some(scope) = project_scope.as_ref() {
                    clear_project_popup_state(scope, &request.id)?;
                }
                return Ok(response);
            }
            Err(error) => {
                if let Some(scope) = project_scope.as_ref() {
                    clear_project_popup_state(scope, &request.id)?;
                }
                return Err(error);
            }
        }
    }
}

fn build_runtime_popup_request(
    request: &PopupRequest,
    project_scope: Option<&ProjectPopupScope>,
    timeout_ms: u64,
    retry_count: u32,
) -> PopupRequest {
    let mut popup_request = request.clone();
    popup_request.timeout_ms = timeout_ms;
    popup_request.retry_count = retry_count;

    if let Some(scope) = project_scope {
        popup_request.project_path = Some(scope.project_path.clone());
        popup_request.project_name = Some(scope.project_name.clone());
    }

    popup_request
}

fn launch_tauri_popup(request: &PopupRequest) -> Result<String> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("mcp_request_{}.json", request.id));
    let request_json = serde_json::to_string_pretty(request)?;
    fs::write(&temp_file, request_json)?;

    let command_path = find_ui_command()?;
    let output = Command::new(&command_path)
        .arg("--mcp-request")
        .arg(temp_file.to_string_lossy().to_string())
        .output()?;

    let _ = fs::remove_file(&temp_file);

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout);
        let response = response.trim();
        if response.is_empty() {
            Ok("用户取消了操作".to_string())
        } else {
            Ok(response.to_string())
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("UI进程失败: {}", error);
    }
}

fn build_project_popup_scope(project_path: Option<&str>) -> Result<Option<ProjectPopupScope>> {
    let Some(project_path) = project_path else {
        return Ok(None);
    };

    let normalized_path = normalize_project_scope_path(project_path)?;
    let normalized_path_str = normalized_path.to_string_lossy().to_string();
    let project_name = normalized_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| normalized_path_str.clone());

    Ok(Some(ProjectPopupScope {
        project_key: hash_project_path(&normalized_path_str),
        project_path: normalized_path_str,
        project_name,
    }))
}

fn normalize_project_scope_path(project_path: &str) -> Result<PathBuf> {
    let normalized_path_str = decode_and_normalize_path(project_path)
        .map_err(|error| anyhow::anyhow!("项目路径格式错误: {}", error))?;
    let input_path = Path::new(&normalized_path_str);
    let absolute_path = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(input_path)
    };
    let canonical_path = absolute_path
        .canonicalize()
        .unwrap_or_else(|_| manual_canonicalize(&absolute_path).unwrap_or(absolute_path));

    if !canonical_path.exists() {
        anyhow::bail!("项目路径不存在: {}", canonical_path.display());
    }

    if !canonical_path.is_dir() {
        anyhow::bail!("项目路径不是目录: {}", canonical_path.display());
    }

    Ok(find_git_root(&canonical_path).unwrap_or(canonical_path))
}

fn manual_canonicalize(path: &Path) -> Result<PathBuf> {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !components.is_empty() {
                    components.pop();
                }
            }
            _ => components.push(component),
        }
    }

    let mut result = PathBuf::new();
    for component in components {
        result.push(component);
    }

    Ok(result)
}

fn find_git_root(start_path: &Path) -> Option<PathBuf> {
    let mut current_path = start_path;

    loop {
        if current_path.join(".git").exists() {
            return Some(current_path.to_path_buf());
        }

        match current_path.parent() {
            Some(parent) => current_path = parent,
            None => break,
        }
    }

    None
}

fn hash_project_path(project_path: &str) -> String {
    let normalized = if cfg!(windows) {
        project_path.to_lowercase()
    } else {
        project_path.to_string()
    };

    let mut context = ShaContext::new(&SHA256);
    context.update(normalized.as_bytes());
    let digest = context.finish();
    hex::encode(digest.as_ref())
}

fn ensure_project_request_available(scope: &ProjectPopupScope, current_request_id: &str) -> Result<()> {
    let Some(state) = read_project_popup_state(scope)? else {
        return Ok(());
    };

    if state.request_id == current_request_id {
        return Ok(());
    }

    let now = Utc::now();
    if state.paused {
        anyhow::bail!(
            "当前项目 `{}` 的 cunzhi 请求计时已暂停，请勿重复发起。",
            state.project_name
        );
    }

    if state.expires_at <= now {
        clear_project_popup_state(scope, &state.request_id)?;
        return Ok(());
    }

    let remaining_seconds = (state.expires_at - now).num_seconds().max(1);
    anyhow::bail!(
        "当前项目 `{}` 已有未过期的寸止请求，剩余 {} 秒，请勿重复发起。",
        state.project_name,
        remaining_seconds
    );
}

fn persist_project_popup_state(
    scope: &ProjectPopupScope,
    request_id: &str,
    timeout_ms: u64,
    retry_count: u32,
    expires_at: DateTime<Utc>,
) -> Result<()> {
    let state = ProjectPopupState {
        request_id: request_id.to_string(),
        project_path: scope.project_path.clone(),
        project_name: scope.project_name.clone(),
        timeout_ms,
        retry_count,
        expires_at,
        paused: false,
        remaining_ms: timeout_ms,
        updated_at: Utc::now(),
    };

    let state_file = project_popup_state_file(scope)?;
    if let Some(parent) = state_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(state_file, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

fn read_project_popup_state(scope: &ProjectPopupScope) -> Result<Option<ProjectPopupState>> {
    let state_file = project_popup_state_file(scope)?;
    if !state_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&state_file)?;
    match serde_json::from_str::<ProjectPopupState>(&content) {
        Ok(state) => Ok(Some(state)),
        Err(_) => {
            let _ = fs::remove_file(&state_file);
            Ok(None)
        }
    }
}

pub fn update_project_popup_timer_state(
    request: &PopupRequest,
    remaining_ms: u64,
    paused: bool,
) -> Result<()> {
    let Some(scope) = build_project_popup_scope(request.project_path.as_deref())? else {
        return Ok(());
    };

    let Some(mut state) = read_project_popup_state(&scope)? else {
        return Ok(());
    };

    if state.request_id != request.id {
        return Ok(());
    }

    let normalized_remaining_ms = remaining_ms.min(request.timeout_ms.max(1000));
    state.timeout_ms = request.timeout_ms;
    state.retry_count = request.retry_count;
    state.paused = paused;
    state.remaining_ms = normalized_remaining_ms;
    state.updated_at = Utc::now();
    state.expires_at = if paused {
        Utc::now() + Duration::days(365)
    } else {
        Utc::now() + Duration::milliseconds(normalized_remaining_ms as i64)
    };

    let state_file = project_popup_state_file(&scope)?;
    fs::write(state_file, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

fn clear_project_popup_state(scope: &ProjectPopupScope, request_id: &str) -> Result<()> {
    let state_file = project_popup_state_file(scope)?;
    if !state_file.exists() {
        return Ok(());
    }

    if let Some(state) = read_project_popup_state(scope)? {
        if state.request_id == request_id {
            let _ = fs::remove_file(state_file);
        }
    }

    Ok(())
}

fn project_popup_state_file(scope: &ProjectPopupScope) -> Result<PathBuf> {
    let runtime_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
        .join("cunzhi")
        .join("runtime")
        .join("zhi-project-state");

    Ok(runtime_dir.join(format!("{}.json", scope.project_key)))
}

fn is_timeout_response(response: &str) -> bool {
    serde_json::from_str::<McpResponse>(response)
        .ok()
        .and_then(|parsed| parsed.metadata.source)
        .map(|source| source == "popup_timeout")
        .unwrap_or(false)
}

/// 查找等一下 UI 命令的路径
///
/// 按优先级查找：同目录 -> 全局版本 -> 开发环境
fn find_ui_command() -> Result<String> {
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let local_ui_path = exe_dir.join("等一下");
            if local_ui_path.exists() && is_executable(&local_ui_path) {
                return Ok(local_ui_path.to_string_lossy().to_string());
            }
        }
    }

    if test_command_available("等一下") {
        return Ok("等一下".to_string());
    }

    anyhow::bail!(
        "找不到等一下 UI 命令。请确保：\n\
         1. 已编译项目：cargo build --release\n\
         2. 或已全局安装：./install.sh\n\
         3. 或等一下命令在同目录下"
    )
}

fn test_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
    }
}
