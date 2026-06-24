use crate::models::McpService;
use crate::AppState;
use tauri::State;
use std::time::Duration;

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
        service.last_checked = Some(chrono_lite_now());
        state.db.upsert_mcp_service(&service).map_err(|e| e.to_string())?;
        Ok(is_healthy)
    } else {
        Ok(false)
    }
}

async fn check_service_health_async(endpoint: &str) -> bool {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };
        
        match client.get(endpoint).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    } else {
        false
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    let hours = remaining_secs / 3600;
    let minutes = remaining_secs % 3600;

    let year = 1970 + days_since_epoch / 365;
    let mut days = days_since_epoch % 365;
    let mut month = 1u64;

    for (i, &d) in MONTH_DAYS.iter().enumerate() {
        let days_in_month = if i == 1 && is_leap_year(year) { 29 } else { d };
        if days < days_in_month {
            month = (i + 1) as u64;
            break;
        }
        days -= days_in_month;
    }

    let day = days + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:00Z", year, month, day, hours, minutes)
}

fn is_leap_year(year: u64) -> bool {
    let y = year as i64;
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

const MONTH_DAYS: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
