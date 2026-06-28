//! AllAgents Tauri Commands - 将 AllAgentsService 暴露为 Tauri IPC 命令
//!
//! 所有命令通过 serde_json 序列化输出，前端可直接调用

use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{command, AppHandle};
use serde::{Deserialize, Serialize};
use crate::services::allagents_service::{
    AllAgentsConfig, AllAgentsService, WorkspaceConfig, WorkspaceSection,
    PluginEntry, McpServerEntry,
};

/// 统一的命令执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl CommandResult {
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// 获取 AllAgentsService 实例（带 AppHandle，用于事件发射）
fn get_service_with_handle(workspace_path: &str, handle: &AppHandle) -> Result<AllAgentsService, String> {
    let config = AllAgentsConfig {
        workspace_path: PathBuf::from(workspace_path),
        cli_path: None,
        version_requirement: "^1.12.0".to_string(),
        auto_install: true,
    };
    let mut service = AllAgentsService::new(config);
    service.set_app_handle(handle.clone());
    Ok(service)
}

/// 获取 AllAgentsService 实例（不带 AppHandle）
fn get_service(workspace_path: &str) -> Result<AllAgentsService, String> {
    let config = AllAgentsConfig {
        workspace_path: PathBuf::from(workspace_path),
        cli_path: None,
        version_requirement: "^1.12.0".to_string(),
        auto_install: true,
    };
    Ok(AllAgentsService::new(config))
}

// ============================================================================
// 工作区管理命令
// ============================================================================

/// 初始化 allagents 工作区
#[command]
pub async fn allagents_init(
    workspace_path: String,
    from: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.init_workspace(from.as_deref()) {
        Ok(()) => Ok(CommandResult::ok(serde_json::json!({
            "message": "Workspace initialized successfully",
            "path": workspace_path
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 同步所有插件到配置的客户端
#[command]
pub async fn allagents_update(
    app_handle: AppHandle,
    workspace_path: String,
    offline: Option<bool>,
    dry_run: Option<bool>,
    client: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service_with_handle(&workspace_path, &app_handle)?;

    match service.update(
        offline.unwrap_or(false),
        dry_run.unwrap_or(false),
        client.as_deref(),
    ) {
        Ok(report) => Ok(CommandResult::ok(serde_json::json!({
            "synced_count": report.synced_files.len(),
            "error_count": report.errors.len(),
            "skipped_count": report.skipped.len(),
            "synced_files": report.synced_files,
            "errors": report.errors,
            "skipped": report.skipped
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 获取工作区状态
#[command]
pub async fn allagents_status(
    workspace_path: String,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.status() {
        Ok(status) => Ok(CommandResult::ok(serde_json::to_value(status)
            .map_err(|e| format!("Serialization error: {}", e))?)),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

// ============================================================================
// 插件管理命令
// ============================================================================

/// 安装插件
#[command]
pub async fn allagents_plugin_install(
    workspace_path: String,
    plugin_spec: String,
    scope: Option<String>,
    skills: Option<Vec<String>>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.plugin_install(
        &plugin_spec,
        scope.as_deref(),
        skills.as_deref(),
    ) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("Plugin {} installed successfully", plugin_spec),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 卸载插件
#[command]
pub async fn allagents_plugin_uninstall(
    workspace_path: String,
    plugin_spec: String,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.plugin_uninstall(&plugin_spec) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("Plugin {} uninstalled successfully", plugin_spec),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 列出已安装的插件
#[command]
pub async fn allagents_plugin_list(
    workspace_path: String,
    marketplace: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.plugin_list(marketplace.as_deref()) {
        Ok(list) => Ok(CommandResult::ok(serde_json::to_value(list)
            .map_err(|e| format!("Serialization error: {}", e))?)),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

// ============================================================================
// 技能管理命令
// ============================================================================

/// 列出所有技能
#[command]
pub async fn allagents_skill_list(
    workspace_path: String,
    scope: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.skill_list(scope.as_deref()) {
        Ok(list) => Ok(CommandResult::ok(serde_json::to_value(list)
            .map_err(|e| format!("Serialization error: {}", e))?)),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 添加技能
#[command]
pub async fn allagents_skill_add(
    workspace_path: String,
    name: String,
    from: Option<String>,
    plugin: Option<String>,
    scope: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.skill_add(
        &name,
        from.as_deref(),
        plugin.as_deref(),
        scope.as_deref(),
    ) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("Skill {} added successfully", name),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 移除技能
#[command]
pub async fn allagents_skill_remove(
    workspace_path: String,
    name: String,
    plugin: Option<String>,
    scope: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.skill_remove(
        &name,
        plugin.as_deref(),
        scope.as_deref(),
    ) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("Skill {} removed successfully", name),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

// ============================================================================
// MCP 管理命令
// ============================================================================

/// 添加 MCP 服务器
#[command]
pub async fn allagents_mcp_add(
    workspace_path: String,
    name: String,
    command_or_url: String,
    transport: Option<String>,
    client: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.mcp_add(
        &name,
        &command_or_url,
        transport.as_deref(),
        client.as_deref(),
    ) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("MCP server {} added successfully", name),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 移除 MCP 服务器
#[command]
pub async fn allagents_mcp_remove(
    workspace_path: String,
    name: String,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.mcp_remove(&name) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("MCP server {} removed successfully", name),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 列出 MCP 服务器
#[command]
pub async fn allagents_mcp_list(
    workspace_path: String,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.mcp_list() {
        Ok(list) => Ok(CommandResult::ok(serde_json::to_value(list)
            .map_err(|e| format!("Serialization error: {}", e))?)),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 同步 MCP 配置
#[command]
pub async fn allagents_mcp_update(
    workspace_path: String,
    offline: Option<bool>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.mcp_update(offline.unwrap_or(false)) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": "MCP configuration updated",
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

// ============================================================================
// Marketplace 管理命令
// ============================================================================

/// 添加 marketplace 源
#[command]
pub async fn allagents_marketplace_add(
    workspace_path: String,
    source: String,
    name: Option<String>,
    branch: Option<String>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.marketplace_add(
        &source,
        name.as_deref(),
        branch.as_deref(),
    ) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("Marketplace {} added successfully", source),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 移除 marketplace 源
#[command]
pub async fn allagents_marketplace_remove(
    workspace_path: String,
    name: String,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.marketplace_remove(&name) {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "message": format!("Marketplace {} removed successfully", name),
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// 列出 marketplace 源
#[command]
pub async fn allagents_marketplace_list(
    workspace_path: String,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    match service.marketplace_list() {
        Ok(output) => Ok(CommandResult::ok(serde_json::json!({
            "output": output
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

// ============================================================================
// 配置管理命令
// ============================================================================

/// 生成 workspace.yaml 配置
#[command]
pub async fn allagents_generate_config(
    workspace_path: String,
    clients: Vec<String>,
    plugins: Vec<String>,
    mcp_servers: Option<HashMap<String, McpServerConfig>>,
) -> Result<CommandResult, String> {
    let service = get_service(&workspace_path)?;

    let mcp_entries = mcp_servers.map(|servers| {
        servers.into_iter().map(|(name, config)| {
            (
                name,
                McpServerEntry {
                    transport_type: config.transport_type,
                    url: config.url,
                    command: config.command,
                    args: config.args,
                    env: config.env,
                    headers: config.headers,
                    clients: config.clients,
                },
            )
        }).collect()
    });

    let config = WorkspaceConfig {
        workspace: Some(WorkspaceSection {
            source: Some(".forge/config".to_string()),
            files: None,
        }),
        repositories: None,
        plugins: Some(plugins.into_iter().map(PluginEntry::Simple).collect()),
        clients: Some(clients),
        mcp_servers: mcp_entries,
        mcp_proxy: None,
        sync_mode: Some("copy".to_string()),
    };

    match service.write_config(&config) {
        Ok(()) => Ok(CommandResult::ok(serde_json::json!({
            "message": "workspace.yaml generated successfully",
            "path": format!("{}/workspace.yaml", workspace_path)
        }))),
        Err(e) => Ok(CommandResult::err(e)),
    }
}

/// MCP 服务器配置（前端传入用）
#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub transport_type: String,
    pub url: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
    pub clients: Option<Vec<String>>,
}
