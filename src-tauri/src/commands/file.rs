use crate::models::FileEntry;
use crate::services::FileService;
use crate::utils::PathGuard;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Build the path guard for `file::*` commands. The guard allow-lists the
/// app data dir (read/write of workspace state, logs, etc.) and the user's
/// home directory (so the file picker can return absolute paths the user
/// explicitly chose). Anything else is rejected — in particular, system
/// roots (`/etc`, `/var`, ...), SSH keys, and dotenv files.
fn build_guard(app: &AppHandle) -> PathGuard {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(data_dir) = app.path().app_data_dir() {
        roots.push(data_dir);
    }
    if let Some(home) = dirs::home_dir() {
        roots.push(home);
    }
    if roots.is_empty() {
        PathGuard::deny_all()
    } else {
        PathGuard::new(roots)
    }
}

/// Convenience wrapper that turns a `PathGuardError` into the `String` error
/// shape Tauri commands return. The error message is intentionally generic so
/// a hostile caller can't probe filesystem layout through error text.
fn guard_err(e: crate::utils::PathGuardError) -> String {
    log::warn!("file command rejected path: {}", e);
    e.to_string()
}

#[tauri::command]
pub fn read_file(app: AppHandle, path: String) -> Result<String, String> {
    log::info!("Reading file: {}", path);
    let guard = build_guard(&app);
    let safe = guard.validate(&path).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service.read_file(&safe).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file(app: AppHandle, path: String, content: String) -> Result<(), String> {
    log::info!("Writing file: {}", path);
    let guard = build_guard(&app);
    let safe = guard.validate(&path).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service.write_file(&safe, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_file(app: AppHandle, path: String) -> Result<(), String> {
    log::info!("Deleting file: {}", path);
    let guard = build_guard(&app);
    let safe = guard.validate(&path).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service.delete_file(&safe).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_directory(app: AppHandle, path: String) -> Result<Vec<FileEntry>, String> {
    log::info!("Listing directory: {}", path);
    let guard = build_guard(&app);
    let safe = guard.validate(&path).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service.list_directory(&safe).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_directory(app: AppHandle, path: String) -> Result<(), String> {
    log::info!("Creating directory: {}", path);
    let guard = build_guard(&app);
    let safe = guard.validate(&path).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service.create_directory(&safe).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_file(app: AppHandle, src: String, dst: String) -> Result<(), String> {
    log::info!("Copying file from {} to {}", src, dst);
    let guard = build_guard(&app);
    let safe_src = guard.validate(&src).map_err(guard_err)?;
    let safe_dst = guard.validate(&dst).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service
        .copy_file(&safe_src, &safe_dst)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_file(app: AppHandle, src: String, dst: String) -> Result<(), String> {
    log::info!("Moving file from {} to {}", src, dst);
    let guard = build_guard(&app);
    let safe_src = guard.validate(&src).map_err(guard_err)?;
    let safe_dst = guard.validate(&dst).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service
        .move_file(&safe_src, &safe_dst)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_info(app: AppHandle, path: String) -> Result<FileEntry, String> {
    log::info!("Getting file info: {}", path);
    let guard = build_guard(&app);
    let safe = guard.validate(&path).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service.get_file_info(&safe).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn file_exists(app: AppHandle, path: String) -> bool {
    let guard = build_guard(&app);
    let validated: Result<std::path::PathBuf, _> = guard.validate(&path);
    match validated {
        Ok(safe) => {
            let file_service = FileService::new();
            file_service.exists(&safe)
        }
        Err(_) => false,
    }
}

#[tauri::command]
pub fn walk_directory(app: AppHandle, path: String) -> Result<Vec<FileEntry>, String> {
    log::info!("Walking directory: {}", path);
    let guard = build_guard(&app);
    let safe = guard.validate(&path).map_err(guard_err)?;
    let file_service = FileService::new();
    file_service.walk_directory(&safe).map_err(|e| e.to_string())
}