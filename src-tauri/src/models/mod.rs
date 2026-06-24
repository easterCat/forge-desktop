use serde::{Deserialize, Serialize};

pub mod skill_marketplace;
pub mod mcp_marketplace;
pub mod plugin_marketplace;
pub mod plugin_capabilities;
pub mod mcp_bridge;

pub use skill_marketplace::{
    InstallProgress as SkillInstallProgress, GitHubContent as SkillGitHubContent,
    GitHubReadme as SkillGitHubReadme, SkillSource, MarketplaceSkill,
    PaginatedSkills, SyncTarget, SyncConfig, SyncProgress, InstallResult, SyncResult,
    FetchSkillsRequest,
};
pub use mcp_marketplace::{
    MCPSource, EnvVar, MCPServer, PaginatedMCPServers, MCPSyncTarget,
    MCPInstallProgress, MCPSyncProgress, MCPInstallResult, MCPSyncResult,
    MCPSyncConfig, FetchServersRequest,
};
pub use plugin_marketplace::{
    InstallProgress as PluginInstallProgress, GitHubContent as PluginGitHubContent,
    GitHubReadme as PluginGitHubReadme, PluginSource, MarketplacePlugin,
    PluginInstallResult, PluginUpdateResult, PluginInstallSource,
    get_preset_sources as get_plugin_preset_sources,
    SourceStatus, SourceInstallResult, SourceInstallProgress,
};
pub use plugin_capabilities::PluginCapabilities;
pub use plugin_capabilities::{
    HookExecutionResult, ValidationIssue, ValidationReport, ValidationCapabilityCounts,
};
pub use mcp_bridge::McpProbeResult;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SoftwareStatus {
    #[default]
    Unknown,
    Installed,
    NotInstalled,
    Outdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Software {
    pub id: String,
    pub name: String,
    pub key: String,
    pub version: Option<String>,
    #[serde(default)]
    pub install_path: Option<String>,
    pub config_path: String,
    pub is_installed: bool,
    #[serde(default)]
    pub last_checked: Option<String>,
    #[serde(default)]
    pub latest_version: Option<String>,
    #[serde(default)]
    pub is_upgradable: bool,
    #[serde(default)]
    pub status: SoftwareStatus,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub software_id: String,
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub installed_path: Option<String>,
    pub enabled: bool,
    pub installed_at: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub software_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub skill_type: String,
    pub config: Option<String>,
    pub file_path: Option<String>,
    pub installed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpService {
    pub id: String,
    pub software_id: String,
    pub name: String,
    pub endpoint: String,
    pub auth_type: String,
    pub config: Option<String>,
    pub is_healthy: bool,
    pub last_checked: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub software_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub file_path: Option<String>,
    pub content: Option<String>,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: Option<i64>,
    pub file_count: Option<i32>,
    pub created_at: Option<String>,
    pub includes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub files: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub emoji: Option<String>,
    pub color: Option<String>,
    pub department: String,
    pub content: String,
    pub source: String,
    pub tags: Option<String>,
    pub installed_targets: Option<String>,
    pub is_custom: bool,
    pub created_at: String,
    pub updated_at: String,
}
