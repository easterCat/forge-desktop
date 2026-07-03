use crate::models::Software;
use crate::services::{SoftwareScanner, SoftwareInstaller};
use crate::services::software_installer::{InstallResponse, UninstallResponse};
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub success: bool,
    pub has_update: bool,
    pub message: String,
    pub new_version: Option<String>,
}

#[tauri::command]
pub fn get_software_list(state: State<AppState>) -> Result<Vec<Software>, String> {
    log::info!("Getting software list");
    state.db.get_all_software().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_software(installed_versions: Option<std::collections::HashMap<String, String>>) -> Vec<Software> {
    log::info!("Detecting installed software (parallel mode)");
    let scanner = SoftwareScanner::new();
    // Use parallel detection for faster startup
    if installed_versions.is_some() {
        // If we have pre-installed versions, use the enhanced detection
        scanner.detect_software_parallel_with_versions(installed_versions)
    } else {
        scanner.detect_software_parallel()
    }
}

#[tauri::command]
pub fn get_software_by_id(state: State<AppState>, id: String) -> Result<Option<Software>, String> {
    log::info!("Getting software by id: {}", id);
    let software_list = state.db.get_all_software().map_err(|e| e.to_string())?;
    Ok(software_list.into_iter().find(|s| s.id == id))
}

#[tauri::command]
pub fn get_software_by_key(state: State<AppState>, key: String) -> Result<Option<Software>, String> {
    log::info!("Getting software by key: {}", key);
    state.db.get_software_by_key(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_software(state: State<AppState>) -> Result<Vec<Software>, String> {
    log::info!("Syncing software list (parallel mode)");
    let scanner = SoftwareScanner::new();
    // Use parallel detection for faster sync
    let detected = scanner.detect_software_parallel();

    for software in &detected {
        state.db.upsert_software(software).map_err(|e| e.to_string())?;
    }

    Ok(detected)
}

#[tauri::command]
pub fn install_software(software_key: String) -> Result<InstallResponse, String> {
    log::info!("Installing software: {}", software_key);
    let installer = SoftwareInstaller::new();
    installer.install(&software_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn uninstall_software(software_key: String) -> Result<UninstallResponse, String> {
    log::info!("Uninstalling software: {}", software_key);
    let installer = SoftwareInstaller::new();
    installer.uninstall(&software_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_software(software_key: String) -> Result<UpdateCheckResult, String> {
    log::info!("Checking update for software: {}", software_key);
    let scanner = SoftwareScanner::new();

    // Detect current version
    let detected = scanner.detect_software_parallel();
    let current = detected.iter().find(|s| s.key == software_key);

    match current {
        Some(sw) if sw.is_installed => {
            // For now, reinstall to update (same as install which upgrades)
            let installer = SoftwareInstaller::new();
            let result = installer.install(&software_key).map_err(|e| e.to_string())?;
            Ok(UpdateCheckResult {
                success: result.success,
                has_update: result.success,
                message: result.message,
                new_version: result.installed_version,
            })
        }
        Some(_) => Ok(UpdateCheckResult {
            success: true,
            has_update: false,
            message: "Software is not installed".to_string(),
            new_version: None,
        }),
        None => Err(format!("Unknown software key: {}", software_key)),
    }
}
