use crate::services::cli_tools::{CliToolManager, CliToolStatus, UpgradeResult, InstallMethod};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliToolInfo {
    pub id: String,
    pub key: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub install_methods: Vec<InstallMethodInfo>,
    pub npm_package: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMethodInfo {
    pub method: String,
    pub command: String,
    pub priority: u32,
}

impl From<&crate::services::cli_tools::InstallCommand> for InstallMethodInfo {
    fn from(cmd: &crate::services::cli_tools::InstallCommand) -> Self {
        InstallMethodInfo {
            method: cmd.method.as_str().to_string(),
            command: cmd.command.clone(),
            priority: cmd.priority,
        }
    }
}

impl From<&crate::services::cli_tools::CliToolConfig> for CliToolInfo {
    fn from(config: &crate::services::cli_tools::CliToolConfig) -> Self {
        CliToolInfo {
            id: config.key.clone(),
            key: config.key.clone(),
            name: config.name.clone(),
            icon: config.icon.clone(),
            description: config.description.clone(),
            install_methods: config.install_methods.iter().map(InstallMethodInfo::from).collect(),
            npm_package: config.npm_package.clone(),
            website_url: config.website_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliToolStatusInfo {
    pub tool_key: String,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub install_method: Option<String>,
    pub install_path: Option<String>,
    pub has_conflict: bool,
    pub conflict_info: Option<String>,
    pub latest_version: Option<String>,
    pub needs_upgrade: bool,
}

impl From<CliToolStatus> for CliToolStatusInfo {
    fn from(status: CliToolStatus) -> Self {
        CliToolStatusInfo {
            tool_key: status.tool_key,
            is_installed: status.is_installed,
            installed_version: status.installed_version,
            install_method: status.install_method.map(|m| m.as_str().to_string()),
            install_path: status.install_path,
            has_conflict: status.has_conflict,
            conflict_info: status.conflict_info,
            latest_version: status.latest_version,
            needs_upgrade: status.needs_upgrade,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeResultInfo {
    pub success: bool,
    pub message: String,
    pub new_version: Option<String>,
    pub method: String,
}

impl From<UpgradeResult> for UpgradeResultInfo {
    fn from(result: UpgradeResult) -> Self {
        UpgradeResultInfo {
            success: result.success,
            message: result.message,
            new_version: result.new_version,
            method: result.method.as_str().to_string(),
        }
    }
}

#[tauri::command]
pub fn get_cli_tools() -> Vec<CliToolInfo> {
    log::info!("Getting CLI tools list");
    CliToolManager::get_supported_tools()
        .iter()
        .map(CliToolInfo::from)
        .collect()
}

#[tauri::command]
pub fn check_cli_tool_status(tool_key: String) -> Result<CliToolStatusInfo, String> {
    log::info!("Checking status for CLI tool: {}", tool_key);
    let manager = CliToolManager::new();
    manager
        .check_installation(&tool_key)
        .map(CliToolStatusInfo::from)
        .map_err(|e| e.to_string())
}

/// Async version - checks all tools in parallel for better performance
/// Use this instead of check_all_cli_tools_status when performance matters
#[tauri::command]
pub async fn check_all_cli_tools_status_parallel() -> Vec<CliToolStatusInfo> {
    log::info!("check_all_cli_tools_status_parallel called - using async parallel execution");
    let manager = CliToolManager::new();
    
    // Run the async parallel check
    let statuses = manager.check_all_installations_parallel().await;
    
    log::info!("check_all_cli_tools_status_parallel returning {} statuses", statuses.len());
    for s in &statuses {
        log::info!("  {}: isInstalled={}", s.tool_key, s.is_installed);
    }
    
    statuses.into_iter().map(CliToolStatusInfo::from).collect()
}

#[tauri::command]
pub fn check_all_cli_tools_status() -> Vec<CliToolStatusInfo> {
    log::info!("check_all_cli_tools_status called");
    let manager = CliToolManager::new();
    let statuses: Vec<CliToolStatusInfo> = manager
        .check_all_installations()
        .into_iter()
        .map(CliToolStatusInfo::from)
        .collect();
    log::info!("check_all_cli_tools_status returning {} statuses", statuses.len());
    for s in &statuses {
        log::info!("  {}: isInstalled={}", s.tool_key, s.is_installed);
    }
    statuses
}

#[tauri::command]
pub async fn upgrade_cli_tool(tool_key: String, method: String) -> Result<UpgradeResultInfo, String> {
    log::info!("Upgrading CLI tool: {} with method: {}", tool_key, method);
    let manager = CliToolManager::new();

    let install_method = InstallMethod::from_str(&method)
        .ok_or_else(|| format!("Unknown install method: {}", method))?;

    manager
        .upgrade_async(&tool_key, install_method)
        .await
        .map(UpgradeResultInfo::from)
        .map_err(|e| e.to_string())
}
