// FEAT-022: MCP Manager Commands
// Tauri commands for MCP server management, tool invocation, and audit logging

use crate::db::mcp_tables::{
    AuditLogFilters, AuditPage, Group as DbGroup, HealthRecord,
};
use crate::services::mcp_protocol::{
    DiscoveryCache, HttpHandler, InvokeResult, MCPServiceConfig, StdioHandler,
};
use crate::utils::now_rfc3339;
use crate::AppState;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use tauri::State;
use uuid::Uuid;

lazy_static! {
    // Discovery cache with 5-minute TTL
    static ref DISCOVERY_CACHE: RwLock<HashMap<String, (DiscoveryCache, std::time::Instant)>> =
        RwLock::new(HashMap::new());
}

const DISCOVERY_TTL_SECS: u64 = 300; // 5 minutes

// ==================== Lock Helpers ====================
//
// `DISCOVERY_CACHE` is a process-global `RwLock`. If any thread panics while
// holding the lock, std marks the mutex as "poisoned" and every subsequent
// `.read()` / `.write()` call panics too, taking the whole app down. The
// helpers below recover from poisoning the same way `db::connection` does:
// warn, then return the inner guard so the cache stays usable.

fn read_discovery_cache() -> std::sync::RwLockReadGuard<'static, HashMap<String, (DiscoveryCache, std::time::Instant)>> {
    match DISCOVERY_CACHE.read() {
        Ok(g) => g,
        Err(poisoned) => {
            log::warn!("DISCOVERY_CACHE was poisoned; recovering in-process");
            poisoned.into_inner()
        }
    }
}

fn write_discovery_cache() -> std::sync::RwLockWriteGuard<'static, HashMap<String, (DiscoveryCache, std::time::Instant)>> {
    match DISCOVERY_CACHE.write() {
        Ok(g) => g,
        Err(poisoned) => {
            log::warn!("DISCOVERY_CACHE was poisoned; recovering in-process");
            poisoned.into_inner()
        }
    }
}

// ==================== Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MCPServiceDetail {
    pub id: String,
    pub software_id: String,
    pub name: String,
    pub endpoint: String,
    pub auth_type: String,
    pub config: Option<String>,
    pub is_healthy: bool,
    pub last_checked: Option<String>,
    pub protocol: String,
    pub group_ids: Vec<String>,
    pub health_history: Vec<HealthRecord>,
    pub discovery_cache: Option<DiscoveryCache>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: String,
    pub is_visible: bool,
    pub created_at: String,
    pub server_count: Option<i32>,
}

impl From<DbGroup> for Group {
    fn from(g: DbGroup) -> Self {
        Group {
            id: g.id,
            name: g.name,
            color: g.color,
            is_visible: g.is_visible,
            created_at: g.created_at,
            server_count: g.server_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: i32,
    pub skipped: i32,
    pub overwritten: i32,
    pub errors: Vec<String>,
}

// ==================== Helper Functions ====================

fn get_current_actor() -> String {
    // In a real app, this would get the current user from auth context
    "user".to_string()
}

fn is_cache_valid(service_id: &str) -> bool {
    let cache = read_discovery_cache();
    if let Some((_, timestamp)) = cache.get(service_id) {
        timestamp.elapsed().as_secs() < DISCOVERY_TTL_SECS
    } else {
        false
    }
}

fn get_cached_discovery(service_id: &str) -> Option<DiscoveryCache> {
    let cache = read_discovery_cache();
    cache.get(service_id).map(|(discovery, _)| discovery.clone())
}

fn set_cached_discovery(service_id: String, discovery: DiscoveryCache) {
    let mut cache = write_discovery_cache();
    cache.insert(service_id, (discovery, std::time::Instant::now()));
}

fn invalidate_cache(service_id: &str) {
    let mut cache = write_discovery_cache();
    cache.remove(service_id);
}

fn evict_if_overflow() {
    let mut cache = write_discovery_cache();
    if cache.len() > 100 {
        // Collect keys to remove before mutating
        let mut items: Vec<_> = cache.iter().collect();
        items.sort_by_key(|(_, (_, instant))| *instant);
        let keys_to_remove: Vec<_> = items.into_iter().take(20).map(|(k, _)| k.clone()).collect();
        for key in keys_to_remove {
            cache.remove(&key);
        }
    }
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub async fn get_mcp_service_detail(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<MCPServiceDetail, String> {
    log::info!("Getting MCP service detail: {}", service_id);

    // Get service from database
    let services = state.db.get_all_mcp_services().map_err(|e| e.to_string())?;
    let service = services
        .into_iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| "Service not found".to_string())?;

    // Get group associations
    let group_ids = state.db.get_service_groups(&service_id).unwrap_or_default();

    // Get health history
    let health_history = state
        .db
        .get_health_history(&service_id, Some(10))
        .unwrap_or_default();

    // Get cached discovery if valid
    let discovery_cache = if is_cache_valid(&service_id) {
        get_cached_discovery(&service_id)
    } else {
        None
    };

    Ok(MCPServiceDetail {
        id: service.id,
        software_id: service.software_id,
        name: service.name,
        endpoint: service.endpoint.clone(),
        auth_type: service.auth_type,
        config: service.config,
        is_healthy: service.is_healthy,
        last_checked: service.last_checked,
        protocol: determine_protocol(&service.endpoint),
        group_ids,
        health_history,
        discovery_cache,
    })
}

#[tauri::command]
pub async fn invoke_mcp_tool(
    state: State<'_, AppState>,
    service_id: String,
    tool_name: String,
    args: HashMap<String, Value>,
) -> Result<InvokeResult, String> {
    log::info!("Invoking MCP tool {} on service {}", tool_name, service_id);

    // Get service from database
    let services = state.db.get_all_mcp_services().map_err(|e| e.to_string())?;
    let service = services
        .into_iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| "Service not found".to_string())?;

    let protocol = determine_protocol(&service.endpoint);
    let config = MCPServiceConfig {
        id: service.id.clone(),
        name: service.name.clone(),
        endpoint: service.endpoint.clone(),
        protocol: protocol.clone(),
        auth_type: service.auth_type.clone(),
        config: service.config.clone(),
    };

    let args_for_audit = args.clone();
    let result = match protocol.as_str() {
        "stdio" => StdioHandler::new(config).invoke_tool(&tool_name, args).await,
        _ => match HttpHandler::new(config) {
            Ok(h) => h.invoke_tool(&tool_name, args).await,
            Err(e) => Err(e),
        },
    };

    // Log to audit — only record toolName and args, not the full response
    let actor = get_current_actor();
    let (status, details) = match result.as_ref() {
        Ok(r) => ("success", serde_json::json!({ "toolName": tool_name, "args": args_for_audit, "success": r.success, "durationMs": r.duration_ms }).to_string()),
        Err(e) => ("failure", serde_json::json!({ "toolName": tool_name, "args": args_for_audit, "error": e }).to_string()),
    };

    let _ = state.db.insert_audit_log(
        &actor,
        "invoke",
        Some(&service_id),
        Some(&service.name),
        Some(&details),
        status,
    );

    result
}

#[tauri::command]
pub async fn discover_mcp_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<DiscoveryCache, String> {
    log::info!("Discovering MCP service: {}", service_id);

    // Check cache first
    if is_cache_valid(&service_id) {
        if let Some(cached) = get_cached_discovery(&service_id) {
            log::info!("Returning cached discovery for {}", service_id);
            return Ok(cached);
        }
    }

    // Get service from database
    let services = state.db.get_all_mcp_services().map_err(|e| e.to_string())?;
    let service = services
        .into_iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| "Service not found".to_string())?;

    let protocol = determine_protocol(&service.endpoint);
    let config = MCPServiceConfig {
        id: service.id.clone(),
        name: service.name.clone(),
        endpoint: service.endpoint.clone(),
        protocol: protocol.clone(),
        auth_type: service.auth_type.clone(),
        config: service.config.clone(),
    };

    let discovery = match protocol.as_str() {
        "stdio" => StdioHandler::new(config).discover().await,
        _ => match HttpHandler::new(config) {
            Ok(h) => h.discover().await,
            Err(e) => Err(e),
        },
    }?;

    // Cache the result
    set_cached_discovery(service_id.clone(), discovery.clone());

    // Evict oldest entries if cache exceeds max size
    evict_if_overflow();

    Ok(discovery)
}

#[tauri::command]
pub async fn export_mcp_services(
    state: State<'_, AppState>,
    ids: Option<Vec<String>>,
    format: String,
) -> Result<String, String> {
    log::info!("Exporting MCP services: {:?} as {}", ids, format);

    let services = state.db.get_all_mcp_services().map_err(|e| e.to_string())?;

    let filtered: Vec<_> = if let Some(ref service_ids) = ids {
        services.into_iter().filter(|s| service_ids.contains(&s.id)).collect()
    } else {
        services
    };

    // Pre-fetch group memberships for every service in one round-trip
    // instead of N+1. The single-service `get_service_groups` call would
    // serialise on the shared `Mutex<Connection>` per row, dominating
    // export time as the service list grows.
    let service_id_refs: Vec<String> = filtered.iter().map(|s| s.id.clone()).collect();
    let group_membership = state
        .db
        .get_service_groups_batch(&service_id_refs)
        .map_err(|e| e.to_string())?;

    // Convert to export format
    let export_services: Vec<serde_json::Value> = filtered
        .iter()
        .map(|s| {
            let group_ids = group_membership
                .get(&s.id)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "name": s.name,
                "endpoint": s.endpoint,
                "protocol": determine_protocol(&s.endpoint),
                "authType": s.auth_type,
                "config": serde_json::from_str::<Value>(s.config.as_ref().unwrap_or(&String::new())).unwrap_or(serde_json::json!({})),
                "groups": group_ids,
            })
        })
        .collect();

    let export_data = serde_json::json!({
        "version": "1.0",
        "exportedAt": now_rfc3339(),
        "services": export_services
    });

    match format.as_str() {
        "yaml" => serde_yaml::to_string(&export_data).map_err(|e| e.to_string()),
        _ => serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub async fn import_mcp_services(
    state: State<'_, AppState>,
    data: String,
    mode: String,
) -> Result<ImportResult, String> {
    log::info!("Importing MCP services with mode: {}", mode);

    // Parse input (JSON or YAML)
    let parsed: serde_json::Value = if data.trim().starts_with('{') || data.trim().starts_with('[') {
        serde_json::from_str(&data).map_err(|e| format!("Invalid JSON: {}", e))?
    } else {
        serde_yaml::from_str(&data).map_err(|e| format!("Invalid YAML: {}", e))?
    };

    // Extract services array
    let services_array: Vec<Value> = if let Some(arr) = parsed.get("services").and_then(|s| s.as_array()) {
        arr.clone()
    } else if parsed.is_array() {
        parsed.as_array().unwrap().clone()
    } else {
        return Err("No services array found in import data".to_string());
    };

    let mut result = ImportResult {
        imported: 0,
        skipped: 0,
        overwritten: 0,
        errors: Vec::new(),
    };

    let actor = get_current_actor();

    // Fetch all existing services once — avoid N+1 per-entry lookup
    let existing_map: HashMap<String, _> = state
        .db
        .get_all_mcp_services()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|s| (s.name.clone(), s))
        .collect();

    for (idx, item) in services_array.iter().enumerate() {
        let name = match item.get("name").and_then(|n| n.as_str()) {
            Some(n) => n.to_string(),
            None => {
                result.errors.push(format!("Entry {}: missing name", idx));
                continue;
            }
        };

        let endpoint = match item.get("endpoint").and_then(|e| e.as_str()) {
            Some(e) => e.to_string(),
            None => {
                result.errors.push(format!("Entry {} ('{}'): missing endpoint", idx, name));
                continue;
            }
        };

        // Check for existing service with same name — O(1) via HashMap
        let existing = existing_map.get(&name);

        match (existing, mode.as_str()) {
            (Some(_existing), "skip") => {
                result.skipped += 1;
                log::info!("Skipping existing service: {}", name);
            }
            (Some(existing), "overwrite") => {
                // Update existing service
                let mut updated = existing.clone();
                updated.endpoint = endpoint;
                updated.auth_type = item
                    .get("authType")
                    .and_then(|a| a.as_str())
                    .unwrap_or("none")
                    .to_string();
                updated.config = item.get("config").map(|c| c.to_string());

                if let Err(e) = state.db.upsert_mcp_service(&updated) {
                    result.errors.push(format!("Entry {} ('{}'): update failed: {}", idx, name, e));
                } else {
                    result.overwritten += 1;
                    let details = serde_json::json!({ "importedCount": result.overwritten, "mode": mode, "serviceName": name }).to_string();
                    let _ = state.db.insert_audit_log(
                        &actor,
                        "import_overwrite",
                        Some(&updated.id),
                        Some(&updated.name),
                        Some(&details),
                        "success",
                    );
                }
            }
            (None, _) => {
                // Create new service
                let new_service = crate::models::McpService {
                    id: Uuid::new_v4().to_string(),
                    software_id: String::new(),
                    name: name.clone(),
                    endpoint,
                    auth_type: item
                        .get("authType")
                        .and_then(|a| a.as_str())
                        .unwrap_or("none")
                        .to_string(),
                    config: item.get("config").map(|c| c.to_string()),
                    is_healthy: false,
                    last_checked: None,
                };

                if let Err(e) = state.db.upsert_mcp_service(&new_service) {
                    result.errors.push(format!("Entry {} ('{}'): insert failed: {}", idx, name, e));
                } else {
                    result.imported += 1;
                    let details = serde_json::json!({ "importedCount": result.imported, "mode": mode, "serviceName": name }).to_string();
                    let _ = state.db.insert_audit_log(
                        &actor,
                        "import",
                        Some(&new_service.id),
                        Some(&new_service.name),
                        Some(&details),
                        "success",
                    );
                }
            }
            _ => {}
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_mcp_health_history(
    state: State<'_, AppState>,
    service_id: String,
    limit: Option<i32>,
) -> Result<Vec<HealthRecord>, String> {
    log::info!("Getting health history for: {}", service_id);
    state
        .db
        .get_health_history(&service_id, limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_mcp_groups(state: State<'_, AppState>) -> Result<Vec<Group>, String> {
    log::info!("Getting MCP groups");
    state
        .db
        .get_groups()
        .map(|groups| groups.into_iter().map(Group::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_mcp_group(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<Group, String> {
    log::info!("Creating MCP group: {}", name);

    let color = color.unwrap_or_else(|| "#F59E0B".to_string());

    let group = state
        .db
        .create_group(&name, &color)
        .map(Group::from)
        .map_err(|e| e.to_string())?;

    // Audit log
    let actor = get_current_actor();
    let _ = state.db.insert_audit_log(
        &actor,
        "create_group",
        None,
        Some(&name),
        Some(&serde_json::json!({ "color": color }).to_string()),
        "success",
    );

    Ok(group)
}

#[tauri::command]
pub async fn update_mcp_group(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    color: Option<String>,
    is_visible: Option<bool>,
) -> Result<Group, String> {
    log::info!("Updating MCP group: {}", id);

    let group = state
        .db
        .update_group(
            &id,
            name.as_deref(),
            color.as_deref(),
            is_visible,
        )
        .map(Group::from)
        .map_err(|e| e.to_string())?;

    // Audit log
    let actor = get_current_actor();
    let details = serde_json::json!({
        "name": name,
        "color": color,
        "isVisible": is_visible
    });
    let _ = state.db.insert_audit_log(
        &actor,
        "update_group",
        None,
        Some(&group.name),
        Some(&details.to_string()),
        "success",
    );

    Ok(group)
}

#[tauri::command]
pub async fn delete_mcp_group(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    log::info!("Deleting MCP group: {}", id);

    // Get group name for audit
    let groups = state.db.get_groups().map_err(|e| e.to_string())?;
    let group_name = groups.iter().find(|g| g.id == id).map(|g| g.name.clone());

    state.db.delete_group(&id).map_err(|e| e.to_string())?;

    // Audit log
    let actor = get_current_actor();
    let _ = state.db.insert_audit_log(
        &actor,
        "delete_group",
        None,
        group_name.as_deref(),
        None,
        "success",
    );

    Ok(())
}

#[tauri::command]
pub async fn get_mcp_audit_log(
    state: State<'_, AppState>,
    filters: AuditLogFilters,
    page: i32,
    page_size: i32,
) -> Result<AuditPage, String> {
    log::info!("Getting MCP audit log: page={}, page_size={}", page, page_size);

    let page_size = if page_size <= 0 || page_size > 100 { 20 } else { page_size };
    let page = if page <= 0 { 1 } else { page };

    state
        .db
        .get_audit_log_paginated(&filters, page, page_size)
        .map_err(|e| e.to_string())
}

// ==================== Helper Functions ====================

fn determine_protocol(endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        "http".to_string()
    } else {
        "stdio".to_string()
    }
}
