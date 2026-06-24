// Installed Registry Tauri Commands — Feature 5

use crate::services::{InstalledRegistry, InstalledEntry};

#[tauri::command]
pub fn get_installed_registry() -> Result<InstalledRegistry, String> {
    log::info!("get_installed_registry");
    Ok(crate::services::read_installed_registry())
}

#[tauri::command]
pub fn update_plugin_inuse_cmd(
    source_id: String,
    plugin_name: String,
) -> Result<(), String> {
    log::info!(
        "update_plugin_inuse_cmd: source_id={}, plugin_name={}",
        source_id,
        plugin_name
    );
    crate::services::update_plugin_inuse(&source_id, &plugin_name)
}

#[tauri::command]
pub fn sweep_inuse_cmd() -> Result<usize, String> {
    log::info!("sweep_inuse_cmd");
    crate::services::sweep_inuse()
}
