use crate::models::BackupRecord;
use crate::services::FileService;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn create_backup(
    state: State<'_, AppState>,
    name: String,
    includes: Vec<String>,
) -> Result<BackupRecord, String> {
    log::info!("Creating backup: {} with includes: {:?}", name, includes);

    let backup_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Forge")
        .join("backups");

    tokio::fs::create_dir_all(&backup_dir)
        .await
        .map_err(|e| e.to_string())?;

    let timestamp = chrono_lite_now();
    let backup_path = backup_dir.join(format!(
        "{}_{}",
        name.replace(" ", "_"),
        timestamp.replace(":", "-")
    ));
    tokio::fs::create_dir_all(&backup_path)
        .await
        .map_err(|e| e.to_string())?;

    let file_service = FileService::new();
    let mut file_count = 0i64;
    let mut total_size: u64 = 0;

    for include_path in &includes {
        let src_path = PathBuf::from(include_path);
        if src_path.exists() {
                    let dest_path = backup_path.join(
                        src_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    );

            let copied = file_service
                .copy_file_async(&src_path, &dest_path)
                .await
                .map_err(|e| e.to_string())?;

            total_size += copied;

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
                        file_count += 1;
                        total_size += metadata.len();
                    }
                }
            }
        }
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
    let backup_record = BackupRecord {
        id: uuid_v4(),
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
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032x}", now)
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
