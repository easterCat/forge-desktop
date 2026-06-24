// Plugin Repository Sync Service
// Handles manual-only cloning of official plugin repositories.
// IMPORTANT: This module is NOT called automatically on app startup.
// Source installation is triggered ONLY by user action in the Plugins UI
// (clicking the "Install" button in the Sources tab).

use std::path::PathBuf;

/// Configuration for plugin repositories to sync
pub struct PluginRepoConfig {
    pub name: String,
    pub url: String,
}

impl PluginRepoConfig {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
}

/// Returns the list of official plugin repositories available for installation.
/// These match the PRESET_MARKETPLACE_SOURCES in the frontend:
/// - anthropics: https://github.com/anthropics/claude-plugins-official
/// - ananddtyagi: https://github.com/ananddtyagi/cc-marketplace
///
/// IMPORTANT: This list is used by the manual source installation flow only.
/// No automatic cloning happens on app startup.
pub fn get_official_repos() -> Vec<PluginRepoConfig> {
    vec![
        PluginRepoConfig::new(
            "anthropics",
            "https://github.com/anthropics/claude-plugins-official",
        ),
        PluginRepoConfig::new(
            "ananddtyagi",
            "https://github.com/ananddtyagi/cc-marketplace",
        ),
    ]
}

/// Get the plugins directory path under forge_home.
/// Destination format: forge/plugins/marketplace/<source_id>/
fn get_plugins_dir() -> PathBuf {
    crate::services::plugin_marketplace::plugins_dir()
}

/// Get the marketplace directory path for source repositories.
/// Format: forge/plugins/marketplace/<source_id>/
fn get_marketplace_dir() -> PathBuf {
    crate::services::plugin_marketplace::plugins_dir().join("marketplace")
}

/// Ensure the plugins directory exists
fn ensure_plugins_dir_exists() -> Result<PathBuf, String> {
    let dir = get_plugins_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
        log::info!("Created plugins directory at {}", dir.display());
    }
    Ok(dir)
}

/// Clone a repository to the specified destination directory.
/// Returns Ok(true) if cloned, Ok(false) if already exists (skipped).
/// 
/// Destination format: forge/plugins/marketplace/<source_id>/
fn clone_repository(url: &str, dest_dir: &PathBuf, repo_name: &str) -> Result<bool, String> {
    // Check if destination already exists
    if dest_dir.exists() {
        // Check if it's a valid git repo
        if dest_dir.join(".git").exists() {
            log::info!(
                "Repository '{}' already exists at {}, skipping clone",
                repo_name,
                dest_dir.display()
            );
            return Ok(false);
        }
        // Not a valid git repo, remove and re-clone
        log::info!(
            "Directory '{}' exists but is not a git repo, removing and re-cloning",
            dest_dir.display()
        );
        std::fs::remove_dir_all(dest_dir)
            .map_err(|e| format!("Failed to remove invalid directory: {}", e))?;
    }

    // Create parent directory if needed
    if let Some(parent) = dest_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    log::info!("Cloning repository '{}' from {} to {}", repo_name, url, dest_dir.display());

    // Perform git clone using system git (shallow clone for efficiency)
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth=1",
            url,
            dest_dir.to_str().unwrap_or("."),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| format!("Failed to spawn git clone: {}", e))?;

    if !status.success() {
        // Try to capture stderr for better error message
        let mut cmd = std::process::Command::new("git");
        cmd.args(["clone", "--depth=1", url, dest_dir.to_str().unwrap_or(".")])
           .stdout(std::process::Stdio::null())
           .stderr(std::process::Stdio::piped());

        if let Ok(output) = cmd.output() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!(
                "Failed to clone repository '{}' (exit {}): {}",
                repo_name,
                status,
                stderr.trim()
            );
            return Err(format!(
                "git clone failed for '{}': {}",
                repo_name,
                stderr.trim()
            ));
        }
        return Err(format!(
            "git clone failed for '{}' with exit code {}",
            repo_name,
            status
        ));
    }

    log::info!("Successfully cloned repository '{}'", repo_name);
    Ok(true)
}

/// Internal sync function that runs in a blocking task
fn sync_repos_blocking() {
    // Ensure plugins directory exists
    if let Err(e) = ensure_plugins_dir_exists() {
        log::error!("Failed to create plugins directory: {}", e);
        return;
    }

    let plugins_dir = get_plugins_dir();
    let repos = get_official_repos();

    for repo in repos {
        let dest_dir = plugins_dir.join(&repo.name);
        match clone_repository(&repo.url, &dest_dir, &repo.name) {
            Ok(cloned) => {
                if cloned {
                    log::info!("Successfully synced plugin repository: {}", repo.name);
                }
            }
            Err(e) => {
                // Log error but continue with other repos
                log::error!("Failed to sync plugin repository '{}': {}", repo.name, e);
            }
        }
    }
}

/// Clone a repository to the marketplace directory.
/// Returns Ok(true) if cloned, Ok(false) if already exists (skipped).
pub fn clone_to_marketplace(url: &str, source_id: &str) -> Result<bool, String> {
    let marketplace_dir = get_marketplace_dir();
    let dest_dir = marketplace_dir.join(source_id);
    clone_repository(url, &dest_dir, source_id)
}

/// Synchronously wait for repository sync to complete.
/// Use this when you need to ensure sync is done before proceeding.
///
/// NOTE: This function is kept for backwards compatibility but should NOT be
/// called during app startup. Source installation should be triggered only by
/// user action (clicking "Install" button in the UI).
pub fn ensure_plugin_repos_initialized_sync() -> Result<(), String> {
    log::info!("Starting synchronous plugin repository sync (manual only)...");

    let marketplace_dir = get_marketplace_dir();

    // Ensure marketplace directory exists
    if !marketplace_dir.exists() {
        std::fs::create_dir_all(&marketplace_dir)
            .map_err(|e| format!("Failed to create marketplace directory: {}", e))?;
    }

    let repos = get_official_repos();

    for repo in repos {
        let dest_dir = marketplace_dir.join(&repo.name);
        match clone_repository(&repo.url, &dest_dir, &repo.name) {
            Ok(cloned) => {
                if cloned {
                    log::info!("Successfully synced plugin repository: {}", repo.name);
                }
            }
            Err(e) => {
                // Log error but continue with other repos
                log::error!("Failed to sync plugin repository '{}': {}", repo.name, e);
            }
        }
    }

    log::info!("Plugin repository sync completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_official_repos() {
        let repos = get_official_repos();
        assert!(!repos.is_empty());
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].name, "anthropics");
        assert_eq!(repos[1].name, "ananddtyagi");
    }

    #[test]
    fn test_get_plugins_dir() {
        let dir = get_plugins_dir();
        // Should end with "plugins" under forge home
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.ends_with("plugins") || dir_str.ends_with("plugins/"));
    }
}
