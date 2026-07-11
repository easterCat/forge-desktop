use crate::models::Rule;
use crate::utils::now_rfc3339;
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
        rule.updated_at = Some(now_rfc3339());
        state.db.upsert_rule(&rule).map_err(|e| e.to_string())?;
    }
    Ok(())
}

