// MCP Bridge Model - McpProbeResult struct

use serde::{Deserialize, Serialize};

/// Result of probing an MCP server's availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpProbeResult {
    pub reachable: bool,
    pub server_info: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}
