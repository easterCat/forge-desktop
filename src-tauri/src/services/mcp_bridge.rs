// MCP Bridge Service - probes MCP server health via JSON-RPC handshake.
// Simulates the Claude Code MCP client connection check.

use crate::models::mcp_bridge::McpProbeResult;
use std::collections::HashMap;
use std::time::Duration;

/// Send an `initialize` JSON-RPC request to an MCP server via stdin/stdout
/// and check if the server responds with a valid result containing `serverInfo`.
pub async fn probe_mcp_server(
    command: &str,
    args: &[String],
    env_vars: &HashMap<String, String>,
) -> Result<McpProbeResult, String> {
    let start = std::time::Instant::now();

    // Build the initialize JSON-RPC request
    let request_id = 1;
    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "forge-desktop",
                "version": "1.0.0"
            }
        }
    });

    let request_json = serde_json::to_string(&initialize_request)
        .map_err(|e| format!("Failed to serialize initialize request: {}", e))?;

    // Run with 5-second timeout using tokio's Timeout
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run_mcp_request(command, args, env_vars, request_json),
    )
    .await;

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(response)) => {
            // Parse response: look for "result" with "serverInfo"
            if let Some(result_obj) = response.get("result").and_then(|r| r.as_object()) {
                if result_obj.contains_key("serverInfo") {
                    return Ok(McpProbeResult {
                        reachable: true,
                        server_info: result_obj.get("serverInfo").cloned(),
                        error: None,
                        duration_ms,
                    });
                }
            }
            Ok(McpProbeResult {
                reachable: false,
                server_info: None,
                error: Some("Server responded but did not include serverInfo in initialize result".to_string()),
                duration_ms,
            })
        }
        Ok(Err(e)) => Ok(McpProbeResult {
            reachable: false,
            server_info: None,
            error: Some(e),
            duration_ms,
        }),
        Err(_) => {
            // Timed out
            Ok(McpProbeResult {
                reachable: false,
                server_info: None,
                error: Some("Connection timed out after 5 seconds".to_string()),
                duration_ms,
            })
        }
    }
}

async fn run_mcp_request(
    command: &str,
    args: &[String],
    env_vars: &HashMap<String, String>,
    request_json: String,
) -> Result<serde_json::Value, String> {
    let mut child = tokio::process::Command::new(command)
        .args(args)
        .envs(env_vars.iter())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn MCP server '{}': {}", command, e))?;

    // Write request to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(request_json.as_bytes()).await;
        let _ = stdin.write_all(b"\n").await;
    }

    // Read response from stdout
    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Failed to read MCP server output: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Try to parse the first JSON line as the response
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(response) = serde_json::from_str::<serde_json::Value>(trimmed) {
            // Check for JSON-RPC error
            if response.get("error").is_some() {
                let err_msg = response["error"]
                    .as_object()
                    .and_then(|o| o.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown MCP error");
                return Err(format!("MCP server returned error: {}", err_msg));
            }
            return Ok(response);
        }
    }

    if !stderr.trim().is_empty() {
        return Err(format!("MCP server stderr: {}", stderr.trim()));
    }

    Err("No valid JSON-RPC response received from MCP server".to_string())
}
