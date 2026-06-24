// Plugin Capabilities Tauri Commands

use crate::models::plugin_capabilities::{
    HookExecutionResult, PluginCapabilities, ValidationReport,
};
use crate::services::plugin_capabilities;
use crate::services::plugin_marketplace::plugins_dir;
use std::collections::HashMap;

/// Fetch the capability matrix for a plugin. Tries the local install directory
/// first; falls back to remote (git archive) for marketplace plugins that have
/// not been installed yet.
#[tauri::command]
pub async fn get_plugin_capabilities(
    source_id: String,
    plugin_name: String,
) -> Result<PluginCapabilities, String> {
    log::info!(
        "get_plugin_capabilities: source_id={}, plugin_name={}",
        source_id,
        plugin_name
    );

    // 1. Try local
    if let Ok(caps) = plugin_capabilities::parse_local_capabilities(&source_id, &plugin_name) {
        log::info!(
            "Local capabilities found for '{}/{}'",
            source_id,
            plugin_name
        );
        return Ok(caps);
    }

    log::info!(
        "Local parse failed for '{}/{}'. Trying remote.",
        source_id,
        plugin_name
    );

    // 2. Remote fallback
    plugin_capabilities::parse_remote_capabilities(&source_id, &plugin_name).await
}

/// Execute a single hook from a plugin's hooks/hooks.json and return structured results.
/// Writes a JSON log to `~/.claude/plugins/data/<source>/<plugin>/hook-logs/`.
#[tauri::command]
pub async fn execute_plugin_hook(
    source_id: String,
    plugin_name: String,
    event: String,
    matcher: Option<String>,
) -> Result<HookExecutionResult, String> {
    log::info!(
        "execute_plugin_hook: source_id={}, plugin={}, event={}, matcher={:?}",
        source_id,
        plugin_name,
        event,
        matcher
    );

    let env_vars = HashMap::new();
    plugin_capabilities::execute_hook(
        &source_id,
        &plugin_name,
        &event,
        matcher.as_deref(),
        env_vars,
    )
    .await
}

/// Validate a plugin directory and return a structured report.
#[tauri::command]
pub fn validate_plugin_path(path: String) -> Result<ValidationReport, String> {
    log::info!("validate_plugin_path: path={}", path);

    let plugin_dir = if std::path::Path::new(&path).is_absolute() {
        std::path::PathBuf::from(&path)
    } else {
        // Resolve relative paths against the plugins directory
        plugins_dir().join(&path)
    };

    Ok(plugin_capabilities::validate_plugin(&plugin_dir))
}
