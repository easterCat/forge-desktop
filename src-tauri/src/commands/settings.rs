//! App-level persistent settings (currently just the GitHub personal access
//! token used to raise the GitHub REST API rate-limit when listing /
//! installing skills from Anthropic / Composio / other remote sources).
//!
//! Storage layout
//! --------------
//! The token is written to `<app_data_dir>/github_token` (a plain UTF-8 text
//! file) with `0600` permissions on Unix. We deliberately avoid a SQLite
//! column so secrets do not end up in database dumps / VACUUM INTO backups
//! that the rest of the app already exports.
//!
//! Read precedence (in `read_github_token`)
//! -----------------------------------------
//! 1. The settings file written by `set_github_token` (highest priority — the
//!    explicit choice the user made in the Settings panel).
//! 2. The `GITHUB_TOKEN` env var (handy for CI / dev runs).
//! 3. None.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use tauri::Manager;

const TOKEN_FILE_NAME: &str = "github_token";

fn token_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app_data_dir: {}", e))?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create app_data_dir {}: {}", dir.display(), e))?;
    Ok(dir.join(TOKEN_FILE_NAME))
}

/// Internal helper used by `anthropic_skills::create_http_client` to pick up
/// the token without going through the Tauri command layer (which requires
/// an `AppHandle` and async context).
///
/// Precedence: settings file > `GITHUB_TOKEN` env var.
pub fn read_github_token() -> Option<String> {
    // 1. Settings file (resolved against the per-user data dir, mirroring
    //    the same path Tauri uses for the sqlite database).
    if let Some(local) = dirs::data_local_dir() {
        let path = local.join("forge").join(TOKEN_FILE_NAME);
        if let Ok(content) = fs::read_to_string(&path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // 2. Environment variable fallback.
    if let Ok(value) = std::env::var("GITHUB_TOKEN") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

/// Tauri command — returns `true` when a token is currently configured
/// (either via the settings file or the `GITHUB_TOKEN` env var). The token
/// itself is *not* returned so the UI never has to put it on the wire or
/// echo it back into logs.
#[tauri::command]
pub fn has_github_token(app: tauri::AppHandle) -> bool {
    if let Ok(path) = token_file_path(&app) {
        if let Ok(content) = fs::read_to_string(&path) {
            if !content.trim().is_empty() {
                return true;
            }
        }
    }
    std::env::var("GITHUB_TOKEN")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

/// Return a masked preview of the stored token (last 4 chars only) so the
/// Settings panel can confirm the right credential is loaded without ever
/// exposing the full secret.
#[tauri::command]
pub fn get_github_token_preview(app: tauri::AppHandle) -> Option<String> {
    if let Ok(path) = token_file_path(&app) {
        if let Ok(content) = fs::read_to_string(&path) {
            let token = content.trim();
            if !token.is_empty() {
                return Some(mask_token(token));
            }
        }
    }
    if let Ok(value) = std::env::var("GITHUB_TOKEN") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(mask_token(trimmed));
        }
    }
    None
}

fn mask_token(token: &str) -> String {
    let len = token.chars().count();
    if len <= 4 {
        return "****".to_string();
    }
    let tail: String = token.chars().rev().take(4).collect::<String>().chars().rev().collect();
    format!("****{}", tail)
}

/// Tauri command — write the token to the settings file. An empty string is
/// rejected to avoid silently creating an empty file.
#[tauri::command]
pub fn set_github_token(app: tauri::AppHandle, token: String) -> Result<(), String> {
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err("Token must not be empty".to_string());
    }
    let path = token_file_path(&app)?;
    write_secret_file(&path, &token)
}

/// Tauri command — remove the settings file (if any). The env-var fallback
/// is not affected (and cannot be).
#[tauri::command]
pub fn clear_github_token(app: tauri::AppHandle) -> Result<(), String> {
    let path = token_file_path(&app)?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove token file: {}", e))?;
    }
    Ok(())
}

fn write_secret_file(path: &PathBuf, contents: &str) -> Result<(), String> {
    // Write to a sibling temp file first, then atomically rename. This
    // avoids leaving a half-written file behind if the process is killed
    // mid-write.
    let tmp = path.with_extension("tmp");
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|e| format!("Failed to open {}: {}", tmp.display(), e))?;

        // Best-effort 0600 on Unix. We don't fail if this errors (e.g. on
        // Windows where the mode is ignored) so the write still succeeds.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = file.set_permissions(perms);
        }

        file.write_all(contents.as_bytes())
            .map_err(|e| format!("Failed to write token: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to fsync token: {}", e))?;
    }
    fs::rename(&tmp, path)
        .map_err(|e| format!("Failed to commit token file: {}", e))
}
