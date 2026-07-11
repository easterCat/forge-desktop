// Plugin Marketplace Tauri Commands
// Backed by `.claude-plugin/marketplace.json` synced via
// `node scripts/plugins/install.mjs`.

use crate::models::{
    MarketplacePlugin, PluginSource, PluginInstallResult, PluginUpdateResult,
    SourceStatus, SourceInstallResult,
};
use crate::services::plugin_marketplace;
use std::collections::HashMap;

/// Return preset (well-known) plugin sources, with the live `pluginCount`
/// injected from the local manifest.
#[tauri::command]
pub async fn get_marketplace_sources() -> Vec<PluginSource> {
    log::info!("Getting marketplace sources");
    plugin_marketplace::get_marketplace_sources().await
}

/// Fetch every plugin from a marketplace source (no pagination).
/// `source_id` is the local key (e.g. `anthropics` / `ccplugins` / `ananddtyagi`).
/// `keyword` is an optional case-insensitive substring match against name
/// and description.
#[tauri::command]
pub async fn fetch_marketplace_plugins(
    source_id: String,
    keyword: Option<String>,
) -> Result<Vec<MarketplacePlugin>, String> {
    log::info!(
        "Fetching marketplace plugins: source_id={}, keyword={:?}",
        source_id,
        keyword
    );
    plugin_marketplace::fetch_plugins_from_source(
        &source_id,
        keyword.as_deref(),
    )
    .await
}

/// Return the full marketplace manifest (sources + plugins + removed).
/// The frontend uses this to render counts and the Installed tab.
#[tauri::command]
pub async fn get_marketplace_manifest(
) -> Result<crate::services::plugin_marketplace::MarketplaceManifest, String> {
    log::info!("Getting marketplace manifest");
    Ok(plugin_marketplace::get_marketplace_manifest().await)
}

/// List locally installed plugins (i.e. plugins that have a directory under
/// `plugins/<source>/<name>/`).
#[tauri::command]
pub async fn get_marketplace_plugins() -> Result<Vec<MarketplacePlugin>, String> {
    log::info!("Getting installed marketplace plugins");
    let dir = plugin_marketplace::plugins_dir();
    plugin_marketplace::get_installed_plugins(&dir).await
}

/// Install a plugin by copying its metadata into `plugins/<source>/<name>/`.
/// The actual plugin source files are already on disk thanks to
/// `node scripts/plugins/install.mjs` — this command just creates a metadata
/// stub so the Installed tab picks it up.
#[tauri::command]
pub async fn install_marketplace_plugin(
    plugin: MarketplacePlugin,
) -> Result<PluginInstallResult, String> {
    log::info!(
        "Installing plugin '{}' from source '{}'",
        plugin.name,
        plugin.source_id
    );
    let dir = plugin_marketplace::plugins_dir();
    plugin_marketplace::install_plugin(&plugin, "", &dir).await
}

/// Uninstall a plugin by removing its `plugins/<source>/<name>/` directory.
#[tauri::command]
pub async fn uninstall_marketplace_plugin(
    plugin_id: String,
) -> Result<PluginInstallResult, String> {
    log::info!("Uninstalling plugin: {}", plugin_id);
    let dir = plugin_marketplace::plugins_dir();
    plugin_marketplace::uninstall_plugin(&plugin_id, &dir).await
}

/// Update a plugin (not implemented in this iteration).
#[tauri::command]
pub async fn update_marketplace_plugin(
    plugin: MarketplacePlugin,
) -> Result<PluginUpdateResult, String> {
    log::info!("Updating plugin '{}'", plugin.name);
    let dir = plugin_marketplace::plugins_dir();
    plugin_marketplace::update_plugin(&plugin, "", &dir).await
}

/// Legacy command — kept for compatibility. With GitHub-backed sources the
/// registration happens in `.claude-plugin/marketplace.json`.
#[tauri::command]
pub async fn add_marketplace_source(
    _source: PluginSource,
) -> Result<PluginInstallResult, String> {
    Ok(PluginInstallResult {
        success: true,
        path: None,
        error: None,
    })
}

/// Check if a plugin is installed on disk.
#[tauri::command]
pub async fn is_plugin_installed(plugin_name: String) -> Result<bool, String> {
    // Look across all source dirs under `plugins/`
    let root = plugin_marketplace::plugins_dir();
    if !root.exists() {
        return Ok(false);
    }
    let read = match std::fs::read_dir(&root) {
        Ok(r) => r,
        Err(_) => return Ok(false),
    };
    for entry in read.filter_map(|e| e.ok()) {
        let p = entry.path().join(&plugin_name);
        if p.exists() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Flip the user-controlled enable/disable flag for an installed plugin.
/// The plugin directory on disk is left untouched; the flag is metadata
/// only and is persisted in `.claude-plugin/marketplace.json`. Returns
/// the new disabled value so the frontend can reconcile its store.
#[tauri::command]
pub async fn set_plugin_disabled(
    source_id: String,
    plugin_name: String,
    disabled: bool,
) -> Result<bool, String> {
    log::info!(
        "set_plugin_disabled: {}/{} -> disabled={}",
        source_id,
        plugin_name,
        disabled
    );
    plugin_marketplace::set_plugin_disabled(&source_id, &plugin_name, disabled).await
}

// ---------------------------------------------------------------------------
// Source installation commands (FEAT-016)
// ---------------------------------------------------------------------------

/// Return installation status for all preset marketplace sources.
#[tauri::command]
pub async fn get_marketplace_source_status() -> Vec<SourceStatus> {
    log::info!("Getting marketplace source installation status");
    plugin_marketplace::get_sources_status()
}

/// Install a single marketplace source by cloning its GitHub repository.
///
/// `repo_url` is optional and only required for user-added sources that
/// are not in the preset list (FEAT-018). For preset sources we look up
/// the URL from `get_preset_sources()`.
#[tauri::command]
pub async fn install_marketplace_source(
    app: tauri::AppHandle,
    source_id: String,
    repo_url: Option<String>,
) -> Result<SourceInstallResult, String> {
    log::info!("Installing marketplace source: {}", source_id);
    plugin_marketplace::install_marketplace_source(&source_id, repo_url.as_deref(), Some(&app)).await
}

/// Batch install all preset marketplace sources (sequential, not parallel).
#[tauri::command]
pub async fn install_all_marketplace_sources(
    app: tauri::AppHandle,
) -> Vec<SourceInstallResult> {
    use crate::models::plugin_marketplace::get_preset_sources;

    log::info!("Installing all marketplace sources");
    let sources = get_preset_sources();
    let mut results = Vec::with_capacity(sources.len());

    for source in sources {
        let result = plugin_marketplace::install_marketplace_source(&source.id, None, Some(&app)).await;
        match result {
            Ok(r) => results.push(r),
            Err(e) => {
                log::error!("Failed to install source '{}': {}", source.id, e);
                results.push(SourceInstallResult {
                    success: false,
                    source_id: source.id,
                    installed_path: None,
                    installed_paths: Vec::new(),
                    error: Some(e),
                });
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// User-added source commands (FEAT-019)
// Persist user-added custom GitHub repos to
// `$FORGE_HOME/plugins/user_sources.json` so they survive restarts.
// ---------------------------------------------------------------------------

/// Return all user-added sources persisted on disk.
#[tauri::command]
pub async fn get_user_marketplace_sources() -> Vec<PluginSource> {
    log::info!("Getting user-added marketplace sources");
    plugin_marketplace::read_user_sources()
}

/// Add a new user-added source. Rejects duplicates by `id` or by
/// `command` (URL). Returns the persisted source on success.
#[tauri::command]
pub async fn add_user_marketplace_source(
    source: PluginSource,
) -> Result<PluginSource, String> {
    log::info!(
        "Adding user marketplace source: id={}, command={}",
        source.id,
        source.command
    );

    if source.id.trim().is_empty() {
        return Err("Source id is required".to_string());
    }
    if source.command.trim().is_empty() {
        return Err("Source command (repo URL) is required".to_string());
    }
    if !is_valid_source_id(&source.id) {
        return Err(
            "Source id must contain only letters, digits, '.', '-', or '_'".to_string(),
        );
    }

    // SECURITY: only allow https://github.com/<owner>/<repo> URLs and
    // reject anything that could be smuggled into a shell / git command.
    plugin_marketplace::validate_github_url(&source.command)
        .map_err(|e| format!("Invalid source URL: {}", e))?;

    let mut current = plugin_marketplace::read_user_sources();

    // Reject by id collision
    if current.iter().any(|s| s.id == source.id) {
        return Err(format!("User source id '{}' already exists", source.id));
    }
    // Reject by command (URL) collision
    let norm = |c: &str| c.trim().trim_end_matches('/').trim_end_matches(".git").to_string();
    let incoming = norm(&source.command);
    if current.iter().any(|s| norm(&s.command) == incoming) {
        return Err(format!(
            "User source with URL '{}' already exists",
            source.command
        ));
    }
    // Reject if URL matches a preset source
    let presets = crate::models::plugin_marketplace::get_preset_sources();
    if presets.iter().any(|s| norm(&s.command) == incoming) {
        return Err(format!(
            "该仓库 '{}' 已是系统预制数据源",
            source.command
        ));
    }

    current.push(source.clone());
    plugin_marketplace::write_user_sources(&current)?;
    Ok(source)
}

/// Allow only safe characters in a user-supplied source id so it cannot
/// be used as a path-traversal vector.
fn is_valid_source_id(id: &str) -> bool {
    let len = id.len();
    (1..=100).contains(&len)
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

/// Remove a user-added source by id. Returns the removed source so the
/// frontend can reconcile. No-op (Ok(None)) if the id is not a user-added
/// source (i.e. it's a preset).
#[tauri::command]
pub async fn remove_user_marketplace_source(source_id: String) -> Result<Option<PluginSource>, String> {
    log::info!("Removing user marketplace source: {}", source_id);

    let mut current = plugin_marketplace::read_user_sources();
    let pos = current.iter().position(|s| s.id == source_id);
    let Some(idx) = pos else {
        return Ok(None);
    };
    let removed = current.remove(idx);
    plugin_marketplace::write_user_sources(&current)?;
    Ok(Some(removed))
}

/// Update the `repo_type` of a user-added source (market ↔ res).
/// Preset sources cannot be modified. Returns the updated source.
#[tauri::command]
pub async fn update_source_repo_type(
    source_id: String,
    repo_type: String,
) -> Result<PluginSource, String> {
    log::info!(
        "Updating source repo_type: source_id={}, repo_type={}",
        source_id,
        repo_type
    );
    plugin_marketplace::update_user_source_repo_type(&source_id, &repo_type)
}

// ---------------------------------------------------------------------------
// Plugin version resolution
// ---------------------------------------------------------------------------

/// Resolve the display version for a plugin at `installed_path` by reading
/// manifest files from disk in priority order:
///
/// 1. `.claude-plugin/marketplace.json` → `version`
/// 2. `.claude-plugin/plugin.json` → `version`
/// 3. `package.json` → `version`
/// 4. `"unknown"` if none found
#[tauri::command]
pub async fn resolve_plugin_version(installed_path: String) -> Result<String, String> {
    log::info!("Resolving plugin version for: {}", installed_path);
    let dir = std::path::Path::new(&installed_path);
    if !dir.is_dir() {
        return Ok("unknown".to_string());
    }

    // Priority 1: .claude-plugin/marketplace.json
    let marketplace_json = dir.join(".claude-plugin").join("marketplace.json");
    if let Some(ver) = read_version_from_json(&marketplace_json) {
        return Ok(ver);
    }

    // Priority 2: .claude-plugin/plugin.json
    let plugin_json = dir.join(".claude-plugin").join("plugin.json");
    if let Some(ver) = read_version_from_json(&plugin_json) {
        return Ok(ver);
    }

    // Priority 3: package.json
    let package_json = dir.join("package.json");
    if let Some(ver) = read_version_from_json(&package_json) {
        return Ok(ver);
    }

    // Priority 4: fallback
    Ok("unknown".to_string())
}

/// Read the `version` field from a JSON file. Returns `None` if the file
/// doesn't exist, can't be parsed, or has no valid version string.
fn read_version_from_json(path: &std::path::Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let version = value.get("version")?.as_str()?;
    let trimmed = version.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

// ---------------------------------------------------------------------------
// Source notes commands (FEAT-020)
// ---------------------------------------------------------------------------

/// Return all source notes as a map of source_id → markdown string.
#[tauri::command]
pub async fn get_source_notes() -> HashMap<String, String> {
    log::info!("Getting source notes");
    plugin_marketplace::read_source_notes()
}

/// Save a note for a source. Passing an empty string deletes the note.
/// Returns `Ok(true)` on success.
#[tauri::command]
pub async fn save_source_note(
    source_id: String,
    note: String,
) -> Result<bool, String> {
    log::info!("Saving source note for: {}", source_id);
    let mut notes = plugin_marketplace::read_source_notes();
    if note.is_empty() {
        notes.remove(&source_id);
    } else {
        notes.insert(source_id, note);
    }
    plugin_marketplace::write_source_notes(&notes)?;
    Ok(true)
}
