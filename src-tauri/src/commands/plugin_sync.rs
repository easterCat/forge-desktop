// Plugin Sync to CLI Tool Commands
// Syncs plugin cache data to CLI tool plugin directories and manages sync records.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::services::cli_tools::CliToolManager;
use crate::services::plugin_marketplace::forge_home;

// -- Sync record types ----------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSyncRecord {
    pub plugin_id: String,
    pub cli_tool_key: String,
    pub source_id: String,
    pub plugin_name: String,
    pub source_repo_name: Option<String>,
    pub synced_at: String,
    pub target_path: String,
    /// Files that were synced (relative paths within the target directory).
    /// Used to cleanly remove synced content on unsync.
    pub synced_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSyncStatusResult {
    pub plugin_id: String,
    pub cli_tool_key: String,
    pub synced: bool,
    pub synced_at: Option<String>,
    pub target_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSyncResult {
    pub success: bool,
    pub target_path: Option<String>,
    pub error: Option<String>,
}

// -- Helpers --------------------------------------------------------------

fn sync_records_path() -> PathBuf {
    forge_home().join("plugins").join("sync_records.json")
}

fn load_sync_records() -> HashMap<String, PluginSyncRecord> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.get("plugin_sync_records").unwrap_or_default();
    }
    // Fallback
    let path = sync_records_path();
    if !path.exists() {
        return HashMap::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_sync_records(records: &HashMap<String, PluginSyncRecord>) -> Result<(), String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.put("plugin_sync_records", records);
    }
    // Fallback
    let path = sync_records_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create sync records dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize sync records: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write sync records: {}", e))
}

/// Expand a path that starts with `~` to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") || path.starts_with("~\\") {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(path.trim_start_matches("~/").trim_start_matches("~\\"))
    } else if path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(path)
    }
}

/// Scan the cache directory for a plugin by name.
/// The cache structure is `cache/<parent>/<plugin_name>/` where `<parent>`
/// is either a repo name or the plugin name itself (repo-is-plugin case).
fn find_plugin_in_cache(cache_root: &PathBuf, plugin_name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(cache_root).ok()?;
    for parent_entry in entries.filter_map(|e| e.ok()) {
        let candidate = parent_entry.path().join(plugin_name);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Recursively copy a directory. Returns only top-level entry names
/// (directories with `/` suffix, files by name) relative to `dst`.
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<Vec<String>, String> {
    copy_dir_inner(src, dst, dst)
}

fn copy_dir_inner(src: &PathBuf, dst: &PathBuf, root: &PathBuf) -> Result<Vec<String>, String> {
    let mut top_level_entries = Vec::new();
    if !src.exists() {
        return Err(format!("Source directory does not exist: {}", src.display()));
    }
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create target dir {}: {}", dst.display(), e))?;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read source dir: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip .git directory — plugin content only, no VCS history
        if file_name_str == ".git" {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            // Recurse to copy all files, but don't collect sub-entries
            copy_dir_inner(&src_path, &dst_path, root)?;
            // Record only the top-level directory name (with / suffix)
            let dir_name = format!("{}/", file_name_str);
            top_level_entries.push(dir_name);
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
            // Record only the file name (no path prefix)
            top_level_entries.push(file_name_str.to_string());
        }
    }
    Ok(top_level_entries)
}

/// Recursively remove entries listed in `synced_files` from the target directory.
/// Directory entries (paths that resolve to directories on disk) are removed with
/// `remove_dir_all`; file entries are removed with `remove_file`.
/// Also removes empty parent directories up to (but not including) the target root.
fn remove_synced_files(target_path: &PathBuf, synced_files: &[String]) -> Result<(), String> {
    for file_name in synced_files {
        let file_path = target_path.join(file_name);
        if !file_path.exists() {
            continue;
        }
        if file_path.is_dir() {
            fs::remove_dir_all(&file_path)
                .map_err(|e| format!("Failed to remove dir {}: {}", file_path.display(), e))?;
        } else {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to remove {}: {}", file_path.display(), e))?;
        }
    }
    // Clean up empty directories (bottom-up)
    cleanup_empty_dirs(target_path);
    Ok(())
}

/// Remove empty directories recursively from bottom up, stopping at the root.
fn cleanup_empty_dirs(dir: &PathBuf) {
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                cleanup_empty_dirs(&path);
            }
        }
        // Try to remove the directory if it's empty (ignore errors - dir not empty)
        let _ = fs::remove_dir(dir);
    }
}

// -- Claude Code installed_plugins.json management -------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledPluginsFile {
    version: u32,
    #[serde(default)]
    plugins: HashMap<String, Vec<InstalledPluginEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledPluginEntry {
    scope: String,
    #[serde(rename = "installPath")]
    install_path: String,
    version: String,
    #[serde(rename = "installedAt")]
    installed_at: String,
    #[serde(rename = "lastUpdated")]
    last_updated: String,
}

/// Path to Claude Code's installed_plugins.json
fn claude_installed_plugins_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json")
}

/// Load or create the installed_plugins.json file.
fn load_installed_plugins() -> InstalledPluginsFile {
    let path = claude_installed_plugins_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or(InstalledPluginsFile { version: 2, plugins: HashMap::new() })
    } else {
        InstalledPluginsFile { version: 2, plugins: HashMap::new() }
    }
}

/// Save the installed_plugins.json file.
fn save_installed_plugins(data: &InstalledPluginsFile) -> Result<(), String> {
    let path = claude_installed_plugins_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create plugins dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize installed_plugins: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write installed_plugins.json: {}", e))
}

/// Add a plugin entry to installed_plugins.json.
/// Key format: `<pluginName>@<repoName>`
fn register_installed_plugin(
    plugin_name: &str,
    repo_name: &str,
    install_path: &str,
) -> Result<(), String> {
    let key = format!("{}@{}", plugin_name, repo_name);
    let now = chrono::Utc::now().to_rfc3339();
    let entry = InstalledPluginEntry {
        scope: "user".to_string(),
        install_path: install_path.to_string(),
        version: "unknown".to_string(),
        installed_at: now.clone(),
        last_updated: now,
    };

    let mut data = load_installed_plugins();
    data.plugins.insert(key, vec![entry]);
    save_installed_plugins(&data)
}

/// Remove a plugin entry from installed_plugins.json.
fn unregister_installed_plugin(
    plugin_name: &str,
    repo_name: &str,
) -> Result<(), String> {
    let key = format!("{}@{}", plugin_name, repo_name);
    let mut data = load_installed_plugins();
    data.plugins.remove(&key);
    save_installed_plugins(&data)
}

// -- Claude Code settings.json enabledPlugins management ------------------

fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("settings.json")
}

/// Load Claude Code settings.json as a serde_json::Value.
fn load_claude_settings() -> serde_json::Value {
    let path = claude_settings_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    }
}

/// Save Claude Code settings.json.
fn save_claude_settings(data: &serde_json::Value) -> Result<(), String> {
    let path = claude_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write settings.json: {}", e))
}

/// Add a plugin to enabledPlugins in settings.json.
fn enable_claude_plugin(plugin_name: &str, repo_name: &str) -> Result<(), String> {
    let key = format!("{}@{}", plugin_name, repo_name);
    let mut settings = load_claude_settings();
    let enabled = settings
        .as_object_mut()
        .ok_or("settings.json is not an object")?
        .entry("enabledPlugins")
        .or_insert_with(|| serde_json::json!({}));
    let enabled_obj = enabled
        .as_object_mut()
        .ok_or("enabledPlugins is not an object")?;
    enabled_obj.insert(key, serde_json::json!(true));
    save_claude_settings(&settings)
}

/// Remove a plugin from enabledPlugins in settings.json.
fn disable_claude_plugin(plugin_name: &str, repo_name: &str) -> Result<(), String> {
    let key = format!("{}@{}", plugin_name, repo_name);
    let mut settings = load_claude_settings();
    if let Some(enabled) = settings.get_mut("enabledPlugins") {
        if let Some(obj) = enabled.as_object_mut() {
            obj.remove(&key);
        }
    }
    save_claude_settings(&settings)
}

// -- Tauri Commands -------------------------------------------------------

/// Sync a plugin's cache data to the specified CLI tool's plugin directory.
/// Copies all files from `$FORGE_HOME/plugins/cache/<plugin_name>/`
/// to the CLI tool's plugin directory and records the sync in `$FORGE_HOME/plugins/sync_records.json`.
#[tauri::command]
pub async fn sync_plugin_to_cli_tool(
    plugin_id: String,
    cli_tool_key: String,
    source_id: String,
    plugin_name: String,
    source_repo_name: Option<String>,
    plugin_version: Option<String>,
) -> PluginSyncResult {
    log::info!(
        "Syncing plugin {}/{} to CLI tool {}",
        source_id,
        plugin_name,
        cli_tool_key
    );

    // Resolve the CLI tool's plugin directory
    let tools = CliToolManager::get_supported_tools();
    let tool = tools.iter().find(|t| t.key == cli_tool_key);
    let plugin_dir_str = match tool.and_then(|t| t.plugin_dir.as_deref()) {
        Some(dir) => dir,
        None => {
            return PluginSyncResult {
                success: false,
                target_path: None,
                error: Some(format!("CLI tool '{}' not found or has no plugin directory", cli_tool_key)),
            };
        }
    };

    let target_base = expand_tilde(plugin_dir_str);

    // Source: plugin cache directory
    // Sub-plugin: cache/<repo_name>/<plugin_name>/
    // Repo-is-plugin: cache/<plugin_name>/<plugin_name>/
    let cache_root = forge_home().join("plugins").join("cache");
    let repo_name = source_repo_name.as_deref().unwrap_or(&source_id);
    let sub_path = cache_root.join(repo_name).join(&plugin_name);
    let root_path = cache_root.join(&plugin_name).join(&plugin_name);
    let sub_path_exists = sub_path.exists();
    let mut source_path = if sub_path_exists {
        sub_path
    } else if root_path.exists() {
        root_path
    } else {
        // Neither path matched. Scan the cache directory for the plugin
        // name — the cache may use a different parent directory name
        // (e.g. when source_id was empty or mismatched after uninstall).
        find_plugin_in_cache(&cache_root, &plugin_name)
            .unwrap_or_else(|| root_path) // fall back to root_path for the error message
    };

    // Use repo name (e.g. "claude-plugins-official") as the source directory,
    // falling back to source_id if not provided.
    // If the source_path was found via cache scan, derive the dir name from
    // the actual parent directory in the cache.
    let source_dir_name = if sub_path_exists || source_repo_name.is_some() {
        repo_name.to_string()
    } else if source_path.exists() {
        // Derive from the actual cache path: cache/<parent>/<plugin_name>/
        source_path.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| source_id.clone())
    } else {
        source_id.clone()
    };
    // Include version as subdirectory, defaulting to "unknown"
    let version_dir = plugin_version.as_deref().unwrap_or("unknown");
    let target_path = target_base.join("cache").join(&source_dir_name).join(&plugin_name).join(version_dir);

    if !source_path.exists() {
        return PluginSyncResult {
            success: false,
            target_path: None,
            error: Some(format!(
                "Plugin cache directory not found: {}",
                source_path.display()
            )),
        };
    }

    // Check if already synced
    let records = load_sync_records();
    if let Some(existing) = records.get(&plugin_id) {
        if existing.cli_tool_key == cli_tool_key {
            return PluginSyncResult {
                success: true,
                target_path: Some(existing.target_path.clone()),
                error: None,
            };
        }
    }

    // Copy files
    let synced_files = match copy_dir_recursive(&source_path, &target_path) {
        Ok(files) => files,
        Err(e) => {
            return PluginSyncResult {
                success: false,
                target_path: None,
                error: Some(e),
            };
        }
    };

    // Record the sync
    let now = chrono::Utc::now().to_rfc3339();
    let repo_name_for_register = source_repo_name.clone().unwrap_or_else(|| source_dir_name.clone());
    let plugin_name_for_register = plugin_name.clone();
    let record = PluginSyncRecord {
        plugin_id: plugin_id.clone(),
        cli_tool_key: cli_tool_key.clone(),
        source_id,
        plugin_name,
        source_repo_name,
        synced_at: now,
        target_path: target_path.to_string_lossy().to_string(),
        synced_files,
    };

    let mut records = load_sync_records();
    records.insert(plugin_id, record);
    if let Err(e) = save_sync_records(&records) {
        log::warn!("Failed to save sync record: {}", e);
    }

    // For Claude Code: register the plugin in installed_plugins.json and settings.json
    if cli_tool_key == "claude-code" {
        if let Err(e) = register_installed_plugin(&plugin_name_for_register, &repo_name_for_register, &target_path.to_string_lossy()) {
            log::warn!("Failed to register installed plugin: {}", e);
        }
        if let Err(e) = enable_claude_plugin(&plugin_name_for_register, &repo_name_for_register) {
            log::warn!("Failed to enable plugin in settings.json: {}", e);
        }
    }

    PluginSyncResult {
        success: true,
        target_path: Some(target_path.to_string_lossy().to_string()),
        error: None,
    }
}

/// Remove previously synced plugin data from a CLI tool's plugin directory.
/// Reads `$FORGE_HOME/plugins/sync_records.json` to locate the synced content, removes it, and
/// clears the sync record.
#[tauri::command]
pub async fn unsync_plugin_from_cli_tool(
    plugin_id: String,
    cli_tool_key: String,
) -> PluginSyncResult {
    log::info!(
        "Unsyncing plugin {} from CLI tool {}",
        plugin_id,
        cli_tool_key
    );

    let mut records = load_sync_records();
    // Try direct lookup first, then fall back to matching by plugin name
    // (the sourceId may have changed, e.g. "agent-skills" → "addyosmani").
    let mut record = records.get(&plugin_id).filter(|r| r.cli_tool_key == cli_tool_key).cloned();
    let mut record_key = if record.is_some() { Some(plugin_id.clone()) } else { None };

    if record.is_none() {
        let plugin_name = plugin_id.split("::").last().unwrap_or(&plugin_id);
        if let Some((key, rec)) = records.iter().find(|(_, r)| r.plugin_name == plugin_name && r.cli_tool_key == cli_tool_key) {
            record = Some(rec.clone());
            record_key = Some(key.clone());
        }
    }

    // Resolve the target path: prefer the record, fall back to deriving
    // it from the CLI tool's plugin_dir + plugin_name.
    let target_path = if let Some(ref rec) = record {
        PathBuf::from(&rec.target_path)
    } else {
        // No sync record — derive the expected target path from the CLI tool config
        let tools = CliToolManager::get_supported_tools();
        let plugin_dir = tools.iter()
            .find(|t| t.key == cli_tool_key)
            .and_then(|t| t.plugin_dir.as_deref());
        match plugin_dir {
            Some(dir) => {
                // Extract source_id and plugin_name from plugin_id (format: "sourceId::pluginName")
                let mut parts = plugin_id.splitn(2, "::");
                let source_id = parts.next().unwrap_or("unknown");
                let plugin_name = parts.next().unwrap_or(&plugin_id);
                expand_tilde(dir).join("cache").join(source_id).join(plugin_name).join("unknown")
            }
            None => {
                return PluginSyncResult {
                    success: false,
                    target_path: None,
                    error: Some("No sync record found and CLI tool has no plugin directory".to_string()),
                };
            }
        }
    };

    // Remove the entire synced directory. This is more reliable than
    // removing individual files (which can fail if the record format
    // changed or subdirectories were involved). The target directory
    // was created by us during sync, so it's safe to remove entirely.
    if target_path.exists() {
        if let Err(e) = fs::remove_dir_all(&target_path) {
            log::warn!("Error removing synced directory {}: {}", target_path.display(), e);
            // Fallback: try removing individual files if we have a record
            if let Some(ref rec) = record {
                if let Err(e2) = remove_synced_files(&target_path, &rec.synced_files) {
                    log::warn!("Error removing synced files: {}", e2);
                }
            }
        }
        // Clean up the parent source directory if it's now empty
        if let Some(parent) = target_path.parent() {
            let _ = fs::remove_dir(parent);
        }
    }

    // For Claude Code: unregister the plugin from installed_plugins.json and settings.json
    if cli_tool_key == "claude-code" {
        let mut parts = plugin_id.splitn(2, "::");
        let source_id = parts.next().unwrap_or("unknown");
        let plugin_name = parts.next().unwrap_or(&plugin_id);
        let repo_name = record.as_ref()
            .and_then(|r| r.source_repo_name.as_deref())
            .unwrap_or(source_id);
        if let Err(e) = unregister_installed_plugin(plugin_name, repo_name) {
            log::warn!("Failed to unregister installed plugin: {}", e);
        }
        if let Err(e) = disable_claude_plugin(plugin_name, repo_name) {
            log::warn!("Failed to disable plugin in settings.json: {}", e);
        }
    }

    // Remove the sync record if it exists
    if let Some(key) = record_key {
        records.remove(&key);
        if let Err(e) = save_sync_records(&records) {
            log::warn!("Failed to save sync records after removal: {}", e);
        }
    }

    PluginSyncResult {
        success: true,
        target_path: Some(target_path.to_string_lossy().to_string()),
        error: None,
    }
}

/// Batch query sync status for multiple plugins.
/// Returns the sync status for each requested plugin_id.
///
/// Records are stored with 2-part keys (`sourceId::pluginName`) but the
/// sourceId may have changed since the record was created (e.g. a preset
/// source was renamed).  We first try an exact/prefix match; if that fails
/// we fall back to matching by plugin_name alone so stale records are still
/// found.  On fallback we return the *queried* id so the frontend can
/// construct the correct 3-part sync key.
#[tauri::command]
pub async fn get_plugin_sync_status(
    plugin_ids: Vec<String>,
) -> Vec<PluginSyncStatusResult> {
    let records = load_sync_records();
    let mut results = Vec::new();
    for id in &plugin_ids {
        let prefix = format!("{}::", id);
        let matched: Vec<(&String, &PluginSyncRecord)> = records
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix) || *k == id)
            .collect();

        if matched.is_empty() {
            // Fallback: the sourceId may have changed (e.g. preset renamed from
            // "agent-skills" to "addyosmani").  Try matching by plugin name —
            // the second segment of the "sourceId::pluginName" key.
            let plugin_name = id.split("::").last().unwrap_or(id);
            let fallback: Vec<(&String, &PluginSyncRecord)> = records
                .iter()
                .filter(|(_, r)| r.plugin_name == plugin_name)
                .collect();

            if fallback.is_empty() {
                results.push(PluginSyncStatusResult {
                    plugin_id: id.clone(),
                    cli_tool_key: String::new(),
                    synced: false,
                    synced_at: None,
                    target_path: None,
                });
            } else {
                // Use the *queried* id so the frontend can construct the
                // correct 3-part sync key with the current sourceId.
                for (_key, record) in fallback {
                    results.push(PluginSyncStatusResult {
                        plugin_id: id.clone(),
                        cli_tool_key: record.cli_tool_key.clone(),
                        synced: true,
                        synced_at: Some(record.synced_at.clone()),
                        target_path: Some(record.target_path.clone()),
                    });
                }
            }
        } else {
            for (key, record) in matched {
                results.push(PluginSyncStatusResult {
                    plugin_id: key.clone(),
                    cli_tool_key: record.cli_tool_key.clone(),
                    synced: true,
                    synced_at: Some(record.synced_at.clone()),
                    target_path: Some(record.target_path.clone()),
                });
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn copy_dir_inner_returns_only_top_level_entries() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        // Create nested structure:
        //   root_file.txt
        //   subdir/
        //     nested_file.txt
        //     deep/
        //       deep_file.txt
        fs::write(src.path().join("root_file.txt"), "root").unwrap();
        fs::create_dir_all(src.path().join("subdir/deep")).unwrap();
        fs::write(src.path().join("subdir/nested_file.txt"), "nested").unwrap();
        fs::write(src.path().join("subdir/deep/deep_file.txt"), "deep").unwrap();

        let result = copy_dir_inner(
            &src.path().to_path_buf(),
            &dst.path().to_path_buf(),
            &dst.path().to_path_buf(),
        ).unwrap();

        // Should contain root_file.txt and subdir/ but NOT nested paths
        assert!(result.contains(&"root_file.txt".to_string()));
        assert!(result.contains(&"subdir/".to_string()));
        assert!(!result.iter().any(|f| f.contains("nested_file")));
        assert!(!result.iter().any(|f| f.contains("deep_file")));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn copy_dir_inner_still_copies_all_files() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();

        fs::write(src.path().join("a.txt"), "a").unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub/b.txt"), "b").unwrap();

        copy_dir_inner(
            &src.path().to_path_buf(),
            &dst.path().to_path_buf(),
            &dst.path().to_path_buf(),
        ).unwrap();

        // All files should actually be copied on disk
        assert!(dst.path().join("a.txt").exists());
        assert!(dst.path().join("sub/b.txt").exists());
    }

    #[test]
    fn remove_synced_files_handles_directory_entries() {
        let dir = TempDir::new().unwrap();

        // Set up: some files and a subdirectory
        fs::write(dir.path().join("file.txt"), "content").unwrap();
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir/nested.txt"), "nested").unwrap();

        let synced_files = vec![
            "file.txt".to_string(),
            "subdir/".to_string(),
        ];

        remove_synced_files(&dir.path().to_path_buf(), &synced_files).unwrap();

        assert!(!dir.path().join("file.txt").exists());
        assert!(!dir.path().join("subdir").exists());
    }

    #[test]
    fn remove_synced_files_handles_old_full_path_format() {
        let dir = TempDir::new().unwrap();

        // Simulate old format: full relative paths
        fs::create_dir_all(dir.path().join("subdir/deep")).unwrap();
        fs::write(dir.path().join("subdir/deep/file.txt"), "content").unwrap();

        let synced_files = vec![
            "subdir/deep/file.txt".to_string(),
        ];

        remove_synced_files(&dir.path().to_path_buf(), &synced_files).unwrap();

        assert!(!dir.path().join("subdir/deep/file.txt").exists());
    }
}
