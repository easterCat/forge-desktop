// FEAT-022: MCP Protocol Handlers
// Implements STDIO and HTTP protocol handlers for MCP server communication

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// MCP Server Configuration
#[derive(Debug, Clone)]
pub struct MCPServiceConfig {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub protocol: String,
    pub auth_type: String,
    pub config: Option<String>,
}

// Discovery Cache Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryCache {
    pub tools: Vec<MCPTool>,
    pub resources: Vec<MCPResource>,
    pub prompts: Vec<MCPPrompt>,
    pub cached_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPResource {
    pub uri: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<MCPArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<Value>,
}

// Invocation Result Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvokeResult {
    pub success: bool,
    pub content: Option<Vec<MCPContentBlock>>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<String>,
    pub data: Option<String>,
    pub mime_type: Option<String>,
    pub uri: Option<String>,
}

// Health Check Result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResult {
    pub reachable: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

// ==================== STDIO Handler ====================

/// Handler for STDIO-based MCP servers
pub struct StdioHandler {
    config: MCPServiceConfig,
}

impl StdioHandler {
    pub fn new(config: MCPServiceConfig) -> Self {
        Self { config }
    }

    /// Discover available tools, resources, and prompts from the MCP server
    pub async fn discover(&self) -> Result<DiscoveryCache, String> {
        // First send initialize
        let _init_response = self
            .send_request("initialize", serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "forge-desktop",
                    "version": "1.0.0"
                }
            }))
            .await?;

        // Then send notifications/initialize completed
        let _notif = self
            .send_request("notifications/initialized", serde_json::json!({}))
            .await;

        // Send tools/list request
        let tools_response = self
            .send_request("tools/list", serde_json::json!({}))
            .await?;

        let tools = parse_tools_list(&tools_response);

        // Send resources/list request
        let resources_response = self
            .send_request("resources/list", serde_json::json!({}))
            .await?;

        let resources = parse_resources_list(&resources_response);

        // Send prompts/list request
        let prompts_response = self
            .send_request("prompts/list", serde_json::json!({}))
            .await?;

        let prompts = parse_prompts_list(&prompts_response);

        let now = chrono_lite_now();
        let expires_at = chrono_lite_future(300); // 5 minutes TTL

        Ok(DiscoveryCache {
            tools,
            resources,
            prompts,
            cached_at: now,
            expires_at,
        })
    }

    /// Invoke a tool on the MCP server
    pub async fn invoke_tool(
        &self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> Result<InvokeResult, String> {
        let start = std::time::Instant::now();

        let request = serde_json::json!({
            "name": name,
            "arguments": args
        });

        match self.send_request("tools/call", request).await {
            Ok(response) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let content = parse_call_response(&response);
                Ok(InvokeResult {
                    success: true,
                    content: Some(content),
                    error: None,
                    duration_ms,
                })
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(InvokeResult {
                    success: false,
                    content: None,
                    error: Some(e),
                    duration_ms,
                })
            }
        }
    }

    /// Perform a health check on the STDIO server
    pub async fn health_check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        match self.discover().await {
            Ok(_) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                HealthCheckResult {
                    reachable: true,
                    latency_ms: Some(latency_ms),
                    error: None,
                }
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                HealthCheckResult {
                    reachable: false,
                    latency_ms: Some(latency_ms),
                    error: Some(e),
                }
            }
        }
    }

    /// Send a JSON-RPC request to the STDIO server
    async fn send_request(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, String> {
        let request_id = uuid::Uuid::new_v4().to_string();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        });

        let request_json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        let _ = request_json;

        // Parse endpoint to get command and args
        let (command, args, env_vars) = parse_endpoint(&self.config.endpoint)?;

        let output = tokio::process::Command::new(&command)
            .args(&args)
            .envs(env_vars)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to spawn MCP server '{}': {}", command, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse response - look for JSON-RPC response with matching id
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(response) = serde_json::from_str::<serde_json::Value>(trimmed) {
                // Check for JSON-RPC error
                if let Some(error) = response.get("error") {
                    let err_msg = error
                        .as_object()
                        .and_then(|o| o.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown MCP error");
                    return Err(format!("MCP server error: {}", err_msg));
                }

                // Return result if present
                if let Some(result) = response.get("result") {
                    return Ok(result.clone());
                }
            }
        }

        if !stderr.trim().is_empty() {
            return Err(format!("MCP server stderr: {}", stderr.trim()));
        }

        Err("No valid JSON-RPC response received from MCP server".to_string())
    }
}

// ==================== HTTP Handler ====================

/// Handler for HTTP(SSE)-based MCP servers
pub struct HttpHandler {
    config: MCPServiceConfig,
    client: reqwest::Client,
}

impl HttpHandler {
    pub fn new(config: MCPServiceConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// Discover available tools, resources, and prompts from the HTTP MCP server
    pub async fn discover(&self) -> Result<DiscoveryCache, String> {
        // Try /tools endpoint first, fallback to root /
        let tools = self.discover_tools().await?;
        let resources = self.discover_resources().await.unwrap_or_default();
        let prompts = self.discover_prompts().await.unwrap_or_default();

        let now = chrono_lite_now();
        let expires_at = chrono_lite_future(300); // 5 minutes TTL

        Ok(DiscoveryCache {
            tools,
            resources,
            prompts,
            cached_at: now,
            expires_at,
        })
    }

    async fn discover_tools(&self) -> Result<Vec<MCPTool>, String> {
        let url = if self.config.endpoint.ends_with('/') {
            format!("{}tools", self.config.endpoint)
        } else {
            format!("{}/tools", self.config.endpoint)
        };

        let mut request = self.client.get(&url);

        // Add auth headers based on auth_type
        if let Some(ref config_json) = self.config.config {
            if let Ok(config) = serde_json::from_str::<Value>(config_json) {
                if let Some(bearer) = config.get("bearer_token").and_then(|v| v.as_str()) {
                    request = request.header("Authorization", format!("Bearer {}", bearer));
                }
                if let Some(api_key) = config.get("api_key").and_then(|v| v.as_str()) {
                    request = request.header("X-API-Key", api_key);
                }
            }
        }

        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text().await.map_err(|e| e.to_string())?;
                    let parsed: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
                    Ok(parse_tools_list(&parsed))
                } else {
                    Err(format!("HTTP error: {}", response.status()))
                }
            }
            Err(e) => Err(format!("Failed to discover tools: {}", e)),
        }
    }

    async fn discover_resources(&self) -> Result<Vec<MCPResource>, String> {
        let url = if self.config.endpoint.ends_with('/') {
            format!("{}resources", self.config.endpoint)
        } else {
            format!("{}/resources", self.config.endpoint)
        };

        let mut request = self.client.get(&url);

        if let Some(ref config_json) = self.config.config {
            if let Ok(config) = serde_json::from_str::<Value>(config_json) {
                if let Some(bearer) = config.get("bearer_token").and_then(|v| v.as_str()) {
                    request = request.header("Authorization", format!("Bearer {}", bearer));
                }
            }
        }

        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text().await.map_err(|e| e.to_string())?;
                    let parsed: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
                    Ok(parse_resources_list(&parsed))
                } else {
                    Err(format!("HTTP error: {}", response.status()))
                }
            }
            Err(_) => Ok(Vec::new()), // Resources are optional
        }
    }

    async fn discover_prompts(&self) -> Result<Vec<MCPPrompt>, String> {
        let url = if self.config.endpoint.ends_with('/') {
            format!("{}prompts", self.config.endpoint)
        } else {
            format!("{}/prompts", self.config.endpoint)
        };

        let mut request = self.client.get(&url);

        if let Some(ref config_json) = self.config.config {
            if let Ok(config) = serde_json::from_str::<Value>(config_json) {
                if let Some(bearer) = config.get("bearer_token").and_then(|v| v.as_str()) {
                    request = request.header("Authorization", format!("Bearer {}", bearer));
                }
            }
        }

        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text().await.map_err(|e| e.to_string())?;
                    let parsed: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
                    Ok(parse_prompts_list(&parsed))
                } else {
                    Err(format!("HTTP error: {}", response.status()))
                }
            }
            Err(_) => Ok(Vec::new()), // Prompts are optional
        }
    }

    /// Invoke a tool on the HTTP MCP server
    pub async fn invoke_tool(
        &self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> Result<InvokeResult, String> {
        let start = std::time::Instant::now();

        let url = if self.config.endpoint.ends_with('/') {
            format!("{}tools/call", self.config.endpoint)
        } else {
            format!("{}/tools/call", self.config.endpoint)
        };

        let body = serde_json::json!({
            "name": name,
            "arguments": args
        });

        let mut request = self.client.post(&url).json(&body);

        // Add auth headers
        if let Some(ref config_json) = self.config.config {
            if let Ok(config) = serde_json::from_str::<Value>(config_json) {
                if let Some(bearer) = config.get("bearer_token").and_then(|v| v.as_str()) {
                    request = request.header("Authorization", format!("Bearer {}", bearer));
                }
                if let Some(api_key) = config.get("api_key").and_then(|v| v.as_str()) {
                    request = request.header("X-API-Key", api_key);
                }
            }
        }

        match request.send().await {
            Ok(response) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let status = response.status();

                if status.is_success() {
                    let body = response.text().await.map_err(|e| e.to_string())?;
                    let parsed: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
                    let content = parse_call_response(&parsed);

                    Ok(InvokeResult {
                        success: true,
                        content: Some(content),
                        error: None,
                        duration_ms,
                    })
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    Ok(InvokeResult {
                        success: false,
                        content: None,
                        error: Some(format!("HTTP {}: {}", status, error_text)),
                        duration_ms,
                    })
                }
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(InvokeResult {
                    success: false,
                    content: None,
                    error: Some(format!("Request failed: {}", e)),
                    duration_ms,
                })
            }
        }
    }

    /// Perform a health check on the HTTP server
    pub async fn health_check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        let mut request = self.client.get(&self.config.endpoint);

        // Add auth headers
        if let Some(ref config_json) = self.config.config {
            if let Ok(config) = serde_json::from_str::<Value>(config_json) {
                if let Some(bearer) = config.get("bearer_token").and_then(|v| v.as_str()) {
                    request = request.header("Authorization", format!("Bearer {}", bearer));
                }
            }
        }

        match request.send().await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                HealthCheckResult {
                    reachable: response.status().is_success(),
                    latency_ms: Some(latency_ms),
                    error: None,
                }
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                HealthCheckResult {
                    reachable: false,
                    latency_ms: Some(latency_ms),
                    error: Some(e.to_string()),
                }
            }
        }
    }
}

// ==================== Protocol Router ====================

/// Route to the appropriate handler based on protocol
pub async fn probe_service(service: &crate::models::McpService) -> Result<HealthCheckResult, String> {
    let config = MCPServiceConfig {
        id: service.id.clone(),
        name: service.name.clone(),
        endpoint: service.endpoint.clone(),
        protocol: determine_protocol(&service.endpoint),
        auth_type: service.auth_type.clone(),
        config: service.config.clone(),
    };

    match config.protocol.as_str() {
        "stdio" => Ok(StdioHandler::new(config).health_check().await),
        "http" | "sse" | _ => Ok(HttpHandler::new(config).health_check().await),
    }
}

/// Determine protocol from endpoint
fn determine_protocol(endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        "http".to_string()
    } else {
        "stdio".to_string()
    }
}

// ==================== Helper Functions ====================

/// Parse endpoint into command, args, and env vars.
/// Uses shell-words for safe shell-argument parsing (handles quoted args).
fn parse_endpoint(endpoint: &str) -> Result<(String, Vec<String>, std::collections::HashMap<String, String>), String> {
    use shell_words::split;

    if endpoint.trim().is_empty() {
        return Err("Empty endpoint".to_string());
    }

    let parts = split(endpoint).map_err(|e| format!("Failed to parse endpoint: {}", e))?;

    if parts.is_empty() {
        return Err("Empty endpoint after parsing".to_string());
    }

    let command = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    Ok((command, args, std::collections::HashMap::new()))
}

/// Parse tools list from discovery response
fn parse_tools_list(response: &Value) -> Vec<MCPTool> {
    response
        .get("tools")
        .and_then(|t| t.get("list").or(Some(t)))
        .and_then(|arr| arr.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|tool| {
                    Some(MCPTool {
                        name: tool.get("name")?.as_str()?.to_string(),
                        description: tool.get("description").and_then(|d| d.as_str()).map(String::from),
                        input_schema: tool.get("inputSchema").cloned().unwrap_or(serde_json::json!({})),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Parse resources list from discovery response
fn parse_resources_list(response: &Value) -> Vec<MCPResource> {
    response
        .get("resources")
        .and_then(|r| r.get("list").or(Some(r)))
        .and_then(|arr| arr.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|res| {
                    Some(MCPResource {
                        uri: res.get("uri")?.as_str()?.to_string(),
                        name: res.get("name").and_then(|d| d.as_str()).map(String::from),
                        description: res.get("description").and_then(|d| d.as_str()).map(String::from),
                        mime_type: res.get("mimeType").and_then(|d| d.as_str()).map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Parse prompts list from discovery response
fn parse_prompts_list(response: &Value) -> Vec<MCPPrompt> {
    response
        .get("prompts")
        .and_then(|p| p.get("list").or(Some(p)))
        .and_then(|arr| arr.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|prompt| {
                    Some(MCPPrompt {
                        name: prompt.get("name")?.as_str()?.to_string(),
                        description: prompt.get("description").and_then(|d| d.as_str()).map(String::from),
                        arguments: prompt.get("arguments").and_then(|a| a.as_array()).map(|arr| {
                            arr.iter()
                                .filter_map(|arg| {
                                    Some(MCPArgument {
                                        name: arg.get("name")?.as_str()?.to_string(),
                                        description: arg.get("description").and_then(|d| d.as_str()).map(String::from),
                                        required: arg.get("required").and_then(|r| r.as_bool()).unwrap_or(false),
                                        default: arg.get("default").cloned(),
                                    })
                                })
                                .collect()
                        }),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Parse call response to content blocks
fn parse_call_response(response: &Value) -> Vec<MCPContentBlock> {
    response
        .get("content")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|block| {
                    let block_type = block.get("type")?.as_str()?.to_string();
                    Some(MCPContentBlock {
                        block_type,
                        text: block.get("text").and_then(|t| t.as_str()).map(String::from),
                        data: block.get("data").and_then(|d| d.as_str()).map(String::from),
                        mime_type: block.get("mimeType").and_then(|m| m.as_str()).map(String::from),
                        uri: block.get("uri").and_then(|u| u.as_str()).map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// ==================== Timestamp Utilities ====================

fn chrono_lite_now() -> String {
    crate::utils::now_rfc3339()
}

fn chrono_lite_future(seconds: u64) -> String {
    crate::utils::future_rfc3339(seconds as i64)
}
