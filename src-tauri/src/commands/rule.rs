use crate::models::Rule;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_rules(state: State<AppState>, software_id: String) -> Result<Vec<Rule>, String> {
    log::info!("Getting rules for software: {}", software_id);
    if software_id.is_empty() {
        state.db.get_all_rules().map_err(|e| e.to_string())
    } else {
        state.db.get_rules_by_software(&software_id).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn create_rule(state: State<AppState>, rule: Rule) -> Result<(), String> {
    log::info!("Creating rule: {}", rule.name);
    state.db.upsert_rule(&rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_rule(state: State<AppState>, rule: Rule) -> Result<(), String> {
    log::info!("Updating rule: {}", rule.name);
    state.db.upsert_rule(&rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_rule(state: State<AppState>, rule_id: String) -> Result<(), String> {
    log::info!("Deleting rule: {}", rule_id);
    state.db.delete_rule(&rule_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_rule(state: State<AppState>, rule_id: String, is_active: bool) -> Result<(), String> {
    log::info!("Toggling rule {} to {}", rule_id, is_active);
    let rules = state.db.get_all_rules().map_err(|e| e.to_string())?;
    if let Some(mut rule) = rules.into_iter().find(|r| r.id == rule_id) {
        rule.is_active = is_active;
        rule.updated_at = Some(chrono_lite_now());
        state.db.upsert_rule(&rule).map_err(|e| e.to_string())?;
    }
    Ok(())
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
