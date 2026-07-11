use crate::models::BackupRecord;
use crate::services::FileService;
use crate::utils::now_rfc3339;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

/// Hard cap on the total bytes a single backup may consume. Defends
/// against a UI bug / hostile IPC call that passes paths like `/` and
/// silently fills the user's disk.
const BACKUP_MAX_TOTAL_BYTES: u64 = 5 * 1024 * 1024 * 1024; // 5 GiB

/// Hard cap on the number of files copied into a single backup.
const BACKUP_MAX_FILE_COUNT: i64 = 100_000;

#[tauri::command]
pub async fn create_backup(
    state: State<'_, AppState>,
    name: String,
    includes: Vec<String>,
) -> Result<BackupRecord, String> {
    log::info!("Creating backup: {} with includes: {:?}", name, includes);

    // Reject empty / too-large input up front to fail fast and avoid
    // touching the filesystem.
    if includes.is_empty() {
        return Err("No paths to include in backup".to_string());
    }
    if name.is_empty() || name.len() > 200 {
        return Err("Backup name must be 1..=200 characters".to_string());
    }

    let backup_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Forge")
        .join("backups");

    tokio::fs::create_dir_all(&backup_dir)
        .await
        .map_err(|e| e.to_string())?;

    let timestamp = now_rfc3339();
    let safe_name = sanitize_backup_name(&name);
    let backup_path = backup_dir.join(format!(
        "{}_{}",
        safe_name,
        timestamp.replace(":", "-")
    ));
    tokio::fs::create_dir_all(&backup_path)
        .await
        .map_err(|e| e.to_string())?;

    let file_service = FileService::new();
    let mut file_count: i64 = 0;
    let mut total_size: u64 = 0;

    for include_path in &includes {
        // Refuse absolute paths that look like system roots. The UI only
        // passes pre-vetted user directories, but a hostile IPC call
        // must not be able to ask us to copy `/`, `C:\`, `/Users`, etc.
        if is_forbidden_backup_root(include_path) {
            return Err(format!(
                "Refusing to back up system root: '{}'",
                include_path
            ));
        }

        let src_path = PathBuf::from(include_path);
        if !src_path.exists() {
            log::warn!("Backup include does not exist, skipping: {}", include_path);
            continue;
        }

        let dest_path = backup_path.join(
            src_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
        );

        let copied = file_service
            .copy_file_async(&src_path, &dest_path)
            .await
            .map_err(|e| e.to_string())?;

        total_size = total_size.saturating_add(copied);
        if total_size > BACKUP_MAX_TOTAL_BYTES {
            let _ = tokio::fs::remove_dir_all(&backup_path).await;
            return Err(format!(
                "Backup exceeded {} GB cap; aborted",
                BACKUP_MAX_TOTAL_BYTES / 1024 / 1024 / 1024
            ));
        }

        // Count files asynchronously
        let mut stack = vec![dest_path.clone()];
        while let Some(current) = stack.pop() {
            let mut dir = tokio::fs::read_dir(&current)
                .await
                .map_err(|e| e.to_string())?;

            while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
                let entry_path = entry.path();
                let metadata = entry.metadata().await.map_err(|e| e.to_string())?;

                if metadata.is_dir() {
                    stack.push(entry_path);
                } else {
                    file_count = file_count.saturating_add(1);
                    if file_count > BACKUP_MAX_FILE_COUNT {
                        let _ = tokio::fs::remove_dir_all(&backup_path).await;
                        return Err(format!(
                            "Backup exceeded {} file cap; aborted",
                            BACKUP_MAX_FILE_COUNT
                        ));
                    }
                    total_size = total_size.saturating_add(metadata.len());
                    if total_size > BACKUP_MAX_TOTAL_BYTES {
                        let _ = tokio::fs::remove_dir_all(&backup_path).await;
                        return Err(format!(
                            "Backup exceeded {} GB cap; aborted",
                            BACKUP_MAX_TOTAL_BYTES / 1024 / 1024 / 1024
                        ));
                    }
                }
            }
        }
    }

    if file_count == 0 {
        let _ = tokio::fs::remove_dir_all(&backup_path).await;
        return Err("Backup produced no files; refusing to persist".to_string());
    }

    let manifest_path = backup_path.join("manifest.json");
    let manifest = serde_json::json!({
        "name": name,
        "includes": includes,
        "created_at": timestamp,
        "file_count": file_count,
        "size": total_size
    });

    tokio::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap())
        .await
        .map_err(|e| e.to_string())?;

    let includes_json = serde_json::to_string(&includes).unwrap_or_default();
    // i64 is the SQLite column type. `total_size` is u64 ≤ BACKUP_MAX_TOTAL_BYTES (5 GiB),
    // well below i64::MAX, so this cast cannot overflow.
    let backup_record = BackupRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path: backup_path.display().to_string(),
        size: Some(total_size as i64),
        file_count: Some(file_count as i32),
        created_at: Some(timestamp),
        includes: Some(includes_json),
    };

    state
        .db
        .upsert_backup_record(&backup_record)
        .map_err(|e| e.to_string())?;

    log::info!("Backup created successfully: {}", backup_record.id);
    Ok(backup_record)
}

/// Replace any path separators or shell metacharacters in the user-supplied
/// backup name with `_` so it cannot escape `backup_dir`.
fn sanitize_backup_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Returns true if `path` is a path we should never back up wholesale.
fn is_forbidden_backup_root(path: &str) -> bool {
    let p = path.replace('\\', "/");
    matches!(
        p.trim().to_ascii_lowercase().as_str(),
        "/" | "/." | "" | "c:/" | "c:" | "d:/" | "d:" | "/users" | "/home" | "/etc" | "/var" | "/tmp"
    )
}

#[tauri::command]
pub fn get_backups(state: State<AppState>) -> Result<Vec<BackupRecord>, String> {
    log::info!("Getting backup records");
    state.db.get_all_backup_records().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_backup(
    state: State<'_, AppState>,
    backup_id: String,
) -> Result<(), String> {
    log::info!("Restoring backup: {}", backup_id);

    let backups = state.db.get_all_backup_records().map_err(|e| e.to_string())?;
    let backup = backups
        .into_iter()
        .find(|b| b.id == backup_id)
        .ok_or_else(|| "Backup not found".to_string())?;

    let backup_path = PathBuf::from(&backup.path);
    if !backup_path.exists() {
        return Err("Backup path does not exist".to_string());
    }

    let includes: Vec<String> = backup
        .includes
        .as_ref()
        .and_then(|i| serde_json::from_str(i).ok())
        .unwrap_or_default();

    let file_service = FileService::new();

    for include_path in &includes {
        let src_path = PathBuf::from(include_path);
        let dest_path = if src_path.exists() {
            src_path
        } else {
            let parent = src_path.parent().unwrap_or(&src_path);
            parent.to_path_buf()
        };

        // Async walk directory
        let mut stack = vec![backup_path.clone()];
        let mut files_to_copy: Vec<(PathBuf, PathBuf)> = Vec::new();

        while let Some(current) = stack.pop() {
            let mut dir = tokio::fs::read_dir(&current)
                .await
                .map_err(|e| e.to_string())?;

            while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
                let entry_path = entry.path();
                let metadata = entry.metadata().await.map_err(|e| e.to_string())?;

                if metadata.is_dir() {
                    stack.push(entry_path);
                } else {
                    let relative_path = entry_path.strip_prefix(&backup_path).unwrap_or(&entry_path);
                    let target = dest_path.join(relative_path);
                    files_to_copy.push((entry_path, target));
                }
            }
        }

        // Copy all files
        for (src, dst) in files_to_copy {
            file_service
                .copy_file_async(&src, &dst)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    log::info!("Backup restored successfully");
    Ok(())
}

#[tauri::command]
pub async fn delete_backup(
    state: State<'_, AppState>,
    backup_id: String,
) -> Result<(), String> {
    log::info!("Deleting backup: {}", backup_id);

    let backups = state.db.get_all_backup_records().map_err(|e| e.to_string())?;
    let backup = backups
        .into_iter()
        .find(|b| b.id == backup_id)
        .ok_or_else(|| "Backup not found".to_string())?;

    let backup_path = PathBuf::from(&backup.path);
    if backup_path.exists() {
        tokio::fs::remove_dir_all(&backup_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    state
        .db
        .delete_backup_record(&backup_id)
        .map_err(|e| e.to_string())?;

    log::info!("Backup deleted successfully");
    Ok(())
}

#[tauri::command]
pub async fn get_backup_contents(path: String) -> Result<Vec<String>, String> {
    let backup_path = PathBuf::from(&path);
    if !backup_path.exists() {
        return Err("Backup path does not exist".to_string());
    }

    let mut files = Vec::new();
    let mut stack = vec![backup_path.clone()];

    while let Some(current) = stack.pop() {
        let mut dir = tokio::fs::read_dir(&current)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
            let entry_path = entry.path();
            let metadata = entry.metadata().await.map_err(|e| e.to_string())?;

            if metadata.is_dir() {
                stack.push(entry_path);
            } else if let Ok(rel_path) = entry_path.strip_prefix(&backup_path) {
                let rel_str = rel_path.display().to_string();
                if rel_str != "manifest.json" {
                    files.push(rel_str);
                }
            }
        }
    }

    Ok(files)
}

fn uuid_v4() -> String {
    // Retained as a thin wrapper for any future caller; delegates to the
    // `uuid` crate to keep timestamps from colliding under high load.
    uuid::Uuid::new_v4().to_string()
}
