use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Software '{0}' is not supported")]
    NotSupported(String),
    #[error("Installation failed: {0}")]
    InstallFailed(String),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Software already installed: {0}")]
    AlreadyInstalled(String),
}

pub type InstallResult<T> = Result<T, InstallError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResponse {
    pub success: bool,
    pub message: String,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallResponse {
    pub success: bool,
    pub message: String,
    pub needs_manual: bool,
    pub manual_commands: Vec<String>,
}

pub struct SoftwareInstaller;

impl SoftwareInstaller {
    pub fn new() -> Self {
        Self
    }

    pub fn install(&self, software_key: &str) -> InstallResult<InstallResponse> {
        log::info!("Installing software: {}", software_key);

        match software_key {
            // Tier 1: Foundation
            "homebrew" => self.install_homebrew(),
            "vscode" => self.install_vscode(),
            "oh-my-posh" => self.install_oh_my_posh(),
            "chocolatey" => self.install_chocolatey(),
            "scoop" => self.install_scoop(),

            // Tier 2: Language Managers
            "nvm" => self.install_nvm(),
            "pyenv" => self.install_pyenv(),

            // Tier 3: Runtime
            "docker" => self.install_docker(),
            "ffmpeg" => self.install_ffmpeg(),
            "docker-compose" => self.install_docker_compose(),

            // Tier 4: Debug Tools
            "postman" => self.install_postman(),
            "filezilla" => self.install_filezilla(),

            // Tier 5: Productivity
            "obsidian" => self.install_obsidian(),
            "snipaste" => self.install_snipaste(),

            // Not directly installable
            "ssh" => Err(InstallError::NotSupported(
                "SSH is system-installed. Please use your package manager.".to_string()
            )),
            "iterm2" => Err(InstallError::NotSupported(
                "iTerm2 requires macOS manual installation from iterm2.com".to_string()
            )),
            "windows-terminal" => Err(InstallError::NotSupported(
                "Windows Terminal should be installed via Microsoft Store".to_string()
            )),
            "goenv" | "jenv" | "asdf" => Err(InstallError::NotSupported(
                "Version managers require manual installation. See project documentation.".to_string()
            )),
            "apifox" => Err(InstallError::NotSupported(
                "Apifox requires manual installation from apifox.com".to_string()
            )),
            "charles" => Err(InstallError::NotSupported(
                "Charles Proxy requires manual installation".to_string()
            )),
            "cyberduck" => Err(InstallError::NotSupported(
                "Cyberduck requires manual installation".to_string()
            )),
            "excalidraw" => Err(InstallError::NotSupported(
                "Excalidraw is a web app at excalidraw.com".to_string()
            )),

            // AI Tools
            "cursor" | "windsurf" | "claude-desktop" | "continue" | "cody" => {
                Err(InstallError::NotSupported(format!(
                    "{} should be installed from the official website",
                    software_key.replace('-', " ").split_whitespace()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )))
            }

            _ => Err(InstallError::NotSupported(software_key.to_string())),
        }
    }

    pub fn uninstall(&self, software_key: &str) -> InstallResult<UninstallResponse> {
        log::info!("Uninstalling software: {}", software_key);

        match software_key {
            "vscode" => self.uninstall_vscode(),
            "oh-my-posh" => self.uninstall_oh_my_posh(),
            "nvm" => self.uninstall_nvm(),
            "pyenv" => self.uninstall_pyenv(),
            "docker" => self.uninstall_docker(),
            "ffmpeg" => self.uninstall_ffmpeg(),
            "obsidian" => self.uninstall_obsidian(),
            "snipaste" => self.uninstall_snipaste(),
            "homebrew" => Ok(UninstallResponse {
                success: true,
                message: "Homebrew should be uninstalled manually".to_string(),
                needs_manual: true,
                manual_commands: vec![
                    "/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/uninstall.sh)\"".to_string(),
                ],
            }),
            _ => self.uninstall_generic(software_key),
        }
    }

    // ============ Installation Methods ============

    fn install_homebrew(&self) -> InstallResult<InstallResponse> {
        // Check if brew is already installed
        if Command::new("brew")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(InstallResponse {
                success: true,
                message: "Homebrew is already installed".to_string(),
                installed_version: None,
            });
        }

        // Try to install using the official script
        // Note: This requires sudo access on macOS
        let result = Command::new("/bin/bash")
            .args(["-c", "NONINTERACTIVE=1 $(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"])
            .output();

        match result {
            Ok(output) if output.status.success() => Ok(InstallResponse {
                success: true,
                message: "Homebrew installed successfully".to_string(),
                installed_version: Some("4.x".to_string()),
            }),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                // If it fails due to permissions, provide helpful message
                if stderr.contains("sudo") || stderr.contains("Permission denied") || stderr.contains("Need sudo access") {
                    Err(InstallError::InstallFailed(
                        "Homebrew requires administrator privileges. Please run the following command in Terminal:\n\n/bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n\nAfter installation, click 'Detect' to refresh.".to_string()
                    ))
                } else {
                    Err(InstallError::InstallFailed(stderr))
                }
            }
            Err(e) => Err(InstallError::CommandFailed(e.to_string())),
        }
    }

    fn install_vscode(&self) -> InstallResult<InstallResponse> {
        let result = self.run_and_capture("brew", &["install", "--cask", "visual-studio-code"])
            .or_else(|_| self.run_and_capture("winget", &["install", "--id", "Microsoft.VisualStudioCode", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"]));

        Ok(match result {
            Ok(version) => InstallResponse {
                success: true,
                message: "VS Code installed successfully".to_string(),
                installed_version: version,
            },
            Err(e) => return Err(e),
        })
    }

    fn install_oh_my_posh(&self) -> InstallResult<InstallResponse> {
        self.run_bash_script("https://ohmyposh.dev/install.sh")
            .map(|_| InstallResponse {
                success: true,
                message: "Oh My Posh installed successfully".to_string(),
                installed_version: Some("latest".to_string()),
            })
    }

    fn install_nvm(&self) -> InstallResult<InstallResponse> {
        self.run_bash_script("https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh")
            .map(|_| InstallResponse {
                success: true,
                message: "nvm installed. Restart shell or run: source ~/.bashrc".to_string(),
                installed_version: Some("v0.39.7".to_string()),
            })
    }

    fn install_pyenv(&self) -> InstallResult<InstallResponse> {
        let result = self.run_and_capture("brew", &["install", "pyenv"]);

        if result.is_ok() {
            return Ok(InstallResponse {
                success: true,
                message: "pyenv installed successfully".to_string(),
                installed_version: result.ok().flatten(),
            });
        }

        // Try bash script
        self.run_bash_script("https://pyenv.run")
            .map(|_| InstallResponse {
                success: true,
                message: "pyenv installed successfully".to_string(),
                installed_version: Some("latest".to_string()),
            })
    }

    fn install_docker(&self) -> InstallResult<InstallResponse> {
        #[cfg(target_os = "linux")]
        {
            self.run_bash_script("https://get.docker.com")
                .map(|_| InstallResponse {
                    success: true,
                    message: "Docker installed successfully".to_string(),
                    installed_version: Some("latest".to_string()),
                })
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(InstallError::NotSupported(
                "Docker Desktop should be installed from docker.com on macOS/Windows".to_string()
            ))
        }
    }

    fn install_ffmpeg(&self) -> InstallResult<InstallResponse> {
        let result = self.run_and_capture("brew", &["install", "ffmpeg"])
            .or_else(|_| self.run_and_capture("sudo", &["apt-get", "install", "-y", "ffmpeg"]))
            .or_else(|_| self.run_and_capture("winget", &["install", "--id", "Gyan.FFmpeg", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"]));

        Ok(match result {
            Ok(version) => InstallResponse {
                success: true,
                message: "FFmpeg installed successfully".to_string(),
                installed_version: version,
            },
            Err(e) => return Err(e),
        })
    }

    fn install_chocolatey(&self) -> InstallResult<InstallResponse> {
        #[cfg(target_os = "windows")]
        {
            let script = r#"Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"#;
            self.run_powershell(script)
                .map(|_| InstallResponse {
                    success: true,
                    message: "Chocolatey installed successfully".to_string(),
                    installed_version: Some("2.x".to_string()),
                })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(InstallError::NotSupported("Chocolatey is for Windows only".to_string()))
        }
    }

    fn install_scoop(&self) -> InstallResult<InstallResponse> {
        #[cfg(target_os = "windows")]
        {
            self.run_powershell("irm get.scoop.sh | iex")
                .map(|_| InstallResponse {
                    success: true,
                    message: "Scoop installed successfully".to_string(),
                    installed_version: Some("1.x".to_string()),
                })
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(InstallError::NotSupported("Scoop is for Windows only".to_string()))
        }
    }

    fn install_docker_compose(&self) -> InstallResult<InstallResponse> {
        #[cfg(target_os = "linux")]
        {
            self.run_command("sudo", &["curl", "-L", "https://github.com/docker/compose/releases/latest/download/docker-compose-linux-x86_64", "-o", "/usr/local/bin/docker-compose"])
                .and_then(|_| self.run_command("sudo", &["chmod", "+x", "/usr/local/bin/docker-compose"]))
                .map(|_| InstallResponse {
                    success: true,
                    message: "Docker Compose installed successfully".to_string(),
                    installed_version: Some("2.x".to_string()),
                })
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(InstallError::NotSupported(
                "Docker Compose is included with Docker Desktop on macOS/Windows".to_string()
            ))
        }
    }

    fn install_postman(&self) -> InstallResult<InstallResponse> {
        let result = self.run_and_capture("brew", &["install", "--cask", "postman"])
            .or_else(|_| self.run_and_capture("winget", &["install", "--id", "Postman.Postman", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"]));

        Ok(match result {
            Ok(version) => InstallResponse {
                success: true,
                message: "Postman installed successfully".to_string(),
                installed_version: version,
            },
            Err(e) => return Err(e),
        })
    }

    fn install_filezilla(&self) -> InstallResult<InstallResponse> {
        let result = self.run_and_capture("brew", &["install", "--cask", "filezilla"])
            .or_else(|_| self.run_and_capture("sudo", &["apt-get", "install", "-y", "filezilla"]))
            .or_else(|_| self.run_and_capture("winget", &["install", "--id", "FileZilla.FileZilla", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"]));

        Ok(match result {
            Ok(version) => InstallResponse {
                success: true,
                message: "FileZilla installed successfully".to_string(),
                installed_version: version,
            },
            Err(e) => return Err(e),
        })
    }

    fn install_obsidian(&self) -> InstallResult<InstallResponse> {
        let result = self.run_and_capture("brew", &["install", "--cask", "obsidian"])
            .or_else(|_| self.run_and_capture("winget", &["install", "--id", "Obsidian.Obsidian", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"]));

        Ok(match result {
            Ok(version) => InstallResponse {
                success: true,
                message: "Obsidian installed successfully".to_string(),
                installed_version: version,
            },
            Err(e) => return Err(e),
        })
    }

    fn install_snipaste(&self) -> InstallResult<InstallResponse> {
        #[cfg(target_os = "macos")]
        {
            self.run_and_capture("brew", &["install", "--cask", "snipaste"])
                .map(|version| InstallResponse {
                    success: true,
                    message: "Snipaste installed successfully".to_string(),
                    installed_version: version,
                })
        }
        #[cfg(target_os = "windows")]
        {
            self.run_and_capture("winget", &["install", "--id", "Snipaste.Snipaste", "--source", "winget", "--accept-package-agreements", "--accept-source-agreements"])
                .map(|version| InstallResponse {
                    success: true,
                    message: "Snipaste installed successfully".to_string(),
                    installed_version: version,
                })
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err(InstallError::NotSupported("Snipaste is not available for Linux".to_string()))
        }
    }

    // ============ Uninstallation Methods ============

    fn uninstall_vscode(&self) -> InstallResult<UninstallResponse> {
        let result = self.run_command("brew", &["uninstall", "--cask", "visual-studio-code"])
            .or_else(|_| self.run_command("sudo", &["apt-get", "remove", "-y", "code"]))
            .or_else(|_| self.run_command("winget", &["uninstall", "--id", "Microsoft.VisualStudioCode"]));

        Ok(match result {
            Ok(_) => Self::ok_response("VS Code uninstalled successfully"),
            Err(e) => return Err(e),
        })
    }

    fn uninstall_oh_my_posh(&self) -> InstallResult<UninstallResponse> {
        let config_path = dirs::home_dir()
            .map(|p| p.join(".oh-my-posh"))
            .unwrap_or_default();

        if config_path.exists() {
            std::fs::remove_dir_all(&config_path)
                .map_err(|e| InstallError::CommandFailed(e.to_string()))?;
        }

        Ok(Self::ok_response("Oh My Posh uninstalled (config removed)"))
    }

    fn uninstall_nvm(&self) -> InstallResult<UninstallResponse> {
        let nvm_dir = dirs::home_dir()
            .map(|p| p.join(".nvm"))
            .unwrap_or_default();

        if nvm_dir.exists() {
            std::fs::remove_dir_all(&nvm_dir)
                .map_err(|e| InstallError::CommandFailed(e.to_string()))?;
        }

        Ok(Self::ok_response("nvm uninstalled (directory removed)"))
    }

    fn uninstall_pyenv(&self) -> InstallResult<UninstallResponse> {
        let pyenv_dir = dirs::home_dir()
            .map(|p| p.join(".pyenv"))
            .unwrap_or_default();

        if pyenv_dir.exists() {
            std::fs::remove_dir_all(&pyenv_dir)
                .map_err(|e| InstallError::CommandFailed(e.to_string()))?;
        }

        Ok(Self::ok_response("pyenv uninstalled (directory removed)"))
    }

    fn uninstall_docker(&self) -> InstallResult<UninstallResponse> {
        #[cfg(target_os = "linux")]
        {
            self.run_command("sudo", &["apt-get", "remove", "-y", "docker-ce", "docker-ce-cli", "containerd", "dockerd", "docker-compose-plugin"])
                .map(|_| Self::ok_response("Docker uninstalled successfully"))
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err(InstallError::NotSupported(
                "Docker Desktop should be uninstalled from Applications".to_string()
            ))
        }
    }

    fn uninstall_ffmpeg(&self) -> InstallResult<UninstallResponse> {
        let result = self.run_command("brew", &["uninstall", "ffmpeg"])
            .or_else(|_| self.run_command("sudo", &["apt-get", "remove", "-y", "ffmpeg"]))
            .or_else(|_| self.run_command("winget", &["uninstall", "--id", "Gyan.FFmpeg"]));

        Ok(match result {
            Ok(_) => Self::ok_response("FFmpeg uninstalled successfully"),
            Err(e) => return Err(e),
        })
    }

    fn uninstall_obsidian(&self) -> InstallResult<UninstallResponse> {
        let result = self.run_command("brew", &["uninstall", "--cask", "obsidian"])
            .or_else(|_| self.run_command("winget", &["uninstall", "--id", "Obsidian.Obsidian"]));

        Ok(match result {
            Ok(_) => Self::ok_response("Obsidian uninstalled successfully"),
            Err(e) => return Err(e),
        })
    }

    fn uninstall_snipaste(&self) -> InstallResult<UninstallResponse> {
        #[cfg(target_os = "macos")]
        {
            self.run_command("brew", &["uninstall", "--cask", "snipaste"])
                .map(|_| Self::ok_response("Snipaste uninstalled successfully"))
        }
        #[cfg(target_os = "windows")]
        {
            self.run_command("winget", &["uninstall", "--id", "Snipaste.Snipaste"])
                .map(|_| Self::ok_response("Snipaste uninstalled successfully"))
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err(InstallError::NotSupported("Snipaste is not available for this platform".to_string()))
        }
    }

    fn uninstall_generic(&self, software_key: &str) -> InstallResult<UninstallResponse> {
        log::info!("Attempting generic uninstall for: {}", software_key);

        // Try npm uninstall first - try both the key and known package name variants.
        // npm uninstall returns exit code 0 even when the package doesn't exist,
        // so we must verify by checking ALL candidates after each attempt.
        let npm_candidates = self.get_npm_package_candidates(software_key);

        // Only attempt npm uninstall if at least one candidate is actually installed.
        // This prevents falsely reporting success for tools not installed via npm
        // (e.g., cursor installed via curl-bash).
        if self.any_npm_candidate_installed(&npm_candidates) {
            for pkg in &npm_candidates {
                log::info!("Trying npm uninstall for package: {}", pkg);
                let _ = Command::new("npm")
                    .args(["uninstall", "-g", pkg])
                    .output();

                // After each attempt, check if ANY candidate is still installed.
                // If all are gone, the uninstall succeeded.
                if self.all_npm_candidates_removed(&npm_candidates) {
                    return Ok(Self::ok_response(&format!("{} uninstalled via npm", software_key)));
                }
                log::info!("Package still exists after uninstalling {}, trying next candidate", pkg);
            }
        }

        // Try brew uninstall (formula)
        if let Ok(output) = Command::new("brew")
            .args(["uninstall", software_key])
            .output()
        {
            if output.status.success() {
                return Ok(Self::ok_response(&format!("{} uninstalled via brew", software_key)));
            }
        }

        // Try brew uninstall --cask (for cask packages)
        let brew_cask_candidates = self.get_brew_cask_candidates(software_key);
        for cask in &brew_cask_candidates {
            if let Ok(output) = Command::new("brew")
                .args(["uninstall", "--cask", cask])
                .output()
            {
                if output.status.success() {
                    return Ok(Self::ok_response(&format!("{} uninstalled via brew cask", software_key)));
                }
            }
        }

        // Try pip uninstall
        if let Ok(output) = Command::new("pip")
            .args(["uninstall", "-y", software_key])
            .output()
        {
            if output.status.success() {
                return Ok(Self::ok_response(&format!("{} uninstalled via pip", software_key)));
            }
        }

        // Try removing curl-bash installed binaries (symlinks in ~/.local/bin + data dir)
        if let Some(manual_cmds) = self.uninstall_curl_bash_binary(software_key) {
            if manual_cmds.is_empty() {
                return Ok(Self::ok_response(&format!("{} uninstalled", software_key)));
            } else {
                // Partial success: user-level files removed, but system-level files need manual deletion
                return Ok(UninstallResponse {
                    success: true,
                    message: format!("{} partially uninstalled. Some files need manual removal.", software_key),
                    needs_manual: true,
                    manual_commands: manual_cmds,
                });
            }
        }

        Err(InstallError::NotSupported(format!(
            "Could not uninstall {} automatically. Please uninstall manually.",
            software_key
        )))
    }

    fn get_npm_package_candidates(&self, software_key: &str) -> Vec<String> {
        let mut candidates = vec![software_key.to_string()];

        // Add known package name mappings (tool_key → npm_package)
        match software_key {
            "deepseek-reasonix" => candidates.push("reasonix".to_string()),
            "claude-code" => {
                candidates.push("@anthropic-ai/claude-code".to_string());
                candidates.push("claude".to_string());
            }
            "codex" => candidates.push("@openai/codex".to_string()),
            "gemini-cli" => candidates.push("@google/gemini-cli".to_string()),
            "opencode" => candidates.push("opencode-ai".to_string()),
            "mimo-code" => candidates.push("@mimo-ai/cli".to_string()),
            "qwen-code" => candidates.push("@qwen-code/qwen-code".to_string()),
            "copilot" => candidates.push("@github/copilot".to_string()),
            _ => {}
        }

        // Also try without hyphens
        if software_key.contains('-') {
            candidates.push(software_key.replace('-', ""));
        }

        candidates
    }

    fn get_brew_cask_candidates(&self, software_key: &str) -> Vec<String> {
        let mut candidates = vec![software_key.to_string()];

        // Add known brew cask name mappings
        match software_key {
            "deepseek-reasonix" => candidates.push("reasonix".to_string()),
            "vscode" => candidates.push("visual-studio-code".to_string()),
            "obsidian" => candidates.push("obsidian".to_string()),
            "snipaste" => candidates.push("snipaste".to_string()),
            _ => {}
        }

        // Also try without hyphens
        if software_key.contains('-') {
            candidates.push(software_key.replace('-', ""));
        }

        candidates
    }

    fn all_npm_candidates_removed(&self, candidates: &[String]) -> bool {
        // Check if ALL candidate package names are gone from the global install.
        // If any one is still present, the uninstall is incomplete.
        // NOTE: npm list can exit with non-zero even when package IS installed
        // (e.g. broken peer deps), so we check stdout content, not exit code.
        for pkg in candidates {
            if let Ok(output) = Command::new("npm")
                .args(["list", "-g", "--depth=0", pkg])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains(pkg) {
                    log::info!("Package {} still installed", pkg);
                    return false;
                }
            }
        }
        true
    }

    fn any_npm_candidate_installed(&self, candidates: &[String]) -> bool {
        // Check if ANY candidate package is currently installed via npm.
        // This prevents false positives for tools not installed via npm.
        // NOTE: npm list can exit with non-zero even when package IS installed
        // (e.g. broken peer deps), so we check stdout content, not exit code.
        for pkg in candidates {
            if let Ok(output) = Command::new("npm")
                .args(["list", "-g", "--depth=0", pkg])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains(pkg) {
                    log::info!("Found installed npm package: {}", pkg);
                    return true;
                }
            }
        }
        false
    }

    /// Returns None if not applicable, Some(vec![]) if fully removed, Some(cmds) if manual steps needed.
    fn uninstall_curl_bash_binary(&self, software_key: &str) -> Option<Vec<String>> {
        // Some tools install via curl-bash and drop binaries into ~/.local/bin
        // with symlinks + a data directory under ~/.local/share/<name>.
        // Remove the symlinks and the data directory if they exist.
        let (binary_names, data_dir_name, system_paths) = match software_key {
            "cursor" => (vec!["cursor-agent", "agent"], "cursor-agent", vec![] as Vec<String>),
            "hermes" => (vec!["hermes", "hermes-agent"], "hermes", vec![]),
            "opencode" => (vec!["opencode"], "opencode", vec![]),
            _ => return None,
        };

        let home = match dirs::home_dir() {
            Some(h) => h,
            None => {
                log::error!("Cannot determine home directory, skipping curl-bash uninstall");
                return Some(Vec::new());
            }
        };
        let local_bin = home.join(".local/bin");
        let local_share = home.join(".local/share");

        // Collect system-level paths that need manual deletion
        let mut manual_cmds: Vec<String> = Vec::new();
        for sys_path in &system_paths {
            let p = PathBuf::from(sys_path);
            if p.exists() || p.symlink_metadata().is_ok() {
                manual_cmds.push(format!("sudo rm -f {}", sys_path));
            }
        }

        // Remove user-level symlinks in ~/.local/bin
        for name in &binary_names {
            let link_path = local_bin.join(name);
            if link_path.exists() || link_path.symlink_metadata().is_ok() {
                log::info!("Removing symlink: {:?}", link_path);
                let _ = std::fs::remove_file(&link_path);
            }
        }

        // Remove data directory (~/.local/share/<name>)
        let data_dir = local_share.join(data_dir_name);
        if data_dir.exists() {
            log::info!("Removing data directory: {:?}", data_dir);
            let _ = std::fs::remove_dir_all(&data_dir);
        }

        // Also remove config directory (~/.<name>) if it exists
        let config_dir = home.join(format!(".{}", data_dir_name));
        if config_dir.exists() {
            log::info!("Removing config directory: {:?}", config_dir);
            let _ = std::fs::remove_dir_all(&config_dir);
        }

        Some(manual_cmds)
    }

    // ============ Helper Methods ============

    fn ok_response(message: &str) -> UninstallResponse {
        UninstallResponse {
            success: true,
            message: message.to_string(),
            needs_manual: false,
            manual_commands: vec![],
        }
    }

    fn run_command(&self, program: &str, args: &[&str]) -> InstallResult<()> {
        Command::new(program)
            .args(args)
            .output()
            .map_err(|e| InstallError::CommandFailed(e.to_string()))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(InstallError::InstallFailed(stderr.to_string()))
                }
            })
    }

    fn run_and_capture(&self, program: &str, args: &[&str]) -> InstallResult<Option<String>> {
        Command::new(program)
            .args(args)
            .output()
            .map_err(|e| InstallError::CommandFailed(e.to_string()))
            .and_then(|output| {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let version = stdout.trim().to_string();
                    if version.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(version))
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(InstallError::InstallFailed(stderr.to_string()))
                }
            })
    }

    fn run_bash_script(&self, url: &str) -> InstallResult<()> {
        Command::new("/bin/bash")
            .args(["-c", &format!(r#"curl -fsSL {} | bash"#, url)])
            .output()
            .map_err(|e| InstallError::CommandFailed(e.to_string()))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(InstallError::InstallFailed(stderr.to_string()))
                }
            })
    }

    fn run_powershell(&self, script: &str) -> InstallResult<()> {
        Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
            .output()
            .map_err(|e| InstallError::CommandFailed(e.to_string()))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(InstallError::InstallFailed(stderr.to_string()))
                }
            })
    }
}

impl Default for SoftwareInstaller {
    fn default() -> Self {
        Self::new()
    }
}
