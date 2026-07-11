// FEAT-022: MCP Protocol Handlers
// Implements STDIO and HTTP protocol handlers for MCP server communication.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Default timeout for an MCP stdio request (server startup + RPC round-trip).
const STDIO_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Cap on the cumulative bytes read from a single MCP server. Protects
/// against an unbounded response buffer being held in memory.
const STDIO_MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024; // 16 MiB

/// Validate that an MCP HTTP endpoint does not target loopback, link-local,
/// or otherwise private / metadata network ranges. Returns `Ok(())` for
/// publicly routable hosts, `Err(message)` otherwise.
///
/// This is a defence-in-depth measure: even if a user adds a service
/// pointing at an internal address, we reject it unless the user explicitly
/// opts in via the endpoint's query string `?allow_internal=1`.
pub fn validate_http_endpoint(endpoint: &str) -> Result<(), String> {
    use std::net::IpAddr;

    let url = url::Url::parse(endpoint).map_err(|e| format!("Invalid URL: {}", e))?;

    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!("Unsupported scheme '{}' (use http or https)", scheme));
    }

    // Allow callers to explicitly opt in to private addresses.
    if url
        .query_pairs()
        .any(|(k, v)| k == "allow_internal" && v == "1")
    {
        return Ok(());
    }

    let host = url
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?;

    // Reject literal addresses; if the host is an IP, check its range.
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_or_loopback_ip(&ip) {
            return Err(format!(
                "Refusing to connect to private/loopback IP {}; \
                 append ?allow_internal=1 to opt in",
                ip
            ));
        }
        return Ok(());
    }

    // For hostnames, refuse the well-known metadata hostnames outright.
    let lower = host.to_ascii_lowercase();
    if lower == "metadata.google.internal"
        || lower == "metadata.goog"
        || lower == "kubernetes.default.svc"
        || lower.ends_with(".internal")
    {
        return Err(format!("Refusing to connect to internal host '{}'", host));
    }

    Ok(())
}

/// Returns true if `ip` is loopback, private, link-local, multicast, or
/// otherwise not safe to contact from a user-supplied config.
fn is_private_or_loopback_ip(ip: &std::net::IpAddr) -> bool {
    use std::net::IpAddr::*;
    match ip {
        V4(v4) => {
            v4.is_loopback()            // 127.0.0.0/8
            || v4.is_private()          // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
            || v4.is_link_local()        // 169.254.0.0/16
            || v4.is_multicast()         // 224.0.0.0/4
            || v4.is_unspecified()       // 0.0.0.0
            || v4.octets()[0] == 100 && (v4.octets()[1] & 0xC0) == 64 // 100.64.0.0/10 (CGNAT)
        }
        V6(v6) => {
            v6.is_loopback()            // ::1
            || v6.is_unspecified()       // ::
            || v6.segments()[0] == 0xfe80 // fe80::/10 link-local
            || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 unique-local
            || (v6.segments()[0] & 0xff00) == 0xff00 // ff00::/8 multicast
        }
    }
}

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

        let now = crate::utils::now_rfc3339();
        let expires_at = crate::utils::future_rfc3339(300); // 5 minutes TTL

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

    /// Send a JSON-RPC request to the STDIO server.
    ///
    /// The MCP server is spawned as a child process with `stdin`, `stdout`,
    /// and `stderr` piped. We:
    /// 1. Spawn the server with `kill_on_drop(true)` so a timeout or panic
    ///    does not leak a zombie process.
    /// 2. Write a single newline-delimited JSON-RPC request to stdin and
    ///    close stdin (signals EOF to the server).
    /// 3. Read stdout line by line, collecting JSON-RPC frames until we
    ///    find one whose `id` matches our request id (notifications arrive
    ///    on the same stream and must be skipped).
    /// 4. Bound the read by both a wall-clock timeout and a cumulative
    ///    byte cap to protect against malicious / buggy servers.
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

        let request_line = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // Parse endpoint to get command and args.
        let (command, args, env_vars) = parse_endpoint(&self.config.endpoint)?;

        // Spawn the server. `kill_on_drop` guarantees the process is
        // terminated if we exit early (timeout, panic, error).
        let mut child = TokioCommand::new(&command)
            .args(&args)
            .envs(env_vars)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server '{}': {}", command, e))?;

        // Write the request to stdin and close it. Doing this in a
        // detached task means a slow / blocked server cannot deadlock
        // the read below — closing stdin signals "no more input" to the
        // server, allowing it to flush its response and exit cleanly.
        if let Some(mut stdin) = child.stdin.take() {
            let payload = format!("{}\n", request_line);
            tokio::spawn(async move {
                let _ = stdin.write_all(payload.as_bytes()).await;
                // Implicit drop closes stdin when the task ends.
            });
        } else {
            // Without stdin we cannot deliver the request — terminate.
            let _ = child.kill().await;
            return Err("MCP server child has no stdin pipe".to_string());
        }

        // Take stdout for line-buffered reading. The MCP server may emit
        // any number of `notifications/*` frames before our response;
        // skip them and only return when we see our `id`.
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "MCP server child has no stdout pipe".to_string())?;
        let stderr = child.stderr.take();

        let mut reader = BufReader::new(stdout);
        let id_for_match = request_id.clone();

        let outcome = timeout(STDIO_REQUEST_TIMEOUT, async {
            let mut buf = String::new();
            let mut total_bytes = 0usize;

            loop {
                buf.clear();
                let n = reader
                    .read_line(&mut buf)
                    .await
                    .map_err(|e| format!("Failed to read from MCP server: {}", e))?;
                if n == 0 {
                    // EOF: server closed without sending our response.
                    return Err("MCP server closed stdout before responding".to_string());
                }
                total_bytes += n;
                if total_bytes > STDIO_MAX_RESPONSE_BYTES {
                    return Err(format!(
                        "MCP server response exceeded {} byte cap",
                        STDIO_MAX_RESPONSE_BYTES
                    ));
                }

                let trimmed = buf.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let frame: Value = match serde_json::from_str(trimmed) {
                    Ok(v) => v,
                    Err(_) => continue, // not JSON or truncated frame — skip
                };

                // Notifications carry no `id`; ignore them.
                let frame_id = frame.get("id").and_then(|v| v.as_str());
                if frame_id.map(|s| s != id_for_match).unwrap_or(true) {
                    continue;
                }

                if let Some(error) = frame.get("error") {
                    let err_msg = error
                        .as_object()
                        .and_then(|o| o.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown MCP error");
                    return Err(format!("MCP server error: {}", err_msg));
                }

                if let Some(result) = frame.get("result") {
                    return Ok(result.clone());
                }
                // Frame matched our id but has neither result nor error —
                // treat as protocol violation.
                return Err("MCP server returned frame with no result/error".to_string());
            }
        })
        .await;

        // Whether or not we succeeded, try to wait the child cleanly so we
        // don't leak resources. `kill_on_drop` is the safety net.
        let _ = child.start_kill();

        match outcome {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(e)) => {
                if let Some(mut err) = stderr {
                    let mut buf = String::new();
                    let _ = tokio::io::AsyncReadExt::read_to_string(&mut err, &mut buf).await;
                    if !buf.trim().is_empty() {
                        return Err(format!("{} (stderr: {})", e, buf.trim()));
                    }
                }
                Err(e)
            }
            Err(_) => Err(format!(
                "MCP server timed out after {}s",
                STDIO_REQUEST_TIMEOUT.as_secs()
            )),
        }
    }
}

// ==================== HTTP Handler ====================

/// Handler for HTTP(SSE)-based MCP servers
pub struct HttpHandler {
    config: MCPServiceConfig,
    client: reqwest::Client,
}

impl HttpHandler {
    /// Construct a handler. Returns `Err` if the endpoint is rejected by
    /// the SSRF guard (private / loopback / metadata addresses).
    pub fn new(config: MCPServiceConfig) -> Result<Self, String> {
        validate_http_endpoint(&config.endpoint)?;
        Ok(Self {
            config,
            client: shared_http_client(),
        })
    }

    /// Discover available tools, resources, and prompts from the HTTP MCP server
    pub async fn discover(&self) -> Result<DiscoveryCache, String> {
        // Try /tools endpoint first, fallback to root /
        let tools = self.discover_tools().await?;
        let resources = self.discover_resources().await.unwrap_or_default();
        let prompts = self.discover_prompts().await.unwrap_or_default();

        let now = crate::utils::now_rfc3339();
        let expires_at = crate::utils::future_rfc3339(300); // 5 minutes TTL

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

/// Process-wide shared `reqwest::Client`. Building a client is expensive
/// (TLS config, connection pool, DNS resolver) and we may construct many
/// short-lived handlers per second; share a single instance instead.
pub fn shared_http_client() -> reqwest::Client {
    use std::sync::OnceLock;
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("forge-desktop/0.1")
                .build()
                .unwrap_or_default()
        })
        .clone()
}

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
        "http" | "sse" | _ => {
            let handler = HttpHandler::new(config)?;
            Ok(handler.health_check().await)
        }
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
