use crate::models::{Software, SoftwareStatus};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

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
            last_checked: Some(chrono_lite_now()),
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
                key: "chocolatey".to_string(),
                name: "Chocolatey".to_string(),
                tier: 1,
                platform: "Windows".to_string(),
                default_config_paths: vec![
                    PathBuf::from("C:/ProgramData/chocolatey"),
                ],
                website_url: Some("https://chocolatey.org".to_string()),
            },
            SoftwareConfig {
                key: "scoop".to_string(),
                name: "Scoop".to_string(),
                tier: 1,
                platform: "Windows".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".local").join("share").join("scoop"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://scoop.sh".to_string()),
            },
            SoftwareConfig {
                key: "ssh".to_string(),
                name: "SSH Config".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".ssh"))
                        .unwrap_or_default(),
                ],
                website_url: None,
            },
            SoftwareConfig {
                key: "windows-terminal".to_string(),
                name: "Windows Terminal".to_string(),
                tier: 1,
                platform: "Windows".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("AppData").join("Local").join("Packages").join("Microsoft.WindowsTerminal"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/microsoft/terminal".to_string()),
            },
            SoftwareConfig {
                key: "iterm2".to_string(),
                name: "iTerm2".to_string(),
                tier: 1,
                platform: "macOS".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("Library").join("Application Support").join("iTerm2"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://iterm2.com".to_string()),
            },
            SoftwareConfig {
                key: "oh-my-posh".to_string(),
                name: "Oh My Posh".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".oh-my-posh.omp.json"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://ohmyposh.dev".to_string()),
            },
            SoftwareConfig {
                key: "vscode".to_string(),
                name: "VS Code".to_string(),
                tier: 1,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".config").join("Code").join("User"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://code.visualstudio.com".to_string()),
            },

            // Tier 2: Language Version Managers (P1)
            SoftwareConfig {
                key: "nvm".to_string(),
                name: "nvm (Node)".to_string(),
                tier: 2,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".nvm"))
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
                    dirs::home_dir()
                        .map(|p| p.join(".pyenv"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/pyenv/pyenv".to_string()),
            },
            SoftwareConfig {
                key: "goenv".to_string(),
                name: "goenv (Go)".to_string(),
                tier: 2,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".goenv"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/syndbg/goenv".to_string()),
            },
            SoftwareConfig {
                key: "jenv".to_string(),
                name: "jenv (Java)".to_string(),
                tier: 2,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".jenv"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://github.com/jenv/jenv".to_string()),
            },
            SoftwareConfig {
                key: "asdf".to_string(),
                name: "asdf".to_string(),
                tier: 2,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".asdf"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://asdf-vm.com".to_string()),
            },

            // Tier 3: Runtime & Containers (P0)
            SoftwareConfig {
                key: "docker".to_string(),
                name: "Docker".to_string(),
                tier: 3,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".docker"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://www.docker.com".to_string()),
            },
            SoftwareConfig {
                key: "docker-compose".to_string(),
                name: "Docker Compose".to_string(),
                tier: 3,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".docker").join("cli-plugins"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://docs.docker.com/compose".to_string()),
            },
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
                key: "apifox".to_string(),
                name: "Apifox".to_string(),
                tier: 4,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("Library").join("Application Support").join("Apifox"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://apifox.com".to_string()),
            },
            SoftwareConfig {
                key: "postman".to_string(),
                name: "Postman".to_string(),
                tier: 4,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("Library").join("Application Support").join("Postman"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://www.postman.com".to_string()),
            },
            SoftwareConfig {
                key: "charles".to_string(),
                name: "Charles Proxy".to_string(),
                tier: 4,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("Library").join("Application Support").join("Charles"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://www.charlesproxy.com".to_string()),
            },
            SoftwareConfig {
                key: "cyberduck".to_string(),
                name: "Cyberduck (SFTP)".to_string(),
                tier: 4,
                platform: "macOS".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("Library").join("Application Support").join("Cyberduck"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://cyberduck.io".to_string()),
            },
            SoftwareConfig {
                key: "filezilla".to_string(),
                name: "FileZilla (SFTP)".to_string(),
                tier: 4,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".config").join("filezilla"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://filezilla-project.org".to_string()),
            },

            // Tier 5: Productivity Tools (P2)
            SoftwareConfig {
                key: "snipaste".to_string(),
                name: "Snipaste".to_string(),
                tier: 5,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("Library").join("Application Support").join("Snipaste"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://www.snipaste.com".to_string()),
            },
            SoftwareConfig {
                key: "obsidian".to_string(),
                name: "Obsidian".to_string(),
                tier: 5,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join("Library").join("Application Support").join("obsidian"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://obsidian.md".to_string()),
            },
            SoftwareConfig {
                key: "excalidraw".to_string(),
                name: "Excalidraw".to_string(),
                tier: 5,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".excalidraw"))
                        .unwrap_or_default(),
                ],
                website_url: Some("https://excalidraw.com".to_string()),
            },

            // Legacy: AI Tools (from original code)
            SoftwareConfig {
                key: "cursor".to_string(),
                name: "Cursor".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".cursor"))
                        .unwrap_or_default(),
                ],
                website_url: None,
            },
            SoftwareConfig {
                key: "windsurf".to_string(),
                name: "Windsurf".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".windsurf"))
                        .unwrap_or_default(),
                ],
                website_url: None,
            },
            SoftwareConfig {
                key: "claude-desktop".to_string(),
                name: "Claude Desktop".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".claude"))
                        .unwrap_or_default(),
                ],
                website_url: None,
            },
            SoftwareConfig {
                key: "continue".to_string(),
                name: "Continue".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".continue"))
                        .unwrap_or_default(),
                ],
                website_url: None,
            },
            SoftwareConfig {
                key: "cody".to_string(),
                name: "Cody".to_string(),
                tier: 0,
                platform: "Cross-platform".to_string(),
                default_config_paths: vec![
                    dirs::home_dir()
                        .map(|p| p.join(".cody"))
                        .unwrap_or_default(),
                ],
                website_url: None,
            },
        ]);

        configs
    }

    /// Detect all software - parallel version using rayon
    pub fn detect_software_parallel(&self) -> Vec<Software> {
        let configs = self.get_supported_software();

        use rayon::prelude::*;

        configs.par_iter().map(|config| {
            self.detect_single_sync(config)
        }).collect()
    }

    /// Synchronous single software detection (for parallel execution)
    fn detect_single_sync(&self, config: &SoftwareConfig) -> Software {
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
            config_path = config.default_config_paths.first().cloned().unwrap_or_default();
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
            last_checked: Some(chrono_lite_now()),
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
                config_path = config.default_config_paths.first().cloned().unwrap_or_default();
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

            let last_checked = chrono_lite_now();
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
            config_path = config.default_config_paths.first().cloned().unwrap_or_default();
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
            last_checked: Some(chrono_lite_now()),
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
            "docker" => self.detect_docker_version(),
            "vscode" => self.detect_vscode_version(),
            "iterm2" => self.detect_iterm2_version(config_path),
            "oh-my-posh" => self.detect_oh_my_posh_version(config_path),
            // Tier 2: Language Managers
            "nvm" => self.detect_nvm_version(),
            "pyenv" => self.detect_pyenv_version(),
            "homebrew" => self.detect_homebrew_version(),
            // Tier 3: Runtime
            "docker-compose" => self.detect_docker_compose_version(),
            "ffmpeg" => self.detect_ffmpeg_version(),
            // Tier 4: Debug Tools
            "apifox" => self.detect_apifox_version(config_path),
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
            .spawn(move || {
                SoftwareScanner::new().detect_version(&key, &config_path)
            })
            .ok()?;
        // Wait up to 1000ms for the version detection thread.
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(1000);
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
        if let Ok(output) = std::process::Command::new("which")
            .arg(key)
            .output()
        {
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

    fn detect_docker_version(&self) -> Option<String> {
        std::process::Command::new("docker")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.split_whitespace().nth(2).map(|s| s.trim_matches(',').to_string())
            })
    }

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

    fn detect_pyenv_version(&self) -> Option<String> {
        std::process::Command::new("pyenv")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
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
                stdout.lines().next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .map(|s| s.to_string())
            })
    }

    fn detect_docker_compose_version(&self) -> Option<String> {
        std::process::Command::new("docker")
            .args(["compose", "version", "--format", "json"])
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(stdout.as_ref()) {
                        return json.get("Version").and_then(|v| v.as_str()).map(|s| s.to_string());
                    }
                }
                // Fallback to docker-compose v1
                std::process::Command::new("docker-compose")
                    .arg("--version")
                    .output()
                    .ok()
                    .and_then(|out| {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        stdout.split_whitespace().nth(2).map(|s| s.to_string())
                    })
            })
    }

    fn detect_ffmpeg_version(&self) -> Option<String> {
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.lines().next()
                    .and_then(|line| line.split_whitespace().nth(3))
                    .map(|s| s.to_string())
            })
    }

    fn detect_apifox_version(&self, config_path: &PathBuf) -> Option<String> {
        let settings = config_path.join("settings.json");
        if settings.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(version) = json.get("appVersion") {
                        return version.as_str().map(|s| s.to_string());
                    }
                }
            }
        }
        None
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
                                    return Some(id.strip_prefix("obsidian-").unwrap_or(id).to_string());
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
                        let combined = if stdout.is_empty() { stderr.as_ref() } else { stdout.as_ref() };
                        combined.lines().next()
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

fn chrono_lite_now() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
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

/// Best-effort fallback for software whose binary is in a well-known location
/// but isn't on PATH. Returning `None` is always safe — it just means the UI
/// will not show an install path, not that detection failed.
fn fallback_install_path(key: &str) -> Option<String> {
    let p = match key {
        "docker" => "/usr/local/bin/docker",
        "docker-compose" => "/usr/local/bin/docker-compose",
        "ffmpeg" => "/usr/local/bin/ffmpeg",
        "vscode" | "code" => "/usr/local/bin/code",
        "homebrew" | "brew" => "/opt/homebrew/bin/brew",
        "nvm" => return dirs::home_dir().map(|h| h.join(".nvm").join("nvm.sh").display().to_string()),
        "pyenv" => return dirs::home_dir().map(|h| h.join(".pyenv").display().to_string()),
        "goenv" => return dirs::home_dir().map(|h| h.join(".goenv").display().to_string()),
        "jenv" => return dirs::home_dir().map(|h| h.join(".jenv").display().to_string()),
        "asdf" => return dirs::home_dir().map(|h| h.join(".asdf").display().to_string()),
        "oh-my-posh" => return dirs::home_dir().map(|h| h.join(".oh-my-posh.omp.json").display().to_string()),
        "node" => "/usr/local/bin/node",
        "python" | "python3" => "/usr/local/bin/python3",
        "go" => "/usr/local/go/bin/go",
        "java" => "/usr/bin/java",
        "rustc" | "cargo" => return dirs::home_dir().map(|h| h.join(".cargo").join("bin").display().to_string()),
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
        assert!(tiers.len() >= 5, "expected entries across all 5 tiers, got {:?}", tiers);
    }

    #[test]
    fn test_compute_status_not_installed() {
        let scanner = SoftwareScanner::new();
        assert_eq!(scanner.compute_status(false, None, None), SoftwareStatus::NotInstalled);
        assert_eq!(scanner.compute_status(false, Some("1.0"), None), SoftwareStatus::NotInstalled);
    }

    #[test]
    fn test_compute_status_installed() {
        let scanner = SoftwareScanner::new();
        assert_eq!(scanner.compute_status(true, Some("4.3.0"), None), SoftwareStatus::Installed);
    }

    #[test]
    fn test_compute_status_unknown_degrades_safely() {
        let scanner = SoftwareScanner::new();
        // Installed but no version command output -> Unknown, not panic.
        assert_eq!(scanner.compute_status(true, None, None), SoftwareStatus::Unknown);
        assert_eq!(scanner.compute_status(true, Some(""), None), SoftwareStatus::Unknown);
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
