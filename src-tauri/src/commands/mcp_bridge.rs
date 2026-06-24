// MCP Bridge Tauri Commands - exposes MCP server health check to the frontend.

use crate::models::mcp_bridge::McpProbeResult;
use crate::services::mcp_bridge;
use std::collections::HashMap;

#[tauri::command]
pub async fn probe_plugin_mcp(
    source_id: String,
    plugin_name: String,
    server_name: String,
) -> Result<McpProbeResult, String> {
    log::info!(
        "probe_plugin_mcp: source_id={}, plugin_name={}, server={}",
        source_id,
        plugin_name,
        server_name
    );

    // Parse plugin.json to get the MCP server config
    let plugin_dir = crate::services::plugin_marketplace::plugins_dir()
        .join(&source_id)
        .join(&plugin_name);

    let plugin_json_path = plugin_dir.join(".claude-plugin").join("plugin.json");
    let json_path = if plugin_json_path.is_file() {
        &plugin_json_path
    } else {
        &plugin_dir.join("plugin.json")
    };

    let content = std::fs::read_to_string(json_path)
        .map_err(|e| format!("Failed to read plugin.json: {}", e))?;

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PluginJsonMcp {
        #[serde(default)]
        mcp_servers: std::collections::HashMap<String, serde_json::Value>,
    }

    let pj: PluginJsonMcp = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse plugin.json: {}", e))?;

    let server_config = pj
        .mcp_servers
        .get(&server_name)
        .ok_or_else(|| format!("MCP server '{}' not found in plugin.json", server_name))?;

    let command = server_config
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("MCP server '{}' missing 'command' field", server_name))?;

    let args: Vec<String> = server_config
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let env: HashMap<String, String> = server_config
        .get("env")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    mcp_bridge::probe_mcp_server(command, &args, &env).await
}
