// Skill Marketplace Tauri Commands

use crate::models::{
    MarketplaceSkill, PaginatedSkills, SkillSource, SyncResult, SyncTarget,
};
use crate::services::skill_marketplace;
use std::path::PathBuf;

/// Get all available skill sources
#[tauri::command]
pub fn get_skill_sources() -> Vec<SkillSource> {
    log::info!("Getting skill sources");
    skill_marketplace::get_preset_sources()
}

/// Fetch skills from a specific source with pagination and filtering
#[tauri::command]
pub async fn fetch_marketplace_skills(
    source_id: String,
    page: u32,
    page_size: u32,
    category: Option<String>,
    keyword: Option<String>,
) -> Result<PaginatedSkills, String> {
    log::info!(
        "Fetching marketplace skills: source={}, page={}, page_size={}, category={:?}, keyword={:?}",
        source_id, page, page_size, category, keyword
    );
    
    let sources = skill_marketplace::get_preset_sources();
    let source = sources
        .iter()
        .find(|s| s.id == source_id)
        .ok_or_else(|| format!("Unknown source: {}", source_id))?;
    
    // Use different fetching strategy based on source type
    match source.region.as_str() {
        "github" => {
            skill_marketplace::fetch_github_skills(
                source,
                page,
                page_size,
                category.as_deref(),
                keyword.as_deref(),
            )
            .await
        }
        _ => {
            // For standard API sources, try to fetch from API
            // If API fails, fall back to sample data
            match skill_marketplace::fetch_skills_from_api(
                source,
                page,
                page_size,
                category.as_deref(),
                keyword.as_deref(),
            )
            .await
            {
                Ok(result) => Ok(result),
                Err(e) => {
                    log::warn!("API fetch failed, using sample data: {}", e);
                    // Fall back to sample data for demonstration
                    skill_marketplace::fetch_github_skills(
                        source,
                        page,
                        page_size,
                        category.as_deref(),
                        keyword.as_deref(),
                    )
                    .await
                }
            }
        }
    }
}

/// Get all categories available
#[tauri::command]
pub fn get_skill_categories() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "key": "development",
            "label": "Development",
            "labelZh": "开发",
            "tags": ["code", "debug", "test", "refactor", "security", "database"]
        }),
        serde_json::json!({
            "key": "business",
            "label": "Business",
            "labelZh": "商业",
            "tags": ["business", "crm", "analytics", "finance", "marketing"]
        }),
        serde_json::json!({
            "key": "search",
            "label": "Search & Research",
            "labelZh": "搜索研究",
            "tags": ["search", "web", "research", "scraping"]
        }),
        serde_json::json!({
            "key": "writing",
            "label": "Writing & Content",
            "labelZh": "写作内容",
            "tags": ["writing", "content", "translation", "summarization"]
        }),
        serde_json::json!({
            "key": "tools",
            "label": "Tools & Automation",
            "labelZh": "工具自动化",
            "tags": ["automation", "system", "file", "network", "cloud"]
        }),
        serde_json::json!({
            "key": "ai",
            "label": "AI & Machine Learning",
            "labelZh": "AI与机器学习",
            "tags": ["llm", "ml", "nlp", "vision", "agent"]
        }),
    ]
}

/// Install a skill to local directory
#[tauri::command]
pub async fn install_marketplace_skill(
    skill: MarketplaceSkill,
    local_path: String,
) -> Result<String, String> {
    log::info!(
        "Installing skill '{}' to {}",
        skill.name,
        local_path
    );
    
    let local = PathBuf::from(local_path);
    skill_marketplace::install_skill(&skill, &local).await
}

/// Get local installed skills
#[tauri::command]
pub async fn get_local_marketplace_skills(
    local_path: String,
) -> Result<Vec<MarketplaceSkill>, String> {
    log::info!("Getting local skills from {}", local_path);
    let local = PathBuf::from(local_path);
    skill_marketplace::get_local_skills(&local).await
}

/// Sync a skill to target directory
#[tauri::command]
pub async fn sync_skill_to_target(
    skill_name: String,
    local_path: String,
    target: SyncTarget,
) -> Result<SyncResult, String> {
    log::info!(
        "Syncing skill '{}' from {} to {} ({})",
        skill_name, local_path, target.path, target.method
    );
    
    let local = PathBuf::from(local_path);
    skill_marketplace::sync_skill_to_target(&skill_name, &local, &target).await
}

/// Get sync targets configuration
#[tauri::command]
pub fn get_sync_targets() -> Vec<SyncTarget> {
    log::info!("Getting sync targets");
    vec![
        SyncTarget {
            id: "cursor-default".to_string(),
            name: "Cursor".to_string(),
            path: "~/.cursor/skills/".to_string(),
            method: "copy".to_string(),
            is_valid: true,
            exists: None,
        },
        SyncTarget {
            id: "claude-default".to_string(),
            name: "Claude Desktop".to_string(),
            path: "~/.claude/skills/".to_string(),
            method: "symlink".to_string(),
            is_valid: true,
            exists: None,
        },
    ]
}

/// Add a new sync target
#[tauri::command]
pub fn add_sync_target(target: SyncTarget) -> Result<SyncTarget, String> {
    log::info!("Adding sync target: {} ({})", target.name, target.path);
    
    // Validate the path
    let is_valid = skill_marketplace::validate_sync_target(&target);
    
    Ok(SyncTarget {
        id: target.id,
        name: target.name,
        path: target.path,
        method: target.method,
        is_valid,
        exists: Some(is_valid),
    })
}

/// Remove a sync target
#[tauri::command]
pub fn remove_sync_target(target_id: String) -> Result<(), String> {
    log::info!("Removing sync target: {}", target_id);
    Ok(())
}

/// Check if a skill is installed locally
#[tauri::command]
pub async fn is_skill_installed(
    skill_name: String,
    local_path: String,
) -> Result<bool, String> {
    let local = PathBuf::from(local_path);
    let skills_dir = local.join("skills").join(&skill_name);
    Ok(skills_dir.exists())
}

/// Get skill details from a specific source
#[tauri::command]
pub async fn get_skill_details(
    source_id: String,
    skill_id: String,
) -> Result<MarketplaceSkill, String> {
    log::info!("Getting skill details: {} from {}", skill_id, source_id);
    
    let sources = skill_marketplace::get_preset_sources();
    let _source = sources
        .iter()
        .find(|s| s.id == source_id)
        .ok_or_else(|| format!("Unknown source: {}", source_id))?;
    
    // For now, fetch first page and find the skill
    let result = fetch_marketplace_skills(source_id, 1, 50, None, None).await?;
    
    result
        .items
        .into_iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| format!("Skill not found: {}", skill_id))
}
