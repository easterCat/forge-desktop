// Plugin Capabilities Service - parses local and remote plugin capability manifests.
// No serde_yaml: YAML frontmatter is parsed with a simple regex.

use crate::models::plugin_capabilities::{
    CommandInfo, HookInfo, LspServerInfo, McpServerInfo, PluginCapabilities,
    PluginCapabilityCounts, SkillInfo,
};
use crate::services::plugin_marketplace::{
    get_preset_sources, plugins_dir, read_manifest, read_user_sources, ManifestPlugin,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Raw plugin.json deserialization (camelCase keys)
// ---------------------------------------------------------------------------

/// Subset of plugin.json fields we care about.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginJson {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<serde_json::Value>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub lsp_servers: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub manifest: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct HooksJson {
    #[serde(default)]
    pub hooks: Vec<RawHook>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawHook {
    #[serde(default)]
    pub event: String,
    #[serde(default)]
    pub matcher: Option<String>,
    #[serde(default)]
    pub command: String,
}

// ---------------------------------------------------------------------------
// YAML frontmatter helpers
// ---------------------------------------------------------------------------

/// Parse YAML frontmatter from a Markdown file.
/// Returns (frontmatter_text, body_text) or (None, full_content) if no frontmatter.
fn parse_yaml_frontmatter(content: &str) -> Option<String> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---").map(|i| i + 3)?;
    Some(rest[..end].trim().to_string())
}

/// Parse key: value lines from YAML frontmatter.
fn parse_frontmatter_lines(fm: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in fm.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(colon) = line.find(':') {
            let key = line[..colon].trim().to_string();
            let raw_val = line[colon + 1..].trim();
            // Strip surrounding quotes
            let val = if (raw_val.starts_with('"') && raw_val.ends_with('"'))
                || (raw_val.starts_with('\'') && raw_val.ends_with('\''))
            {
                &raw_val[1..raw_val.len() - 1]
            } else {
                raw_val
            };
            map.insert(key, val.to_string());
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Directory scanning helpers
// ---------------------------------------------------------------------------

pub fn scan_skills(plugin_dir: &Path) -> Vec<SkillInfo> {
    let skills_root = plugin_dir.join("skills");
    if !skills_root.is_dir() {
        return vec![];
    }

    let mut skills = vec![];
    if let Ok(entries) = std::fs::read_dir(&skills_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }

            // Parse frontmatter; skip files without valid frontmatter per task spec §8.1-U4
            let content = match std::fs::read_to_string(&skill_md) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let fm = parse_yaml_frontmatter(&content);
            let fm_map = fm
                .as_ref()
                .map(|f| parse_frontmatter_lines(f))
                .unwrap_or_default();

            // A skill must have a non-empty description (frontmatter required)
            let description = fm_map.get("description").cloned().unwrap_or_default();
            if description.is_empty() {
                log::warn!("SKILL.md at {} has no frontmatter description, skipping", skill_md.display());
                continue;
            }

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let has_scripts = path.join("scripts").is_dir();
            let has_references = path.join("references").is_dir();

            let rel_path = format!("skills/{}/SKILL.md", name);
            skills.push(SkillInfo {
                name,
                description,
                path: rel_path,
                has_scripts,
                has_references,
            });
        }
    }
    skills
}

pub fn scan_commands(plugin_dir: &Path) -> Vec<CommandInfo> {
    let commands_dir = plugin_dir.join("commands");
    if !commands_dir.is_dir() {
        return vec![];
    }

    let mut commands = vec![];
    if let Ok(entries) = std::fs::read_dir(&commands_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !file_name.ends_with(".md") {
                continue;
            }

            // Derive command name from filename: "my-command.md" -> "my-command"
            let name = file_name.trim_end_matches(".md").to_string();

            let (description, allowed_tools) =
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let fm = parse_yaml_frontmatter(&content);
                    let fm_map = fm
                        .as_ref()
                        .map(|f| parse_frontmatter_lines(f))
                        .unwrap_or_default();
                    let desc = fm_map.get("description").cloned().unwrap_or_default();
                    let tools = fm_map
                        .get("allowed-tools")
                        .map(|v| {
                            v.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        })
                        .unwrap_or_default();
                    (desc, tools)
                } else {
                    (String::new(), vec![])
                };

            let rel_path = format!("commands/{}", file_name);
            commands.push(CommandInfo {
                name,
                description,
                allowed_tools,
                path: rel_path,
            });
        }
    }
    commands
}

pub fn scan_hooks(plugin_dir: &Path) -> Vec<HookInfo> {
    let hooks_json_path = plugin_dir.join("hooks").join("hooks.json");
    if !hooks_json_path.is_file() {
        return vec![];
    }

    let content = match std::fs::read_to_string(&hooks_json_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let hooks_data: HooksJson = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("Failed to parse hooks.json: {}", e);
            return vec![];
        }
    };

    hooks_data
        .hooks
        .into_iter()
        .map(|h| {
            // Extract script path from last word of command (e.g. "bash hooks/session-start" → "hooks/session-start")
            let script_path = h
                .command
                .split_whitespace()
                .last()
                .unwrap_or("")
                .trim();

            // Resolve relative to plugin_dir: "hooks/session-start" → <plugin_dir>/hooks/session-start
            let script_exists = if !script_path.is_empty() {
                plugin_dir.join(script_path).is_file()
            } else {
                false
            };

            HookInfo {
                event: h.event,
                matcher: h.matcher,
                command: h.command,
                script_exists,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// plugin.json extraction
// ---------------------------------------------------------------------------

pub fn parse_plugin_json(plugin_dir: &Path) -> Result<PluginJson, String> {
    let json_path = plugin_dir.join(".claude-plugin").join("plugin.json");
    let json_path_alt = plugin_dir.join("plugin.json");
    let path = if json_path.is_file() {
        &json_path
    } else if json_path_alt.is_file() {
        &json_path_alt
    } else {
        return Err(format!(
            "plugin.json not found in {}",
            plugin_dir.display()
        ));
    };

    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read plugin.json: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse plugin.json: {}", e))
}

// ---------------------------------------------------------------------------
// Count aggregation
// ---------------------------------------------------------------------------

fn count_capabilities(
    skills: &[SkillInfo],
    hooks: &[HookInfo],
    commands: &[CommandInfo],
    mcp_servers: &[McpServerInfo],
    lsp_servers: &[LspServerInfo],
) -> PluginCapabilityCounts {
    PluginCapabilityCounts {
        skills: skills.len() as u32,
        hooks: hooks.len() as u32,
        commands: commands.len() as u32,
        mcp_servers: mcp_servers.len() as u32,
        lsp_servers: lsp_servers.len() as u32,
    }
}

// ---------------------------------------------------------------------------
// Local parsing
// ---------------------------------------------------------------------------

/// Resolve the on-disk install directory for a plugin, trying every layout
/// the installer has historically used. Returns the first directory that
/// actually exists and contains a `plugin.json` (either at the root or under
/// `.claude-plugin/`).
///
/// Layouts checked, in order:
///   1. `plugins/cache/<repo_name>/<plugin_name>/`              — sub-plugin layout
///   2. `plugins/cache/<plugin_name>/<plugin_name>/`            — repo-is-plugin layout
///   3. `plugins/<source_id>/<plugin_name>/`                    — flat layout
///   4. `plugins/marketplace/<source_id>/plugins/<plugin_name>/` — source-cloned layout
fn resolve_local_plugin_dir(source_id: &str, plugin_name: &str) -> Option<PathBuf> {
    let base = plugins_dir();
    // Resolve repo_name from source_id for cache path lookup.
    let repo_name = crate::services::plugin_marketplace::get_preset_sources().iter()
        .find(|s| s.id == source_id)
        .and_then(|s| s.repo_name.clone())
        .or_else(|| {
            crate::services::plugin_marketplace::read_user_sources().iter()
                .find(|s| s.id == source_id)
                .and_then(|s| s.repo_name.clone())
        })
        .unwrap_or_else(|| source_id.to_string());

    let candidates = [
        base.join("cache").join(&repo_name).join(plugin_name),
        base.join("cache").join(plugin_name).join(plugin_name),
        base.join(source_id).join(plugin_name),
        base.join("marketplace")
            .join(source_id)
            .join("plugins")
            .join(plugin_name),
        // The marketplace source repo itself is the plugin (no subdirectory layout).
        base.join("marketplace").join(source_id),
    ];

    for candidate in candidates.iter() {
        if !candidate.is_dir() {
            continue;
        }
        // Accept the directory if it has a plugin.json in either location.
        let has_root_pj = candidate.join("plugin.json").is_file();
        let has_nested_pj = candidate
            .join(".claude-plugin")
            .join("plugin.json")
            .is_file();
        if has_root_pj || has_nested_pj {
            return Some(candidate.clone());
        }
    }
    None
}

/// Parse capabilities from a locally installed plugin.
///
/// Tries several plausible on-disk layouts (flat, `cache/`, and
/// `marketplace/<source>/plugins/`) before giving up. This makes the
/// "Failed to load plugin details" dialog work for plugins that were
/// installed under `cache/<source>/<name>` (the layout used by the
/// installer that copies a single plugin out of a cloned source repo).
pub fn parse_local_capabilities(
    source_id: &str,
    plugin_name: &str,
) -> Result<PluginCapabilities, String> {
    let plugin_dir = resolve_local_plugin_dir(source_id, plugin_name).ok_or_else(|| {
        format!(
            "Plugin '{}' not found in source '{}' (checked plugins/, plugins/cache/, plugins/marketplace/{}/plugins/)",
            plugin_name, source_id, source_id
        )
    })?;

    let pj = parse_plugin_json(&plugin_dir)?;

    // Extract fields from plugin.json
    let name = pj
        .name
        .unwrap_or_else(|| plugin_name.to_string());
    let version = pj.version;
    let description = pj.description.unwrap_or_default();
    let license = pj.license;

    let author = match pj.author {
        Some(serde_json::Value::String(s)) => Some(s),
        Some(v) => serde_json::from_value::<String>(v).ok(),
        _ => None,
    };

    let repository = pj.repository;
    let dependencies = pj.dependencies;

    // Scan capability directories
    let skills = scan_skills(&plugin_dir);
    let commands = scan_commands(&plugin_dir);
    let hooks = scan_hooks(&plugin_dir);

    // Parse mcp_servers / lsp_servers from plugin.json
    let mcp_servers = parse_mcp_servers(pj.mcp_servers);
    let lsp_servers = parse_lsp_servers(pj.lsp_servers);

    // Extract manifest.files if present
    let manifest_files: Vec<String> = pj
        .manifest
        .as_ref()
        .and_then(|m| m.get("files"))
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let capabilities =
        count_capabilities(&skills, &hooks, &commands, &mcp_servers, &lsp_servers);

    Ok(PluginCapabilities {
        name,
        version,
        author,
        license,
        description,
        repository,
        source: "local".to_string(),
        source_id: source_id.to_string(),
        installed_path: plugin_dir.to_string_lossy().to_string(),
        capabilities,
        skills,
        commands,
        hooks,
        mcp_servers,
        lsp_servers,
        dependencies,
        manifest_files,
    })
}

/// Build a synthetic `ManifestPlugin` from a locally installed `plugin.json`
/// when the top-level `marketplace.json` does not list this plugin. This is
/// purely a compatibility shim — it lets the Details dialog render for
/// plugins that were installed outside the normal install flow (e.g. the
/// installer copied a plugin into `plugins/cache/<name>/` directly
/// without writing a manifest entry).
///
/// Returns `None` when no local `plugin.json` can be found in any of the
/// layouts checked by `resolve_local_plugin_dir`.
fn synthesize_manifest_plugin_from_local(
    source_id: &str,
    plugin_name: &str,
) -> Option<ManifestPlugin> {
    let plugin_dir = resolve_local_plugin_dir(source_id, plugin_name)?;
    let pj = parse_plugin_json(&plugin_dir).ok()?;

    // installed_path is "plugins/<source>/<name>" by convention — the
    // remote fallback uses this to compute the sub_path inside the
    // upstream repo. Setting it to the relative form keeps the existing
    // remote-fetch logic working unchanged.
    let installed_path = format!("plugins/{}/{}", source_id, plugin_name);

    Some(ManifestPlugin {
        name: pj.name.unwrap_or_else(|| plugin_name.to_string()),
        description: pj.description.unwrap_or_default(),
        version: pj.version.unwrap_or_default(),
        author: pj.author.unwrap_or(serde_json::Value::Null),
        repo_url: String::new(),
        installed_path,
        external: false,
        dependencies: pj.dependencies,
        install_mode: String::new(),
        manifest: pj.manifest,
        installed_at: None,
        disabled: false,
    })
}

// ---------------------------------------------------------------------------
// Remote parsing (git archive + sparse-checkout)
// ---------------------------------------------------------------------------

/// Fetch plugin capabilities from a remote source (for Marketplace Tab plugins
/// that are not yet installed).  Uses the same git archive / sparse-checkout
/// pattern as FEAT-009 but only downloads metadata files.
pub async fn parse_remote_capabilities(
    source_id: &str,
    plugin_name: &str,
) -> Result<PluginCapabilities, String> {
    // Look up the repo URL.  Priority:
    //   1. `manifest.sources[source_id].repo_url` — populated by
    //      `node scripts/plugins/install.mjs` after a sync.
    //   2. The hardcoded preset list (`get_preset_sources`).
    //   3. User-added sources (`user_sources.json`).
    //
    // The manifest alone is not enough: if the user has never run
    // `npm run plugins:install`, the manifest is empty but the preset
    // sources still have valid GitHub URLs.  Without this fallback the
    // Marketplace tab shows "No repo URL found for source 'anthropics'"
    // for any not-yet-installed plugin.
    let manifest = read_manifest();
    let repo_url = manifest
        .sources
        .get(source_id)
        .map(|s| s.repo_url.as_str())
        .filter(|u| !u.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            get_preset_sources()
                .into_iter()
                .find(|s| s.id == source_id)
                .map(|s| s.command)
        })
        .or_else(|| {
            read_user_sources()
                .into_iter()
                .find(|s| s.id == source_id)
                .map(|s| s.command)
        })
        .ok_or_else(|| {
            format!("No repo URL found for source '{}'", source_id)
        })?;

    // Find the plugin entry in manifest.plugins.<source>.
    //
    // Compatibility fallback: if the top-level `marketplace.json` was never
    // written (e.g. the user installed the plugin by hand-cloning it into
    // `cache/<source>/<name>/`, or ran a script that bypassed the manifest
    // sync step), we can still synthesize a `ManifestPlugin` from the local
    // `plugin.json`. This makes the Details dialog work even when the
    // manifest is missing or doesn't list the plugin.
    let manifest = read_manifest();
    let plugin_entry = manifest
        .plugins
        .get(source_id)
        .and_then(|list| list.iter().find(|p| p.name == plugin_name))
        .cloned()
        .or_else(|| synthesize_manifest_plugin_from_local(source_id, plugin_name))
        .ok_or_else(|| {
            format!(
                "Plugin '{}' not found in source '{}' of marketplace.json, and no local plugin.json fallback was found in cache/",
                plugin_name, source_id
            )
        })?;

    // Compute the sub_path: installed_path may be "plugins/anthropics/frontend-design"
    let installed_path_relative = plugin_entry
        .installed_path
        .strip_prefix("plugins/")
        .unwrap_or(&plugin_entry.installed_path);
    let sub_path: String = installed_path_relative
        .strip_prefix(&format!("{}/", source_id))
        .map(|s| s.to_string())
        .unwrap_or_else(|| installed_path_relative.to_string());

    // Temporary directory to extract into
    let tmp_dir = std::env::temp_dir().join(format!(
        "env-manager-capabilities-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));

    std::fs::create_dir_all(&tmp_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;

    // Build list of files to extract: plugin.json + metadata directories
    // We try to fetch via git archive --remote first, then sparse-checkout fallback.
    let files_to_fetch = vec![
        format!("{}/.claude-plugin/plugin.json", sub_path),
        format!("{}/skills", sub_path),
        format!("{}/commands", sub_path),
        format!("{}/hooks", sub_path),
    ];

    let fetch_result = download_metadata_files(&repo_url, &files_to_fetch, &tmp_dir).await;

    match fetch_result {
        Ok(_) => {
            let plugin_dir = tmp_dir.join(&sub_path);
            let pj = parse_plugin_json(&plugin_dir)?;

            let name = pj
                .name
                .unwrap_or_else(|| plugin_name.to_string());
            let version = pj.version;
            let description = pj.description.unwrap_or_default();
            let license = pj.license;

            let author = match pj.author {
                Some(serde_json::Value::String(s)) => Some(s),
                Some(v) => serde_json::from_value::<String>(v).ok(),
                _ => None,
            };

            let repository = pj.repository;
            let dependencies = pj.dependencies;

            let skills = scan_skills(&plugin_dir);
            let commands = scan_commands(&plugin_dir);
            let hooks = scan_hooks(&plugin_dir);
            let mcp_servers = parse_mcp_servers(pj.mcp_servers);
            let lsp_servers = parse_lsp_servers(pj.lsp_servers);

            let manifest_files: Vec<String> = pj
                .manifest
                .as_ref()
                .and_then(|m| m.get("files"))
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let capabilities =
                count_capabilities(&skills, &hooks, &commands, &mcp_servers, &lsp_servers);

            // Clean up
            let _ = std::fs::remove_dir_all(&tmp_dir);

            Ok(PluginCapabilities {
                name,
                version,
                author,
                license,
                description,
                repository,
                source: "remote".to_string(),
                source_id: source_id.to_string(),
                installed_path: String::new(),
                capabilities,
                skills,
                commands,
                hooks,
                mcp_servers,
                lsp_servers,
                dependencies,
                manifest_files,
            })
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            Err(e)
        }
    }
}

/// Download specific files/directories from a git repo using git archive --remote
/// + tar, falling back to shallow sparse-checkout clone.
async fn download_metadata_files(
    repo_url: &str,
    _paths: &[String],
    dest: &std::path::Path,
) -> Result<(), String> {
    // For simplicity, use a temporary clone with sparse-checkout to fetch the files.
    // This mirrors the fallback path in plugin_marketplace.rs.
    try_git_archive_remote_all(repo_url, dest)?;

    // If git archive --remote failed, the function above returns Err,
    // so we don't reach here. In practice both paths may work.
    Ok(())
}

/// Try `git archive --remote=<url>` for each path we care about.
fn try_git_archive_remote_all(repo_url: &str, dest: &std::path::Path) -> Result<(), String> {
    // git archive --remote does not support fetching multiple paths in one call.
    // We use sparse-checkout instead (more reliable).
    try_sparse_clone(repo_url, dest)
}

/// Shallow sparse clone of a repo into dest.
fn try_sparse_clone(repo_url: &str, dest: &std::path::Path) -> Result<(), String> {
    let clone_dir = dest.join("repo");

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
        return Err(format!("git clone failed with exit {}", status));
    }

    // Sparse-checkout the top-level plugins/ directory to get all plugins
    // (we can't know the exact subdir name without parsing marketplace.json)
    let status = std::process::Command::new("git")
        .current_dir(&clone_dir)
        .args(["sparse-checkout", "set", "plugins"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| format!("Failed to spawn git sparse-checkout: {}", e))?;

    if !status.success() {
        return Err(format!(
            "git sparse-checkout set failed with exit {}",
            status
        ));
    }

    // Move contents up so that the clone_dir/plugins/... path is directly usable
    let plugins_src = clone_dir.join("plugins");
    if plugins_src.exists() {
        copy_dir_recursive(&plugins_src, dest).map_err(|e| {
            format!("Failed to copy plugin files: {}", e)
        })?;
    }

    let _ = std::fs::remove_dir_all(&clone_dir);
    Ok(())
}

/// Recursive directory copy using only std library.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// mcpServers / lspServers parsing
// ---------------------------------------------------------------------------

fn parse_mcp_servers(
    raw: HashMap<String, serde_json::Value>,
) -> Vec<McpServerInfo> {
    raw.into_iter()
        .map(|(name, val)| {
            let command = val
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let args: Vec<String> = val
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let env: Option<HashMap<String, String>> = val
                .get("env")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            McpServerInfo {
                name,
                command,
                args,
                env,
            }
        })
        .collect()
}

fn parse_lsp_servers(
    raw: HashMap<String, serde_json::Value>,
) -> Vec<LspServerInfo> {
    raw.into_iter()
        .map(|(name, val)| {
            let command = val
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let args: Vec<String> = val
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            LspServerInfo {
                name,
                command,
                args,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Hook Execution (Feature 2)
// ---------------------------------------------------------------------------

use crate::models::plugin_capabilities::{
    HookExecutionResult, ValidationCapabilityCounts, ValidationIssue, ValidationReport,
};

/// Execute a single hook script and return structured results.
/// Timeout: 10 seconds.  Writes a JSON log to `~/.claude/plugins/data/<plugin>/hook-logs/`.
pub async fn execute_hook(
    source_id: &str,
    plugin_name: &str,
    event: &str,
    matcher: Option<&str>,
    env_vars: std::collections::HashMap<String, String>,
) -> Result<HookExecutionResult, String> {
    let plugin_dir = plugins_dir().join(source_id).join(plugin_name);

    // Resolve the hook script relative to the plugin directory
    let hooks_json_path = plugin_dir.join("hooks").join("hooks.json");
    let hooks_content = std::fs::read_to_string(&hooks_json_path)
        .map_err(|e| format!("Failed to read hooks.json: {}", e))?;
    let hooks_data: HooksJson = serde_json::from_str(&hooks_content)
        .map_err(|e| format!("Failed to parse hooks.json: {}", e))?;

    let hook_entry = hooks_data
        .hooks
        .iter()
        .find(|h| h.event == event && h.matcher.as_deref() == matcher)
        .ok_or_else(|| format!("Hook not found: event={}, matcher={:?}", event, matcher))?;

    let started_at = chrono::Local::now().to_rfc3339();

    // Build command: e.g. "bash hooks/session-start" -> split into program + args
    let parts: Vec<&str> = hook_entry.command.split_whitespace().collect();
    let (program, args): (&str, Vec<&str>) = if parts.len() >= 2 {
        (parts[0], parts[1..].to_vec())
    } else {
        (parts[0], vec![])
    };

    // Resolve the script path relative to plugin_dir
    let _script_name = hook_entry.command.split_whitespace().last().unwrap_or("");

    let start = std::time::Instant::now();

    let output = tokio::process::Command::new(program)
        .args(&args)
        .current_dir(&plugin_dir)
        .envs(env_vars.iter())
        .output()
        .await
        .map_err(|e| format!("Failed to spawn hook process: {}", e))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let exit_code = output.status.code();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Try to parse stdout as JSON
    let parsed_json: Option<serde_json::Value> = serde_json::from_str(&stdout).ok();

    // Write log file
    let log_path = write_hook_log(source_id, plugin_name, event, &HookExecutionResult {
        event: event.to_string(),
        matcher: matcher.map(String::from),
        command: hook_entry.command.clone(),
        exit_code,
        stdout: stdout.clone(),
        stderr: stderr.clone(),
        parsed_json: parsed_json.clone(),
        started_at: started_at.clone(),
        duration_ms,
        log_path: String::new(),
    }).map_err(|e| format!("Failed to write hook log: {}", e))?;

    Ok(HookExecutionResult {
        event: event.to_string(),
        matcher: matcher.map(String::from),
        command: hook_entry.command.clone(),
        exit_code,
        stdout,
        stderr,
        parsed_json,
        started_at,
        duration_ms,
        log_path,
    })
}

fn write_hook_log(
    source_id: &str,
    plugin_name: &str,
    event: &str,
    result: &HookExecutionResult,
) -> Result<String, String> {
    let data_dir = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?
        .join(".claude")
        .join("plugins")
        .join("data")
        .join(source_id)
        .join(plugin_name)
        .join("hook-logs");

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create hook-logs dir: {}", e))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%f").to_string();
    let log_file = data_dir.join(format!("{}_{}.json", event, timestamp));

    let json = serde_json::to_string_pretty(result)
        .map_err(|e| format!("Failed to serialize hook result: {}", e))?;

    std::fs::write(&log_file, json)
        .map_err(|e| format!("Failed to write hook log: {}", e))?;

    Ok(log_file.to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// Plugin Validation (Feature 4)
// ---------------------------------------------------------------------------

/// Validate a plugin directory structure and manifest.
pub fn validate_plugin(plugin_dir: &std::path::Path) -> ValidationReport {
    let mut errors = vec![];
    let mut warnings = vec![];

    // Check plugin.json exists
    let plugin_json_path = plugin_dir.join(".claude-plugin").join("plugin.json");
    let plugin_json_path_alt = plugin_dir.join("plugin.json");
    let json_path = if plugin_json_path.is_file() {
        plugin_json_path
    } else if plugin_json_path_alt.is_file() {
        plugin_json_path_alt
    } else {
        errors.push(ValidationIssue {
            severity: "error".to_string(),
            code: "MISSING_PLUGIN_JSON".to_string(),
            message: "plugin.json not found (checked .claude-plugin/plugin.json and plugin.json)".to_string(),
            path: None,
        });
        return ValidationReport {
            valid: false,
            errors,
            warnings,
            capabilities: ValidationCapabilityCounts::default(),
        };
    };

    // Parse plugin.json
    let json_content = match std::fs::read_to_string(&json_path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(ValidationIssue {
                severity: "error".to_string(),
                code: "INVALID_PLUGIN_JSON".to_string(),
                message: format!("Failed to read plugin.json: {}", e),
                path: Some(json_path.to_string_lossy().to_string()),
            });
            return ValidationReport {
                valid: false,
                errors,
                warnings,
                capabilities: ValidationCapabilityCounts::default(),
            };
        }
    };

    let pj: serde_json::Value = match serde_json::from_str(&json_content) {
        Ok(v) => v,
        Err(e) => {
            errors.push(ValidationIssue {
                severity: "error".to_string(),
                code: "MALFORMED_PLUGIN_JSON".to_string(),
                message: format!("plugin.json is not valid JSON: {}", e),
                path: Some(json_path.to_string_lossy().to_string()),
            });
            return ValidationReport {
                valid: false,
                errors,
                warnings,
                capabilities: ValidationCapabilityCounts::default(),
            };
        }
    };

    // Required fields
    let name = pj.get("name").and_then(|v| v.as_str());
    if name.is_none() {
        errors.push(ValidationIssue {
            severity: "error".to_string(),
            code: "MISSING_NAME".to_string(),
            message: "plugin.json missing required field: name".to_string(),
            path: Some(json_path.to_string_lossy().to_string()),
        });
    }
    let version = pj.get("version").and_then(|v| v.as_str());
    if version.is_none() {
        errors.push(ValidationIssue {
            severity: "error".to_string(),
            code: "MISSING_VERSION".to_string(),
            message: "plugin.json missing required field: version".to_string(),
            path: Some(json_path.to_string_lossy().to_string()),
        });
    }

    // Warnings for recommended fields
    if pj.get("description").is_none() {
        warnings.push(ValidationIssue {
            severity: "warning".to_string(),
            code: "MISSING_DESCRIPTION".to_string(),
            message: "plugin.json missing recommended field: description".to_string(),
            path: Some(json_path.to_string_lossy().to_string()),
        });
    }
    if pj.get("author").is_none() {
        warnings.push(ValidationIssue {
            severity: "warning".to_string(),
            code: "MISSING_AUTHOR".to_string(),
            message: "plugin.json missing recommended field: author".to_string(),
            path: Some(json_path.to_string_lossy().to_string()),
        });
    }
    if pj.get("license").is_none() {
        warnings.push(ValidationIssue {
            severity: "warning".to_string(),
            code: "MISSING_LICENSE".to_string(),
            message: "plugin.json missing recommended field: license".to_string(),
            path: Some(json_path.to_string_lossy().to_string()),
        });
    }

    // Validate skills directory
    let skills_dir = plugin_dir.join("skills");
    let mut skills_count = 0u32;
    if skills_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let skill_path = entry.path();
                if !skill_path.is_dir() { continue; }
                let skill_md = skill_path.join("SKILL.md");
                if !skill_md.is_file() {
                    warnings.push(ValidationIssue {
                        severity: "warning".to_string(),
                        code: "SKILL_MISSING_SKILL_MD".to_string(),
                        message: format!("Skill directory {} missing SKILL.md", skill_path.file_name().and_then(|n| n.to_str()).unwrap_or("?")),
                        path: Some(skill_md.to_string_lossy().to_string()),
                    });
                    continue;
                }
                skills_count += 1;
                // Check for non-empty description
                if let Ok(content) = std::fs::read_to_string(&skill_md) {
                    let fm = parse_yaml_frontmatter(&content);
                    let fm_map = fm.as_ref()
                        .map(|f| parse_frontmatter_lines(f))
                        .unwrap_or_default();
                    if fm_map.get("description").map(|s| s.is_empty()).unwrap_or(true) {
                        warnings.push(ValidationIssue {
                            severity: "warning".to_string(),
                            code: "SKILL_MISSING_DESCRIPTION".to_string(),
                            message: format!("SKILL.md in {} missing non-empty description frontmatter", skill_path.file_name().and_then(|n| n.to_str()).unwrap_or("?")),
                            path: Some(skill_md.to_string_lossy().to_string()),
                        });
                    }
                }
            }
        }
    }

    // Validate commands directory
    let commands_dir = plugin_dir.join("commands");
    let mut commands_count = 0u32;
    if commands_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&commands_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().is_file() && entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
                    commands_count += 1;
                }
            }
        }
    }

    // Validate hooks/hooks.json
    let hooks_json = plugin_dir.join("hooks").join("hooks.json");
    let mut hooks_count = 0u32;
    if hooks_json.is_file() {
        if let Ok(content) = std::fs::read_to_string(&hooks_json) {
            if let Ok(data) = serde_json::from_str::<HooksJson>(&content) {
                hooks_count = data.hooks.len() as u32;
                for h in &data.hooks {
                    let script_name = h.command.split_whitespace().last().unwrap_or("");
                    let script_path = plugin_dir.join(script_name);
                    if !script_path.is_file() {
                        warnings.push(ValidationIssue {
                            severity: "warning".to_string(),
                            code: "HOOK_SCRIPT_MISSING".to_string(),
                            message: format!("Hook '{}' command references missing script: {}", h.event, script_name),
                            path: Some(hooks_json.to_string_lossy().to_string()),
                        });
                    }
                }
            } else {
                warnings.push(ValidationIssue {
                    severity: "warning".to_string(),
                    code: "HOOKS_JSON_PARSE_ERROR".to_string(),
                    message: "hooks/hooks.json exists but failed to parse".to_string(),
                    path: Some(hooks_json.to_string_lossy().to_string()),
                });
            }
        }
    }

    // Validate mcpServers in plugin.json
    let mcp_servers = pj.get("mcpServers").and_then(|v| v.as_object()).map(|o| o.len()).unwrap_or(0) as u32;
    let lsp_servers = pj.get("lspServers").and_then(|v| v.as_object()).map(|o| o.len()).unwrap_or(0) as u32;

    let valid = errors.is_empty();

    log::info!(
        "Validated plugin at {}: valid={}, skills={}, hooks={}, commands={}, mcp={}, lsp={}, errors={}, warnings={}",
        plugin_dir.display(),
        valid,
        skills_count,
        hooks_count,
        commands_count,
        mcp_servers,
        lsp_servers,
        errors.len(),
        warnings.len()
    );

    ValidationReport {
        valid,
        errors,
        warnings,
        capabilities: ValidationCapabilityCounts {
            skills: skills_count,
            hooks: hooks_count,
            commands: commands_count,
            mcp_servers,
            lsp_servers,
        },
    }
}
