use crate::models::McpService;
use crate::utils::now_rfc3339;
use crate::AppState;
use crate::services::mcp_protocol::shared_http_client;
use crate::services::mcp_protocol::validate_http_endpoint;
use tauri::State;

#[tauri::command]
pub fn get_mcp_services(state: State<AppState>, software_id: String) -> Result<Vec<McpService>, String> {
    log::info!("Getting MCP services for software: {}", software_id);
    if software_id.is_empty() {
        state.db.get_all_mcp_services().map_err(|e| e.to_string())
    } else {
        state.db.get_mcp_services_by_software(&software_id).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn add_mcp_service(state: State<AppState>, mcp_service: McpService) -> Result<(), String> {
    log::info!("Adding MCP service: {}", mcp_service.name);
    state.db.upsert_mcp_service(&mcp_service).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_mcp_service(state: State<AppState>, mcp_service: McpService) -> Result<(), String> {
    log::info!("Updating MCP service: {}", mcp_service.name);
    state.db.upsert_mcp_service(&mcp_service).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_mcp_service(state: State<AppState>, service_id: String) -> Result<(), String> {
    log::info!("Deleting MCP service: {}", service_id);
    state.db.delete_mcp_service(&service_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_mcp_service_health(state: State<'_, AppState>, service_id: String) -> Result<bool, String> {
    log::info!("Checking MCP service health: {}", service_id);
    let mcp_services = state.db.get_all_mcp_services().map_err(|e| e.to_string())?;
    if let Some(mut service) = mcp_services.into_iter().find(|s| s.id == service_id) {
        let is_healthy = check_service_health_async(&service.endpoint).await;
        service.is_healthy = is_healthy;
        service.last_checked = Some(now_rfc3339());
        state.db.upsert_mcp_service(&service).map_err(|e| e.to_string())?;
        Ok(is_healthy)
    } else {
        Ok(false)
    }
}

async fn check_service_health_async(endpoint: &str) -> bool {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        // Reuse the process-wide client and refuse internal addresses.
        if validate_http_endpoint(endpoint).is_err() {
            return false;
        }
        let client = shared_http_client();
        match client.get(endpoint).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    } else {
        false
    }
}

