use crate::models::{Software, SoftwareStatus};
use crate::utils::now_rfc3339;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

use crate::commands_ext::CommandExt;

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type ScannerResult<T> = Result<T, ScannerError>;

#[derive(Debug, Clone)]
pub struct SoftwareConfig {
    pub key: String,
    pub name: String,
    pub tier: u8,
    pub platform: String,
    pub default_config_paths: Vec<PathBuf>,
    pub website_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DetectedSoftware {
    pub key: String,
    pub name: String,
    pub tier: u8,
    pub platform: String,
    pub version: Option<String>,
    pub config_path: String,
    pub is_installed: bool,
    pub website_url: Option<String>,
}

impl From<DetectedSoftware> for Software {
    fn from(detected: DetectedSoftware) -> Self {
        let status = SoftwareScanner::new().compute_status(
            detected.is_installed,
            detected.version.as_deref(),
            None,
        );
        Software {
            id: uuid_from_key(&detected.key),
            name: detected.name,
            key: detected.key,
            version: detected.version,
            install_path: None,
            config_path: detected.config_path,
            is_installed: detected.is_installed,
            last_checked: Some(now_rfc3339()),
            latest_version: None,
            is_upgradable: false,
            status,
            website_url: detected.website_url,
            platform: Some(detected.platform),
        }
    }
}

pub struct SoftwareScanner;

impl SoftwareScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn get_supported_software(&self) -> Vec<SoftwareConfig> {
        let mut configs = Vec::new();

        // Tier 1: Foundation Environment (P0)
        configs.extend(vec![
            SoftwareConfig {
                key: "homebrew".to_string(),
                name: "Homebrew".to_string(),
                tier: 1,
                platform: "macOS".to_string(),
                default_config_paths: vec![
                    // Apple Silicon Mac
                    PathBuf::from("/opt/homebrew"),
                    // Intel Mac
                    PathBuf::from("/usr/local/Homebrew"),
                    // Fallback: check brew command in PATH
                    PathBuf::from("/opt/homebrew/bin/brew"),
                    PathBuf::from("/usr/local/bin/brew"),
                ],
                website_url: Some("https://brew.sh".to_string()),
            },
            SoftwareConfig {
                key: "scoop".to_string(),
                name: "Scoop".to_string(),
                tier: 1,
                platform: "Windows".to_string(),
                default_config_paths: vec![
                    // 默认安装路径
                    dirs::home_dir()
                        .map(|p| p.join("scoop"))
                        .unwrap_or_default(),
                    // 备选路径：.local/share/scoop
                    dirs::home_dir()
                        .map(|p| p.join(".local").join("share").join("scoop"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://scoop.sh".to_string()),
            },
            SoftwareConfig {
                key: "windows-terminal".to_string(),
                name: "Windows Terminal".to_string(),
                tier: 1,
                platform: "Windows".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| {
                        p.join("AppData")
                            .join("Local")
                            .join("Packages")
                            .join("Microsoft.WindowsTerminal")
                    })
                    .unwrap_or_default()],
                website_url: Some("https://github.com/microsoft/terminal".to_string()),
            },
            SoftwareConfig {
                key: "sudo".to_string(),
                name: "Sudo".to_string(),
                tier: 1,
                platform: "Windows".to_string(),
                default_config_paths: vec![
                    // gsudo path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("gsudo").join("current"))
                        .unwrap_or_default(),
                    // sudo path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("sudo").join("current"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/gerardog/gsudo".to_string()),
            },
            SoftwareConfig {
                key: "git".to_string(),
                name: "Git".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    // Scoop installation path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("git").join("current"))
                        .unwrap_or_default(),
                    // Git for Windows default path
                    PathBuf::from("C:\\Program Files\\Git"),
                    // Portable Git
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("git").join("current").join("cmd").join("git.exe"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://git-scm.com".to_string()),
            },
            SoftwareConfig {
                key: "7zip".to_string(),
                name: "7-Zip".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    // Scoop installation path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("7zip").join("current"))
                        .unwrap_or_default(),
                    // 7-Zip default installation path
                    PathBuf::from("C:\\Program Files\\7-Zip"),
                ],
                website_url: Some("https://www.7-zip.org".to_string()),
            },
            SoftwareConfig {
                key: "switchhosts".to_string(),
                name: "SwitchHosts".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    // Scoop installation path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("switchhosts").join("current"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/oldj/SwitchHosts".to_string()),
            },
            SoftwareConfig {
                key: "colortool".to_string(),
                name: "ColorTool".to_string(),
                tier: 1,
                platform: "Windows".to_string(),
                default_config_paths: vec![
                    // Scoop installation path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("colortool").join("current"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/microsoft/terminal".to_string()),
            },
            SoftwareConfig {
                key: "iterm2".to_string(),
                name: "iTerm2".to_string(),
                tier: 1,
                platform: "macOS".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join("Library").join("Application Support").join("iTerm2"))
                    .unwrap_or_default()],
                website_url: Some("https://iterm2.com".to_string()),
            },
            SoftwareConfig {
                key: "oh-my-posh".to_string(),
                name: "Oh My Posh".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".oh-my-posh.omp.json"))
                    .unwrap_or_default()],
                website_url: Some("https://ohmyposh.dev".to_string()),
            },
            SoftwareConfig {
                key: "vscode".to_string(),
                name: "VS Code".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".config").join("Code").join("User"))
                    .unwrap_or_default()],
                website_url: Some("https://code.visualstudio.com".to_string()),
            },
            // Tier 2: Language Version Managers (P1)
            SoftwareConfig {
                key: "nvm".to_string(),
                name: "nvm (Node)".to_string(),
                tier: 2,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    // Unix/Linux/macOS path
                    dirs::home_dir()
                        .map(|p| p.join(".nvm"))
                        .unwrap_or_default(),
                    // Windows: Scoop installation path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("nvm").join("current"))
                        .unwrap_or_default(),
                    // Windows: nvm-windows default path
                    dirs::home_dir()
                        .map(|p| p.join("AppData").join("Roaming").join("nvm"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/nvm-sh/nvm".to_string()),
            },
            SoftwareConfig {
                key: "pyenv".to_string(),
                name: "pyenv (Python)".to_string(),
                tier: 2,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    // Unix/Linux/macOS path
                    dirs::home_dir()
                        .map(|p| p.join(".pyenv"))
                        .unwrap_or_default(),
                    // Windows: Scoop installation path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("pyenv").join("current"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/pyenv/pyenv".to_string()),
            },
            SoftwareConfig {
                key: "jenv".to_string(),
                name: "jenv (Java)".to_string(),
                tier: 2,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    // Unix/Linux/macOS path
                    dirs::home_dir()
                        .map(|p| p.join(".jenv"))
                        .unwrap_or_default(),
                    // Windows: Scoop installation path
                    dirs::home_dir()
                        .map(|p| p.join("scoop").join("apps").join("jenv").join("current"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/jenv/jenv".to_string()),
            },
            // Tier 3: Runtime & Containers (P0)
            SoftwareConfig {
                key: "ffmpeg".to_string(),
                name: "FFmpeg".to_string(),
                tier: 3,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    PathBuf::from("/usr/local/bin/ffmpeg"),
                    PathBuf::from("/usr/bin/ffmpeg"),
                ],
                website_url: Some("https://ffmpeg.org".to_string()),
            },
            // Tier 4: Debug & Collaboration (P1)
            SoftwareConfig {
                key: "postman".to_string(),
                name: "Postman".to_string(),
                tier: 4,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| {
                        p.join("Library")
                            .join("Application Support")
                            .join("Postman")
                    })
                    .unwrap_or_default()],
                website_url: Some("https://www.postman.com".to_string()),
            },
            SoftwareConfig {
                key: "cyberduck".to_string(),
                name: "Cyberduck (SFTP)".to_string(),
                tier: 4,
                platform: "macOS".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| {
                        p.join("Library")
                            .join("Application Support")
                            .join("Cyberduck")
                    })
                    .unwrap_or_default()],
                website_url: Some("https://cyberduck.io".to_string()),
            },
            // Tier 5: Productivity Tools (P2)
            SoftwareConfig {
                key: "snipaste".to_string(),
                name: "Snipaste".to_string(),
                tier: 5,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| {
                        p.join("Library")
                            .join("Application Support")
                            .join("Snipaste")
                    })
                    .unwrap_or_default()],
                website_url: Some("https://www.snipaste.com".to_string()),
            },
            SoftwareConfig {
                key: "obsidian".to_string(),
                name: "Obsidian".to_string(),
                tier: 5,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| {
                        p.join("Library")
                            .join("Application Support")
                            .join("obsidian")
                    })
                    .unwrap_or_default()],
                website_url: Some("https://obsidian.md".to_string()),
            },
            SoftwareConfig {
                key: "excalidraw".to_string(),
                name: "Excalidraw".to_string(),
                tier: 5,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".excalidraw"))
                    .unwrap_or_default()],
                website_url: Some("https://excalidraw.com".to_string()),
            },
            // Legacy: AI Tools (from original code)
            SoftwareConfig {
                key: "cursor".to_string(),
                name: "Cursor".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".cursor"))
                    .unwrap_or_default()],
                website_url: None,
            },
            SoftwareConfig {
                key: "windsurf".to_string(),
                name: "Windsurf".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".windsurf"))
                    .unwrap_or_default()],
                website_url: None,
            },
            SoftwareConfig {
                key: "claude-desktop".to_string(),
                name: "Claude Desktop".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".claude"))
                    .unwrap_or_default()],
                website_url: None,
            },
            SoftwareConfig {
                key: "continue".to_string(),
                name: "Continue".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".continue"))
                    .unwrap_or_default()],
                website_url: None,
            },
            SoftwareConfig {
                key: "cody".to_string(),
                name: "Cody".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![dirs::home_dir()
                    .map(|p| p.join(".cody"))
                    .unwrap_or_default()],
                website_url: None,
            },
        ]);

        configs
    }

    /// Detect all software - parallel version using rayon
    pub fn detect_software_parallel(&self) -> Vec<Software> {
        let configs = self.get_supported_software();

        use rayon::prelude::*;

        configs
            .par_iter()
            .map(|config| self.detect_single_sync(config))
            .collect()
    }

    /// Parallel detection with pre-installed version information
    /// Used after installation to ensure the newly installed version is captured
    pub fn detect_software_parallel_with_versions(
        &self,
        installed_versions: Option<std::collections::HashMap<String, String>>,
    ) -> Vec<Software> {
        let configs = self.get_supported_software();
        let installed_versions = installed_versions.unwrap_or_default();

        use rayon::prelude::*;

        configs
            .par_iter()
            .map(|config| {
                let mut software = self.detect_single_sync(config);

                // If we have a pre-installed version for this software and it's not detected yet
                if let Some(version) = installed_versions.get(&config.key) {
                    if !version.is_empty() {
                        // If software was not detected as installed, but we have version info
                        // from the installation command, use that version
                        if !software.is_installed || software.version.is_none() {
                            software.is_installed = true;
                            software.version = Some(version.clone());
                            software.status = SoftwareStatus::Installed;
                        }
                    }
                }

                software
            })
            .collect()
    }

    /// Synchronous single software detection (for parallel execution)
    fn detect_single_sync(&self, config: &SoftwareConfig) -> Software {
        // Check all config paths and use the first one that exists
        // Add retry mechanism for Scoop symbolic links that may take time to create
        let mut config_path = PathBuf::new();
        let mut is_installed = false;
        let max_retries = 3;
        let retry_delay = Duration::from_millis(500);

        for _ in 0..max_retries {
            for path in &config.default_config_paths {
                if path.exists() {
                    config_path = path.clone();
                    is_installed = true;
                    break;
                }
            }

            if is_installed {
                break;
            }

            // Wait before retry (only if we have paths to check)
            if !config.default_config_paths.is_empty() {
                std::thread::sleep(retry_delay);
            }
        }

        // If no path found, use the first one as default
        if config_path == PathBuf::new() {
            config_path = config
                .default_config_paths
                .first()
                .cloned()
                .unwrap_or_default();
        }

        let version = if is_installed {
            self.detect_version_with_timeout(&config.key, &config_path)
        } else {
            None
        };

        let install_path = if is_installed {
            self.detect_install_path(&config.key)
        } else {
            None
        };

        let status = self.compute_status(is_installed, version.as_deref(), None);

        Software {
            id: uuid_from_key(&config.key),
            name: config.name.clone(),
            key: config.key.clone(),
            version,
            install_path,
            config_path: config_path.display().to_string(),
            is_installed,
            last_checked: Some(now_rfc3339()),
            latest_version: None,
            is_upgradable: false,
            status,
            website_url: config.website_url.clone(),
            platform: Some(config.platform.clone()),
        }
    }

    pub fn detect_software(&self) -> Vec<Software> {
        let configs = self.get_supported_software();
        let mut results = Vec::new();

        for config in configs {
            // Check all config paths and use the first one that exists
            let mut config_path = PathBuf::new();
            let mut is_installed = false;

            for path in &config.default_config_paths {
                if path.exists() {
                    config_path = path.clone();
                    is_installed = true;
                    break;
                }
            }

            // If no path found, use the first one as default
            if config_path == PathBuf::new() {
                config_path = config
                    .default_config_paths
                    .first()
                    .cloned()
                    .unwrap_or_default();
            }

            let version = if is_installed {
                self.detect_version_with_timeout(&config.key, &config_path)
            } else {
                None
            };

            let install_path = if is_installed {
                self.detect_install_path(&config.key)
            } else {
                None
            };

            let last_checked = now_rfc3339();
            let status = self.compute_status(is_installed, version.as_deref(), None);

            results.push(Software {
                id: uuid_from_key(&config.key),
                name: config.name,
                key: config.key,
                version,
                install_path,
                config_path: config_path.display().to_string(),
                is_installed,
                last_checked: Some(last_checked),
                latest_version: None,
                is_upgradable: false,
                status,
                website_url: config.website_url.clone(),
                platform: Some(config.platform.clone()),
            });
        }

        results
    }

    pub fn detect_single(&self, key: &str) -> Option<Software> {
        let configs = self.get_supported_software();
        let config = configs.into_iter().find(|c| c.key == key)?;

        // Check all config paths and use the first one that exists
        let mut config_path = PathBuf::new();
        let mut is_installed = false;

        for path in &config.default_config_paths {
            if path.exists() {
                config_path = path.clone();
                is_installed = true;
                break;
            }
        }

        // If no path found, use the first one as default
        if config_path == PathBuf::new() {
            config_path = config
                .default_config_paths
                .first()
                .cloned()
                .unwrap_or_default();
        }

        let version = if is_installed {
            self.detect_version_with_timeout(key, &config_path)
        } else {
            None
        };

        let install_path = if is_installed {
            self.detect_install_path(key)
        } else {
            None
        };

        let status = self.compute_status(is_installed, version.as_deref(), None);

        Some(Software {
            id: uuid_from_key(key),
            name: config.name,
            key: key.to_string(),
            version,
            install_path,
            config_path: config_path.display().to_string(),
            is_installed,
            last_checked: Some(now_rfc3339()),
            latest_version: None,
            is_upgradable: false,
            status,
            website_url: config.website_url.clone(),
            platform: Some(config.platform.clone()),
        })
    }

    pub fn get_config_path(&self, key: &str) -> Option<PathBuf> {
        let configs = self.get_supported_software();
        let config = configs.into_iter().find(|c| c.key == key)?;
        config.default_config_paths.first().cloned()
    }

    fn detect_version(&self, key: &str, config_path: &PathBuf) -> Option<String> {
        match key {
            // AI Tools (Tier 0)
            "cursor" => self.detect_cursor_version(config_path),
            "windsurf" => self.detect_windsurf_version(config_path),
            "claude-desktop" => self.detect_claude_version(config_path),
            "continue" => self.detect_continue_version(config_path),
            "cody" => self.detect_cody_version(config_path),
            // Tier 1: Foundation
            "vscode" => self.detect_vscode_version(),
            "iterm2" => self.detect_iterm2_version(config_path),
            "oh-my-posh" => self.detect_oh_my_posh_version(config_path),
            "scoop" => self.detect_scoop_version(),
            "sudo" => self.detect_sudo_version(),
            "git" => self.detect_git_version(),
            "7zip" => self.detect_7zip_version(),
            "switchhosts" => self.detect_switchhosts_version(),
            "colortool" => self.detect_colortool_version(),
            // Tier 2: Language Managers
            "nvm" => self.detect_nvm_version(),
            "pyenv" => self.detect_pyenv_version(),
            "jenv" => self.detect_jenv_version(),
            "homebrew" => self.detect_homebrew_version(),
            // Tier 3: Runtime
            "ffmpeg" => self.detect_ffmpeg_version(),
            // Tier 4: Debug Tools
            "postman" => self.detect_postman_version(config_path),
            // Tier 5: Productivity
            "obsidian" => self.detect_obsidian_version(config_path),
            "snipaste" => self.detect_snipaste_version(config_path),
            _ => self.detect_generic_version(key),
        }
    }

    fn detect_version_sync(&self, key: &str, config_path: &PathBuf) -> Option<String> {
        self.detect_version_with_timeout(key, config_path)
    }

    /// Run version detection with a 1000ms timeout to avoid blocking parallel scans.
    /// Returns None on timeout, IO error, or non-zero exit.
    fn detect_version_with_timeout(&self, key: &str, config_path: &PathBuf) -> Option<String> {
        let key = key.to_string();
        let config_path = config_path.clone();
        // Use a small dedicated thread to enforce a hard timeout without async runtime.
        let handle = std::thread::Builder::new()
            .name(format!("detect-{}", key))
            .spawn(move || SoftwareScanner::new().detect_version(&key, &config_path))
            .ok()?;
        // Wait up to 3000ms for the version detection thread.
        // Increased from 1000ms to allow newly installed software to initialize
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(3000);
        loop {
            if handle.is_finished() {
                return handle.join().ok().flatten();
            }
            if start.elapsed() >= timeout {
                // Thread is taking too long; abandon it and return None.
                return None;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Compute the InstallStatus from detection results.
    ///
    /// NOTE: System software (git, docker, brew, vscode...) has no reliable
    /// "latest version" data source we can hit from a desktop app — the
    /// canonical way to check is `brew outdated` / `apt list --upgradable`.
    /// For now `Outdated` is reserved for a future `check_software_updates`
    /// command; the current pipeline only produces `Installed` / `NotInstalled`
    /// / `Unknown` (degraded when version detection fails).
    fn compute_status(
        &self,
        is_installed: bool,
        version: Option<&str>,
        _latest_version: Option<&str>,
    ) -> SoftwareStatus {
        if !is_installed {
            return SoftwareStatus::NotInstalled;
        }
        match version {
            Some(v) if !v.is_empty() => SoftwareStatus::Installed,
            _ => SoftwareStatus::Unknown,
        }
    }

    /// Try to locate the on-disk path of the software's executable.
    /// Strategy:
    ///   1. `which <key>` to find the binary in PATH.
    ///   2. Fallback to known standard locations per key.
    ///   3. Otherwise None — never blocks the scan.
    pub fn detect_install_path(&self, key: &str) -> Option<String> {
        if let Ok(output) = std::process::Command::new("which").arg(key).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
        fallback_install_path(key)
    }

    // ============ Version Detection Methods ============

    fn detect_vscode_version(&self) -> Option<String> {
        std::process::Command::new("code")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().next().map(|s| s.to_string())
            })
    }

    fn detect_iterm2_version(&self, config_path: &PathBuf) -> Option<String> {
        let prefs = config_path.join("PrefsAsJson.plist");
        if prefs.exists() {
            if let Ok(content) = std::fs::read_to_string(&prefs) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("BundleVersion") {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    fn detect_oh_my_posh_version(&self, config_path: &PathBuf) -> Option<String> {
        if config_path.exists() {
            std::process::Command::new("oh-my-posh")
                .arg("--version")
                .output()
                .ok()
                .and_then(|output| {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Some(stdout.trim().to_string())
                })
        } else {
            None
        }
    }

    fn detect_nvm_version(&self) -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            // Windows: 先尝试获取当前 Node.js 版本（nvm use 设置的版本）
            // nvm-windows 的 "nvm version" 返回的是 nvm-windows 自身版本，不是 Node 版本
            // 所以我们直接用 "node -v" 来获取当前活跃的 Node.js 版本
            let node_output = std::process::Command::new("node")
                .arg("-v")
                .output()
                .ok();

            if let Some(output) = node_output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let version = stdout.trim().to_string();
                    if !version.is_empty()
                        && version.starts_with('v')
                        && version.len() > 1
                        && version[1..].chars().next().map_or(false, |c| c.is_ascii_digit())
                    {
                        return Some(version);
                    }
                }
            }

            // 回退：尝试通过 nvm current 获取
            let nvm_output = std::process::Command::new("nvm")
                .arg("current")
                .output()
                .ok();

            nvm_output.and_then(|output| {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let version = stdout.trim().to_string();
                    // nvm-windows current 输出可能带 "v" 前缀或不带
                    let version_clean = version.trim_start_matches('v').trim_start_matches('V');
                    if !version_clean.is_empty()
                        && version_clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
                    {
                        Some(format!("v{}", version_clean))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            // macOS/Linux: nvm 使用 bash 脚本
            let nvm_sh = dirs::home_dir()?.join(".nvm").join("nvm.sh");
            if nvm_sh.exists() {
                std::process::Command::new("bash")
                    .args(["-c", "source ~/.nvm/nvm.sh && nvm --version"])
                    .output()
                    .ok()
                    .and_then(|output| {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let version = stdout.trim();
                        if version.starts_with('v') {
                            Some(version.to_string())
                        } else {
                            None
                        }
                    })
            } else {
                None
            }
        }
    }

    fn detect_pyenv_version(&self) -> Option<String> {
        // Windows 上通过 PowerShell 执行，以支持 pyenv.cmd 等批处理文件
        // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
        let output = if cfg!(target_os = "windows") {
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let creation_flags = flags;

            std::process::Command::new("powershell.exe")
                .args(["-Command", "& { pyenv --version }"])
                .creation_flags(creation_flags)
                .output()
                .ok()
        } else {
            std::process::Command::new("pyenv")
                .arg("--version")
                .output()
                .ok()
        };

        output.and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.split_whitespace().nth(1).map(|s| s.to_string())
        })
    }

    fn detect_jenv_version(&self) -> Option<String> {
        // Windows 上通过 PowerShell 执行，以支持 jenv.cmd 等批处理文件
        // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
        let output = if cfg!(target_os = "windows") {
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let creation_flags = flags;

            std::process::Command::new("powershell.exe")
                .args(["-Command", "& { jenv --version }"])
                .creation_flags(creation_flags)
                .output()
                .ok()
        } else {
            std::process::Command::new("jenv")
                .arg("--version")
                .output()
                .ok()
        };

        output.and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.split_whitespace().nth(1).map(|s| s.to_string())
        })
    }

    fn detect_homebrew_version(&self) -> Option<String> {
        std::process::Command::new("brew")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .map(|s| s.to_string())
            })
    }

    fn detect_scoop_version(&self) -> Option<String> {
        // 首先检查 Scoop 安装目录是否存在
        if let Some(home) = dirs::home_dir() {
            let scoop_dir = home.join("scoop");
            let scoop_apps_dir = scoop_dir.join("apps").join("scoop").join("current");

            // 如果 Scoop 目录存在，尝试从 scoop.ps1 文件读取版本
            if scoop_apps_dir.exists() {
                let scoop_ps1 = scoop_apps_dir.join("bin").join("scoop.ps1");
                if scoop_ps1.exists() {
                    if let Ok(content) = std::fs::read_to_string(&scoop_ps1) {
                        // 查找版本信息
                        for line in content.lines() {
                            if line.contains("$version") || line.contains("version") {
                                // 提取版本号
                                if let Some(pos) = line.find('=') {
                                    let version_part = &line[pos + 1..].trim();
                                    // 去除引号
                                    let version =
                                        version_part.trim_matches(|c| c == '\'' || c == '"');
                                    if !version.is_empty()
                                        && version
                                            .chars()
                                            .next()
                                            .map(|c| c.is_ascii_digit())
                                            .unwrap_or(false)
                                    {
                                        return Some(version.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 尝试使用 scoop --version 命令
        if let Ok(output) = std::process::Command::new("scoop")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Scoop 输出格式:
                // Current Scoop version:
                // b588a06e chore(release): Bump to version 0.5.3 (resync) (#6436)
                //
                // 我们需要提取 "0.5.3" 这样的版本号
                for line in stdout.lines() {
                    if line.contains("version") && !line.starts_with("Current") {
                        // 查找 "version X.X.X" 格式
                        if let Some(pos) = line.find("version ") {
                            let after_version = &line[pos + 8..];
                            // 跳过前导空格
                            let after_version = after_version.trim_start();
                            // 提取版本号（直到遇到空格或括号）
                            let version: String = after_version
                                .chars()
                                .take_while(|c| c.is_ascii_digit() || *c == '.')
                                .collect();
                            if !version.is_empty() {
                                return Some(version);
                            }
                        }
                    }
                }
            }
        }

        // 尝试从 scoop 配置文件读取版本
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".config").join("scoop").join("config.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(version) = json.get("last_update") {
                            return version.as_str().map(|s| s.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    fn detect_sudo_version(&self) -> Option<String> {
        // Windows 上通过 PowerShell 执行，以支持 gsudo.cmd 等批处理文件
        // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
        let output = if cfg!(target_os = "windows") {
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let creation_flags = flags;

            std::process::Command::new("powershell.exe")
                .args(["-Command", "& { gsudo --version }"])
                .creation_flags(creation_flags)
                .output()
                .ok()
        } else {
            std::process::Command::new("sudo")
                .arg("--version")
                .output()
                .ok()
        };

        output.and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // gsudo 输出格式: "gsudo version: 2.6.1"
            for line in stdout.lines() {
                if line.contains("version") || line.contains("Version") {
                    if let Some(pos) = line.find(':') {
                        let version = line[pos + 1..].trim();
                        if !version.is_empty() {
                            return Some(version.to_string());
                        }
                    }
                    // 尝试提取数字版本号
                    let version: String = line
                        .chars()
                        .skip_while(|c| !c.is_ascii_digit())
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !version.is_empty() {
                        return Some(version);
                    }
                }
            }
            None
        })
    }

    fn detect_git_version(&self) -> Option<String> {
        // Windows 上通过 PowerShell 执行
        // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
        let output = if cfg!(target_os = "windows") {
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let creation_flags = flags;

            std::process::Command::new("powershell.exe")
                .args(["-Command", "& { git --version }"])
                .creation_flags(creation_flags)
                .output()
                .ok()
        } else {
            std::process::Command::new("git")
                .arg("--version")
                .output()
                .ok()
        };

        output.and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // git 输出格式: "git version 2.54.0.windows.1"
            stdout
                .split_whitespace()
                .nth(2)
                .map(|s| s.to_string())
        })
    }

    fn detect_7zip_version(&self) -> Option<String> {
        // Windows 上通过 PowerShell 执行
        // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
        let output = if cfg!(target_os = "windows") {
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let creation_flags = flags;

            std::process::Command::new("powershell.exe")
                .args(["-Command", "& { 7z i }"])
                .creation_flags(creation_flags)
                .output()
                .ok()
        } else {
            std::process::Command::new("7z")
                .arg("i")
                .output()
                .ok()
        };

        output.and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // 7-Zip 输出格式: "7-Zip 26.01 (x64) : ..."
            for line in stdout.lines() {
                if line.starts_with("7-Zip") {
                    let version: String = line
                        .chars()
                        .skip(6) // Skip "7-Zip "
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !version.is_empty() {
                        return Some(version);
                    }
                }
            }
            None
        })
    }

    fn detect_switchhosts_version(&self) -> Option<String> {
        // SwitchHosts 通过 scoop 安装，从 manifest.json 读取版本
        if let Some(home) = dirs::home_dir() {
            let manifest_path = home.join("scoop").join("apps").join("switchhosts").join("current").join("manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(version) = json.get("version") {
                            return version.as_str().map(|s| s.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_colortool_version(&self) -> Option<String> {
        // Windows 上通过 PowerShell 执行
        // 添加 CREATE_NO_WINDOW 标志防止控制台弹窗
        let output = if cfg!(target_os = "windows") {
            let flags: u32 = 0x08000000; // CREATE_NO_WINDOW
            let creation_flags = flags;

            std::process::Command::new("powershell.exe")
                .args(["-Command", "& { colortool --version }"])
                .creation_flags(creation_flags)
                .output()
                .ok()
        } else {
            std::process::Command::new("colortool")
                .arg("--version")
                .output()
                .ok()
        };

        output.and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout.trim().to_string();
            if !version.is_empty() && version.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                Some(version)
            } else {
                None
            }
        })
    }

    fn detect_ffmpeg_version(&self) -> Option<String> {
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(3))
                    .map(|s| s.to_string())
            })
    }

    fn detect_postman_version(&self, config_path: &PathBuf) -> Option<String> {
        let prefs = config_path.join("(window-state).json");
        if prefs.exists() {
            if let Ok(content) = std::fs::read_to_string(&prefs) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("version") {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    fn detect_obsidian_version(&self, config_path: &PathBuf) -> Option<String> {
        let config = config_path.join("obsidian.json");
        if config.exists() {
            if let Ok(content) = std::fs::read_to_string(&config) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(vals) = json.get("values").and_then(|v| v.as_array()) {
                        for val in vals {
                            if let Some(id) = val.get("id").and_then(|i| i.as_str()) {
                                if id.starts_with("obsidian-") {
                                    return Some(
                                        id.strip_prefix("obsidian-").unwrap_or(id).to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_snipaste_version(&self, config_path: &PathBuf) -> Option<String> {
        let conf = config_path.join("conf.ini");
        if conf.exists() {
            if let Ok(content) = std::fs::read_to_string(&conf) {
                for line in content.lines() {
                    if line.starts_with("version=") {
                        return Some(line.split('=').nth(1).unwrap_or("").trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn detect_generic_version(&self, key: &str) -> Option<String> {
        // Generic detection using which command
        std::process::Command::new("which")
            .arg(key)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|_| {
                std::process::Command::new(key)
                    .arg("--version")
                    .output()
                    .ok()
                    .and_then(|output| {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let combined = if stdout.is_empty() {
                            stderr.as_ref()
                        } else {
                            stdout.as_ref()
                        };
                        combined
                            .lines()
                            .next()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty() && s.len() < 50)
                    })
            })
    }

    fn detect_cursor_version(&self, config_path: &PathBuf) -> Option<String> {
        let settings_path = config_path.join("settings.json");
        if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("cursor.version").or(json.get("version")) {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }

        let product_info = config_path.join("product.json");
        if product_info.exists() {
            if let Ok(content) = std::fs::read_to_string(&product_info) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("version") {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }

        None
    }

    fn detect_windsurf_version(&self, config_path: &PathBuf) -> Option<String> {
        let settings_path = config_path.join("settings.json");
        if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("windsurf.version") {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    fn detect_claude_version(&self, config_path: &PathBuf) -> Option<String> {
        let local_storage = config_path.join("localStorage.json");
        if local_storage.exists() {
            if let Ok(content) = std::fs::read_to_string(&local_storage) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(data) = json.get("desktop") {
                        if let Some(version) = data.get("lastKnownVersion") {
                            return version.as_str().map(|s| s.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_continue_version(&self, config_path: &PathBuf) -> Option<String> {
        let config_file = config_path.join("config.json");
        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("version") {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    fn detect_cody_version(&self, config_path: &PathBuf) -> Option<String> {
        let settings_path = config_path.join("settings.json");
        if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("cody.version") {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn verify_installation(&self, key: &str) -> bool {
        if let Some(path) = self.get_config_path(key) {
            path.exists() && path.is_dir()
        } else {
            false
        }
    }

    pub fn get_software_keys(&self) -> Vec<String> {
        self.get_supported_software()
            .into_iter()
            .map(|c| c.key)
            .collect()
    }
}

impl Default for SoftwareScanner {
    fn default() -> Self {
        Self::new()
    }
}

fn uuid_from_key(key: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{:016x}-0000-0000-0000-{:012x}", hash, hash >> 4)
}

/// Best-effort fallback for software whose binary is in a well-known location
/// but isn't on PATH. Returning `None` is always safe — it just means the UI
/// will not show an install path, not that detection failed.
fn fallback_install_path(key: &str) -> Option<String> {
    let p = match key {
        "ffmpeg" => "/usr/local/bin/ffmpeg",
        "vscode" | "code" => "/usr/local/bin/code",
        "homebrew" | "brew" => "/opt/homebrew/bin/brew",
        "nvm" => {
            return dirs::home_dir().map(|h| h.join(".nvm").join("nvm.sh").display().to_string())
        }
        "pyenv" => return dirs::home_dir().map(|h| h.join(".pyenv").display().to_string()),
        "jenv" => return dirs::home_dir().map(|h| h.join(".jenv").display().to_string()),
        "oh-my-posh" => {
            return dirs::home_dir().map(|h| h.join(".oh-my-posh.omp.json").display().to_string())
        }
        "node" => "/usr/local/bin/node",
        "python" | "python3" => "/usr/local/bin/python3",
        "go" => "/usr/local/go/bin/go",
        "java" => "/usr/bin/java",
        "rustc" | "cargo" => {
            return dirs::home_dir().map(|h| h.join(".cargo").join("bin").display().to_string())
        }
        _ => return None,
    };
    Some(p.to_string())
}

/// Compare two semver-ish version strings. Returns positive if `a > b`,
/// negative if `a < b`, zero if equal. Non-numeric segments are ignored,
/// missing components treated as 0.
fn version_compare(a: &str, b: &str) -> i32 {
    let parse = |s: &str| -> Vec<u32> {
        s.trim()
            .trim_start_matches('v')
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };
    let av = parse(a);
    let bv = parse(b);
    let n = av.len().max(bv.len());
    for i in 0..n {
        let x = *av.get(i).unwrap_or(&0);
        let y = *bv.get(i).unwrap_or(&0);
        if x != y {
            return (x as i32) - (y as i32);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_software() {
        let scanner = SoftwareScanner::new();
        let software_list = scanner.detect_software();
        assert!(!software_list.is_empty());
    }

    #[test]
    fn test_get_supported_software() {
        let scanner = SoftwareScanner::new();
        let configs = scanner.get_supported_software();
        // The catalog spans 5 tiers plus a legacy AI tools block — assert
        // non-empty and at least one per tier to guard against regressions
        // without coupling to a hard-coded total.
        assert!(!configs.is_empty());
        let tiers: std::collections::HashSet<u8> = configs.iter().map(|c| c.tier).collect();
        assert!(
            tiers.len() >= 5,
            "expected entries across all 5 tiers, got {:?}",
            tiers
        );
    }

    #[test]
    fn test_compute_status_not_installed() {
        let scanner = SoftwareScanner::new();
        assert_eq!(
            scanner.compute_status(false, None, None),
            SoftwareStatus::NotInstalled
        );
        assert_eq!(
            scanner.compute_status(false, Some("1.0"), None),
            SoftwareStatus::NotInstalled
        );
    }

    #[test]
    fn test_compute_status_installed() {
        let scanner = SoftwareScanner::new();
        assert_eq!(
            scanner.compute_status(true, Some("4.3.0"), None),
            SoftwareStatus::Installed
        );
    }

    #[test]
    fn test_compute_status_unknown_degrades_safely() {
        let scanner = SoftwareScanner::new();
        // Installed but no version command output -> Unknown, not panic.
        assert_eq!(
            scanner.compute_status(true, None, None),
            SoftwareStatus::Unknown
        );
        assert_eq!(
            scanner.compute_status(true, Some(""), None),
            SoftwareStatus::Unknown
        );
    }

    #[test]
    fn test_version_compare_basic() {
        assert!(version_compare("2.45.0", "2.40.0") > 0);
        assert!(version_compare("2.40.0", "2.45.0") < 0);
        assert_eq!(version_compare("2.40.0", "2.40.0"), 0);
        assert!(version_compare("v1.0.1", "1.0.0") > 0);
    }

    #[test]
    fn test_fallback_install_path_returns_known_paths() {
        assert!(fallback_install_path("nvm").is_some());
        assert!(fallback_install_path("unknown-xyz").is_none());
    }

    #[test]
    fn test_detect_install_path_does_not_panic() {
        let scanner = SoftwareScanner::new();
        // Whatever happens, the function must not panic for any key.
        let _ = scanner.detect_install_path("git");
        let _ = scanner.detect_install_path("definitely-not-installed-xyz");
    }
}
