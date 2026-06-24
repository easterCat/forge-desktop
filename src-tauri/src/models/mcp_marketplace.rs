// MCP Marketplace Models - Rust side

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPSource {
    pub id: String,
    pub name: String,
    pub name_zh: Option<String>,
    pub region: String, // international, china, github, mcp-specific
    pub url: String,
    pub api_endpoint: String,
    pub description: String,
    pub icon: Option<String>,
    pub is_available: bool,
    pub last_checked: Option<String>,
    pub server_count: Option<u32>,
    pub requires_auth: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub description: String,
    pub long_description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub install_command: Option<String>,
    pub install_path: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub npm_package: Option<String>,
    pub protocol: String, // stdio, sse, http
    pub required_env_vars: Option<Vec<EnvVar>>,
    pub required_permissions: Option<Vec<String>>,
    pub last_updated: Option<String>,
    pub stars: Option<u32>,
    pub downloads: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedMCPServers {
    pub items: Vec<MCPServer>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPSyncTarget {
    pub id: String,
    pub name: String,
    pub path: String,
    pub method: String, // copy, symlink
    pub is_valid: bool,
    pub exists: Option<bool>,
    pub config_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPInstallProgress {
    pub server_id: String,
    pub server_name: String,
    pub stage: String, // pending, downloading, installing, extracting, syncing, success, failed, conflict
    pub progress: u32,
    pub message: String,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPSyncProgress {
    pub server_name: String,
    pub target_name: String,
    pub method: String,
    pub stage: String, // pending, syncing, success, failed
    pub progress: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPInstallResult {
    pub success: bool,
    pub server_name: String,
    pub path: Option<String>,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPSyncResult {
    pub success: bool,
    pub server_name: String,
    pub target_path: String,
    pub method: String,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPSyncConfig {
    pub targets: Vec<MCPSyncTarget>,
    pub default_method: String,
}

// Request/Response types for API calls
#[derive(Debug, Serialize)]
pub struct FetchServersRequest {
    pub page: u32,
    pub page_size: u32,
    pub category: Option<String>,
    pub keyword: Option<String>,
}
