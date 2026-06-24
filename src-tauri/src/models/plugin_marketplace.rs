// Plugin Marketplace Models - Rust side

use serde::{Deserialize, Serialize};

// Source installation status types (FEAT-016)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStatus {
    pub source_id: String,
    pub name: String,
    pub name_zh: Option<String>,
    pub repo_url: String,
    pub is_installed: bool,
    /// Primary installed path (kept for backwards compatibility)
    pub installed_path: Option<String>,
    /// All installed paths (forge/plugins/ and forge/marketplace/)
    pub installed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInstallResult {
    pub success: bool,
    pub source_id: String,
    /// Primary installed path (kept for backwards compatibility)
    pub installed_path: Option<String>,
    /// All installed paths (forge/plugins/ and forge/marketplace/)
    pub installed_paths: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInstallProgress {
    pub source_id: String,
    pub stage: String, // preparing, cloning, extracting, success, failed
    pub progress: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSource {
    pub id: String,
    pub name: String,
    pub name_zh: Option<String>,
    pub command: String, // The exact marketplace add command (GitHub URL)
    pub description: String,
    pub icon: Option<String>,
    pub plugin_count: Option<u32>,
    /// GitHub repository name (extracted from URL)
    pub repo_name: Option<String>,
    /// Repository type: `"market"` (marketplace index) or `"res"` (single-plugin repo).
    /// Persisted so the frontend can distinguish source types across restarts.
    pub repo_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstallSource {
    /// Source type discriminator:
    /// - `"local"` — relative path inside the marketplace source repo
    ///   (e.g. `./plugins/agent-sdk-dev`). Files are already on disk once
    ///   the source is cloned; install_plugin just copies them.
    /// - `"git-subdir"` — a subdirectory of another Git repository. install_plugin
    ///   does a sparse `git clone` of `<url>` and copies `<path>` out of it.
    /// - `"url"` — the whole plugin lives in an external Git repository.
    ///   install_plugin does a `git clone` of `<url>` and uses the repo root.
    /// Empty when the marketplace manifest did not declare a `source` (older
    /// manifests / fallback filesystem scan); treated as `"local"` by
    /// install_plugin.
    #[serde(default, rename = "type")]
    pub kind: String,
    /// Git clone URL (used by `git-subdir` and `url`).
    #[serde(default)]
    pub url: String,
    /// Subdirectory inside the cloned URL (used by `git-subdir`).
    #[serde(default)]
    pub path: String,
    /// Git ref (branch / tag) to check out. Optional.
    #[serde(default)]
    pub r#ref: String,
    /// Pinned commit SHA. When present we `git checkout` this exact SHA after
    /// cloning (so the installed plugin is reproducible). Optional.
    #[serde(default)]
    pub sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketplacePlugin {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub description: String,
    pub long_description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub has_update: Option<bool>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub install_command: Option<String>,
    /// Absolute on-disk path of the installed plugin directory.
    /// Populated by `get_installed_plugins` (and the marketplace→installed
    /// mapper) so the frontend can render "Installed at: …" on each row of
    /// the PluginsView Installed tab. Mirrors `MarketplaceSkill.installPath`.
    pub install_path: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub stars: Option<u32>,
    pub downloads: Option<u32>,
    pub last_updated: Option<String>,
    #[serde(default)]
    pub is_installed: bool,
    /// User-controlled enable/disable flag (lifecycle.ts in the Phase 1
    /// plan). Persisted in `.claude-plugin/marketplace.json` so the
    /// state survives app restarts and CLI syncs. Mirrors the
    /// `MarketplaceSkill.enabled` UI concept on the Skills side.
    #[serde(default)]
    pub disabled: bool,
    /// Original `source` block from the upstream marketplace manifest.
    /// install_plugin inspects this to decide between local copy and
    /// remote clone. Defaults to `None` for plugins that came from the
    /// legacy fallback scan (no manifest entry).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_source: Option<PluginInstallSource>,
    /// Key of the CLI tool this plugin is associated with (e.g. "claude-code").
    /// Used to display the CLI tool icon and determine the sync target directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_tool_key: Option<String>,
    /// All CLI tool keys this plugin supports (detected from marker directories).
    /// When non-empty, the Installed tab shows a sync button for each tool.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cli_tool_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub plugin_id: String,
    pub plugin_name: String,
    pub stage: String, // pending, downloading, installing, success, failed
    pub progress: u32,
    pub message: String,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallResult {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateResult {
    pub success: bool,
    pub new_version: Option<String>,
    pub error: Option<String>,
}

// GitHub API types
#[derive(Debug, Deserialize)]
pub struct GitHubContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
    pub download_url: Option<String>,
    #[serde(rename = "type")]
    pub file_type: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubReadme {
    pub content: String,
    pub encoding: String,
}

// Preset marketplace sources (4 real GitHub repos).
// These IDs (`anthropics` / `ccplugins` / `ananddtyagi` / `addyosmani`) match the source
// keys used in `.claude-plugin/marketplace.json` (see FEAT-008).
pub fn get_preset_sources() -> Vec<PluginSource> {
    vec![
        PluginSource {
            id: "anthropics".to_string(),
            name: "Anthropic Official".to_string(),
            name_zh: Some("Anthropic 官方".to_string()),
            command: "https://github.com/anthropics/claude-plugins-official".to_string(),
            description: "Anthropic 一手官方插件仓库（35+ 插件）".to_string(),
            icon: None,
            plugin_count: None,
            repo_name: Some("claude-plugins-official".to_string()),
            repo_type: Some("market".to_string()),
        },
        PluginSource {
            id: "ananddtyagi".to_string(),
            name: "cc-marketplace".to_string(),
            name_zh: Some("市场索引".to_string()),
            command: "https://github.com/ananddtyagi/cc-marketplace".to_string(),
            description: "Claude Code marketplace 索引仓库".to_string(),
            icon: None,
            plugin_count: None,
            repo_name: Some("cc-marketplace".to_string()),
            repo_type: Some("market".to_string()),
        },
        PluginSource {
            id: "addyosmani".to_string(),
            name: "agent-skills".to_string(),
            name_zh: Some("Agent 技能库".to_string()),
            command: "https://github.com/addyosmani/agent-skills".to_string(),
            description: "Addy Osmani 的 Agent 技能集合，提供多种实用技能".to_string(),
            icon: None,
            plugin_count: None,
            repo_name: Some("agent-skills".to_string()),
            repo_type: Some("res".to_string()),
        },
    ]
}

/// Extract repository name from a GitHub URL.
/// Example: "https://github.com/anthropics/claude-plugins-official" -> "claude-plugins-official"
pub fn extract_repo_name_from_url(url: &str) -> Option<String> {
    url.trim_end_matches('/')
        .split('/')
        .last()
        .map(|s| s.to_string())
}
