// Plugin Marketplace Services - reads real data from
// `.claude-plugin/marketplace.json` (synced by scripts/plugins/install.mjs).

use crate::models::{
    MarketplacePlugin, PluginSource, PluginInstallResult, PluginUpdateResult,
    PluginInstallSource,
    SourceStatus, SourceInstallResult, SourceInstallProgress,
};
use crate::models::plugin_marketplace::extract_repo_name_from_url;
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// -- Manifest types (mirror of `.claude-plugin/marketplace.json`) ---------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketplaceManifest {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub last_sync_at: Option<String>,
    #[serde(default)]
    pub sources: std::collections::HashMap<String, ManifestSource>,
    #[serde(default)]
    pub plugins: std::collections::HashMap<String, Vec<ManifestPlugin>>,
    #[serde(default)]
    pub removed: Vec<RemovedEntry>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestSource {
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub external: bool,
    #[serde(default)]
    pub last_sync_at: Option<String>,
    #[serde(default)]
    pub plugin_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPlugin {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: serde_json::Value,
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub installed_path: String,
    #[serde(default)]
    pub external: bool,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub install_mode: String,
    #[serde(default)]
    pub manifest: Option<serde_json::Value>,
    /// Timestamp when user clicked Install (written by install_plugin).
    /// Not written during CLI --all sync. Re-installing updates this value.
    #[serde(default)]
    pub installed_at: Option<String>,
    /// User-controlled enable/disable flag. Persisted in
    /// `.claude-plugin/marketplace.json` so toggling survives app
    /// restarts and CLI syncs. Skipped (treated as `false`) when the
    /// entry was first written by `scripts/plugins/install.mjs`.
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovedEntry {
    pub name: String,
    pub source: String,
    pub removed_at: String,
    #[serde(default)]
    pub reason: String,
}

// -- Local project paths --------------------------------------------------

/// Resolve the Forge home directory (`~/.forge` by default, overridable
/// with the `FORGE_HOME` environment variable). All installed content
/// (marketplace manifest, plugin files, agents, rules, skills, prompts,
/// logs, db) lives under this directory.
pub fn forge_home() -> PathBuf {
    if let Ok(raw) = std::env::var("FORGE_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let expanded = if trimmed == "~" {
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
            } else if trimmed.starts_with("~/") || trimmed.starts_with("~\\") {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.join(trimmed.trim_start_matches("~/").trim_start_matches("~\\"))
            } else if std::path::Path::new(trimmed).is_absolute() {
                PathBuf::from(trimmed)
            } else {
                // Relative paths are anchored to the current working directory.
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(trimmed)
            };
            return expanded;
        }
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".forge")
}

/// Backwards-compatible alias for `forge_home()`. The previous
/// implementation walked up from cwd looking for `.claude-plugin/`; that
/// project-local storage has been replaced by the user-level FORGE_HOME
/// directory, so any legacy callers now resolve to the same global
/// location.
pub fn project_root() -> PathBuf {
    forge_home()
}

pub fn manifest_path() -> PathBuf {
    forge_home().join("plugins").join("marketplace.json")
}

pub fn plugins_dir() -> PathBuf {
    forge_home().join("plugins")
}

/// Detect CLI tool associations from a plugin directory by looking for
/// well-known marker directories (e.g. `.claude-plugin/`, `.cursor-plugin/`).
/// Returns all matching CLI tool keys, or empty Vec if no markers found.
pub fn detect_cli_tool_keys(plugin_dir: &std::path::Path) -> Vec<String> {
    const MARKERS: &[(&str, &str)] = &[
        (".claude-plugin", "claude-code"),
        (".cursor-plugin", "cursor"),
        (".codex-plugin", "codex"),
        (".opencode", "opencode"),
        (".openclaw", "openclaw"),
        (".hermes", "hermes"),
        (".gemini", "gemini-cli"),
    ];
    MARKERS.iter()
        .filter(|(marker, _)| plugin_dir.join(marker).is_dir())
        .map(|(_, key)| key.to_string())
        .collect()
}

/// Backward-compatible single-key helper: returns the first detected key.
pub fn detect_cli_tool_key(plugin_dir: &std::path::Path) -> Option<String> {
    detect_cli_tool_keys(plugin_dir).into_iter().next()
}

/// Directory for marketplace source repositories (cloned GitHub repos).
/// Structure: forge/plugins/marketplace/<source_id>/
/// 
/// IMPORTANT: Source repositories are stored here, NOT in forge/marketplace.
/// This directory contains the raw cloned GitHub repositories with their
/// full plugin source code. The marketplace.json manifest references plugins
/// stored here.
pub fn marketplace_sources_dir() -> PathBuf {
    forge_home().join("plugins").join("marketplace")
}

pub fn installed_plugins_path() -> PathBuf {
    forge_home().join("plugins").join("installed_plugins.json")
}

// ---------------------------------------------------------------------------
// User-added source registry (FEAT-019)
// Persisted at `$FORGE_HOME/plugins/user_sources.json`. Lives next to the
// other plugin-scope config files (`plugins/`), not under `marketplace/`,
// because it configures the *plugin* feature surface itself. Survives app
// restarts so users do not have to re-add custom GitHub repos every time.
// ---------------------------------------------------------------------------

pub fn user_sources_path() -> PathBuf {
    plugins_dir().join("user_sources.json")
}

// ---------------------------------------------------------------------------
// Source notes (FEAT-020)
// User-authored Markdown notes for each source, persisted to
// `$FORGE_HOME/plugins/source_notes.json`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceNotesRegistry {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub notes: HashMap<String, String>,
}

pub fn source_notes_path() -> PathBuf {
    plugins_dir().join("source_notes.json")
}

/// Read all source notes from disk.
/// Returns an empty map if the file does not exist or cannot be parsed.
pub fn read_source_notes() -> HashMap<String, String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        if let Some(reg) = kv.get::<SourceNotesRegistry>("plugin_source_notes") {
            return reg.notes;
        }
    }
    // Fallback
    let path = source_notes_path();
    if !path.exists() {
        return HashMap::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<SourceNotesRegistry>(&content) {
            Ok(reg) => reg.notes,
            Err(e) => {
                log::warn!("Failed to parse source_notes.json: {}", e);
                HashMap::new()
            }
        },
        Err(e) => {
            log::warn!("Failed to read source_notes.json: {}", e);
            HashMap::new()
        }
    }
}

/// Write all source notes to disk.
pub fn write_source_notes(notes: &HashMap<String, String>) -> Result<(), String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        let reg = SourceNotesRegistry {
            version: "1".to_string(),
            notes: notes.clone(),
        };
        return kv.put("plugin_source_notes", &reg);
    }
    // Fallback
    let path = source_notes_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create plugins dir: {}", e))?;
    }
    let reg = SourceNotesRegistry {
        version: "1".to_string(),
        notes: notes.clone(),
    };
    let json = serde_json::to_string_pretty(&reg)
        .map_err(|e| format!("Failed to serialize source_notes.json: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write source_notes.json: {}", e))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserSourceRegistry {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub sources: Vec<PluginSource>,
}

pub fn read_user_sources() -> Vec<PluginSource> {
    let mut sources: Vec<PluginSource> = if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        kv.get::<UserSourceRegistry>("plugin_user_sources")
            .map(|reg| reg.sources)
            .unwrap_or_default()
    } else {
        let path = user_sources_path();
        if !path.exists() {
            return Vec::new();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<UserSourceRegistry>(&content) {
                Ok(reg) => reg.sources,
                Err(e) => {
                    log::error!("Failed to parse user_sources.json: {}", e);
                    return Vec::new();
                }
            },
            Err(e) => {
                log::error!("Failed to read user_sources.json: {}", e);
                return Vec::new();
            }
        }
    };
    // Compute plugin counts for user sources using their repoName
    for s in &mut sources {
        // res-type repos are single-plugin: always 1
        if s.repo_type.as_deref() == Some("res") {
            s.plugin_count = Some(1);
            continue;
        }
        let repo_name = s.repo_name.clone().unwrap_or_else(|| s.id.clone());
        let source_path = marketplace_sources_dir().join(&repo_name);
        if source_path.exists() && is_git_repo(&source_path) {
            if let Some(count) = read_source_marketplace_json(&source_path) {
                s.plugin_count = Some(count);
            } else {
                // Fallback: count plugin directories
                let count = count_plugin_dirs(&source_path);
                s.plugin_count = Some(count);
            }
        }
    }
    sources
}

pub fn write_user_sources(sources: &[PluginSource]) -> Result<(), String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        let reg = UserSourceRegistry {
            version: "1".to_string(),
            sources: sources.to_vec(),
        };
        return kv.put("plugin_user_sources", &reg);
    }
    // Fallback
    let path = user_sources_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create marketplace dir: {}", e))?;
    }
    let reg = UserSourceRegistry {
        version: "1".to_string(),
        sources: sources.to_vec(),
    };
    let json = serde_json::to_string_pretty(&reg)
        .map_err(|e| format!("Failed to serialize user_sources.json: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write user_sources.json: {}", e))?;
    Ok(())
}

/// Update the `repo_type` field of a user-added source.
/// Only user-added sources may be modified; preset sources are rejected.
/// When switching to `"res"`, `plugin_count` is forced to `1`.
/// Returns the updated `PluginSource` on success.
pub fn update_user_source_repo_type(
    source_id: &str,
    repo_type: &str,
) -> Result<PluginSource, String> {
    if repo_type != "market" && repo_type != "res" {
        return Err(format!(
            "Invalid repo_type '{}': must be 'market' or 'res'",
            repo_type
        ));
    }

    // Reject modifications to preset sources
    let presets = get_preset_sources();
    if presets.iter().any(|s| s.id == source_id) {
        return Err(format!(
            "Cannot change repo_type of preset source '{}'",
            source_id
        ));
    }

    let mut sources = read_user_sources();
    let source = sources
        .iter_mut()
        .find(|s| s.id == source_id)
        .ok_or_else(|| format!("User source '{}' not found", source_id))?;

    source.repo_type = Some(repo_type.to_string());
    if repo_type == "res" {
        source.plugin_count = Some(1);
    }

    let updated = source.clone();
    write_user_sources(&sources)?;
    Ok(updated)
}

// -- Manifest IO ----------------------------------------------------------

pub fn read_manifest() -> MarketplaceManifest {
    // Try KV store first; fall back to legacy JSON file during migration window
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        if let Some(manifest) = kv.get::<MarketplaceManifest>("marketplace_manifest") {
            return manifest;
        }
    }
    // Fallback: read from JSON file (handles first-launch before DB is ready)
    let path = manifest_path();
    if !path.exists() {
        log::warn!("marketplace.json not found at {}", path.display());
        return MarketplaceManifest::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            log::error!("Failed to parse marketplace.json: {}", e);
            MarketplaceManifest::default()
        }),
        Err(e) => {
            log::error!("Failed to read marketplace.json: {}", e);
            MarketplaceManifest::default()
        }
    }
}

pub fn write_manifest(manifest: &MarketplaceManifest) -> Result<(), String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.put("marketplace_manifest", manifest);
    }
    // Fallback: write to JSON file
    let path = manifest_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude-plugin dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;
    Ok(())
}

pub fn get_preset_sources() -> Vec<PluginSource> {
    crate::models::plugin_marketplace::get_preset_sources()
}

// ---------------------------------------------------------------------------
// Source plugin scanning — primary: .claude-plugin/marketplace.json,
// fallback: filesystem scan of cloned repo
// ---------------------------------------------------------------------------

/// Parse a `MarketplacePlugin.source` block from a marketplace manifest
/// into a structured `PluginInstallSource`.
///
/// Three shapes are recognised:
/// - Object form: `{"source":"git-subdir", "url":"...", "path":"...", "ref":"...", "sha":"..."}`
///   → `kind = "git-subdir"`.
/// - Object form: `{"source":"url", "url":"...", "sha":"..."}` (no `path`)
///   → `kind = "url"`.
/// - String form: `"./plugins/<name>"` (a relative path inside the
///   marketplace repo) → `kind = "local"`. We synthesise a `url` from the
///   manifest-level `repository` field for completeness, even though
///   install_plugin will short-circuit to a local copy and never use it.
///
/// Returns `None` for missing or unrecognised input (so the caller can
/// fall back to the local-copy path in `install_plugin`).
fn parse_install_source(value: Option<&serde_json::Value>, repo_url: Option<&str>) -> Option<PluginInstallSource> {
    let v = value?;
    if let Some(obj) = v.as_object() {
        let kind = obj
            .get("source")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let url = obj
            .get("url")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let path = obj
            .get("path")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let r#ref = obj
            .get("ref")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let sha = obj
            .get("sha")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        // Normalise the kind to one of: "local" | "git-subdir" | "url".
        // Anything with both `url` + `path` is a subdir clone; with just
        // `url` it's a whole-repo clone; without `url` it's a local copy.
        let normalised = match kind.as_str() {
            "git-subdir" if !path.is_empty() => "git-subdir",
            "url" | "git-subdir" | "" if !url.is_empty() => "url",
            _ => "local",
        };

        // Use the manifest-level `repository` as a fallback URL for
        // string-form sources (e.g. `"./plugins/agent-sdk-dev"`).
        let resolved_url = if url.is_empty() {
            repo_url.unwrap_or("").to_string()
        } else {
            url
        };

        return Some(PluginInstallSource {
            kind: normalised.to_string(),
            url: resolved_url,
            path,
            r#ref,
            sha,
        });
    }
    if let Some(s) = v.as_str() {
        // String form: relative path inside the marketplace source repo.
        return Some(PluginInstallSource {
            kind: "local".to_string(),
            url: repo_url.unwrap_or("").to_string(),
            path: s.to_string(),
            r#ref: String::new(),
            sha: String::new(),
        });
    }
    None
}

/// Read the `plugins` array from `<source_path>/.claude-plugin/marketplace.json`
/// and convert each entry to a `MarketplacePlugin`. Returns `None` if the file
/// is missing, unreadable, or unparseable (callers should fall back to the
/// filesystem scan in that case).
///
/// **Special case — empty marketplace, repo is a plugin**: if the
/// `plugins` array is empty but the repository root has plugin markers
/// (SKILL.md, manifest.json, plugin.json, etc.) we synthesize a single
/// `MarketplacePlugin` representing the repo itself, so the marketplace
/// tab stops showing an empty grid for single-plugin repos.
fn read_source_marketplace_plugins(source_path: &PathBuf, source_id: &str) -> Option<Vec<MarketplacePlugin>> {
    let json_path = source_path.join(".claude-plugin").join("marketplace.json");
    if !json_path.exists() {
        log::debug!(
            "No .claude-plugin/marketplace.json at {} — falling back to filesystem scan",
            source_path.display()
        );
        return None;
    }

    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read {}: {}", json_path.display(), e);
            return None;
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse {}: {}", json_path.display(), e);
            return None;
        }
    };

    let plugins_array = match value.get("plugins").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            log::warn!("No 'plugins' array in {}", json_path.display());
            return None;
        }
    };

    let repo_url = value.get("repository")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let plugins: Vec<MarketplacePlugin> = plugins_array
        .iter()
        .filter_map(|p| {
            let name = p.get("name")?.as_str()?.to_string();

            let description = p.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            // author may be { name, email? } or a plain string
            let author = p.get("author").and_then(|a| {
                if let Some(name_str) = a.as_str() {
                    Some(name_str.to_string())
                } else if let Some(obj) = a.as_object() {
                    obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            });

            let category = p.get("category")
                .and_then(|v| v.as_str())
                .map(|s| vec![s.to_string()])
                .unwrap_or_default();

            let homepage = p.get("homepage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // source: either { source, url, path?, ref?, sha? } or a string like "./plugins/..."
            let repository = p.get("source").and_then(|s| {
                if let Some(url) = s.get("url").and_then(|v| v.as_str()) {
                    Some(url.to_string())
                } else if let Some(_path_str) = s.as_str() {
                    // Relative path like "./plugins/agent-sdk-dev" — combine with repo_url
                    repo_url.clone()
                } else {
                    None
                }
            });

            // Capture the raw `source` block so install_plugin can decide
            // whether the plugin is local (already on disk) or remote
            // (needs a `git clone`). Without this, plugins whose
            // `source` is `{url, sha}` pointing at a completely different
            // GitHub repo — e.g. `superpowers` from `obra/superpowers.git`
            // — were silently un-installable because install_plugin only
            // looked at `<marketplace-repo>/plugins/<name>/`.
            let install_source = parse_install_source(p.get("source"), repo_url.as_deref());

            // Resolve a plausible on-disk install path so the Install button works.
            // Try plugins/<name>/ first (claude-plugins-official layout), then <name>/ (root layout),
            // then the source root itself (repo IS the plugin).
            let install_path: Option<String> = if source_path.join("plugins").join(&name).exists() {
                Some(source_path.join("plugins").join(&name).to_string_lossy().to_string())
            } else if source_path.join(&name).exists() {
                Some(source_path.join(&name).to_string_lossy().to_string())
            } else if repo_root_has_plugin_marker(source_path) {
                // The marketplace source repo itself is the plugin (no subdirectory layout).
                Some(source_path.to_string_lossy().to_string())
            } else {
                // Best-effort guess; install handler will fail gracefully if missing
                Some(source_path.join("plugins").join(&name).to_string_lossy().to_string())
            };

            // Determine on-disk install state: a plugin is installed iff
            // its cache directory exists under
            // `$FORGE_HOME/plugins/cache/<repo_name>/<plugin_name>/`.
            // This is the single source of truth used by `install_plugin`
            // and `uninstall_plugin`, so we mirror it here so the
            // Marketplace tab survives an app restart without forgetting
            // what the user already installed.
            let repo_name = resolve_repo_name(source_id);
            let cache_dir = forge_home()
                .join("plugins")
                .join("cache")
                .join(&repo_name)
                .join(&name);
            let is_installed = cache_dir.is_dir();
            // Detect CLI tool association from the installed plugin directory.
            let cli_tool_keys = if is_installed {
                detect_cli_tool_keys(&cache_dir)
            } else {
                install_path.as_deref().map(std::path::Path::new).map(detect_cli_tool_keys).unwrap_or_default()
            };
            let cli_tool_key = cli_tool_keys.first().cloned();

            Some(MarketplacePlugin {
                id: name.clone(),
                source_id: source_id.to_string(),
                name,
                description,
                long_description: None,
                author,
                version: None,
                latest_version: None,
                has_update: Some(false),
                categories: category,
                tags: vec![],
                install_command: None,
                install_path,
                repository,
                homepage,
                license: None,
                stars: None,
                downloads: None,
                last_updated: None,
                is_installed,
                disabled: false,
                install_source,
                cli_tool_key,
                cli_tool_keys,
            })
        })
        .collect();

    // Empty marketplace + repo is itself a plugin → synthesize a single
    // entry for the repo. The count fix in `read_source_marketplace_json`
    // is mirrored here so the grid doesn't show an empty placeholder.
    if plugins.is_empty() && repo_root_has_plugin_marker(source_path) {
        let url = resolve_source_repo_url(source_id);
        let synthesized = synthesize_repo_root_plugin(source_path, source_id, &url);
        log::info!(
            "Synthesized 1 plugin entry for repo {} (empty marketplace.json + plugin marker at root)",
            source_path.display()
        );
        return Some(vec![synthesized]);
    }

    log::info!(
        "Read {} plugins from {}/.claude-plugin/marketplace.json",
        plugins.len(),
        source_path.display()
    );
    Some(plugins)
}

/// Read a single plugin's metadata from `.claude-plugin/plugin.json` for
/// res-type (single-plugin) repositories.
///
/// When a `res`-type repo doesn't ship a `marketplace.json`, this function
/// reads the plugin info from `.claude-plugin/plugin.json` instead. Returns
/// `None` if the file doesn't exist or is malformed.
fn read_res_plugin_from_json(source_path: &PathBuf, source_id: &str) -> Option<MarketplacePlugin> {
    let json_path = source_path.join(".claude-plugin").join("plugin.json");
    if !json_path.exists() {
        log::debug!(
            "No .claude-plugin/plugin.json at {} — cannot read res plugin",
            source_path.display()
        );
        return None;
    }

    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read {}: {}", json_path.display(), e);
            return None;
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse {}: {}", json_path.display(), e);
            return None;
        }
    };

    let plugin_name = source_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| source_id.to_string());

    let description = value.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let version = value.get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let author = value.get("author").and_then(|a| {
        if let Some(name_str) = a.as_str() {
            Some(name_str.to_string())
        } else if let Some(obj) = a.as_object() {
            obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
        } else {
            None
        }
    });

    let repository = value.get("repository")
        .or_else(|| value.get("repo"))
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    let homepage = value.get("homepage")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let license = value.get("license")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let categories = value.get("category")
        .and_then(|v| v.as_str())
        .map(|s| vec![s.to_string()])
        .or_else(|| {
            value.get("categories").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str().map(|s| s.to_string()))
                    .collect()
            })
        })
        .unwrap_or_else(|| vec!["Claude Code Plugin".to_string()]);

    let tags = value.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Check install state
    let repo_name = resolve_repo_name(source_id);
    let cache_dir = forge_home()
        .join("plugins")
        .join("cache")
        .join(&repo_name)
        .join(&plugin_name);
    let is_installed = cache_dir.is_dir();
    let cli_tool_keys = detect_cli_tool_keys(if is_installed { &cache_dir } else { source_path });
    let cli_tool_key = cli_tool_keys.first().cloned();

    Some(MarketplacePlugin {
        id: plugin_name.clone(),
        source_id: source_id.to_string(),
        name: plugin_name,
        description,
        long_description: None,
        author,
        version: version.clone(),
        latest_version: version,
        has_update: Some(false),
        categories,
        tags,
        install_command: None,
        install_path: Some(source_path.to_string_lossy().to_string()),
        repository,
        homepage,
        license,
        stars: None,
        downloads: None,
        last_updated: None,
        is_installed,
        disabled: false,
        install_source: Some(PluginInstallSource {
            kind: "url".to_string(),
            url: String::new(),
            path: String::new(),
            r#ref: String::new(),
            sha: String::new(),
        }),
        cli_tool_key,
        cli_tool_keys,
    })
}

/// Scan a cloned marketplace source repository for available plugins.
///
/// **Primary path**: reads the authoritative `plugins` array from
/// `<source_path>/.claude-plugin/marketplace.json` (shipped inside the cloned repo).
///
/// **Fallback**: filesystem scan of `plugins/` or root-level directories if
/// marketplace.json is absent/unparseable.
///
/// Returns `Ok(vec![])` when the source directory does not exist (never `Err`);
/// `fetch_plugins_from_source` swallows the `Result` via `.ok()` anyway, so an
/// empty vec is the correct sentinel for "not installed".
///
/// Directory resolution uses `repo_name` (from the presets) as the on-disk folder
/// name, not `source_id` — e.g. `source_id = "anthropics"` resolves to
/// `~/.forge/plugins/marketplace/claude-plugins-official/`, not `anthropics/`.
pub fn scan_source_for_plugins(source_id: &str, repo_type: Option<&str>) -> Result<Vec<MarketplacePlugin>, String> {
    // Resolve the actual on-disk directory via repo_name, not source_id.
    // Check presets first, then user sources.
    let presets = get_preset_sources();
    let user_sources = read_user_sources();
    let repo_name = presets
        .iter()
        .find(|s| s.id == source_id)
        .and_then(|s| s.repo_name.clone())
        .or_else(|| user_sources.iter().find(|s| s.id == source_id).and_then(|s| s.repo_name.clone()))
        .unwrap_or_else(|| source_id.to_string());

    let source_path = marketplace_sources_dir().join(&repo_name);

    if !source_path.exists() || !is_git_repo(&source_path) {
        // Source not installed — return empty list, not an error.
        log::debug!(
            "Source '{}' (repo '{}') not installed at {}, returning empty list",
            source_id,
            repo_name,
            source_path.display()
        );
        return Ok(vec![]);
    }

    // Primary path: read from .claude-plugin/marketplace.json
    if let Some(plugins) = read_source_marketplace_plugins(&source_path, source_id) {
        // For res-type repos, only return the first plugin (the repo itself)
        let plugins = if repo_type == Some("res") {
            plugins.into_iter().take(1).collect()
        } else {
            plugins
        };
        let mut plugins = plugins;
        plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        log::info!(
            "scan_source_for_plugins('{}'): {} plugins from marketplace.json",
            source_id,
            plugins.len()
        );
        return Ok(plugins);
    }

    // For res-type repos, try .claude-plugin/plugin.json as fallback
    if repo_type == Some("res") {
        if let Some(plugin) = read_res_plugin_from_json(&source_path, source_id) {
            log::info!(
                "scan_source_for_plugins('{}'): 1 plugin from plugin.json (res-type)",
                source_id
            );
            return Ok(vec![plugin]);
        }
    }

    // Fallback: filesystem scan (handles repos that don't ship marketplace.json)
    log::info!(
        "scan_source_for_plugins('{}'): marketplace.json unavailable, falling back to filesystem scan",
        source_id
    );
    scan_source_for_plugins_fallback(&source_path, source_id)
}

/// Filesystem-based fallback scan for sources that don't ship a marketplace.json.
/// Keeps the original `extract_plugin_info` logic but uses the correct
/// `repo_name`-resolved `source_path`.
fn scan_source_for_plugins_fallback(source_path: &PathBuf, source_id: &str) -> Result<Vec<MarketplacePlugin>, String> {
    let plugins_dir = source_path.join("plugins");
    let mut plugins = Vec::new();

    if plugins_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let plugin_path = entry.path();
                if plugin_path.is_dir() {
                    if let Some(plugin) = extract_plugin_info(&plugin_path, source_id) {
                        plugins.push(plugin);
                    }
                }
            }
        }
    } else {
        // Some repos have plugins at root level (no plugins/ subdirectory)
        log::warn!("No 'plugins/' directory in '{}', scanning root", source_path.display());
        let mut found_any = false;
        if let Ok(entries) = std::fs::read_dir(source_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() { continue; }
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if name.starts_with('.') || name == "node_modules" { continue; }
                if is_likely_plugin_dir(&path) {
                    found_any = true;
                    if let Some(plugin) = extract_plugin_info(&path, source_id) {
                        plugins.push(plugin);
                    }
                }
            }
        }
        // If no subdirectory plugins found, check if the root itself is a plugin
        // (handles repos that ARE the plugin, not just repos CONTAINING plugins)
        if !found_any && is_likely_plugin_dir(source_path) {
            if let Some(plugin) = extract_plugin_info(source_path, source_id) {
                plugins.push(plugin);
            }
        }
    }

    plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    log::info!("scan_source_for_plugins fallback: {} plugins from '{}'", plugins.len(), source_id);
    Ok(plugins)
}

/// Check if a directory looks like a plugin (has plugin-defining files).
fn is_likely_plugin_dir(path: &PathBuf) -> bool {
    let plugin_indicators = [
        "SKILL.md",
        "skill.json",
        "manifest.json",
        "plugin.json",
        ".claude-plugin",
    ];
    
    for indicator in &plugin_indicators {
        if path.join(indicator).exists() {
            return true;
        }
    }
    
    // Also check for directories that might contain agent skills
    if path.join("agents").exists() || path.join("commands").exists() {
        return true;
    }
    
    false
}

/// Extract basic plugin information from a plugin directory.
fn extract_plugin_info(plugin_path: &PathBuf, source_id: &str) -> Option<MarketplacePlugin> {
    let plugin_name = plugin_path.file_name()
        .map(|n| n.to_string_lossy().to_string())?;

    // Skip hidden directories
    if plugin_name.starts_with('.') {
        return None;
    }

    // Try to read description from plugin manifest first, then README
    let description = extract_description_from_plugin_manifest(plugin_path)
        .unwrap_or_else(|| extract_description_from_readme(plugin_path));

    // Try to read manifest.json if present
    let (version, author, repo_url) = extract_manifest_info(plugin_path);

    // Fallback scan: a plugin is "installed" when its cache directory
    // exists under `$FORGE_HOME/plugins/cache/<repo_name>/<plugin_name>/`.
    // Mirrors the same check used in `read_source_marketplace_plugins`
    // so the UI shows consistent state across both code paths.
    let repo_name = resolve_repo_name(source_id);
    let cache_dir = forge_home()
        .join("plugins")
        .join("cache")
        .join(&repo_name)
        .join(&plugin_name);
    let is_installed = cache_dir.is_dir();
    let cli_tool_keys = detect_cli_tool_keys(if is_installed { &cache_dir } else { plugin_path });
    let cli_tool_key = cli_tool_keys.first().cloned();

    Some(MarketplacePlugin {
        id: plugin_name.clone(),
        source_id: source_id.to_string(),
        name: plugin_name.clone(),
        description,
        long_description: None,
        author,
        version: version.clone(),
        latest_version: version,
        has_update: Some(false),
        categories: vec!["Claude Code Plugin".to_string()],
        tags: vec![],
        install_command: None,
        install_path: Some(plugin_path.to_string_lossy().to_string()),
        repository: repo_url,
        homepage: None,
        license: None,
        stars: None,
        downloads: None,
        last_updated: None,
        is_installed,
        disabled: false,
        // Fallback scan: no upstream `source` block available, so we
        // treat this as a local install (file already on disk in the
        // cloned source repo).
        install_source: Some(PluginInstallSource {
            kind: "local".to_string(),
            url: String::new(),
            path: String::new(),
            r#ref: String::new(),
            sha: String::new(),
        }),
        cli_tool_key,
        cli_tool_keys,
    })
}

/// Extract description from README.md file.
fn extract_description_from_readme(plugin_path: &PathBuf) -> String {
    let readme_variants = ["README.md", "readme.md", "README.MD", "README.txt"];

    for variant in &readme_variants {
        let readme_path = plugin_path.join(variant);
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                // Extract first non-empty, non-heading line as description
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.len() > 10 {
                        // Truncate to reasonable length
                        if trimmed.len() > 200 {
                            return format!("{}...", &trimmed[..200]);
                        }
                        return trimmed.to_string();
                    }
                }
            }
        }
    }

    "No description available".to_string()
}

/// Try to read description from `.claude-plugin/plugin.json` or similar marker files.
fn extract_description_from_plugin_manifest(plugin_path: &PathBuf) -> Option<String> {
    let manifest_variants = [
        ".claude-plugin/plugin.json",
        ".cursor-plugin/plugin.json",
        ".codex-plugin/plugin.json",
        "plugin.json",
    ];
    for variant in &manifest_variants {
        let manifest_path = plugin_path.join(variant);
        if manifest_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
                        if !desc.is_empty() {
                            return Some(desc.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract information from manifest.json if present.
fn extract_manifest_info(plugin_path: &PathBuf) -> (Option<String>, Option<String>, Option<String>) {
    let manifest_variants = ["manifest.json", "skill.json", "plugin.json"];
    
    for variant in &manifest_variants {
        let manifest_path = plugin_path.join(variant);
        if manifest_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let version = json.get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let author = json.get("author")
                        .and_then(|a| a.as_str())
                        .map(|s| s.to_string());
                    let repo_url = json.get("repository")
                        .or_else(|| json.get("repo"))
                        .and_then(|r| r.as_str())
                        .map(|s| s.to_string());
                    return (version, author, repo_url);
                }
            }
        }
    }
    
    (None, None, None)
}

// ---------------------------------------------------------------------------
// Marketplace.json reader — canonical plugin count from upstream source repo
// ---------------------------------------------------------------------------

/// Detect the "repo itself is a plugin" case.
///
/// A repository is itself a single plugin (as opposed to a marketplace of
/// many) when one of the well-known plugin-defining files is present at
/// the repository root. Mirrors the same heuristics the
/// filesystem-scan fallback uses in `is_likely_plugin_dir` — but here we
/// apply it to the repo root instead of any subdirectory, so we can
/// distinguish "this repo IS the plugin" from "this repo COLLECTS
/// plugins".
///
/// We deliberately ignore `.claude-plugin/marketplace.json` itself: a
/// repo can ship a marketplace manifest with zero entries and still be
/// a regular plugin repo. In that case we want to count 1 (the repo
/// itself), not 0.
fn repo_root_has_plugin_marker(source_path: &PathBuf) -> bool {
    const MARKERS: &[&str] = &[
        "SKILL.md",
        "skill.md",
        "skill.json",
        "manifest.json",
        "plugin.json",
    ];
    for marker in MARKERS {
        if source_path.join(marker).exists() {
            return true;
        }
    }
    // Repos that are themselves a plugin often use a `plugin/` or
    // `skills/` directory at the root instead of manifest files.
    if source_path.join("plugin").is_dir() || source_path.join("skills").is_dir() {
        return true;
    }
    false
}

/// Build a `MarketplacePlugin` entry for the case where the entire
/// repository is itself the plugin (the marketplace.json `plugins`
/// array is empty, but the repo root has plugin-defining files).
///
/// Mirrors the metadata we'd otherwise get from a manifest entry:
/// - `name` / `id` fall back to the directory name
/// - `description` comes from the README if any
/// - `version` / `author` / `repository` come from any local manifest
/// - `install_source` is `url`-shaped: the repo URL we cloned IS the
///   plugin, no sub-path to extract.
fn synthesize_repo_root_plugin(source_path: &PathBuf, source_id: &str, repo_url: &str) -> MarketplacePlugin {
    let plugin_name = source_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| source_id.to_string());

    let description = extract_description_from_plugin_manifest(source_path)
        .unwrap_or_else(|| extract_description_from_readme(source_path));
    let (version, author, repo_from_manifest) = extract_manifest_info(source_path);

    // A synthesized "the whole repo is the plugin" entry is installed iff
    // its cache directory exists. For repo-is-plugin, the cache path is
    // `cache/<plugin_name>/<plugin_name>/`.
    let cache_dir = forge_home()
        .join("plugins")
        .join("cache")
        .join(&plugin_name)
        .join(&plugin_name);
    let is_installed = cache_dir.is_dir();
    let cli_tool_keys = detect_cli_tool_keys(if is_installed { &cache_dir } else { source_path });
    let cli_tool_key = cli_tool_keys.first().cloned();

    MarketplacePlugin {
        id: plugin_name.clone(),
        source_id: source_id.to_string(),
        name: plugin_name,
        description,
        long_description: None,
        author,
        version: version.clone(),
        latest_version: None,
        has_update: Some(false),
        categories: vec!["Claude Code Plugin".to_string()],
        tags: vec![],
        install_command: None,
        install_path: Some(source_path.to_string_lossy().to_string()),
        repository: if repo_from_manifest.is_some() {
            repo_from_manifest
        } else if repo_url.is_empty() {
            None
        } else {
            Some(repo_url.to_string())
        },
        homepage: None,
        license: None,
        stars: None,
        downloads: None,
        last_updated: None,
        is_installed,
        disabled: false,
        // The whole repo IS the plugin: install_plugin will see kind=url
        // and clone <repo_url> into forge/plugins/cache/<name>/.
        install_source: Some(PluginInstallSource {
            kind: "url".to_string(),
            url: repo_url.to_string(),
            path: String::new(),
            r#ref: String::new(),
            sha: String::new(),
        }),
        cli_tool_key,
        cli_tool_keys,
    }
}

/// Read the `plugins` array length from `<source_path>/.claude-plugin/marketplace.json`.
///
/// Returns `None` if the file is missing, unreadable, or unparseable.
///
/// **Special case — empty marketplace, repo is a plugin**: if the
/// `plugins` array is empty (length 0) but the repository root contains
/// a plugin marker (`SKILL.md`, `manifest.json`, …), the repo itself
/// IS the plugin. In that case we return `Some(1)` instead of `Some(0)`
/// so the marketplace tab badge stops showing "0" for a single-plugin
/// repository.
fn read_source_marketplace_json(source_path: &PathBuf) -> Option<u32> {
    let json_path = source_path.join(".claude-plugin").join("marketplace.json");
    if !json_path.exists() {
        log::debug!(
            "No .claude-plugin/marketplace.json at {} — will fall back to filesystem scan",
            source_path.display()
        );
        return None;
    }

    let content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read {}: {}", json_path.display(), e);
            return None;
        }
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse {}: {}", json_path.display(), e);
            return None;
        }
    };

    let count = value
        .get("plugins")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len() as u32)?;

    // Empty plugins array + repo is itself a plugin → count as 1.
    if count == 0 && repo_root_has_plugin_marker(source_path) {
        log::info!(
            "Read plugin count 0 from marketplace.json but repo {} has plugin markers — counting as 1",
            source_path.display()
        );
        return Some(1);
    }

    log::info!(
        "Read plugin count {} from {}/.claude-plugin/marketplace.json",
        count,
        source_path.display()
    );
    Some(count)
}

/// Resolve the GitHub URL for a source (preset or user-added). Used
/// when synthesizing a "repo is the plugin" entry so the synthesized
/// `install_source` carries a usable clone URL.
fn resolve_source_repo_url(source_id: &str) -> String {
    let presets = get_preset_sources();
    if let Some(s) = presets.iter().find(|s| s.id == source_id) {
        return s.command.clone();
    }
    // Fall back to user-added sources
    for s in read_user_sources() {
        if s.id == source_id {
            return s.command;
        }
    }
    String::new()
}

/// Count plugin directories in a source repo.
/// Handles two layouts:
/// 1. Single-plugin repo: `.claude-plugin/` at the root → count = 1
/// 2. Multi-plugin repo: subdirectories with `.claude-plugin/` or `plugin.json`
fn count_plugin_dirs(source_path: &std::path::Path) -> u32 {
    // Case 1: the repo root itself is a plugin
    if source_path.join(".claude-plugin").is_dir()
        || source_path.join("plugin.json").exists()
    {
        return 1;
    }
    // Case 2: scan subdirectories for plugins
    let mut count = 0u32;
    if let Ok(entries) = std::fs::read_dir(source_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "plugins" { continue; }
            if path.join(".claude-plugin").is_dir()
                || path.join("plugin.json").exists()
                || path.join("SKILL.md").exists()
            {
                count += 1;
            }
            // Check plugins/ subdirectory inside each entry
            let plugins_sub = path.join("plugins");
            if plugins_sub.is_dir() {
                if let Ok(sub_entries) = std::fs::read_dir(&plugins_sub) {
                    for sub in sub_entries.filter_map(|e| e.ok()) {
                        let sub_path = sub.path();
                        if !sub_path.is_dir() { continue; }
                        if sub_path.join(".claude-plugin").is_dir()
                            || sub_path.join("plugin.json").exists()
                        {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    // Also check top-level plugins/ directory
    let plugins_dir = source_path.join("plugins");
    if plugins_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_dir() { continue; }
                if path.join(".claude-plugin").is_dir()
                    || path.join("plugin.json").exists()
                {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Get the plugin count for a source.
/// Priority: live count from `.claude-plugin/marketplace.json` (if installed) >
/// fallback filesystem scan (for sources without marketplace.json).
pub fn get_source_plugin_count(source_id: &str) -> u32 {
    let presets = get_preset_sources();
    let user_sources = read_user_sources();
    let source_info = presets
        .iter()
        .find(|s| s.id == source_id)
        .or_else(|| user_sources.iter().find(|s| s.id == source_id));
    let repo_name = source_info
        .and_then(|s| s.repo_name.clone())
        .unwrap_or_else(|| source_id.to_string());
    let repo_type = source_info
        .and_then(|s| s.repo_type.clone());

    // res-type repos always have exactly 1 plugin
    if repo_type.as_deref() == Some("res") {
        return 1;
    }

    let source_path = marketplace_sources_dir().join(&repo_name);
    if !source_path.exists() || !is_git_repo(&source_path) {
        return 0;
    }

    // Primary: read from the canonical marketplace.json shipped inside the cloned repo
    if let Some(count) = read_source_marketplace_json(&source_path) {
        return count;
    }

    // Fallback: filesystem scan for sources that don't ship a marketplace.json
    match scan_source_for_plugins(source_id, repo_type.as_deref()) {
        Ok(plugins) => plugins.len() as u32,
        Err(e) => {
            log::warn!("Failed to scan source '{}': {}", source_id, e);
            0
        }
    }
}

/// Fetch all plugins from a marketplace source (no pagination — return the
/// full merged list so the UI can render every card at once).
///
/// The function is still async to match the Tauri command signature, but
/// the result is the same `Vec<MarketplacePlugin>` you would get from a
/// single "give me everything" call. If we later need to scale this past
/// several thousand plugins we can reintroduce cursor-based pagination
/// without breaking the command shape.
pub async fn fetch_plugins_from_source(
    source_id: &str,
    keyword: Option<&str>,
) -> Result<Vec<MarketplacePlugin>, String> {
    // Get repo_name and repo_type for the source
    let presets = get_preset_sources();
    let user_sources = read_user_sources();
    let source_info = presets.iter()
        .find(|s| s.id == source_id)
        .or_else(|| user_sources.iter().find(|s| s.id == source_id));
    let repo_name = source_info
        .and_then(|s| s.repo_name.clone())
        .unwrap_or_else(|| source_id.to_string());
    let repo_type = source_info
        .and_then(|s| s.repo_type.clone());

    // First, try to get plugins from the cloned repository (if source is installed)
    let source_path = marketplace_sources_dir().join(&repo_name);
    let plugins_from_repo = if source_path.exists() && is_git_repo(&source_path) {
        scan_source_for_plugins(source_id, repo_type.as_deref()).ok()
    } else {
        None
    };

    // Also try the manifest as fallback
    let manifest = read_manifest();
    let plugins_from_manifest: Vec<MarketplacePlugin> = manifest
        .plugins
        .get(source_id)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(manifest_plugin_to_marketplace)
        .collect();

    // Merge plugins from repo and manifest, deduplicating by name
    let mut all_plugins: Vec<MarketplacePlugin> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Add repo plugins first (they're from the cloned source)
    if let Some(repo_plugins) = plugins_from_repo {
        for plugin in repo_plugins {
            if seen_names.insert(plugin.name.clone()) {
                all_plugins.push(plugin);
            }
        }
    }

    // Add manifest plugins that weren't already added
    for plugin in plugins_from_manifest {
        if seen_names.insert(plugin.name.clone()) {
            all_plugins.push(plugin);
        }
    }

    // Apply keyword filter
    let filtered: Vec<MarketplacePlugin> = if let Some(kw) = keyword {
        let kw = kw.to_lowercase();
        all_plugins
            .into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&kw)
                    || p.description.to_lowercase().contains(&kw)
            })
            .collect()
    } else {
        all_plugins
    };

    Ok(filtered)
}

pub async fn get_marketplace_manifest() -> MarketplaceManifest {
    read_manifest()
}

pub async fn get_marketplace_sources() -> Vec<PluginSource> {
    let presets = get_preset_sources();
    presets
        .into_iter()
        .map(|mut s| {
            // Priority: live count from .claude-plugin/marketplace.json (if installed) >
            // stored manifest count > 0 > 0.
            let live_count = get_source_plugin_count(&s.id);
            if live_count > 0 {
                s.plugin_count = Some(live_count);
            } else if s.plugin_count.unwrap_or(0) > 0 {
                // Keep existing positive count from manifest
            } else {
                s.plugin_count = Some(0);
            }
            s
        })
        .collect()
}

fn manifest_plugin_to_marketplace(p: ManifestPlugin) -> MarketplacePlugin {
    let author = match p.author {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Object(_) => serde_json::from_value(p.author.clone()).ok(),
        _ => None,
    };
    let version_opt = if p.version.is_empty() {
        None
    } else {
        Some(p.version)
    };
    let repo_url_opt = if p.repo_url.is_empty() {
        None
    } else {
        Some(p.repo_url)
    };

    // Determine on-disk install state by checking the cache directory
    // (`$FORGE_HOME/plugins/cache/<repo_name>/<name>/`). This is the same
    // path `install_plugin` writes to, so it's the single source of truth
    // — both for the installed flag and for the `install_path` exposed
    // to the UI.
    let source_id = derive_source_from_path(&p.installed_path)
        .unwrap_or_else(|| "unknown".to_string());
    let repo_name = resolve_repo_name(&source_id);
    let cache_path = forge_home()
        .join("plugins")
        .join("cache")
        .join(&repo_name)
        .join(&p.name);
    let is_installed = cache_path.is_dir();
    // Always expose the canonical cache path so the Installed tab and
    // its "copy install path" affordance resolve to a real location
    // (matches what `get_installed_plugins` returns).
    let install_path = cache_path;
    let cli_tool_keys = detect_cli_tool_keys(&install_path);
    let cli_tool_key = cli_tool_keys.first().cloned();

    let categories = vec!["Claude Code Plugin".to_string()];
    let tags: Vec<String> = p.dependencies.clone();

    MarketplacePlugin {
        id: p.name.clone(),
        source_id,
        name: p.name,
        description: p.description,
        long_description: None,
        author,
        version: version_opt.clone(),
        latest_version: None,
        has_update: Some(false),
        categories,
        tags,
        install_command: None,
        install_path: Some(install_path.to_string_lossy().to_string()),
        repository: repo_url_opt,
        homepage: None,
        license: None,
        stars: None,
        downloads: None,
        last_updated: None,
        is_installed,
        // Persist the user-controlled enable/disable state from the
        // manifest. Marketplace entries that the user has never toggled
        // default to `disabled = false` (= enabled).
        disabled: p.disabled,
        // Manifest entries coming from the on-disk registry (this
        // function) don't carry an `install_source` block; the install
        // path was already resolved above, so we don't need to download
        // anything for these. Treat them as "local" so install_plugin
        // re-runs would just re-copy the directory.
        install_source: None,
        cli_tool_key,
        cli_tool_keys,
    }
}

fn derive_source_from_path(installed_path: &str) -> Option<String> {
    let trimmed = installed_path.strip_prefix("plugins/")?;
    let mut parts = trimmed.splitn(2, '/');
    parts.next().map(|s| s.to_string())
}

/// Resolve the repo_name for a given source_id.
/// Preset sources have repo_name in their definition; user sources likewise.
/// Falls back to source_id itself when no explicit repo_name is set.
fn resolve_repo_name(source_id: &str) -> String {
    let presets = get_preset_sources();
    let user_sources = read_user_sources();
    presets.iter()
        .find(|s| s.id == source_id)
        .and_then(|s| s.repo_name.clone())
        .or_else(|| user_sources.iter().find(|s| s.id == source_id).and_then(|s| s.repo_name.clone()))
        .unwrap_or_else(|| source_id.to_string())
}

// -- Local install/uninstall -----------------------------------------------

/// Get the cache directory for installed plugins.
fn get_cache_dir() -> PathBuf {
    forge_home().join("plugins").join("cache")
}

pub async fn get_installed_plugins(_cursor_dir: &PathBuf) -> Result<Vec<MarketplacePlugin>, String> {
    let cache_root = get_cache_dir();
    if !cache_root.exists() {
        return Ok(vec![]);
    }

    let manifest = read_manifest();
    let mut installed = Vec::new();

    // Scan nested cache structure: cache/<parent>/<plugin>/
    // parent is repo_name for sub-plugins, or plugin_name for repo-is-plugins.
    if let Ok(parent_entries) = std::fs::read_dir(&cache_root) {
        for parent_entry in parent_entries.filter_map(|e| e.ok()) {
            let parent_path = parent_entry.path();
            if !parent_path.is_dir() { continue; }

            if let Ok(plugin_entries) = std::fs::read_dir(&parent_path) {
                for plugin_entry in plugin_entries.filter_map(|e| e.ok()) {
                    let plugin_path = plugin_entry.path();
                    if !plugin_path.is_dir() { continue; }

                    let plugin_name = plugin_path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Look up plugin manifest entry across all sources
                    let manifest_entry = manifest.plugins.values()
                        .flat_map(|list| list.iter())
                        .find(|p| p.name == plugin_name);

                    let source_id = manifest.plugins.iter()
                        .find(|(_, list)| list.iter().any(|p| p.name == plugin_name))
                        .map(|(k, _)| k.clone())
                        .unwrap_or_else(|| {
                            // Manifest doesn't have this plugin (e.g. it was
                            // uninstalled but the cache directory remains).
                            // Derive source_id from the cache directory structure:
                            // cache/<repo_name>/<plugin_name>/ — the parent dir
                            // is the repo_name, which maps to a preset source_id.
                            let parent_name = parent_path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let presets = get_preset_sources();
                            presets.iter()
                                .find(|s| s.repo_name.as_deref() == Some(&parent_name))
                                .map(|s| s.id.clone())
                                .unwrap_or_else(|| parent_name.clone())
                        });

                    let version = manifest_entry
                        .as_ref()
                        .and_then(|p| if p.version.is_empty() { None } else { Some(p.version.clone()) });

                    let manifest_desc = manifest_entry
                        .as_ref()
                        .map(|p| p.description.clone())
                        .filter(|d| !d.is_empty() && d != "No description");
                    let description = manifest_desc
                        .or_else(|| extract_description_from_plugin_manifest(&plugin_path))
                        .unwrap_or_else(|| "No description".to_string());

                    let disabled = manifest_entry
                        .as_ref()
                        .map(|p| p.disabled)
                        .unwrap_or(false);

                    let repo_url = manifest_entry
                        .as_ref()
                        .and_then(|p| if p.repo_url.is_empty() { None } else { Some(p.repo_url.clone()) });

                    installed.push(MarketplacePlugin {
                        id: plugin_name.clone(),
                        source_id,
                        name: plugin_name.clone(),
                        description,
                        long_description: None,
                        author: None,
                        version,
                        latest_version: None,
                        has_update: Some(false),
                        categories: vec!["Claude Code Plugin".to_string()],
                        tags: vec![],
                        install_command: None,
                        install_path: Some(plugin_path.to_string_lossy().to_string()),
                        repository: repo_url,
                        homepage: None,
                        license: None,
                        stars: None,
                        downloads: None,
                        last_updated: None,
                        is_installed: true,
                        disabled,
                        install_source: None,
                        cli_tool_key: detect_cli_tool_key(&plugin_path),
                        cli_tool_keys: detect_cli_tool_keys(&plugin_path),
                    });
                }
            }
        }
    }

    Ok(installed)
}

// ---------------------------------------------------------------------------
// Single-plugin download via `git archive --remote` + sparse-checkout fallback
// ---------------------------------------------------------------------------

/// Download a single plugin directory.  Tries `git archive --remote` first;
/// falls back to shallow clone + sparse-checkout if the server does not
/// support remote archive.  Uses system `git` and `tar` — no new dependencies.
fn download_single_plugin(
    repo_url: &str,
    sub_path: &str,
    dest_parent: &PathBuf,
) -> Result<PathBuf, String> {
    let dest = dest_parent.to_path_buf();

    // Method 1: git archive --remote=<url> HEAD:<sub-path> | tar -x
    log::info!(
        "Attempting git archive --remote for '{}' from '{}'",
        sub_path,
        repo_url
    );
    if try_git_archive_remote(repo_url, sub_path, &dest).is_ok() {
        if dest.exists() {
            log::info!("git archive --remote succeeded for '{}'", sub_path);
            return Ok(dest);
        }
    }

    // Method 2: fallback — shallow clone + sparse-checkout
    log::info!("Falling back to sparse-checkout for '{}'", sub_path);
    try_sparse_checkout_fallback(repo_url, sub_path, &dest)?;

    Ok(dest)
}

/// Try `git archive --remote=<repo_url> HEAD:<sub_path>` piped to `tar -x`.
fn try_git_archive_remote(repo_url: &str, sub_path: &str, dest: &PathBuf) -> Result<(), String> {
    let dest_parent = dest
        .parent()
        .ok_or_else(|| "dest has no parent directory".to_string())?;

    let mut git_proc = std::process::Command::new("git")
        .args([
            "archive",
            "--remote",
            repo_url,
            &format!("HEAD:{}", sub_path),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn git archive: {}", e))?;

    let mut tar_child = std::process::Command::new("tar")
        .args(["-x", "-C", &dest_parent.to_string_lossy()])
        .stdin(git_proc.stdout.take().ok_or("Failed to pipe git to tar")?)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn tar: {}", e))?;

    let git_status = git_proc
        .wait()
        .map_err(|e| format!("git archive wait failed: {}", e))?;

    let tar_status = tar_child
        .wait()
        .map_err(|e| format!("tar wait failed: {}", e))?;

    if !git_status.success() {
        let mut stderr = String::new();
        if let Some(s) = git_proc.stderr.take() {
            let mut s = s;
            let _ = std::io::Read::read_to_string(&mut s, &mut stderr);
        }
        return Err(format!(
            "git archive --remote failed (exit {}): {}",
            git_status,
            stderr.trim()
        ));
    }
    if !tar_status.success() {
        return Err(format!("tar extraction failed with exit {}", tar_status));
    }

    Ok(())
}

/// Fallback: `git clone --depth 1 --filter=blob:none --sparse` + sparse-checkout.
fn try_sparse_checkout_fallback(repo_url: &str, sub_path: &str, dest: &PathBuf) -> Result<(), String> {
    let tmp_base = std::env::temp_dir().join(format!(
        "env-manager-clone-{}",
        std::process::id()
    ));
    let clone_dir = tmp_base.join("repo");

    let _ = std::fs::remove_dir_all(&tmp_base);

    // git clone --depth 1 --filter=blob:none --sparse <url> <dir>
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--filter=blob:none",
            "--sparse",
            repo_url,
            clone_dir.to_str().unwrap_or("."),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| format!("Failed to spawn git clone: {}", e))?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp_base);
        return Err(format!("git clone failed with exit {}", status));
    }

    // git sparse-checkout set <sub-path>
    let status = std::process::Command::new("git")
        .current_dir(&clone_dir)
        .args(["sparse-checkout", "set", sub_path])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| format!("Failed to spawn git sparse-checkout: {}", e))?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp_base);
        return Err(format!(
            "git sparse-checkout set failed with exit {}",
            status
        ));
    }

    let sparse_src = clone_dir.join(sub_path);
    if !sparse_src.exists() {
        let _ = std::fs::remove_dir_all(&tmp_base);
        return Err(format!(
            "Plugin directory '{}' not found after sparse-checkout (path may not exist in this repo)",
            sub_path
        ));
    }

    copy_dir_recursive(&sparse_src, dest).map_err(|e| {
        let _ = std::fs::remove_dir_all(&tmp_base);
        format!("Failed to copy plugin files: {}", e)
    })?;

    let _ = std::fs::remove_dir_all(&tmp_base);
    Ok(())
}

/// Recursive directory copy using only std library.
/// Copies all files and directories including hidden ones (dot-files/dirs).
/// Skips `.git` directory to avoid copying VCS metadata.
/// Handles symlinks: resolves directory symlinks and copies their contents.
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(&name);

        // Skip only .git — it's VCS metadata, not plugin content.
        if ty.is_dir() && name_str == ".git" {
            continue;
        }

        if ty.is_symlink() {
            // Resolve the symlink target relative to the source directory.
            let link_target = std::fs::read_link(&src_path)?;
            let resolved = if link_target.is_relative() {
                src_path.parent().unwrap_or(src).join(&link_target)
            } else {
                link_target
            };
            // Canonicalize to get the real path (follows chain of symlinks).
            let canonical = std::fs::canonicalize(&resolved)
                .unwrap_or_else(|_| resolved.clone());
            if canonical.is_dir() {
                // Symlink to directory: create dir and copy contents recursively.
                copy_dir_recursive(&canonical, &dst_path)?;
            } else {
                // Symlink to file: copy the file content.
                std::fs::copy(&canonical, &dst_path)?;
            }
        } else if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// install_plugin — single plugin download from cloned marketplace source (FEAT-009)
//
// The marketplace manifest describes every plugin with a `source` block that
// falls into one of three shapes:
//
//   1. `string-path`  — `"./plugins/<name>"`.
//                       Plugin directory is already inside the cloned
//                       marketplace source repo. We just copy it out.
//   2. `git-subdir`   — `{source:"git-subdir", url, path, ref?, sha?}`.
//                       Sub-directory of *another* Git repo. Sparse-clone
//                       `<url>` and copy `<path>` from the result.
//   3. `url`          — `{source:"url", url, ref?, sha?}` (no `path`).
//                       The whole repo IS the plugin. Clone `<url>`.
//
// We dispatch on `plugin.install_source.kind`. The previous implementation
// only handled (1) and therefore failed on every (2)/(3) plugin — e.g.
// `superpowers` from `obra/superpowers.git` — with the misleading
// "Plugin 'X' not found in marketplace source 'Y'" error.
// ---------------------------------------------------------------------------

pub async fn install_plugin(
    plugin: &MarketplacePlugin,
    _source_command: &str,
    _plugins_dir: &PathBuf,
) -> Result<PluginInstallResult, String> {
    let source_key = plugin.source_id.clone();

    // Get repo_name for the source (directory is named by repo_name, not source_id)
    let presets = get_preset_sources();
    let user_sources = read_user_sources();
    let repo_name = presets.iter()
        .find(|s| s.id == source_key)
        .and_then(|s| s.repo_name.clone())
        .or_else(|| user_sources.iter().find(|s| s.id == source_key).and_then(|s| s.repo_name.clone()))
        .unwrap_or_else(|| source_key.clone());

    // Cache layout:
    //   Sub-plugin  → cache/<repo_name>/<plugin_name>/   (e.g. claude-plugins-official/skill-creator/)
    //   Repo-is-plugin → cache/<plugin_name>/<plugin_name>/  (e.g. agent-skills/agent-skills/)
    //
    // We detect which case applies by checking the marketplace source:
    // if the plugin is found in a subdirectory (plugins/X, external_plugins/X, etc.)
    // it's a sub-plugin; if only the repo root qualifies, it's a repo-is-plugin.

    let cache_dir = forge_home().join("plugins").join("cache");
    let is_repo_root = detect_repo_root_plugin(&repo_name, &plugin.name, plugin.install_source.as_ref());
    let cache_parent = if is_repo_root { &plugin.name } else { &repo_name };
    let dest = cache_dir.join(cache_parent).join(&plugin.name);

    // Idempotent: already installed — skip re-download
    if dest.exists() {
        log::info!(
            "Plugin '{}' already exists at {}, skipping download",
            plugin.name,
            dest.display()
        );
        return Ok(PluginInstallResult {
            success: true,
            path: Some(dest.to_string_lossy().to_string()),
            error: None,
        });
    }

    // Create parent directory up front so each branch can write into it.
    std::fs::create_dir_all(dest.parent().unwrap_or(&cache_dir))
        .map_err(|e| format!("Failed to create plugin cache dir: {}", e))?;

    // Decide the install strategy from install_source.kind. Fall back to the
    // legacy local-copy path when the manifest didn't carry an install_source
    // (older manifests / fallback scan) so we stay backwards compatible.
    let kind = plugin
        .install_source
        .as_ref()
        .map(|s| s.kind.as_str())
        .unwrap_or("local");

    log::info!(
        "install_plugin: plugin='{}', source='{}', kind='{}', repo_root={}, dest='{}'",
        plugin.name,
        source_key,
        kind,
        is_repo_root,
        dest.display()
    );

    let result = match kind {
        "git-subdir" => install_from_git_subdir(plugin, &dest).await,
        "url" => install_from_url(plugin, &dest).await,
        // "local" and the unknown-fallback case
        _ => install_from_local(plugin, &source_key, &repo_name, &dest).map(|_| ()),
    };

    match result {
        Ok(()) => finalize_installation(plugin, &source_key, &dest),
        Err(e) => {
            // Clean up any partial state on failure.
            let _ = std::fs::remove_dir_all(&dest);
            Err(e)
        }
    }
}

/// Detect whether a plugin is "the entire repo" (repo-is-plugin) vs a
/// sub-directory plugin. Returns `true` when the marketplace source repo
/// root itself is the plugin — meaning the cache path should be
/// `cache/<plugin_name>/<plugin_name>/` instead of
/// `cache/<repo_name>/<plugin_name>/`.
fn detect_repo_root_plugin(
    repo_name: &str,
    plugin_name: &str,
    install_source: Option<&PluginInstallSource>,
) -> bool {
    let marketplace_source_path = marketplace_sources_dir().join(repo_name);
    if !marketplace_source_path.exists() {
        return false;
    }

    // If install_source has an explicit path, the plugin is a sub-directory.
    if let Some(src) = install_source {
        if !src.path.is_empty() {
            return false;
        }
    }

    // Check if the plugin exists as a sub-directory (steps 1-3 of install_from_local).
    // If so, it's a sub-plugin, not a repo-is-plugin.
    // Step 1: explicit path already checked above.
    // Step 2: plugins/<plugin_name>/
    if marketplace_source_path.join("plugins").join(plugin_name).is_dir() {
        return false;
    }
    // Step 3: <plugin_name>/ at repo root
    if marketplace_source_path.join(plugin_name).is_dir() {
        return false;
    }

    // Step 4: The repo root itself has plugin markers → repo-is-plugin.
    repo_root_has_plugin_marker(&marketplace_source_path)
}

/// Local install: the plugin is a directory already living inside the cloned
/// marketplace source repo. Just copy it out.
fn install_from_local(
    plugin: &MarketplacePlugin,
    source_key: &str,
    repo_name: &str,
    dest: &PathBuf,
) -> Result<(), String> {
    let marketplace_source_path = marketplace_sources_dir().join(repo_name);
    if !marketplace_source_path.exists() || !is_git_repo(&marketplace_source_path) {
        return Err(format!(
            "Source '{}' is not installed. Please install the source first from the Sources tab.",
            source_key
        ));
    }

    let mut plugin_source_path: Option<PathBuf> = None;

    // 1. Try the explicit relative path from install_source.path (e.g. "./external_plugins/context7")
    if let Some(ref install_source) = plugin.install_source {
        if !install_source.path.is_empty() {
            let candidate = marketplace_source_path.join(&install_source.path);
            if candidate.exists() && candidate.is_dir() {
                plugin_source_path = Some(candidate);
            }
        }
    }

    // 2. Try plugins/<plugin_name>/ (claude-plugins-official layout)
    if plugin_source_path.is_none() {
        let candidate = marketplace_source_path
            .join("plugins")
            .join(&plugin.name);
        if candidate.exists() && candidate.is_dir() {
            plugin_source_path = Some(candidate);
        }
    }

    // 3. Try <plugin_name>/ at repo root
    if plugin_source_path.is_none() {
        let candidate = marketplace_source_path.join(&plugin.name);
        if candidate.exists() && candidate.is_dir() {
            plugin_source_path = Some(candidate);
        }
    }

    // 4. The repo itself is the plugin (no subdirectory layout).
    //    This happens when the marketplace source repo IS a single plugin
    //    (e.g. addyosmani/agent-skills — has plugin.json + skills/ at root
    //    but no plugins/ or external_plugins/ subdirectory).
    if plugin_source_path.is_none() && repo_root_has_plugin_marker(&marketplace_source_path) {
        log::info!(
            "install_from_local: using marketplace source root as plugin dir for '{}'",
            plugin.name
        );
        plugin_source_path = Some(marketplace_source_path.clone());
    }

    let plugin_source = plugin_source_path.ok_or_else(|| {
        format!(
            "Plugin '{}' not found in marketplace source '{}' (expected under {} or {})",
            plugin.name,
            source_key,
            marketplace_source_path.join("plugins").join(&plugin.name).display(),
            marketplace_source_path.join(&plugin.name).display(),
        )
    })?;

    log::info!(
        "install_from_local: copying '{}' -> '{}'",
        plugin_source.display(),
        dest.display()
    );
    copy_dir_recursive(&plugin_source, dest).map_err(|e| {
        format!(
            "Failed to copy plugin '{}' from local source: {}",
            plugin.name, e
        )
    })?;
    Ok(())
}

/// `git-subdir` install: clone the external repo into a temporary
/// scratch directory, optionally check out a pinned SHA, then copy the
/// declared sub-directory out.
async fn install_from_git_subdir(plugin: &MarketplacePlugin, dest: &PathBuf) -> Result<(), String> {
    let src = plugin.install_source.as_ref().ok_or_else(|| {
        format!(
            "Plugin '{}' has install_source.kind='git-subdir' but no install_source payload",
            plugin.name
        )
    })?;
    if src.url.is_empty() {
        return Err(format!(
            "Plugin '{}' git-subdir source has no url",
            plugin.name
        ));
    }
    if src.path.is_empty() {
        return Err(format!(
            "Plugin '{}' git-subdir source has no path",
            plugin.name
        ));
    }

    let scratch = clone_to_scratch(&plugin.name, &src.url, &src.r#ref, &src.sha).await?;
    let subdir = scratch.join(src.path.trim_start_matches("./").trim_start_matches('/'));

    if !subdir.exists() || !subdir.is_dir() {
        let _ = std::fs::remove_dir_all(&scratch);
        return Err(format!(
            "Plugin '{}' git-subdir path '{}' not found after cloning {}",
            plugin.name,
            subdir.display(),
            src.url
        ));
    }

    log::info!(
        "install_from_git_subdir: copying '{}' -> '{}'",
        subdir.display(),
        dest.display()
    );
    let res = copy_dir_recursive(&subdir, dest);
    let _ = std::fs::remove_dir_all(&scratch);
    res.map_err(|e| {
        format!(
            "Failed to copy git-subdir plugin '{}': {}",
            plugin.name, e
        )
    })?;
    Ok(())
}

/// `url` install: the whole external repo IS the plugin. Clone it and use
/// the repo root as the plugin directory.
async fn install_from_url(plugin: &MarketplacePlugin, dest: &PathBuf) -> Result<(), String> {
    let src = plugin.install_source.as_ref().ok_or_else(|| {
        format!(
            "Plugin '{}' has install_source.kind='url' but no install_source payload",
            plugin.name
        )
    })?;
    if src.url.is_empty() {
        return Err(format!(
            "Plugin '{}' url source has no url",
            plugin.name
        ));
    }

    let scratch = clone_to_scratch(&plugin.name, &src.url, &src.r#ref, &src.sha).await?;

    log::info!(
        "install_from_url: copying repo root '{}' -> '{}'",
        scratch.display(),
        dest.display()
    );
    let res = copy_dir_recursive(&scratch, dest);
    let _ = std::fs::remove_dir_all(&scratch);
    res.map_err(|e| {
        format!(
            "Failed to copy url plugin '{}': {}",
            plugin.name, e
        )
    })?;
    Ok(())
}

/// Clone `<url>` into `$FORGE_HOME/.tmp/plugin-clone-<name>-<pid>-<nonce>/`,
/// then optionally check out a pinned ref / SHA. Returns the scratch path
/// (caller is responsible for cleaning it up).
async fn clone_to_scratch(
    plugin_name: &str,
    url: &str,
    git_ref: &str,
    sha: &str,
) -> Result<PathBuf, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Build a unique scratch directory under forge_home.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let safe_name: String = plugin_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let scratch = forge_home()
        .join(".tmp")
        .join(format!("plugin-clone-{}-{}-{}", safe_name, pid, nonce));
    if let Some(parent) = scratch.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create scratch dir: {}", e))?;
    }

    log::info!(
        "clone_to_scratch: cloning '{}' for plugin '{}' into '{}'",
        url,
        plugin_name,
        scratch.display()
    );

    let status = std::process::Command::new("git")
        .args(["clone", "--depth=1", url, scratch.to_str().unwrap_or(".")])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| format!("Failed to spawn git clone: {}", e))?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&scratch);
        return Err(format!(
            "git clone of '{}' for plugin '{}' failed with exit code {}",
            url, plugin_name, status
        ));
    }

    // If a SHA is pinned, do a full fetch + checkout so the SHA is
    // reachable. `--depth=1` above only fetches the default branch tip.
    if !sha.is_empty() {
        let fetch_status = std::process::Command::new("git")
            .current_dir(&scratch)
            .args(["fetch", "--depth=1", "origin", sha])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .map_err(|e| format!("Failed to git fetch pinned SHA: {}", e))?;
        if !fetch_status.success() {
            let _ = std::fs::remove_dir_all(&scratch);
            return Err(format!(
                "git fetch of pinned SHA '{}' failed for plugin '{}'",
                sha, plugin_name
            ));
        }
        let co_status = std::process::Command::new("git")
            .current_dir(&scratch)
            .args(["checkout", sha])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .map_err(|e| format!("Failed to git checkout SHA: {}", e))?;
        if !co_status.success() {
            let _ = std::fs::remove_dir_all(&scratch);
            return Err(format!(
                "git checkout of pinned SHA '{}' failed for plugin '{}'",
                sha, plugin_name
            ));
        }
    } else if !git_ref.is_empty() {
        // Refs that aren't default branches also need a fetch.
        let co_status = std::process::Command::new("git")
            .current_dir(&scratch)
            .args(["checkout", git_ref])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .status()
            .map_err(|e| format!("Failed to git checkout ref: {}", e))?;
        if !co_status.success() {
            // Best-effort: the ref may already be the default branch tip.
            log::warn!(
                "git checkout of ref '{}' failed (may already be at HEAD); continuing",
                git_ref
            );
        }
    }

    Ok(scratch)
}

/// Shared post-install steps: validation, activation, manifest update.
fn finalize_installation(
    plugin: &MarketplacePlugin,
    source_key: &str,
    dest: &PathBuf,
) -> Result<PluginInstallResult, String> {
    if !dest.exists() {
        return Err(format!(
            "Plugin '{}' installation failed: destination '{}' not found after copy",
            plugin.name,
            dest.display()
        ));
    }

    activate_plugin(dest)?;

    // Update manifest: write installed_at
    let mut manifest = read_manifest();
    let now = chrono::Local::now().to_rfc3339();
    if let Some(plugins_list) = manifest.plugins.get_mut(source_key) {
        if let Some(p) = plugins_list.iter_mut().find(|p| p.name == plugin.name) {
            p.installed_at = Some(now);
        }
    }
    write_manifest(&manifest).ok(); // Don't fail if manifest write fails

    log::info!(
        "Successfully installed plugin '{}' from source '{}' to {}",
        plugin.name,
        source_key,
        dest.display()
    );

    Ok(PluginInstallResult {
        success: true,
        path: Some(dest.to_string_lossy().to_string()),
        error: None,
    })
}

/// Activate a plugin after installation.
/// This may include:
/// - Extracting any compressed files
/// - Running post-install scripts
/// - Validating plugin structure
fn activate_plugin(plugin_dir: &PathBuf) -> Result<(), String> {
    log::info!("Activating plugin at '{}'", plugin_dir.display());
    
    // Check for and extract .zip files if present
    let plugin_contents = std::fs::read_dir(plugin_dir)
        .map_err(|e| format!("Failed to read plugin directory: {}", e))?;
    
    for entry in plugin_contents.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "zip").unwrap_or(false) {
            log::info!("Found zip file '{}', extracting...", path.display());
            // For now, we just log - zip extraction would require additional dependencies
            // In a production app, you might use a crate like zip
        }
    }
    
    // Check for SKILL.md or manifest files to validate plugin structure
    let has_skill_md = plugin_dir.join("SKILL.md").exists();
    let has_manifest = plugin_dir.join("manifest.json").exists()
        || plugin_dir.join("skill.json").exists()
        || plugin_dir.join("plugin.json").exists();
    
    if !has_skill_md && !has_manifest {
        log::warn!(
            "Plugin at '{}' may not be fully valid (no SKILL.md or manifest found)",
            plugin_dir.display()
        );
        // This is a warning, not an error - some plugins may have different structures
    }
    
    log::info!("Plugin activated successfully");
    Ok(())
}

// ---------------------------------------------------------------------------
// uninstall_plugin — physical delete + manifest.removed[] write (FEAT-009)
// ---------------------------------------------------------------------------

pub async fn uninstall_plugin(
    plugin_id: &str,
    _plugins_dir: &PathBuf,
) -> Result<PluginInstallResult, String> {
    let cache_root = get_cache_dir();
    if !cache_root.exists() {
        return Ok(PluginInstallResult {
            success: false,
            path: None,
            error: Some(format!("Plugin '{}' not found (cache not initialized)", plugin_id)),
        });
    }

    // Resolve source_id and repo_name from manifest for the removed-entry record
    let mut manifest = read_manifest();
    let source_key = manifest.plugins.iter()
        .find(|(_, list)| list.iter().any(|p| p.name == plugin_id))
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let repo_name = resolve_repo_name(&source_key);

    // Find the plugin directory in nested cache structure:
    //   Sub-plugin:     cache/<repo_name>/<plugin_id>/
    //   Repo-is-plugin: cache/<plugin_id>/<plugin_id>/
    let plugin_dir = {
        let sub = cache_root.join(&repo_name).join(plugin_id);
        let root = cache_root.join(plugin_id).join(plugin_id);
        if sub.exists() {
            sub
        } else if root.exists() {
            root
        } else {
            // Fallback: scan all parent directories
            let mut found = None;
            if let Ok(entries) = std::fs::read_dir(&cache_root) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let candidate = entry.path().join(plugin_id);
                    if candidate.is_dir() {
                        found = Some(candidate);
                        break;
                    }
                }
            }
            match found {
                Some(d) => d,
                None => {
                    return Ok(PluginInstallResult {
                        success: false,
                        path: None,
                        error: Some(format!("Plugin '{}' not found in cache", plugin_id)),
                    });
                }
            }
        }
    };

    // Physical delete of the plugin directory
    std::fs::remove_dir_all(&plugin_dir)
        .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
    log::info!(
        "Physically deleted plugin '{}' at {}",
        plugin_id,
        plugin_dir.display()
    );

    // Append to manifest.removed[] — keep plugin entry in manifest.plugins.<source>
    manifest.removed.push(RemovedEntry {
        name: plugin_id.to_string(),
        source: source_key.clone(),
        removed_at: chrono::Local::now().to_rfc3339(),
        reason: "manual uninstall".to_string(),
    });
    write_manifest(&manifest).ok(); // Don't fail if manifest write fails

    log::info!(
        "Recorded uninstall of '{}' from source '{}' in manifest.removed[]",
        plugin_id,
        source_key
    );

    Ok(PluginInstallResult {
        success: true,
        path: Some(plugin_dir.to_string_lossy().to_string()),
        error: None,
    })
}

pub async fn update_plugin(
    _plugin: &MarketplacePlugin,
    _source_command: &str,
    _plugins_dir: &PathBuf,
) -> Result<PluginUpdateResult, String> {
    Ok(PluginUpdateResult {
        success: false,
        new_version: None,
        error: Some(
            "Update not yet implemented; please reinstall via the marketplace".to_string(),
        ),
    })
}

// ---------------------------------------------------------------------------
// set_plugin_disabled — flip the user-controlled enable/disable flag
// persisted in `.claude-plugin/marketplace.json` (lifecycle.ts in the
// Phase 1 plan). The plugin stays on disk; the toggle is purely a UI
// filter hint for the host agent.
// ---------------------------------------------------------------------------

/// Toggle `disabled` for a single plugin. The plugin is identified by
/// `(source_id, name)`. The directory on disk is left untouched — the
/// flag is metadata only. Returns the new disabled value.
pub async fn set_plugin_disabled(
    source_id: &str,
    plugin_name: &str,
    disabled: bool,
) -> Result<bool, String> {
    let mut manifest = read_manifest();

    let entry = manifest
        .plugins
        .get_mut(source_id)
        .and_then(|list| list.iter_mut().find(|p| p.name == plugin_name))
        .ok_or_else(|| {
            format!(
                "Plugin '{}' not found in source '{}' of marketplace.json",
                plugin_name, source_id
            )
        })?;

    entry.disabled = disabled;
    write_manifest(&manifest)?;

    log::info!(
        "Plugin '{}/{}' disabled={} persisted to manifest",
        source_id,
        plugin_name,
        disabled
    );
    Ok(disabled)
}

// ---------------------------------------------------------------------------
// Source installation (FEAT-016) — clone full repo to forge/marketplace/<repo_name>/
// ---------------------------------------------------------------------------

/// Get installation status for all marketplace sources — both the
/// hardcoded preset list AND the user-added sources persisted in
/// `user_sources.json` (FEAT-019).
pub fn get_sources_status() -> Vec<SourceStatus> {
    let presets = get_preset_sources();
    let user_sources = read_user_sources();
    let marketplace_root = marketplace_sources_dir();

    log::info!("[DEBUG] get_sources_status called");
    log::info!("[DEBUG] marketplace_root: {}", marketplace_root.display());
    log::info!("[DEBUG] marketplace_root exists: {}", marketplace_root.exists());
    log::info!(
        "[DEBUG] get_sources_status: {} preset + {} user-added sources",
        presets.len(),
        user_sources.len()
    );

    // Walk the preset list first, then user-added ones. If a user-added
    // id collides with a preset id, or its URL matches a preset URL,
    // skip the user one (preset wins — it's the canonical definition).
    let preset_ids: std::collections::HashSet<String> =
        presets.iter().map(|s| s.id.clone()).collect();
    let preset_urls: std::collections::HashSet<String> = presets
        .iter()
        .map(|s| norm_url(&s.command))
        .collect();
    let mut all: Vec<PluginSource> = presets;
    for us in user_sources {
        if preset_ids.contains(&us.id) {
            continue;
        }
        if preset_urls.contains(&norm_url(&us.command)) {
            continue;
        }
        all.push(us);
    }

    all.into_iter()
        .map(|source| {
            // Use repo_name for directory path (e.g., "claude-plugins-official")
            let repo_name = source.repo_name.as_ref().unwrap_or(&source.id);
            let marketplace_path = marketplace_root.join(repo_name);
            let path_exists = marketplace_path.exists();
            let has_git = is_git_repo(&marketplace_path);
            let marketplace_installed = path_exists && has_git;

            log::info!("[DEBUG] source {}: repo_name={}, path={}, exists={}, has_git={}, installed={}",
                source.id, repo_name, marketplace_path.display(), path_exists, has_git, marketplace_installed);

            // Source is considered installed if marketplace location has it
            let is_installed = marketplace_installed;

            // Collect all installed paths
            let mut installed_paths = Vec::new();
            if marketplace_installed {
                installed_paths.push(marketplace_path.to_string_lossy().to_string());
            }

            // Primary path is the marketplace location
            let primary_path = if marketplace_installed {
                Some(marketplace_path.to_string_lossy().to_string())
            } else {
                None
            };

            SourceStatus {
                source_id: source.id.clone(),
                name: source.name,
                name_zh: source.name_zh,
                repo_url: source.command,
                is_installed,
                installed_path: primary_path.clone(),
                installed_paths,
            }
        })
        .collect()
}

/// Check if a directory is a valid git repository.
fn is_git_repo(path: &PathBuf) -> bool {
    path.join(".git").exists()
}

/// Normalize a repo URL for comparison: strip trailing slashes and `.git`.
fn norm_url(url: &str) -> String {
    url.trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string()
}

/// Install a single marketplace source by cloning its GitHub repository.
///
/// If `source_id` matches a preset, we use the preset's `command` and
/// `repo_name`. Otherwise (user-added source), we accept a `repo_url`
/// fallback and derive the on-disk directory name from the URL via
/// `extract_repo_name_from_url`. This lets the frontend add a custom
/// GitHub source in-memory (FEAT-018) and install it without first
/// registering it server-side.
pub async fn install_marketplace_source(
    source_id: &str,
    repo_url: Option<&str>,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<SourceInstallResult, String> {
    // Try preset first; fall back to a user-provided repo URL for
    // user-added sources (FEAT-018).
    let presets = get_preset_sources();
    let (resolved_url, repo_name) = if let Some(source) = presets.iter().find(|s| s.id == source_id) {
        let rn = source.repo_name.clone().ok_or_else(|| {
            format!("No repo_name configured for source: {}", source_id)
        })?;
        (source.command.clone(), rn)
    } else {
        // User-added source path: require a repo_url and derive repo_name.
        let url = repo_url.ok_or_else(|| {
            format!(
                "Unknown source: {} (and no repo_url provided for user-added source)",
                source_id
            )
        })?;
        let trimmed = url.trim().trim_end_matches('/').to_string();
        if trimmed.is_empty() {
            return Err(format!("Unknown source: {} (empty repo_url)", source_id));
        }
        let rn = extract_repo_name_from_url(&trimmed).ok_or_else(|| {
            format!("Cannot derive repo_name from URL: {}", trimmed)
        })?;
        (trimmed, rn)
    };

    let marketplace_dest = marketplace_sources_dir().join(&repo_name);

    // Collect all installed paths
    let mut installed_paths = Vec::new();

    // Check if already installed in marketplace location
    let marketplace_installed = marketplace_dest.exists() && is_git_repo(&marketplace_dest);

    if marketplace_installed {
        installed_paths.push(marketplace_dest.to_string_lossy().to_string());
    }

    // Idempotent: already installed — skip
    if marketplace_installed {
        log::info!("Source '{}' (repo: '{}') already installed at marketplace", source_id, repo_name);
        return Ok(SourceInstallResult {
            success: true,
            source_id: source_id.to_string(),
            installed_path: Some(marketplace_dest.to_string_lossy().to_string()),
            installed_paths,
            error: None,
        });
    }

    // Clone to marketplace directory
    clone_source_to_path(source_id, &resolved_url, &marketplace_dest, "marketplace", app_handle)?;
    installed_paths.push(marketplace_dest.to_string_lossy().to_string());

    // Emit final progress
    if let Some(handle) = app_handle {
        let _ = handle.emit("source-install-progress", SourceInstallProgress {
            source_id: source_id.to_string(),
            stage: "success".to_string(),
            progress: 100,
            message: "安装完成".to_string(),
        });
    }

    log::info!("Successfully installed marketplace source '{}' (repo: '{}') to {}", source_id, repo_name, marketplace_dest.display());

    Ok(SourceInstallResult {
        success: true,
        source_id: source_id.to_string(),
        installed_path: Some(marketplace_dest.to_string_lossy().to_string()),
        installed_paths,
        error: None,
    })
}

/// Internal helper to clone a source repository to a specific path.
/// Emits progress events for UI updates.
fn clone_source_to_path(
    source_id: &str,
    repo_url: &str,
    dest: &PathBuf,
    location_name: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    // Create parent directory
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {} directory: {}", location_name, e))?;
    }

    // Clean up existing directory if it exists but is not a valid git repo
    if dest.exists() && !is_git_repo(dest) {
        std::fs::remove_dir_all(dest)
            .map_err(|e| format!("Failed to clean existing {} directory: {}", location_name, e))?;
    }

    // Emit progress: preparing
    if let Some(handle) = app_handle {
        let _ = handle.emit("source-install-progress", SourceInstallProgress {
            source_id: source_id.to_string(),
            stage: "preparing".to_string(),
            progress: 10,
            message: format!("正在准备下载到 {}...", location_name),
        });
    }

    log::info!("Cloning marketplace source '{}' from '{}' to '{}' ({})",
        source_id, repo_url, dest.display(), location_name);

    // Emit progress: cloning
    if let Some(handle) = app_handle {
        let _ = handle.emit("source-install-progress", SourceInstallProgress {
            source_id: source_id.to_string(),
            stage: "cloning".to_string(),
            progress: 30,
            message: "正在克隆仓库...".to_string(),
        });
    }

    // Clone the repository (non-bare for full repo access)
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth=1",
            repo_url,
            dest.to_str().unwrap_or("."),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| format!("Failed to spawn git clone: {}", e))?;

    if !status.success() {
        // Emit progress: failed
        if let Some(handle) = app_handle {
            let _ = handle.emit("source-install-progress", SourceInstallProgress {
                source_id: source_id.to_string(),
                stage: "failed".to_string(),
                progress: 0,
                message: format!("克隆失败: git clone exited with {}", status),
            });
        }
        return Err(format!("git clone failed for {} location with exit code: {}", location_name, status));
    }

    // Emit progress: success
    if let Some(handle) = app_handle {
        let _ = handle.emit("source-install-progress", SourceInstallProgress {
            source_id: source_id.to_string(),
            stage: "success".to_string(),
            progress: 100,
            message: format!("已克隆到 {}", location_name),
        });
    }

    log::info!("Successfully cloned to {} at '{}'", location_name, dest.display());
    Ok(())
}

pub async fn add_source(
    _source: &PluginSource,
    _cursor_dir: &PathBuf,
) -> Result<PluginInstallResult, String> {
    Ok(PluginInstallResult {
        success: true,
        path: None,
        error: None,
    })
}

// ---------------------------------------------------------------------------
// Installed Registry — Feature 5
// ---------------------------------------------------------------------------

/// Installed plugin registry persisted at `$FORGE_HOME/plugins/installed_plugins.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstalledRegistry {
    pub version: String,
    pub plugins: std::collections::HashMap<String, InstalledEntry>,
    #[serde(default)]
    pub last_sweep_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledEntry {
    pub source_id: String,
    pub plugin_name: String,
    pub version: Option<String>,
    pub installed_at: Option<String>,
    pub last_inuse_at: Option<String>,
    pub install_path: String,
}

pub fn read_installed_registry() -> InstalledRegistry {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        if let Some(reg) = kv.get::<InstalledRegistry>("installed_plugin_registry") {
            return reg;
        }
    }
    let path = installed_plugins_path();
    if !path.exists() {
        return InstalledRegistry::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            log::error!("Failed to parse installed_plugins.json: {}", e);
            InstalledRegistry::default()
        }),
        Err(e) => {
            log::error!("Failed to read installed_plugins.json: {}", e);
            InstalledRegistry::default()
        }
    }
}

pub fn write_installed_registry(reg: &InstalledRegistry) -> Result<(), String> {
    if let Some(db) = crate::db::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.put("installed_plugin_registry", reg);
    }
    let path = installed_plugins_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create marketplace dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(reg)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write installed_plugins.json: {}", e))?;
    Ok(())
}

/// Update the last_inuse_at timestamp for a plugin.
pub fn update_plugin_inuse(source_id: &str, plugin_name: &str) -> Result<(), String> {
    let mut reg = read_installed_registry();
    let key = format!("{}/{}", source_id, plugin_name);
    if let Some(entry) = reg.plugins.get_mut(&key) {
        entry.last_inuse_at = Some(chrono::Local::now().to_rfc3339());
    }
    write_installed_registry(&reg)
}

/// Scan all installed plugin directories and update their mtime-based inuse timestamps.
/// Returns the number of plugins updated.
pub fn sweep_inuse() -> Result<usize, String> {
    let root = plugins_dir();
    if !root.exists() {
        return Ok(0);
    }

    let mut reg = read_installed_registry();
    let mut updated = 0usize;

    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let source_path = entry.path();
            if !source_path.is_dir() { continue; }
            let source_id = source_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Ok(plugin_entries) = std::fs::read_dir(&source_path) {
                for plugin_entry in plugin_entries.filter_map(|e| e.ok()) {
                    let plugin_path = plugin_entry.path();
                    if !plugin_path.is_dir() { continue; }
                    let plugin_name = plugin_path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let key = format!("{}/{}", source_id, plugin_name);
                    let mtime = plugin_path.metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .map(|t| {
                            let datetime: chrono::DateTime<chrono::Local> = t.into();
                            datetime.to_rfc3339()
                        });

                    if let Some(entry) = reg.plugins.get_mut(&key) {
                        if let Some(ts) = mtime {
                            if entry.last_inuse_at.as_ref() != Some(&ts) {
                                entry.last_inuse_at = Some(ts);
                                updated += 1;
                            }
                        }
                    } else {
                        // Register any new plugin that wasn't previously tracked
                        reg.plugins.insert(key.clone(), InstalledEntry {
                            source_id: source_id.clone(),
                            plugin_name: plugin_name.clone(),
                            version: None,
                            installed_at: mtime.clone(),
                            last_inuse_at: mtime,
                            install_path: plugin_path.to_string_lossy().to_string(),
                        });
                        updated += 1;
                    }
                }
            }
        }
    }

    reg.last_sweep_at = Some(chrono::Local::now().to_rfc3339());
    write_installed_registry(&reg)?;

    log::info!("sweep_inuse: updated {} plugin inuse timestamps", updated);
    Ok(updated)
}

// ---------------------------------------------------------------------------
// Tests — "repo is itself a plugin" detection
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Build a unique temp directory under `$TMPDIR/forge-test-self-plugin-<id>-<pid>/`
    /// so each test gets an isolated fixture.
    fn make_temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "forge-test-self-plugin-{}-{}-{}",
            label,
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        if dir.exists() {
            let _ = fs::remove_dir_all(&dir);
        }
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Write a minimal `.claude-plugin/marketplace.json` with an empty
    /// `plugins` array. Mirrors what a "single-plugin repo" ships:
    /// the manifest exists, but it advertises zero entries because the
    /// repo itself is the plugin.
    fn write_empty_marketplace(source_dir: &Path) {
        let claude_plugin_dir = source_dir.join(".claude-plugin");
        fs::create_dir_all(&claude_plugin_dir).expect("create .claude-plugin");
        let manifest = serde_json::json!({
            "name": source_dir.file_name().unwrap().to_string_lossy(),
            "owner": {
                "name": "Test Author"
            },
            "plugins": []
        });
        fs::write(
            claude_plugin_dir.join("marketplace.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .expect("write marketplace.json");
    }

    #[test]
    fn empty_marketplace_with_skill_md_counts_as_one() {
        let dir = make_temp_dir("skill-md");
        write_empty_marketplace(&dir);
        // Repo root has SKILL.md → repo IS the plugin
        fs::write(dir.join("SKILL.md"), "# Some skill").unwrap();

        let count = read_source_marketplace_json(&dir).expect("read_source_marketplace_json");
        assert_eq!(count, 1, "SKILL.md at root + empty marketplace → count must be 1");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_marketplace_with_manifest_json_counts_as_one() {
        let dir = make_temp_dir("manifest");
        write_empty_marketplace(&dir);
        fs::write(
            dir.join("manifest.json"),
            r#"{"name":"demo","version":"0.1.0","author":"alice"}"#,
        )
        .unwrap();

        let count = read_source_marketplace_json(&dir).expect("read_source_marketplace_json");
        assert_eq!(count, 1, "manifest.json at root + empty marketplace → count must be 1");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_marketplace_with_plugin_json_counts_as_one() {
        let dir = make_temp_dir("plugin-json");
        write_empty_marketplace(&dir);
        fs::write(dir.join("plugin.json"), r#"{"name":"demo"}"#).unwrap();

        let count = read_source_marketplace_json(&dir).expect("read_source_marketplace_json");
        assert_eq!(count, 1, "plugin.json at root + empty marketplace → count must be 1");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_marketplace_without_marker_stays_zero() {
        // No plugin markers at root → repo is NOT itself a plugin,
        // empty marketplace truly means 0.
        let dir = make_temp_dir("no-marker");
        write_empty_marketplace(&dir);
        // Add some unrelated file so it's not an empty dir
        fs::write(dir.join("README.md"), "# Just a readme").unwrap();

        let count = read_source_marketplace_json(&dir).expect("read_source_marketplace_json");
        assert_eq!(count, 0, "no plugin marker at root → count must stay 0");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn nonempty_marketplace_uses_explicit_count() {
        // When the marketplace.json explicitly lists plugins, we use
        // that count — the "repo is a plugin" inference must NOT
        // override an explicit list.
        let dir = make_temp_dir("nonempty");
        let claude_plugin_dir = dir.join(".claude-plugin");
        fs::create_dir_all(&claude_plugin_dir).unwrap();
        let manifest = serde_json::json!({
            "plugins": [
                { "name": "alpha", "source": "./plugins/alpha" },
                { "name": "beta",  "source": "./plugins/beta" },
            ]
        });
        fs::write(
            claude_plugin_dir.join("marketplace.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        // Even with SKILL.md at the root, the explicit count wins.
        fs::write(dir.join("SKILL.md"), "# marker").unwrap();

        let count = read_source_marketplace_json(&dir).expect("read_source_marketplace_json");
        assert_eq!(count, 2, "explicit plugins[] length wins over marker inference");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_marketplace_with_marker_synthesizes_plugin_entry() {
        let dir = make_temp_dir("synthesize");
        write_empty_marketplace(&dir);
        fs::write(
            dir.join("SKILL.md"),
            "# Awesome Plugin\n\nA one-line description that is long enough to pass the README heuristic.",
        )
        .unwrap();

        let plugins = read_source_marketplace_plugins(&dir, "anthropics")
            .expect("read_source_marketplace_plugins");
        assert_eq!(
            plugins.len(),
            1,
            "empty marketplace + marker at root → 1 synthesized plugin entry"
        );
        let p = &plugins[0];
        assert_eq!(p.id, dir.file_name().unwrap().to_string_lossy());
        assert_eq!(p.source_id, "anthropics");
        // The synthesized entry uses install_source.kind = "url" so
        // install_plugin clones the whole repo.
        let src = p.install_source.as_ref().expect("install_source set");
        assert_eq!(src.kind, "url");
        // install_path must point at the on-disk repo so the UI can
        // render the "Installed at:" row.
        assert_eq!(
            p.install_path.as_deref(),
            Some(dir.to_string_lossy().as_ref())
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_marketplace_without_marker_returns_empty_vec() {
        // No marker at root → marketplace.json's empty list stays
        // empty, no synthesized entry.
        let dir = make_temp_dir("empty-no-synth");
        write_empty_marketplace(&dir);

        let plugins = read_source_marketplace_plugins(&dir, "anthropics")
            .expect("read_source_marketplace_plugins");
        assert!(
            plugins.is_empty(),
            "no marker at root → no synthesized plugin"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn repo_root_has_plugin_marker_handles_known_filenames() {
        // Spot-check the heuristic: each known marker individually.
        for marker in ["SKILL.md", "skill.md", "manifest.json", "plugin.json"] {
            let dir = make_temp_dir(&format!("marker-{}", marker));
            write_empty_marketplace(&dir);
            fs::write(dir.join(marker), "x").unwrap();
            assert!(
                repo_root_has_plugin_marker(&dir),
                "{} should be recognized as a plugin marker",
                marker
            );
            let _ = fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn missing_marketplace_json_returns_none() {
        let dir = make_temp_dir("missing-json");
        // No .claude-plugin/marketplace.json at all
        let count = read_source_marketplace_json(&dir);
        assert!(count.is_none(), "missing marketplace.json → None");

        let _ = fs::remove_dir_all(&dir);
    }
}
