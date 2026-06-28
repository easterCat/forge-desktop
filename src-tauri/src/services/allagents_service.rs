//! AllAgents Service - 封装 allagents CLI 的所有交互逻辑
//!
//! 职责：
//! 1. 检查/安装 allagents CLI
//! 2. 生成/更新 workspace.yaml 配置
//! 3. 执行 allagents 命令并解析 JSON 输出
//! 4. 管理同步状态和错误处理

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

// ============================================================================
// 事件系统
// ============================================================================

/// AllAgents 事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AllAgentsEvent {
    /// 同步进度
    #[serde(rename = "sync-progress")]
    SyncProgress {
        phase: String,
        current: u32,
        total: u32,
        message: String,
    },
    /// 同步完成
    #[serde(rename = "sync-complete")]
    SyncComplete {
        success: bool,
        synced_count: u32,
        error_count: u32,
        skipped_count: u32,
    },
    /// 同步错误
    #[serde(rename = "sync-error")]
    SyncError {
        file: String,
        client: String,
        error: String,
        recoverable: bool,
    },
    /// 配置变更
    #[serde(rename = "config-changed")]
    ConfigChanged {
        section: String,
        action: String,
    },
}

/// 事件发射器（供 Tauri 命令使用）
pub struct EventEmitter {
    app_handle: Option<tauri::AppHandle>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self { app_handle: None }
    }

    pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn emit(&self, event: &AllAgentsEvent) {
        if let Some(ref handle) = self.app_handle {
            let _ = handle.emit("allagents:event", event);
        }
    }
}

/// AllAgents 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllAgentsConfig {
    /// 工作区路径
    pub workspace_path: PathBuf,
    /// allagents CLI 可执行文件路径（可选，自动检测）
    pub cli_path: Option<PathBuf>,
    /// allagents 版本要求
    pub version_requirement: String,
    /// 是否自动安装 allagents
    pub auto_install: bool,
}

impl Default for AllAgentsConfig {
    fn default() -> Self {
        Self {
            workspace_path: PathBuf::new(),
            cli_path: None,
            version_requirement: "^1.12.0".to_string(),
            auto_install: true,
        }
    }
}

/// workspace.yaml 配置结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<RepositoryEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<PluginEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clients: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, McpServerEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_proxy: Option<McpProxyConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<WorkspaceFile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkspaceFile {
    Simple(String),
    Detailed {
        source: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryEntry {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginEntry {
    Simple(String),
    Detailed {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        clients: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        install: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclude: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        skills: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pin: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntry {
    #[serde(rename = "type")]
    pub transport_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clients: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpProxyConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clients: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<HashMap<String, McpProxyServer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpProxyServer {
    pub proxy: Vec<String>,
}

// ============================================================================
// AllAgents CLI 输出结构
// ============================================================================

/// allagents update --json 输出
#[derive(Debug, Deserialize)]
pub struct SyncReport {
    #[serde(default)]
    pub synced_files: Vec<SyncedFile>,
    #[serde(default)]
    pub errors: Vec<SyncError>,
    #[serde(default)]
    pub skipped: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncedFile {
    pub source: String,
    pub destination: String,
    pub client: String,
    pub action: String, // "created" | "updated" | "unchanged"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncError {
    pub file: String,
    pub client: String,
    pub error: String,
}

/// allagents workspace status --json 输出
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceStatus {
    pub workspace_path: String,
    pub clients: Vec<String>,
    pub plugins: Vec<PluginStatus>,
    pub mcp_servers: Vec<McpServerStatus>,
    pub last_sync: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginStatus {
    pub name: String,
    pub installed: bool,
    pub skills_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerStatus {
    pub name: String,
    pub transport: String,
    pub url: Option<String>,
}

/// allagents skill list --json 输出
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillListResult {
    pub skills: Vec<SkillEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillEntry {
    pub name: String,
    pub plugin: Option<String>,
    pub path: String,
    pub enabled: bool,
}

/// allagents plugin list --json 输出
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginListResult {
    pub plugins: Vec<PluginListItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginListItem {
    pub name: String,
    pub source: String,
    pub installed: bool,
    pub skills_count: u32,
    pub agents_count: u32,
}

/// allagents mcp list --json 输出
#[derive(Debug, Serialize, Deserialize)]
pub struct McpListResult {
    pub servers: Vec<McpListItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpListItem {
    pub name: String,
    #[serde(rename = "type")]
    pub transport_type: String,
    pub url: Option<String>,
    pub command: Option<String>,
}

// ============================================================================
// AllAgentsService 实现
// ============================================================================

pub struct AllAgentsService {
    config: AllAgentsConfig,
    event_emitter: EventEmitter,
}

impl AllAgentsService {
    /// 创建新的 AllAgentsService
    pub fn new(config: AllAgentsConfig) -> Self {
        Self {
            config,
            event_emitter: EventEmitter::new(),
        }
    }

    /// 设置 Tauri AppHandle（用于事件发射）
    pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
        self.event_emitter.set_app_handle(handle);
    }

    /// 发射事件
    fn emit_event(&self, event: &AllAgentsEvent) {
        self.event_emitter.emit(event);
    }

    /// 确保 allagents CLI 已安装，返回可执行文件路径
    pub fn ensure_installed(&self) -> Result<PathBuf, String> {
        // 1. 检查配置中指定的路径
        if let Some(ref path) = self.config.cli_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // 2. 检查 PATH 中的 allagents
        let which_cmd = if cfg!(target_os = "windows") {
            Command::new("where").arg("allagents").output()
        } else {
            Command::new("which").arg("allagents").output()
        };

        if let Ok(output) = which_cmd {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // Windows where 命令可能返回多行，取第一行
                let first_line = path_str.lines().next().unwrap_or(&path_str);
                return Ok(PathBuf::from(first_line));
            }
        }

        // 3. 检查 npx 是否可用
        if self.config.auto_install {
            info!("allagents not found, attempting installation via npm...");
            let install_result = Command::new("npm")
                .args(["install", "-g", "allagents"])
                .output();

            match install_result {
                Ok(output) if output.status.success() => {
                    info!("allagents installed successfully");
                    // 重新检查路径
                    return self.ensure_installed();
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to install allagents: {}", stderr);
                    return Err(format!("Failed to install allagents: {}", stderr));
                }
                Err(e) => {
                    error!("Failed to run npm: {}", e);
                    return Err(format!("Failed to run npm: {}", e));
                }
            }
        }

        Err("allagents CLI not found and auto_install is disabled".to_string())
    }

    /// 执行 allagents 命令并返回 JSON 输出
    fn execute_command(
        &self,
        args: &[&str],
        workspace_path: Option<&Path>,
    ) -> Result<String, String> {
        let cli_path = self.ensure_installed()?;

        let mut cmd = Command::new(&cli_path);
        cmd.args(args);

        if let Some(path) = workspace_path {
            cmd.current_dir(path);
        } else {
            cmd.current_dir(&self.config.workspace_path);
        }

        // 确保 JSON 输出
        if !args.contains(&"--json") {
            cmd.arg("--json");
        }

        let output = cmd.output().map_err(|e| {
            let msg = format!("Failed to execute allagents: {}", e);
            error!("{}", msg);
            msg
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let msg = format!(
                "allagents command failed (exit {}):\nstdout: {}\nstderr: {}",
                output.status.code().unwrap_or(-1),
                stdout,
                stderr
            );
            error!("{}", msg);
            return Err(msg);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    // ========================================================================
    // 工作区管理
    // ========================================================================

    /// 初始化工作区
    pub fn init_workspace(&self, from: Option<&str>) -> Result<(), String> {
        let mut args = vec!["init", "."];
        let from_arg;
        if let Some(source) = from {
            from_arg = format!("--from={}", source);
            args.push(&from_arg);
        }

        self.execute_command(&args, None)?;
        info!("Workspace initialized at {:?}", self.config.workspace_path);
        Ok(())
    }

    /// 同步所有插件到配置的客户端
    pub fn update(
        &self,
        offline: bool,
        dry_run: bool,
        client_filter: Option<&str>,
    ) -> Result<SyncReport, String> {
        let mut args = vec!["update"];

        let offline_arg;
        if offline {
            offline_arg = "--offline".to_string();
            args.push("--offline");
        }

        let dry_run_arg;
        if dry_run {
            dry_run_arg = "--dry-run".to_string();
            args.push("--dry-run");
        }

        let client_arg;
        if let Some(client) = client_filter {
            client_arg = format!("--client={}", client);
            args.push("--client");
            args.push(&client_arg);
        }

        // 发射开始同步事件
        self.emit_event(&AllAgentsEvent::SyncProgress {
            phase: "starting".to_string(),
            current: 0,
            total: 0,
            message: "Starting sync...".to_string(),
        });

        let output = self.execute_command(&args, None)?;

        // 发射同步中事件
        self.emit_event(&AllAgentsEvent::SyncProgress {
            phase: "transforming".to_string(),
            current: 1,
            total: 2,
            message: "Transforming files...".to_string(),
        });

        let report: SyncReport = serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse sync report: {}", e))?;

        // 发射同步完成事件
        self.emit_event(&AllAgentsEvent::SyncComplete {
            success: report.errors.is_empty(),
            synced_count: report.synced_files.len() as u32,
            error_count: report.errors.len() as u32,
            skipped_count: report.skipped.len() as u32,
        });

        // 发射每个错误事件
        for err in &report.errors {
            self.emit_event(&AllAgentsEvent::SyncError {
                file: err.file.clone(),
                client: err.client.clone(),
                error: err.error.clone(),
                recoverable: true,
            });
        }

        Ok(report)
    }

    /// 获取工作区状态
    pub fn status(&self) -> Result<WorkspaceStatus, String> {
        let output = self.execute_command(&["workspace", "status"], None)?;
        serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse status: {}", e))
    }

    // ========================================================================
    // 插件管理
    // ========================================================================

    /// 安装插件
    pub fn plugin_install(
        &self,
        plugin_spec: &str,
        scope: Option<&str>,
        skills: Option<&[String]>,
    ) -> Result<String, String> {
        let mut args = vec!["plugin", "install", plugin_spec];

        let scope_arg;
        if let Some(s) = scope {
            scope_arg = format!("--scope={}", s);
            args.push(&scope_arg);
        }

        // 注意: allagents 的 --skill 参数用于过滤，需要逐个传递
        let skill_args: Vec<String> = skills
            .unwrap_or(&[])
            .iter()
            .map(|s| format!("--skill={}", s))
            .collect();

        let mut refs: Vec<&str> = skill_args.iter().map(|s| s.as_str()).collect();
        args.append(&mut refs);

        self.execute_command(&args, None)
    }

    /// 卸载插件
    pub fn plugin_uninstall(&self, plugin_spec: &str) -> Result<String, String> {
        self.execute_command(&["plugin", "uninstall", plugin_spec], None)
    }

    /// 列出已安装的插件
    pub fn plugin_list(&self, marketplace: Option<&str>) -> Result<PluginListResult, String> {
        let mut args = vec!["plugin", "list"];

        let marketplace_arg;
        if let Some(m) = marketplace {
            marketplace_arg = m.to_string();
            args.push(&marketplace_arg);
        }

        let output = self.execute_command(&args, None)?;
        serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse plugin list: {}", e))
    }

    // ========================================================================
    // 技能管理
    // ========================================================================

    /// 列出所有技能
    pub fn skill_list(&self, scope: Option<&str>) -> Result<SkillListResult, String> {
        let mut args = vec!["skill", "list"];

        let scope_arg;
        if let Some(s) = scope {
            scope_arg = format!("--scope={}", s);
            args.push(&scope_arg);
        }

        let output = self.execute_command(&args, None)?;
        serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse skill list: {}", e))
    }

    /// 添加技能
    pub fn skill_add(
        &self,
        name: &str,
        from: Option<&str>,
        plugin: Option<&str>,
        scope: Option<&str>,
    ) -> Result<String, String> {
        let mut args = vec!["skill", "add", name];

        let from_arg;
        if let Some(f) = from {
            from_arg = format!("--from={}", f);
            args.push("--from");
            args.push(&from_arg);
        }

        let plugin_arg;
        if let Some(p) = plugin {
            plugin_arg = format!("--plugin={}", p);
            args.push("--plugin");
            args.push(&plugin_arg);
        }

        let scope_arg;
        if let Some(s) = scope {
            scope_arg = format!("--scope={}", s);
            args.push("--scope");
            args.push(&scope_arg);
        }

        self.execute_command(&args, None)
    }

    /// 移除技能
    pub fn skill_remove(
        &self,
        name: &str,
        plugin: Option<&str>,
        scope: Option<&str>,
    ) -> Result<String, String> {
        let mut args = vec!["skill", "remove", name];

        let plugin_arg;
        if let Some(p) = plugin {
            plugin_arg = format!("--plugin={}", p);
            args.push("--plugin");
            args.push(&plugin_arg);
        }

        let scope_arg;
        if let Some(s) = scope {
            scope_arg = format!("--scope={}", s);
            args.push("--scope");
            args.push(&scope_arg);
        }

        self.execute_command(&args, None)
    }

    // ========================================================================
    // MCP 管理
    // ========================================================================

    /// 添加 MCP 服务器
    pub fn mcp_add(
        &self,
        name: &str,
        command_or_url: &str,
        transport: Option<&str>,
        client: Option<&str>,
    ) -> Result<String, String> {
        let mut args = vec!["mcp", "add", name, command_or_url];

        let transport_arg;
        if let Some(t) = transport {
            transport_arg = format!("--transport={}", t);
            args.push("--transport");
            args.push(&transport_arg);
        }

        let client_arg;
        if let Some(c) = client {
            client_arg = format!("--client={}", c);
            args.push("--client");
            args.push(&client_arg);
        }

        self.execute_command(&args, None)
    }

    /// 移除 MCP 服务器
    pub fn mcp_remove(&self, name: &str) -> Result<String, String> {
        self.execute_command(&["mcp", "remove", name], None)
    }

    /// 列出 MCP 服务器
    pub fn mcp_list(&self) -> Result<McpListResult, String> {
        let output = self.execute_command(&["mcp", "list"], None)?;
        serde_json::from_str(&output)
            .map_err(|e| format!("Failed to parse MCP list: {}", e))
    }

    /// 同步 MCP 配置（不执行完整文件同步）
    pub fn mcp_update(&self, offline: bool) -> Result<String, String> {
        let mut args = vec!["mcp", "update"];

        let offline_arg;
        if offline {
            offline_arg = "--offline".to_string();
            args.push("--offline");
        }

        self.execute_command(&args, None)
    }

    // ========================================================================
    // Marketplace 管理
    // ========================================================================

    /// 添加 marketplace 源
    pub fn marketplace_add(
        &self,
        source: &str,
        name: Option<&str>,
        branch: Option<&str>,
    ) -> Result<String, String> {
        let mut args = vec!["plugin", "marketplace", "add", source];

        let name_arg;
        if let Some(n) = name {
            name_arg = format!("--name={}", n);
            args.push("--name");
            args.push(&name_arg);
        }

        let branch_arg;
        if let Some(b) = branch {
            branch_arg = format!("--branch={}", b);
            args.push("--branch");
            args.push(&branch_arg);
        }

        self.execute_command(&args, None)
    }

    /// 移除 marketplace 源
    pub fn marketplace_remove(&self, name: &str) -> Result<String, String> {
        self.execute_command(&["plugin", "marketplace", "remove", name], None)
    }

    /// 列出 marketplace 源
    pub fn marketplace_list(&self) -> Result<String, String> {
        self.execute_command(&["plugin", "marketplace", "list"], None)
    }

    // ========================================================================
    // 配置生成
    // ========================================================================

    /// 写入 workspace.yaml 配置文件
    pub fn write_config(&self, config: &WorkspaceConfig) -> Result<(), String> {
        let yaml = serde_yaml::to_string(config)
            .map_err(|e| format!("YAML serialization failed: {}", e))?;

        let config_path = self.config.workspace_path.join("workspace.yaml");
        std::fs::write(&config_path, &yaml)
            .map_err(|e| format!("Failed to write workspace.yaml: {}", e))?;

        info!("workspace.yaml written to {:?}", config_path);
        Ok(())
    }

    /// 读取现有的 workspace.yaml 配置
    pub fn read_config(&self) -> Result<WorkspaceConfig, String> {
        let config_path = self.config.workspace_path.join("workspace.yaml");

        if !config_path.exists() {
            return Err(format!("workspace.yaml not found at {:?}", config_path));
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read workspace.yaml: {}", e))?;

        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse workspace.yaml: {}", e))
    }

    /// 更新 workspace.yaml 中的指定部分
    pub fn update_config(&self, updates: &WorkspaceConfig) -> Result<(), String> {
        let mut config = self.read_config().unwrap_or_default();

        // 合并更新
        if let Some(updates_workspace) = &updates.workspace {
            let mut ws = config.workspace.unwrap_or(WorkspaceSection {
                source: None,
                files: None,
            });
            if let Some(source) = &updates_workspace.source {
                ws.source = Some(source.clone());
            }
            if let Some(files) = &updates_workspace.files {
                ws.files = Some(files.clone());
            }
            config.workspace = Some(ws);
        }

        if let Some(clients) = &updates.clients {
            config.clients = Some(clients.clone());
        }

        if let Some(plugins) = &updates.plugins {
            let mut existing = config.plugins.unwrap_or_default();
            existing.extend(plugins.clone());
            config.plugins = Some(existing);
        }

        if let Some(mcp_servers) = &updates.mcp_servers {
            let mut existing = config.mcp_servers.unwrap_or_default();
            for (key, value) in mcp_servers {
                existing.insert(key.clone(), value.clone());
            }
            config.mcp_servers = Some(existing);
        }

        self.write_config(&config)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_serialization() {
        let mut mcp_servers = HashMap::new();
        mcp_servers.insert(
            "test-server".to_string(),
            McpServerEntry {
                transport_type: "http".to_string(),
                url: Some("https://example.com/mcp".to_string()),
                command: None,
                args: None,
                env: None,
                headers: None,
                clients: None,
            },
        );

        let config = WorkspaceConfig {
            workspace: Some(WorkspaceSection {
                source: Some(".forge/config".to_string()),
                files: None,
            }),
            repositories: None,
            plugins: Some(vec![
                PluginEntry::Simple("code-review@claude-plugins-official".to_string()),
                PluginEntry::Detailed {
                    name: "superpowers@obra/superpowers".to_string(),
                    source: None,
                    clients: Some(vec!["claude".to_string(), "cursor".to_string()]),
                    install: None,
                    exclude: None,
                    skills: None,
                    pin: None,
                },
            ]),
            clients: Some(vec![
                "claude".to_string(),
                "copilot".to_string(),
                "cursor".to_string(),
            ]),
            mcp_servers: Some(mcp_servers),
            mcp_proxy: None,
            sync_mode: Some("copy".to_string()),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("code-review@claude-plugins-official"));
        assert!(yaml.contains("test-server"));
        assert!(yaml.contains("https://example.com/mcp"));
    }

    #[test]
    fn test_default_config() {
        let config = AllAgentsConfig::default();
        assert_eq!(config.version_requirement, "^1.12.0");
        assert!(config.auto_install);
    }
}
