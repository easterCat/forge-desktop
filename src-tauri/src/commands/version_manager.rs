use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;
use regex::Regex;

#[derive(Error, Debug)]
pub enum VersionManagerError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Software not supported: {0}")]
    NotSupported(String),
    #[error("Version not found: {0}")]
    VersionNotFound(String),
}

pub type VersionManagerResult<T> = Result<T, VersionManagerError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub version: String,
    pub is_installed: bool,
    pub is_current: bool,
    pub is_global: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionListResult {
    pub success: bool,
    pub versions: Vec<VersionInfo>,
    pub current_version: Option<String>,
    pub global_version: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionOperationResult {
    pub success: bool,
    pub message: String,
    pub new_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableVersion {
    pub version: String,
    pub lts: Option<String>,
}

/// Remove ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

pub struct VersionManager;

impl VersionManager {
    pub fn new() -> Self {
        Self
    }

    /// Get available versions for a software
    pub fn get_versions(&self, software_key: &str) -> VersionManagerResult<VersionListResult> {
        log::info!("Getting versions for: {}", software_key);

        match software_key {
            "nvm" => self.get_nvm_versions(),
            "pyenv" => self.get_pyenv_versions(),
            "homebrew" => self.get_homebrew_info(),
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    /// Install a specific version
    pub fn install_version(
        &self,
        software_key: &str,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        log::info!("Installing version {} for {}", version, software_key);

        match software_key {
            "nvm" => self.install_nvm_version(version),
            "pyenv" => self.install_pyenv_version(version),
            "homebrew" => Err(VersionManagerError::NotSupported(
                "Homebrew cannot install multiple versions".to_string(),
            )),
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    /// Switch to a specific version
    pub fn switch_version(
        &self,
        software_key: &str,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        log::info!("Switching to version {} for {}", version, software_key);

        match software_key {
            "nvm" => self.switch_nvm_version(version),
            "pyenv" => self.switch_pyenv_version(version),
            "homebrew" => Err(VersionManagerError::NotSupported(
                "Homebrew does not support version switching".to_string(),
            )),
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    /// Set global version
    pub fn set_global_version(
        &self,
        software_key: &str,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        log::info!("Setting global version {} for {}", version, software_key);

        match software_key {
            "nvm" => self.set_nvm_global_version(version),
            "pyenv" => self.set_pyenv_global_version(version),
            "homebrew" => Err(VersionManagerError::NotSupported(
                "Homebrew does not support global version".to_string(),
            )),
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    /// Remove a specific version
    pub fn remove_version(
        &self,
        software_key: &str,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        log::info!("Removing version {} for {}", version, software_key);

        match software_key {
            "nvm" => self.remove_nvm_version(version),
            "pyenv" => self.remove_pyenv_version(version),
            "homebrew" => Err(VersionManagerError::NotSupported(
                "Homebrew cannot remove versions".to_string(),
            )),
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    // ============ NVM Methods ============

    fn get_nvm_versions(&self) -> VersionManagerResult<VersionListResult> {
        // Get installed versions
        let installed_output = Command::new("bash")
            .args(["-c", "source ~/.nvm/nvm.sh && nvm ls"])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        let mut installed_versions = Vec::new();
        if installed_output.status.success() {
            let stdout = String::from_utf8_lossy(&installed_output.stdout);
            for line in stdout.lines() {
                let line = strip_ansi_codes(line);
                let trimmed = line.trim();

                // Skip empty lines
                if trimmed.is_empty() {
                    continue;
                }

                // Skip alias lines (like "default ->", "iojs ->", "node ->", "lts/* ->", etc.)
                // These lines contain " -> " followed by a non-version value
                if trimmed.contains(" -> ") {
                    continue;
                }

                // Extract version number from lines like "-> v24.16.0 *" or "  v24.16.0 *"
                // These are the actual installed versions
                let version = trimmed
                    .trim_start_matches("-> ")
                    .trim_end_matches(" *")  // Remove trailing " *"
                    .trim();

                // Only include versions that start with "v" followed by numbers
                if version.starts_with('v') && version.len() > 1 && version[1..].chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    installed_versions.push(version.to_string());
                }
            }
        }

        // Get current version
        let current_output = Command::new("bash")
            .args(["-c", "source ~/.nvm/nvm.sh && nvm current"])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        let current_version = if current_output.status.success() {
            let stdout = String::from_utf8_lossy(&current_output.stdout);
            let version = strip_ansi_codes(stdout.trim());
            if version.is_empty() || version == "N/A" {
                None
            } else {
                Some(version)
            }
        } else {
            None
        };

        // Get default (global) version
        let default_output = Command::new("bash")
            .args(["-c", "source ~/.nvm/nvm.sh && nvm alias default"])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        let global_version = if default_output.status.success() {
            let stdout = String::from_utf8_lossy(&default_output.stdout);
            let version = strip_ansi_codes(stdout.trim());
            if version.is_empty() || version == "N/A" {
                None
            } else {
                // Extract just the version number from "default -> 24.16.0 (-> v24.16.0)" format
                let version = version
                    .split("->")
                    .nth(1)
                    .map(|s| s.trim().split_whitespace().next().unwrap_or("").to_string())
                    .unwrap_or(version);
                Some(version)
            }
        } else {
            None
        };

        // Build version list
        let versions: Vec<VersionInfo> = installed_versions
            .iter()
            .map(|v| VersionInfo {
                version: v.clone(),
                is_installed: true,
                is_current: current_version.as_deref() == Some(v.as_str()),
                is_global: global_version.as_deref() == Some(v.as_str()),
            })
            .collect();

        let version_count = versions.len();
        Ok(VersionListResult {
            success: true,
            versions,
            current_version,
            global_version,
            message: format!("Found {} installed versions", version_count),
        })
    }

    fn install_nvm_version(&self, version: &str) -> VersionManagerResult<VersionOperationResult> {
        let version = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };

        let output = Command::new("bash")
            .args(["-c", &format!("source ~/.nvm/nvm.sh && nvm install {}", version)])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("{} installed successfully", version),
                new_version: Some(version),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    fn switch_nvm_version(&self, version: &str) -> VersionManagerResult<VersionOperationResult> {
        let output = Command::new("bash")
            .args(["-c", &format!("source ~/.nvm/nvm.sh && nvm use {}", version)])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("Switched to {}", version),
                new_version: Some(version.to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    fn set_nvm_global_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let output = Command::new("bash")
            .args(["-c", &format!("source ~/.nvm/nvm.sh && nvm alias default {}", version)])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("Global version set to {}", version),
                new_version: Some(version.to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    fn remove_nvm_version(&self, version: &str) -> VersionManagerResult<VersionOperationResult> {
        let output = Command::new("bash")
            .args(["-c", &format!("source ~/.nvm/nvm.sh && nvm uninstall {}", version)])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("{} removed successfully", version),
                new_version: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    // ============ Pyenv Methods ============

    fn get_pyenv_versions(&self) -> VersionManagerResult<VersionListResult> {
        // Get installed versions
        let installed_output = Command::new("pyenv")
            .arg("versions")
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        let mut installed_versions = Vec::new();
        let mut current_version = None;

        if installed_output.status.success() {
            let stdout = String::from_utf8_lossy(&installed_output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with('*') {
                    // Current version
                    let version = line.trim_start_matches("* ").trim();
                    if !version.is_empty() {
                        current_version = Some(version.to_string());
                        installed_versions.push(version.to_string());
                    }
                } else if !line.is_empty() {
                    installed_versions.push(line.to_string());
                }
            }
        }

        // Get global version
        let global_output = Command::new("pyenv")
            .args(["global"])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        let global_version = if global_output.status.success() {
            let stdout = String::from_utf8_lossy(&global_output.stdout);
            let version = stdout.trim().to_string();
            if version.is_empty() || version == "system" {
                None
            } else {
                Some(version)
            }
        } else {
            None
        };

        // Build version list
        let versions: Vec<VersionInfo> = installed_versions
            .iter()
            .map(|v| VersionInfo {
                version: v.clone(),
                is_installed: true,
                is_current: current_version.as_deref() == Some(v.as_str()),
                is_global: global_version.as_deref() == Some(v.as_str()),
            })
            .collect();

        let version_count = versions.len();
        Ok(VersionListResult {
            success: true,
            versions,
            current_version,
            global_version,
            message: format!("Found {} installed versions", version_count),
        })
    }

    fn install_pyenv_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let output = Command::new("pyenv")
            .args(["install", version])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("{} installed successfully", version),
                new_version: Some(version.to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    fn switch_pyenv_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let output = Command::new("pyenv")
            .args(["local", version])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("Switched to {} locally", version),
                new_version: Some(version.to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    fn set_pyenv_global_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let output = Command::new("pyenv")
            .args(["global", version])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("Global version set to {}", version),
                new_version: Some(version.to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    fn remove_pyenv_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let output = Command::new("pyenv")
            .args(["uninstall", "-f", version])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            Ok(VersionOperationResult {
                success: true,
                message: format!("{} removed successfully", version),
                new_version: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(VersionManagerError::CommandFailed(stderr.to_string()))
        }
    }

    // ============ Homebrew Methods ============

    fn get_homebrew_info(&self) -> VersionManagerResult<VersionListResult> {
        let output = Command::new("brew")
            .arg("--version")
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            Ok(VersionListResult {
                success: true,
                versions: vec![VersionInfo {
                    version: version.clone(),
                    is_installed: true,
                    is_current: true,
                    is_global: false,
                }],
                current_version: Some(version.clone()),
                global_version: None,
                message: "Homebrew info retrieved".to_string(),
            })
        } else {
            Err(VersionManagerError::CommandFailed(
                "Failed to get Homebrew version".to_string(),
            ))
        }
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionManager {
    /// Get available versions for a software (remote versions that can be installed)
    pub fn get_available_versions(&self, software_key: &str) -> VersionManagerResult<Vec<AvailableVersion>> {
        log::info!("Getting available versions for: {}", software_key);

        match software_key {
            "nvm" => self.get_nvm_available_versions(),
            "pyenv" => self.get_pyenv_available_versions(),
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    /// Get available NVM versions from remote
    fn get_nvm_available_versions(&self) -> VersionManagerResult<Vec<AvailableVersion>> {
        let output = Command::new("bash")
            .args(["-c", "source ~/.nvm/nvm.sh && nvm ls-remote --no-colors 2>/dev/null | grep -E 'v[0-9]+\\.[0-9]+\\.[0-9]+' | awk '{print $1}'"])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let versions: Vec<AvailableVersion> = stdout
                .lines()
                .filter_map(|line| {
                    let version = strip_ansi_codes(line).trim().to_string();
                    if version.is_empty() || !version.starts_with('v') {
                        return None;
                    }
                    Some(AvailableVersion {
                        version,
                        lts: None,
                    })
                })
                .collect();
            Ok(versions)
        } else {
            Err(VersionManagerError::CommandFailed("Failed to get remote versions".to_string()))
        }
    }

    /// Get available Pyenv versions from remote
    fn get_pyenv_available_versions(&self) -> VersionManagerResult<Vec<AvailableVersion>> {
        let output = Command::new("pyenv")
            .args(["install", "--list"])
            .output()
            .map_err(|e| VersionManagerError::CommandFailed(e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let versions: Vec<AvailableVersion> = stdout
                .lines()
                .skip(1) // Skip header line
                .filter_map(|line| {
                    let version = line.trim().to_string();
                    if version.is_empty() || !version.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                        return None;
                    }
                    Some(AvailableVersion {
                        version,
                        lts: None,
                    })
                })
                .collect();
            Ok(versions)
        } else {
            Err(VersionManagerError::CommandFailed("Failed to get remote versions".to_string()))
        }
    }
}

// ============ Tauri Commands ============

#[tauri::command]
pub fn get_version_list(software_key: String) -> Result<VersionListResult, String> {
    let manager = VersionManager::new();
    manager
        .get_versions(&software_key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_available_versions(software_key: String) -> Result<Vec<AvailableVersion>, String> {
    let manager = VersionManager::new();
    manager
        .get_available_versions(&software_key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn install_version(software_key: String, version: String) -> Result<VersionOperationResult, String> {
    let manager = VersionManager::new();
    manager
        .install_version(&software_key, &version)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn switch_version(software_key: String, version: String) -> Result<VersionOperationResult, String> {
    let manager = VersionManager::new();
    manager
        .switch_version(&software_key, &version)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_global_version(software_key: String, version: String) -> Result<VersionOperationResult, String> {
    let manager = VersionManager::new();
    manager
        .set_global_version(&software_key, &version)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_version(software_key: String, version: String) -> Result<VersionOperationResult, String> {
    let manager = VersionManager::new();
    manager
        .remove_version(&software_key, &version)
        .map_err(|e| e.to_string())
}
