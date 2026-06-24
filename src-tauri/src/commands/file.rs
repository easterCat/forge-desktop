use crate::models::FileEntry;
use crate::services::FileService;
use std::path::PathBuf;

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    log::info!("Reading file: {}", path);
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.read_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    log::info!("Writing file: {}", path);
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.write_file(&path, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    log::info!("Deleting file: {}", path);
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.delete_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<FileEntry>, String> {
    log::info!("Listing directory: {}", path);
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.list_directory(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_directory(path: String) -> Result<(), String> {
    log::info!("Creating directory: {}", path);
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.create_directory(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_file(src: String, dst: String) -> Result<(), String> {
    log::info!("Copying file from {} to {}", src, dst);
    let file_service = FileService::new();
    let src_path = PathBuf::from(&src);
    let dst_path = PathBuf::from(&dst);
    file_service.copy_file(&src_path, &dst_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_file(src: String, dst: String) -> Result<(), String> {
    log::info!("Moving file from {} to {}", src, dst);
    let file_service = FileService::new();
    let src_path = PathBuf::from(&src);
    let dst_path = PathBuf::from(&dst);
    file_service.move_file(&src_path, &dst_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileEntry, String> {
    log::info!("Getting file info: {}", path);
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.get_file_info(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn file_exists(path: String) -> bool {
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.exists(&path)
}

#[tauri::command]
pub fn walk_directory(path: String) -> Result<Vec<FileEntry>, String> {
    log::info!("Walking directory: {}", path);
    let file_service = FileService::new();
    let path = PathBuf::from(&path);
    file_service.walk_directory(&path).map_err(|e| e.to_string())
}
