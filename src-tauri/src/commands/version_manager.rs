use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use thiserror::Error;
use regex::Regex;

/// Allow only characters that appear in real semver-like version strings
/// (`v18.20.4`, `20.11.1`, `3.12.1`, `21.0.2+13`). Anything else (`;`, `|`,
/// `` ` ``, `$`, newline, ...) is rejected before being concatenated into a
/// shell command line. The check is intentionally a *positive* allow-list
/// rather than a block-list of dangerous characters so a future caller can't
/// smuggle a new metachar through.
fn validate_version_string(version: &str) -> Result<(), VersionManagerError> {
    if version.is_empty() {
        return Err(VersionManagerError::CommandFailed(
            "Version string is empty".to_string(),
        ));
    }
    if version.len() > 64 {
        return Err(VersionManagerError::CommandFailed(
            "Version string is too long".to_string(),
        ));
    }
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | '_'))
    {
        return Err(VersionManagerError::CommandFailed(format!(
            "Version string '{}' contains illegal characters",
            version
        )));
    }
    Ok(())
}

/// Default timeout for a version-manager subprocess (nvm, pyenv, jenv).
/// These are interactive CLIs that can hang if the user's shell is in
/// a broken state, so we always cap the wait.
const SUBPROCESS_TIMEOUT: Duration = Duration::from_secs(30);

/// Run a synchronous `Command` with a wall-clock timeout. The subprocess
/// is spawned on a dedicated thread; if it does not finish within the
/// timeout, the function returns a `CommandFailed` error. The subprocess
/// itself is *not* killable from this helper (std::process::Child has
/// no safe cross-platform kill-on-drop for sync APIs), so the orphan
/// will be reaped when the OS shuts down or the parent exits. For
/// critical paths prefer `tokio::process::Command` instead.
fn run_with_timeout(mut cmd: Command, label: &str) -> Result<String, VersionManagerError> {
    let (tx, rx) = mpsc::channel::<Result<std::process::Output, std::io::Error>>();
    thread::Builder::new()
        .name(format!("forge-{}", label))
        .spawn(move || {
            let result = cmd.output();
            let _ = tx.send(result);
        })
        .map_err(|e| VersionManagerError::CommandFailed(format!(
            "Failed to spawn {} thread: {}", label, e
        )))?;

    match rx.recv_timeout(SUBPROCESS_TIMEOUT) {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.success() {
                Ok(stdout.to_string())
            } else {
                let msg = if stderr.is_empty() { stdout.to_string() } else { stderr.to_string() };
                Err(VersionManagerError::CommandFailed(msg))
            }
        }
        Ok(Err(e)) => Err(VersionManagerError::CommandFailed(format!(
            "Failed to run {}: {}", label, e
        ))),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(VersionManagerError::CommandFailed(format!(
            "{} timed out after {}s", label, SUBPROCESS_TIMEOUT.as_secs()
        ))),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(VersionManagerError::CommandFailed(format!(
            "{} worker thread disconnected unexpectedly", label
        ))),
    }
}

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
            "jenv" => self.get_jenv_versions(),
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
        validate_version_string(version)?;

        match software_key {
            "nvm" => self.install_nvm_version(version),
            "pyenv" => self.install_pyenv_version(version),
            "jenv" => Err(VersionManagerError::NotSupported(
                "jenv does not install Java versions, use jenv add instead".to_string(),
            )),
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
        validate_version_string(version)?;

        match software_key {
            "nvm" => self.switch_nvm_version(version),
            "pyenv" => self.switch_pyenv_version(version),
            "jenv" => self.switch_jenv_version(version),
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
        validate_version_string(version)?;

        match software_key {
            "nvm" => self.set_nvm_global_version(version),
            "pyenv" => self.set_pyenv_global_version(version),
            "jenv" => self.set_jenv_global_version(version),
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
        validate_version_string(version)?;

        match software_key {
            "nvm" => self.remove_nvm_version(version),
            "pyenv" => self.remove_pyenv_version(version),
            "jenv" => self.remove_jenv_version(version),
            "homebrew" => Err(VersionManagerError::NotSupported(
                "Homebrew cannot remove versions".to_string(),
            )),
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    // ============ NVM Methods ============

    /// Run an nvm command, using nvm.exe directly on Windows and bash on Unix
    fn run_nvm_command(&self, nvm_args: &str) -> VersionManagerResult<String> {
        // Defence-in-depth: the Unix path forwards `nvm_args` into a
        // `bash -c` string, so any untrusted caller that reaches this
        // function with metacharacters (`;`, `|`, backticks, ...) would
        // be able to execute arbitrary commands. The Tauri command
        // boundary validates the `version` field, but this layer also
        // validates so internal callers can't bypass the check.
        if nvm_args
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && !matches!(c, '.' | '-' | '_' | ' ' | '/' | '=' | '*'))
        {
            return Err(VersionManagerError::CommandFailed(
                "nvm arguments contain illegal characters".to_string(),
            ));
        }
        #[cfg(target_os = "windows")]
        let cmd = {
            let args: Vec<&str> = nvm_args.split_whitespace().collect();
            let mut cmd = Command::new("nvm");
            cmd.args(&args);
            cmd
        };
        #[cfg(not(target_os = "windows"))]
        let cmd = {
            // On Unix/macOS, source nvm.sh inside a bash subshell so we
            // pick up the user's installed nvm regardless of PATH.
            let mut cmd = Command::new("bash");
            cmd.args([
                "-c",
                &format!("source ~/.nvm/nvm.sh && nvm {}", nvm_args),
            ]);
            cmd
        };
        run_with_timeout(cmd, "nvm")
    }

    fn get_nvm_versions(&self) -> VersionManagerResult<VersionListResult> {
        // Get installed versions using platform-appropriate command
        let installed_output = self.run_nvm_command("list")?;

        let mut installed_versions = Vec::new();
        let mut detected_current_from_list: Option<String> = None;

        for line in installed_output.lines() {
            let line = strip_ansi_codes(line);
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Skip alias lines (like "default ->", "iojs ->", "node ->", "lts/* ->", etc.)
            if trimmed.contains(" -> ") {
                continue;
            }

            // Windows nvm-windows format: "  * 20.19.0    (Currently using 64-bit executable)"
            // Unix nvm-sh format: "-> v24.16.0 *" or "  v24.16.0 *"
            let is_current = trimmed.starts_with('*')
                || trimmed.starts_with("->")
                || trimmed.contains("Currently using");

            // Extract version number
            let version = if cfg!(target_os = "windows") {
                // nvm-windows: version may have "v" prefix or not
                // "  * 20.19.0    (Currently using 64-bit executable)"
                let cleaned = trimmed
                    .trim_start_matches('*')
                    .trim_start_matches("->")
                    .trim();

                // Take only the version part (before any whitespace or parentheses)
                let version: String = cleaned
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == 'v' || *c == 'V')
                    .collect();
                version.trim().to_string()
            } else {
                // nvm-sh: "-> v24.16.0 *" or "  v24.16.0 *"
                trimmed
                    .trim_start_matches("->")
                    .trim_end_matches('*')
                    .trim()
                    .to_string()
            };

            if version.is_empty() {
                continue;
            }

            // Validate version looks like a version number
            let version_clean = version.trim_start_matches('v').trim_start_matches('V');
            if !version_clean.is_empty()
                && version_clean
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_digit())
            {
                // Normalize to always include "v" prefix for consistency
                let normalized = if version.starts_with('v') || version.starts_with('V') {
                    format!("v{}", &version[1..])
                } else if version_clean.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    format!("v{}", version)
                } else {
                    version
                };

                if is_current {
                    detected_current_from_list = Some(normalized.clone());
                }
                installed_versions.push(normalized);
            }
        }

        // Get current version
        let current_version = self.run_nvm_command("current").ok().and_then(|output| {
            let version = strip_ansi_codes(output.trim());
            if version.is_empty() || version == "N/A" || version.contains("error") {
                detected_current_from_list.clone()
            } else {
                // Normalize version
                let v = version.trim();
                let normalized = if v.starts_with('v') || v.starts_with('V') {
                    format!("v{}", &v[1..])
                } else {
                    format!("v{}", v)
                };
                Some(normalized)
            }
        }).or(detected_current_from_list);

        // Get default (global) version
        // nvm-sh uses "nvm alias default", nvm-windows doesn't have this
        let global_version = if cfg!(target_os = "windows") {
            // On Windows, the "current" version IS the global default (persists across sessions)
            current_version.clone()
        } else {
            self.run_nvm_command("alias default").ok().and_then(|output| {
                let version = strip_ansi_codes(output.trim());
                if version.is_empty() || version == "N/A" {
                    None
                } else {
                    let version = version
                        .split("->")
                        .nth(1)
                        .map(|s| s.trim().split_whitespace().next().unwrap_or("").to_string())
                        .unwrap_or(version);
                    // Normalize
                    let v = version.trim();
                    let normalized = if v.starts_with('v') || v.starts_with('V') {
                        format!("v{}", &v[1..])
                    } else if v.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                        format!("v{}", v)
                    } else {
                        v.to_string()
                    };
                    Some(normalized)
                }
            })
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
        // nvm-windows uses versions without "v" prefix, nvm-sh uses "v" prefix
        let install_version = if cfg!(target_os = "windows") {
            version.trim_start_matches('v').trim_start_matches('V').to_string()
        } else {
            if version.starts_with('v') {
                version.to_string()
            } else {
                format!("v{}", version)
            }
        };

        let output_str = self.run_nvm_command(&format!("install {}", install_version))?;

        if output_str.to_lowercase().contains("error") || output_str.to_lowercase().contains("not found") {
            Err(VersionManagerError::CommandFailed(output_str))
        } else {
            Ok(VersionOperationResult {
                success: true,
                message: format!("{} installed successfully", install_version),
                new_version: Some(format!("v{}", install_version)),
            })
        }
    }

    fn switch_nvm_version(&self, version: &str) -> VersionManagerResult<VersionOperationResult> {
        // nvm-windows uses versions without "v" prefix
        let use_version = if cfg!(target_os = "windows") {
            version.trim_start_matches('v').trim_start_matches('V').to_string()
        } else {
            version.to_string()
        };

        let output_str = self.run_nvm_command(&format!("use {}", use_version))?;

        if output_str.to_lowercase().contains("error") {
            Err(VersionManagerError::CommandFailed(output_str))
        } else {
            let normalized = if use_version.starts_with('v') || use_version.starts_with('V') {
                use_version
            } else {
                format!("v{}", use_version)
            };
            Ok(VersionOperationResult {
                success: true,
                message: format!("Switched to {}", normalized),
                new_version: Some(normalized),
            })
        }
    }

    fn set_nvm_global_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        #[cfg(target_os = "windows")]
        {
            // nvm-windows: "nvm use <version>" sets the persistent default
            self.switch_nvm_version(version)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let output_str = self.run_nvm_command(&format!("alias default {}", version))?;
            if output_str.to_lowercase().contains("error") {
                Err(VersionManagerError::CommandFailed(output_str))
            } else {
                Ok(VersionOperationResult {
                    success: true,
                    message: format!("Global version set to {}", version),
                    new_version: Some(version.to_string()),
                })
            }
        }
    }

    fn remove_nvm_version(&self, version: &str) -> VersionManagerResult<VersionOperationResult> {
        // nvm-windows uses versions without "v" prefix
        let uninstall_version = if cfg!(target_os = "windows") {
            version.trim_start_matches('v').trim_start_matches('V').to_string()
        } else {
            version.to_string()
        };

        let output_str = self.run_nvm_command(&format!("uninstall {}", uninstall_version))?;

        if output_str.to_lowercase().contains("error") {
            Err(VersionManagerError::CommandFailed(output_str))
        } else {
            Ok(VersionOperationResult {
                success: true,
                message: format!("{} removed successfully", uninstall_version),
                new_version: None,
            })
        }
    }

    // ============ Pyenv Methods ============

    /// Run a pyenv command, using PowerShell on Windows for .cmd/.ps1 file support
    /// 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
    fn run_pyenv_command(&self, pyenv_args: &str) -> VersionManagerResult<String> {
        #[cfg(target_os = "windows")]
        let cmd = {
            // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoProfile", "-Command", &format!("& pyenv {}", pyenv_args)]);
            cmd.creation_flags(flags);
            cmd
        };
        #[cfg(not(target_os = "windows"))]
        let cmd = {
            let args: Vec<&str> = pyenv_args.split_whitespace().collect();
            let mut cmd = Command::new("pyenv");
            cmd.args(&args);
            cmd
        };
        run_with_timeout(cmd, "pyenv")
    }

    fn get_pyenv_versions(&self) -> VersionManagerResult<VersionListResult> {
        // Get installed versions
        let installed_output = self.run_pyenv_command("versions")?;

        let mut installed_versions = Vec::new();
        let mut current_version = None;

        for line in installed_output.lines() {
            let line = line.trim();
            if line.starts_with('*') {
                // Current version: "* 3.12.0 (set by ...)"
                let version = line
                    .trim_start_matches("* ")
                    .trim()
                    // Remove "(set by ...)" suffix
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !version.is_empty() && version != "system" {
                    current_version = Some(version.to_string());
                    installed_versions.push(version.to_string());
                }
            } else if !line.is_empty() && line != "system" {
                // Remove "(set by ...)" suffix if present
                let version = line
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !version.is_empty() {
                    installed_versions.push(version.to_string());
                }
            }
        }

        // Get global version
        let global_output = self.run_pyenv_command("global")?;

        let global_version = {
            let version = global_output.trim().to_string();
            if version.is_empty() || version == "system" {
                None
            } else {
                Some(version)
            }
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
        let _output_str = self.run_pyenv_command(&format!("install {}", version))?;

        // pyenv install success output usually just says "Installing Python-..."
        // Error output would contain "ERROR" or the command would have non-zero exit
        Ok(VersionOperationResult {
            success: true,
            message: format!("{} installed successfully", version),
            new_version: Some(version.to_string()),
        })
    }

    fn switch_pyenv_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let _output_str = self.run_pyenv_command(&format!("local {}", version))?;

        Ok(VersionOperationResult {
            success: true,
            message: format!("Switched to {} locally", version),
            new_version: Some(version.to_string()),
        })
    }

    fn set_pyenv_global_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let _output_str = self.run_pyenv_command(&format!("global {}", version))?;

        Ok(VersionOperationResult {
            success: true,
            message: format!("Global version set to {}", version),
            new_version: Some(version.to_string()),
        })
    }

    fn remove_pyenv_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let _output_str = self.run_pyenv_command(&format!("uninstall -f {}", version))?;

        Ok(VersionOperationResult {
            success: true,
            message: format!("{} removed successfully", version),
            new_version: None,
        })
    }

    // ============ Jenv Methods ============

    /// Run a jenv command, using PowerShell on Windows
    /// 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
    fn run_jenv_command(&self, jenv_args: &str) -> VersionManagerResult<String> {
        #[cfg(target_os = "windows")]
        let cmd = {
            // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoProfile", "-Command", &format!("& jenv {}", jenv_args)]);
            cmd.creation_flags(flags);
            cmd
        };
        #[cfg(not(target_os = "windows"))]
        let cmd = {
            let args: Vec<&str> = jenv_args.split_whitespace().collect();
            let mut cmd = Command::new("jenv");
            cmd.args(&args);
            cmd
        };
        run_with_timeout(cmd, "jenv")
    }

    fn get_jenv_versions(&self) -> VersionManagerResult<VersionListResult> {
        // jenv versions output:
        //   1.8
        //   11.0
        // * 17.0  (set by /path/to/.java-version)
        let installed_output = self.run_jenv_command("versions")?;

        let mut installed_versions = Vec::new();
        let mut current_version = None;

        for line in installed_output.lines() {
            let line = line.trim();
            if line.starts_with('*') {
                let version = line
                    .trim_start_matches("* ")
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim();
                if !version.is_empty() {
                    current_version = Some(version.to_string());
                    installed_versions.push(version.to_string());
                }
            } else if !line.is_empty() {
                let version = line.split_whitespace().next().unwrap_or("").trim();
                if !version.is_empty() {
                    installed_versions.push(version.to_string());
                }
            }
        }

        // jenv global version
        let global_output = self.run_jenv_command("global").ok();
        let global_version = global_output.and_then(|output| {
            let version = output.trim().to_string();
            if version.is_empty() || version.contains("not found") {
                None
            } else {
                Some(version)
            }
        });

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
            message: format!("Found {} managed Java versions", version_count),
        })
    }

    fn switch_jenv_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        // jenv local sets the version for the current directory
        let _output_str = self.run_jenv_command(&format!("local {}", version))?;

        Ok(VersionOperationResult {
            success: true,
            message: format!("Switched to {} locally", version),
            new_version: Some(version.to_string()),
        })
    }

    fn set_jenv_global_version(
        &self,
        version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        let _output_str = self.run_jenv_command(&format!("global {}", version))?;

        Ok(VersionOperationResult {
            success: true,
            message: format!("Global Java version set to {}", version),
            new_version: Some(version.to_string()),
        })
    }

    fn remove_jenv_version(
        &self,
        _version: &str,
    ) -> VersionManagerResult<VersionOperationResult> {
        // jenv doesn't "remove" versions - it only manages which existing Java to use.
        // Users need to uninstall Java themselves.
        Err(VersionManagerError::NotSupported(
            "jenv manages existing Java installations. Uninstall Java from your system to remove a version.".to_string(),
        ))
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
            "jenv" => Ok(Vec::new()), // jenv doesn't install Java versions
            _ => Err(VersionManagerError::NotSupported(software_key.to_string())),
        }
    }

    /// Get available NVM versions from remote
    fn get_nvm_available_versions(&self) -> VersionManagerResult<Vec<AvailableVersion>> {
        #[cfg(target_os = "windows")]
        {
            // nvm-windows: "nvm list available" shows a table of available versions
            // Output format:
            // |   CURRENT    |     LTS      |  OLD STABLE  | OLD UNSTABLE |
            // |--------------|--------------|--------------|--------------|
            // |    26.4.0    |   24.18.0    |   0.12.18    |   0.11.16    |
            let mut cmd = Command::new("nvm");
            cmd.args(["list", "available"]);
            let stdout = run_with_timeout(cmd, "nvm list available")?;

            let mut all_versions: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut lts_versions: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            let mut lts_column_index: Option<usize> = None;
            let _columns: Vec<String> = Vec::new();

            for line in stdout.lines() {
                let line = strip_ansi_codes(line);
                let trimmed = line.trim();

                if trimmed.is_empty()
                    || trimmed.chars().all(|c| c == '|' || c == '-' || c == ' ')
                {
                    continue;
                }

                if trimmed.contains("CURRENT") || trimmed.contains("LTS") {
                    let parts: Vec<&str> = trimmed.split('|').collect();
                    for (i, part) in parts.iter().enumerate() {
                        if part.trim() == "LTS" {
                            lts_column_index = Some(i);
                        }
                    }
                    continue;
                }

                let parts: Vec<&str> = trimmed.split('|').collect();
                for (i, part) in parts.iter().enumerate() {
                    let version = part.trim().to_string();
                    if !version.is_empty()
                        && version.chars().next().map_or(false, |c| c.is_ascii_digit())
                        && version.contains('.')
                    {
                        let normalized = format!("v{}", version);
                        all_versions.insert(normalized.clone());
                        if lts_column_index == Some(i) {
                            lts_versions.insert(normalized, "LTS".to_string());
                        }
                    }
                }
            }

            let mut versions: Vec<AvailableVersion> = all_versions
                .into_iter()
                .map(|v| AvailableVersion {
                    version: v.clone(),
                    lts: lts_versions.get(&v).cloned(),
                })
                .collect();

            versions.sort_by(|a, b| {
                let a_parts: Vec<u32> = a
                    .version
                    .trim_start_matches('v')
                    .split('.')
                    .filter_map(|p| p.parse().ok())
                    .collect();
                let b_parts: Vec<u32> = b
                    .version
                    .trim_start_matches('v')
                    .split('.')
                    .filter_map(|p| p.parse().ok())
                    .collect();
                for i in 0..a_parts.len().max(b_parts.len()) {
                    let a_val = a_parts.get(i).unwrap_or(&0);
                    let b_val = b_parts.get(i).unwrap_or(&0);
                    if a_val != b_val {
                        return b_val.cmp(a_val);
                    }
                }
                std::cmp::Ordering::Equal
            });

            Ok(versions)
        }
        #[cfg(not(target_os = "windows"))]
        {
            // nvm-sh: "nvm ls-remote" shows all remote versions. The
            // network round-trip can hang; bound it with our shared
            // timeout runner.
            let mut cmd = Command::new("bash");
            cmd.args([
                "-c",
                "source ~/.nvm/nvm.sh && nvm ls-remote --no-colors 2>/dev/null \
                 | grep -E 'v[0-9]+\\.[0-9]+\\.[0-9]+' \
                 | awk '{print $1}'",
            ]);
            let stdout = run_with_timeout(cmd, "nvm ls-remote")?;

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
        }
    }

    /// Get available Pyenv versions from remote
    /// 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
    fn get_pyenv_available_versions(&self) -> VersionManagerResult<Vec<AvailableVersion>> {
        #[cfg(target_os = "windows")]
        let cmd = {
            // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let mut cmd = Command::new("powershell.exe");
            cmd.args(["-NoProfile", "-Command", "& pyenv install --list"]);
            cmd.creation_flags(flags);
            cmd
        };
        #[cfg(not(target_os = "windows"))]
        let cmd = {
            let mut cmd = Command::new("pyenv");
            cmd.args(["install", "--list"]);
            cmd
        };
        let stdout = run_with_timeout(cmd, "pyenv install --list")?;

        let versions: Vec<AvailableVersion> = stdout
            .lines()
            .skip(1) // Skip header line
            .filter_map(|line| {
                let version = line.trim().to_string();
                if version.is_empty()
                    || !version.chars().next().map_or(false, |c| c.is_ascii_digit())
                {
                    return None;
                }
                Some(AvailableVersion {
                    version,
                    lts: None,
                })
            })
            .collect();
        Ok(versions)
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
