// Plugin Capabilities Model - describes a plugin's capability matrix
// (Skills / Hooks / Commands / MCP / LSP).
// Serialized to camelCase so the frontend TypeScript can consume it directly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level capabilities response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilities {
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub description: String,
    pub repository: Option<String>,
    /// "local" when the plugin is installed; "remote" when fetched from the network.
    pub source: String,
    /// The source identifier from MarketplacePlugin.sourceId (e.g. "anthropics").
    pub source_id: String,
    /// Absolute path to the plugin directory (empty string for remote-only).
    pub installed_path: String,
    pub capabilities: PluginCapabilityCounts,
    pub skills: Vec<SkillInfo>,
    pub commands: Vec<CommandInfo>,
    pub hooks: Vec<HookInfo>,
    pub mcp_servers: Vec<McpServerInfo>,
    pub lsp_servers: Vec<LspServerInfo>,
    pub dependencies: Vec<String>,
    /// Files listed in the plugin's manifest.files (if present).
    pub manifest_files: Vec<String>,
}

/// Count of each capability type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilityCounts {
    pub skills: u32,
    pub hooks: u32,
    pub commands: u32,
    pub mcp_servers: u32,
    pub lsp_servers: u32,
}

/// A Skill defined in skills/<name>/SKILL.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInfo {
    /// CamelCase skill identifier derived from the directory name.
    pub name: String,
    /// Human-readable description from YAML frontmatter.
    pub description: String,
    /// Relative path from plugin root, e.g. "skills/using-superpowers/SKILL.md".
    pub path: String,
    /// True when scripts/ subdirectory exists.
    #[serde(default)]
    pub has_scripts: bool,
    /// True when references/ subdirectory exists.
    #[serde(default)]
    pub has_references: bool,
}

/// A slash-command defined in commands/<name>.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandInfo {
    /// CamelCase command identifier derived from the file name.
    pub name: String,
    /// Human-readable description from YAML frontmatter.
    pub description: String,
    /// Tools the command is allowed to invoke.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Relative path from plugin root, e.g. "commands/my-command.md".
    pub path: String,
}

/// A hook defined in hooks/hooks.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookInfo {
    /// Lifecycle event name, e.g. "SessionStart".
    pub event: String,
    /// Regex matcher, e.g. "startup|clear|compact".
    #[serde(default)]
    pub matcher: Option<String>,
    /// Command to execute, e.g. "bash hooks/session-start".
    pub command: String,
    /// True when the command script file exists under hooks/.
    #[serde(default)]
    pub script_exists: bool,
}

/// An MCP server declared in plugin.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

/// An LSP server declared in plugin.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspServerInfo {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

/// Result of executing a hook script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookExecutionResult {
    pub event: String,
    pub matcher: Option<String>,
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub parsed_json: Option<serde_json::Value>,
    pub started_at: String,
    pub duration_ms: u64,
    pub log_path: String,
}

/// Validation issue (error or warning) for plugin validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

/// Capability counts snapshot embedded in ValidationReport.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ValidationCapabilityCounts {
    pub skills: u32,
    pub hooks: u32,
    pub commands: u32,
    pub mcp_servers: u32,
    pub lsp_servers: u32,
}

/// Validation report for a plugin directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub capabilities: ValidationCapabilityCounts,
}
