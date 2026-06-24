// Skill Marketplace Models - Rust side

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSource {
    pub id: String,
    pub name: String,
    pub name_zh: Option<String>,
    pub region: String, // international, china, github
    pub url: String,
    pub api_endpoint: String,
    pub description: String,
    pub icon: Option<String>,
    pub is_available: bool,
    pub last_checked: Option<String>,
    pub skill_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSkill {
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
    pub stars: Option<u32>,
    pub downloads: Option<u32>,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedSkills {
    pub items: Vec<MarketplaceSkill>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTarget {
    pub id: String,
    pub name: String,
    pub path: String,
    pub method: String, // copy, symlink
    pub is_valid: bool,
    pub exists: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub targets: Vec<SyncTarget>,
    pub default_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub skill_id: String,
    pub skill_name: String,
    pub stage: String, // pending, downloading, installing, syncing, success, failed, conflict
    pub progress: u32,
    pub message: String,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub skill_name: String,
    pub target_name: String,
    pub method: String,
    pub stage: String, // pending, syncing, success, failed
    pub progress: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub skill_name: String,
    pub local_path: String,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub skill_name: String,
    pub target_path: String,
    pub method: String,
    pub message: String,
    pub error: Option<String>,
}

// Request/Response types for API calls
#[derive(Debug, Serialize)]
pub struct FetchSkillsRequest {
    pub page: u32,
    pub page_size: u32,
    pub category: Option<String>,
    pub keyword: Option<String>,
}

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
