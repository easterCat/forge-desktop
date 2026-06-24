use crate::models::Plugin;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_plugins(state: State<AppState>, software_id: String) -> Result<Vec<Plugin>, String> {
    log::info!("Getting plugins for software: {}", software_id);
    if software_id.is_empty() {
        state.db.get_all_plugins().map_err(|e| e.to_string())
    } else {
        state.db.get_plugins_by_software(&software_id).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn install_plugin(state: State<AppState>, plugin: Plugin) -> Result<(), String> {
    log::info!("Installing plugin: {}", plugin.name);
    state.db.upsert_plugin(&plugin).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uninstall_plugin(state: State<AppState>, plugin_id: String) -> Result<(), String> {
    log::info!("Uninstalling plugin: {}", plugin_id);
    state.db.delete_plugin(&plugin_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_plugin(state: State<AppState>, plugin: Plugin) -> Result<(), String> {
    log::info!("Updating plugin: {}", plugin.name);
    state.db.upsert_plugin(&plugin).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_plugin(state: State<AppState>, plugin_id: String, enabled: bool) -> Result<(), String> {
    log::info!("Toggling plugin {} to {}", plugin_id, enabled);
    let plugins = state.db.get_all_plugins().map_err(|e| e.to_string())?;
    if let Some(mut plugin) = plugins.into_iter().find(|p| p.id == plugin_id) {
        plugin.enabled = enabled;
        state.db.upsert_plugin(&plugin).map_err(|e| e.to_string())?;
    }
    Ok(())
}
