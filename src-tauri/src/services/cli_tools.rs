use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Installation conflict: {0}")]
    Conflict(String),
}

pub type CliResult<T> = Result<T, CliToolError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallMethod {
    Npm,
    CurlBash,
    NpmCurlFallback,
    Brew,
}

impl InstallMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallMethod::Npm => "npm",
            InstallMethod::CurlBash => "curl-bash",
            InstallMethod::NpmCurlFallback => "npm-curl-fallback",
            InstallMethod::Brew => "brew",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "npm" => Some(InstallMethod::Npm),
            "curl-bash" => Some(InstallMethod::CurlBash),
            "npm-curl-fallback" => Some(InstallMethod::NpmCurlFallback),
            "brew" => Some(InstallMethod::Brew),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallCommand {
    pub method: InstallMethod,
    pub command: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliToolConfig {
    pub key: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub install_methods: Vec<InstallCommand>,
    pub npm_package: Option<String>,
    pub website_url: Option<String>,
    /// Plugin directory path for this CLI tool (e.g. ~/.claude/plugins/).
    /// Used as the sync target when syncing plugins from the Forge cache.
    pub plugin_dir: Option<String>,
    /// Per-tool install/upgrade timeout in seconds. Defaults to 300 if None.
    #[serde(default)]
    pub install_timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliToolStatus {
    pub tool_key: String,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub install_method: Option<InstallMethod>,
    pub install_path: Option<String>,
    pub has_conflict: bool,
    pub conflict_info: Option<String>,
    pub latest_version: Option<String>,
    pub needs_upgrade: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeResult {
    pub success: bool,
    pub message: String,
    pub new_version: Option<String>,
    pub method: InstallMethod,
}

pub struct CliToolManager;

// ============== Async Command Execution ==============

/// Async command execution with timeout using tokio
async fn run_command_with_timeout_async(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Option<std::process::Output> {
    let program = program.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    tokio::select! {
        result = tokio::process::Command::new(&program)
            .args(&args)
            .kill_on_drop(true)
            .output() => result.ok(),
        _ = tokio::time::sleep(timeout) => {
            None
        }
    }
}

/// Async shell command execution
async fn run_shell_async(command: &str, timeout: Duration) -> Option<std::process::Output> {
    tokio::select! {
        result = tokio::process::Command::new("sh")
            .args(["-c", command])
            .kill_on_drop(true)
            .output() => result.ok(),
        _ = tokio::time::sleep(timeout) => {
            None
        }
    }
}

// ============== Legacy Sync Wrapper (for backward compatibility) ==============

fn run_command_with_timeout_sync(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Option<std::process::Output> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let program = program.to_string();
    let program_log = program.clone();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    std::thread::spawn(move || {
        let result = Command::new(&program)
            .args(&args)
            .output();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => result.ok(),
        Err(_) => {
            // Timeout fired — the thread is still blocking on Command::output().
            // We cannot kill the child process from here without its PID, but we
            // can at least drop the receiver so the sender's channel is broken.
            // The thread will eventually finish (or not), but we stop waiting.
            log::warn!(
                "Command {:?} timed out after {:?}, thread may still be running",
                program_log, timeout
            );
            None
        }
    }
}

// ============== CliToolManager Implementation ==============

impl CliToolManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_supported_tools() -> Vec<CliToolConfig> {
        vec![
            CliToolConfig {
                key: "claude-code".to_string(),
                name: "Claude Code".to_string(),
                icon: "anthropic".to_string(),
                description: "Anthropic's official CLI tool for AI-assisted coding".to_string(),
                install_methods: vec![
                    InstallCommand {
                        method: InstallMethod::CurlBash,
                        command: "bash -c 'tmp=$(mktemp) && curl -fsSL https://claude.ai/install.sh -o $tmp && bash $tmp; status=$?; rm -f $tmp; exit $status'".to_string(),
                        priority: 1,
                    },
                    InstallCommand {
                        method: InstallMethod::Npm,
                        command: "npm i -g @anthropic-ai/claude-code@latest".to_string(),
                        priority: 2,
                    },
                ],
                npm_package: Some("@anthropic-ai/claude-code".to_string()),
                website_url: Some("https://docs.anthropic.com/en/docs/claude-code".to_string()),
                plugin_dir: Some("~/.claude/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "codex".to_string(),
                name: "Codex".to_string(),
                icon: "openai".to_string(),
                description: "OpenAI's CLI tool for AI code generation".to_string(),
                install_methods: vec![InstallCommand {
                    method: InstallMethod::Npm,
                    command: "npm i -g @openai/codex@latest".to_string(),
                    priority: 1,
                }],
                npm_package: Some("@openai/codex".to_string()),
                website_url: Some("https://codex.openai.com".to_string()),
                plugin_dir: Some("~/.codex/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "gemini-cli".to_string(),
                name: "Gemini CLI".to_string(),
                icon: "google".to_string(),
                description: "Google's CLI tool for Gemini AI interactions".to_string(),
                install_methods: vec![InstallCommand {
                    method: InstallMethod::Npm,
                    command: "npm i -g @google/gemini-cli@latest".to_string(),
                    priority: 1,
                }],
                npm_package: Some("@google/gemini-cli".to_string()),
                website_url: Some("https://ai.google.dev/gemini-api".to_string()),
                plugin_dir: Some("~/.gemini/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "opencode".to_string(),
                name: "OpenCode".to_string(),
                icon: "opencode".to_string(),
                description: "OpenCode AI CLI for code assistance".to_string(),
                install_methods: vec![
                    InstallCommand {
                        method: InstallMethod::CurlBash,
                        command: "bash -c 'tmp=$(mktemp) && curl -fsSL https://opencode.ai/install -o $tmp && bash $tmp; status=$?; rm -f $tmp; exit $status'".to_string(),
                        priority: 1,
                    },
                    InstallCommand {
                        method: InstallMethod::Npm,
                        command: "npm i -g opencode-ai@latest".to_string(),
                        priority: 2,
                    },
                ],
                npm_package: Some("opencode-ai".to_string()),
                website_url: Some("https://opencode.ai".to_string()),
                plugin_dir: Some("~/.opencode/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "openclaw".to_string(),
                name: "OpenClaw".to_string(),
                icon: "claw".to_string(),
                description: "OpenClaw CLI for AI-powered development".to_string(),
                install_methods: vec![InstallCommand {
                    method: InstallMethod::Npm,
                    command: "npm i -g openclaw@latest".to_string(),
                    priority: 1,
                }],
                npm_package: Some("openclaw".to_string()),
                website_url: Some("https://openclaw.dev".to_string()),
                plugin_dir: Some("~/.openclaw/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "hermes".to_string(),
                name: "Hermes".to_string(),
                icon: "hermes".to_string(),
                description: "NousResearch's agent framework for AI interactions".to_string(),
                install_methods: vec![InstallCommand {
                    method: InstallMethod::CurlBash,
                    command: "bash -c 'tmp=$(mktemp) && curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh -o $tmp && bash $tmp --skip-setup --non-interactive; status=$?; rm -f $tmp; exit $status'".to_string(),
                    priority: 1,
                }],
                npm_package: None,
                website_url: Some("https://github.com/NousResearch/hermes-agent".to_string()),
                plugin_dir: Some("~/.hermes/plugins".to_string()),
                install_timeout_secs: Some(600),
            },
            CliToolConfig {
                key: "cursor".to_string(),
                name: "Cursor CLI".to_string(),
                icon: "cursor".to_string(),
                description: "Cursor's official CLI tool for AI-assisted coding".to_string(),
                install_methods: vec![
                    InstallCommand {
                        method: InstallMethod::CurlBash,
                        command: "curl https://cursor.com/install -fsS | bash".to_string(),
                        priority: 1,
                    },
                ],
                npm_package: None,
                website_url: Some("https://cursor.com".to_string()),
                plugin_dir: Some("~/.cursor/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "deepseek-reasonix".to_string(),
                name: "reasonix".to_string(),
                icon: "deepseek-reasonix".to_string(),
                description: "DeepSeek's reasoning CLI tool for AI-assisted problem solving".to_string(),
                install_methods: vec![
                    InstallCommand {
                        method: InstallMethod::Npm,
                        command: "npm i -g reasonix".to_string(),
                        priority: 1,
                    },
                    InstallCommand {
                        method: InstallMethod::Brew,
                        command: "brew install esengine/reasonix/reasonix".to_string(),
                        priority: 2,
                    },
                ],
                npm_package: Some("reasonix".to_string()),
                website_url: Some("https://github.com/esengine/DeepSeek-Reasonix".to_string()),
                plugin_dir: Some("~/.reasonix/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "mimo-code".to_string(),
                name: "MiMo Code".to_string(),
                icon: "mimo-code".to_string(),
                description: "Xiaomi's MiMo AI CLI for agentic coding".to_string(),
                install_methods: vec![
                    InstallCommand {
                        method: InstallMethod::CurlBash,
                        command: "curl -fsSL https://mimo.xiaomi.com/install | bash".to_string(),
                        priority: 1,
                    },
                    InstallCommand {
                        method: InstallMethod::Npm,
                        command: "npm install -g @mimo-ai/cli".to_string(),
                        priority: 2,
                    },
                ],
                npm_package: Some("@mimo-ai/cli".to_string()),
                website_url: Some("https://github.com/XiaomiMiMo/MiMo-Code".to_string()),
                plugin_dir: Some("~/.mimo/plugins".to_string()),
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "qwen-code".to_string(),
                name: "Qwen Code".to_string(),
                icon: "qwen-code".to_string(),
                description: "Alibaba's Qwen AI CLI for agentic coding".to_string(),
                install_methods: vec![
                    InstallCommand {
                        method: InstallMethod::Npm,
                        command: "npm install -g @qwen-code/qwen-code@latest".to_string(),
                        priority: 1,
                    },
                    InstallCommand {
                        method: InstallMethod::Brew,
                        command: "brew install qwen-code".to_string(),
                        priority: 2,
                    },
                ],
                npm_package: Some("@qwen-code/qwen-code".to_string()),
                website_url: Some("https://github.com/QwenLM/qwen-code".to_string()),
                plugin_dir: None,
                install_timeout_secs: None,
            },
            CliToolConfig {
                key: "copilot".to_string(),
                name: "GitHub Copilot CLI".to_string(),
                icon: "copilot".to_string(),
                description: "GitHub's Copilot CLI for AI-powered coding assistance in the terminal".to_string(),
                install_methods: vec![
                    InstallCommand {
                        method: InstallMethod::Npm,
                        command: "npm install -g @github/copilot".to_string(),
                        priority: 1,
                    },
                    InstallCommand {
                        method: InstallMethod::Brew,
                        command: "brew install copilot-cli".to_string(),
                        priority: 2,
                    },
                ],
                npm_package: Some("@github/copilot".to_string()),
                website_url: Some("https://github.com/github/copilot-cli".to_string()),
                plugin_dir: None,
                install_timeout_secs: None,
            },
        ]
    }

    // ============== Async Methods ==============

    /// Async check installation status for a single tool
    pub async fn check_installation_async(&self, tool_key: &str) -> CliResult<CliToolStatus> {
        let tools = Self::get_supported_tools();
        let _tool_config = tools
            .iter()
            .find(|t| t.key == tool_key)
            .ok_or_else(|| CliToolError::NotFound(tool_key.to_string()))?;

        let (is_installed, installed_version, install_method, install_path) =
            self.detect_installation_async(tool_key).await;

        let (has_conflict, conflict_info) = self.check_conflicts_async(tool_key, &install_path).await;

        let latest_version = self.fetch_latest_version_async(tool_key).await;

        let needs_upgrade = if is_installed {
            if let (Some(current), Some(latest)) = (&installed_version, &latest_version) {
                current != latest
            } else {
                false
            }
        } else {
            false
        };

        Ok(CliToolStatus {
            tool_key: tool_key.to_string(),
            is_installed,
            installed_version,
            install_method,
            install_path,
            has_conflict,
            conflict_info,
            latest_version,
            needs_upgrade,
        })
    }

    /// Async check all tool installations in parallel
    pub async fn check_all_installations_parallel(&self) -> Vec<CliToolStatus> {
        let tools = Self::get_supported_tools();

        let futures: Vec<_> = tools
            .iter()
            .map(|t| self.check_installation_async(&t.key))
            .collect();

        // Execute all checks in parallel with timeout
        let results = tokio::time::timeout(
            Duration::from_secs(60),
            futures::future::join_all(futures),
        )
        .await
        .unwrap_or_default();

        results.into_iter().map(|r| r.unwrap_or_else(|_| CliToolStatus {
            tool_key: String::new(),
            is_installed: false,
            installed_version: None,
            install_method: None,
            install_path: None,
            has_conflict: false,
            conflict_info: None,
            latest_version: None,
            needs_upgrade: false,
        })).collect()
    }

    /// Async detect installation
    async fn detect_installation_async(&self, tool_key: &str) -> (bool, Option<String>, Option<InstallMethod>, Option<String>) {
        let tools = Self::get_supported_tools();
        let tool_config = tools.iter().find(|t| t.key == tool_key);

        // Check npm first
        if let Some((version, path)) = self.check_npm_version_async(tool_key, tool_config).await {
            return (true, Some(version), Some(InstallMethod::Npm), Some(path));
        }

        // Probe every known binary candidate. We need to be lenient here
        // because different official installers (e.g. Cursor's curl-bash
        // installer) rename the executable — so `cursor` won't be on PATH
        // but `cursor-agent` (or its `agent` symlink) will.
        for binary_name in self.binary_name_candidates(tool_key) {
            if let Some(path) = self.which_async(&binary_name).await {
                let version = self.get_version_from_binary_async(tool_key, &path).await;
                return (true, version, Some(InstallMethod::CurlBash), Some(path));
            }
        }

        (false, None, None, None)
    }

    async fn check_npm_version_async(&self, tool_key: &str, config: Option<&CliToolConfig>) -> Option<(String, String)> {
        let npm_pkg = config.and_then(|c| c.npm_package.clone());
        let pkg_to_check = npm_pkg.as_deref().unwrap_or(tool_key);

        let output = run_command_with_timeout_async(
            "npm",
            &["list", "-g", "--depth=0", pkg_to_check],
            Duration::from_secs(10),
        ).await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(version) = self.parse_npm_version(&stdout, pkg_to_check) {
                let path = self.get_npm_global_path_sync(pkg_to_check);
                return Some((version, path));
            }
        }
        None
    }

    async fn which_async(&self, binary: &str) -> Option<String> {
        // Direct which
        if let Some(output) = run_command_with_timeout_async("which", &[binary], Duration::from_secs(5)).await {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some(path);
            }
        }

        // npm global paths
        if let Some(output) = run_command_with_timeout_async("npm", &["root", "-g"], Duration::from_secs(5)).await {
            if output.status.success() {
                let root_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let bin_path = root_path.replace("/lib/node_modules", "/bin");
                let full_path = format!("{}/{}", bin_path, binary);

                if std::path::Path::new(&full_path).exists() {
                    return Some(full_path);
                }

                let bin_link = format!("{}/.bin/{}", bin_path, binary);
                if std::path::Path::new(&bin_link).exists() {
                    return Some(bin_link);
                }
            }
        }
        None
    }

    async fn get_version_from_binary_async(&self, _tool_key: &str, path: &str) -> Option<String> {
        // Try --version
        if let Some(output) = run_command_with_timeout_async(path, &["--version"], Duration::from_secs(5)).await {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }

        // Try -v
        if let Some(output) = run_command_with_timeout_async(path, &["-v"], Duration::from_secs(5)).await {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    async fn check_conflicts_async(&self, _tool_key: &str, install_path: &Option<String>) -> (bool, Option<String>) {
        if let Some(path) = install_path {
            let path_str = path.to_string();

            if let Some(output) = run_command_with_timeout_async("npm", &["root", "-g"], Duration::from_secs(5)).await {
                let global_npm = String::from_utf8_lossy(&output.stdout).trim().to_string();

                if !global_npm.is_empty() && path_str.contains(&global_npm) {
                    if let Some(output) = run_command_with_timeout_async("npm", &["list", "-g", "--depth=0"], Duration::from_secs(10)).await {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let lines: Vec<&str> = stdout.lines().collect();
                        let pkg_count = lines.iter().filter(|l| !l.contains("npm@") && !l.contains("undici@")).count();

                        if pkg_count > 10 {
                            return (true, Some(format!("Heavy npm global installation detected ({} packages). Consider using nvm.", pkg_count - 1)));
                        }
                    }
                }
            }
        }
        (false, None)
    }

    async fn fetch_latest_version_async(&self, tool_key: &str) -> Option<String> {
        let tools = Self::get_supported_tools();
        let config = tools.iter().find(|t| t.key == tool_key)?;

        if let Some(pkg) = &config.npm_package {
            let output = run_command_with_timeout_async(
                "npm",
                &["view", pkg, "version"],
                Duration::from_secs(10),
            ).await?;

            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    pub async fn upgrade_async(&self, tool_key: &str, method: InstallMethod) -> CliResult<UpgradeResult> {
        let tools = Self::get_supported_tools();
        let tool_config = tools
            .iter()
            .find(|t| t.key == tool_key)
            .ok_or_else(|| CliToolError::NotFound(tool_key.to_string()))?;

        let command = tool_config
            .install_methods
            .iter()
            .find(|m| m.method == method)
            .map(|m| m.command.clone())
            .ok_or_else(|| CliToolError::NotFound(format!("Install method {:?} for {}", method, tool_key)))?;

        let needs_shell = command.contains('|')
            || command.contains("&&")
            || command.contains("||")
            || command.contains('>')
            || command.starts_with("bash -c")
            || command.starts_with("sh -c");

        let timeout = Duration::from_secs(tool_config.install_timeout_secs.unwrap_or(300));

        let output = if needs_shell {
            run_shell_async(&command, timeout).await
        } else {
            let parts = shell_words::split(&command).unwrap_or_else(|_| vec![command.clone()]);
            if parts.is_empty() {
                return Err(CliToolError::CommandFailed("Empty command".to_string()));
            }
            let program = &parts[0];
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            run_command_with_timeout_async(program, &args, timeout).await
        };

        let output = output.ok_or_else(|| CliToolError::CommandFailed("Command timed out".to_string()))?;

        if output.status.success() {
            let binary_name = self.get_binary_name(tool_key);
            let new_version = self.get_version_from_binary_async(tool_key, &binary_name).await;

            Ok(UpgradeResult {
                success: true,
                message: format!("Successfully upgraded {} to latest version", tool_key),
                new_version,
                method,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = if stderr.is_empty() { stdout.to_string() } else { stderr.to_string() };
            Err(CliToolError::CommandFailed(error_msg))
        }
    }

    // ============== Sync Methods (Legacy, for Tauri commands) ==============

    pub fn check_installation(&self, tool_key: &str) -> CliResult<CliToolStatus> {
        let tools = Self::get_supported_tools();
        let _tool_config = tools
            .iter()
            .find(|t| t.key == tool_key)
            .ok_or_else(|| CliToolError::NotFound(tool_key.to_string()))?;

        let (is_installed, installed_version, install_method, install_path) =
            self.detect_installation_sync(tool_key);

        let (has_conflict, conflict_info) = self.check_conflicts_sync(tool_key, &install_path);

        let latest_version = self.fetch_latest_version_sync(tool_key);

        let needs_upgrade = if is_installed {
            if let (Some(current), Some(latest)) = (&installed_version, &latest_version) {
                current != latest
            } else {
                false
            }
        } else {
            false
        };

        Ok(CliToolStatus {
            tool_key: tool_key.to_string(),
            is_installed,
            installed_version,
            install_method,
            install_path,
            has_conflict,
            conflict_info,
            latest_version,
            needs_upgrade,
        })
    }

    pub fn check_all_installations(&self) -> Vec<CliToolStatus> {
        let tools = Self::get_supported_tools();
        let statuses: Vec<CliToolStatus> = tools
            .iter()
            .map(|t| {
                match self.check_installation(&t.key) {
                    Ok(status) => status,
                    Err(e) => {
                        log::warn!("CLI check failed for {}: {}", t.key, e);
                        CliToolStatus {
                            tool_key: t.key.clone(),
                            is_installed: false,
                            installed_version: None,
                            install_method: None,
                            install_path: None,
                            has_conflict: false,
                            conflict_info: None,
                            latest_version: None,
                            needs_upgrade: false,
                        }
                    }
                }
            })
            .collect();
        statuses
    }

    fn detect_installation_sync(&self, tool_key: &str) -> (bool, Option<String>, Option<InstallMethod>, Option<String>) {
        let tools = Self::get_supported_tools();
        let tool_config = tools.iter().find(|t| t.key == tool_key);

        // Check npm first
        if let Some((version, path)) = self.check_npm_version_sync(tool_key, tool_config) {
            return (true, Some(version), Some(InstallMethod::Npm), Some(path));
        }

        // Probe every known binary candidate. The official Cursor installer
        // drops `cursor-agent` (with an `agent` symlink) into ~/.local/bin,
        // NOT a binary called `cursor`, so we must walk every alias.
        for binary_name in self.binary_name_candidates(tool_key) {
            if let Some(path) = self.which_sync(&binary_name) {
                let version = self.get_version_from_binary_sync(tool_key, &path);
                return (true, version, Some(InstallMethod::CurlBash), Some(path));
            }
        }

        (false, None, None, None)
    }

    fn check_npm_version_sync(&self, tool_key: &str, config: Option<&CliToolConfig>) -> Option<(String, String)> {
        let npm_pkg = config.and_then(|c| c.npm_package.clone());
        let pkg_to_check = npm_pkg.as_deref().unwrap_or(tool_key);

        let output = run_command_with_timeout_sync(
            "npm",
            &["list", "-g", "--depth=0", pkg_to_check],
            Duration::from_secs(10),
        )?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(version) = self.parse_npm_version(&stdout, pkg_to_check) {
                let path = self.get_npm_global_path_sync(pkg_to_check);
                return Some((version, path));
            }
        }
        None
    }

    fn which_sync(&self, binary: &str) -> Option<String> {
        // Direct which
        if let Some(output) = run_command_with_timeout_sync("which", &[binary], Duration::from_secs(5)) {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some(path);
            }
        }

        // npm global paths
        if let Some(output) = run_command_with_timeout_sync("npm", &["root", "-g"], Duration::from_secs(5)) {
            if output.status.success() {
                let root_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let bin_path = root_path.replace("/lib/node_modules", "/bin");
                let full_path = format!("{}/{}", bin_path, binary);

                if std::path::Path::new(&full_path).exists() {
                    return Some(full_path);
                }

                let bin_link = format!("{}/.bin/{}", bin_path, binary);
                if std::path::Path::new(&bin_link).exists() {
                    return Some(bin_link);
                }
            }
        }
        None
    }

    fn get_version_from_binary_sync(&self, _tool_key: &str, path: &str) -> Option<String> {
        if let Some(output) = run_command_with_timeout_sync(path, &["--version"], Duration::from_secs(5)) {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }

        if let Some(output) = run_command_with_timeout_sync(path, &["-v"], Duration::from_secs(5)) {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    fn check_conflicts_sync(&self, _tool_key: &str, install_path: &Option<String>) -> (bool, Option<String>) {
        if let Some(path) = install_path {
            let path_str = path.to_string();

            if let Some(output) = run_command_with_timeout_sync("npm", &["root", "-g"], Duration::from_secs(5)) {
                let global_npm = String::from_utf8_lossy(&output.stdout).trim().to_string();

                if !global_npm.is_empty() && path_str.contains(&global_npm) {
                    if let Some(output) = run_command_with_timeout_sync("npm", &["list", "-g", "--depth=0"], Duration::from_secs(10)) {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let lines: Vec<&str> = stdout.lines().collect();
                        let pkg_count = lines.iter().filter(|l| !l.contains("npm@") && !l.contains("undici@")).count();

                        if pkg_count > 10 {
                            return (true, Some(format!("Heavy npm global installation detected ({} packages). Consider using nvm.", pkg_count - 1)));
                        }
                    }
                }
            }
        }
        (false, None)
    }

    fn fetch_latest_version_sync(&self, tool_key: &str) -> Option<String> {
        let tools = Self::get_supported_tools();
        let config = tools.iter().find(|t| t.key == tool_key)?;

        if let Some(pkg) = &config.npm_package {
            let output = run_command_with_timeout_sync(
                "npm",
                &["view", pkg, "version"],
                Duration::from_secs(10),
            )?;

            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    pub fn upgrade(&self, tool_key: &str, method: InstallMethod) -> CliResult<UpgradeResult> {
        let tools = Self::get_supported_tools();
        let tool_config = tools
            .iter()
            .find(|t| t.key == tool_key)
            .ok_or_else(|| CliToolError::NotFound(tool_key.to_string()))?;

        let command = tool_config
            .install_methods
            .iter()
            .find(|m| m.method == method)
            .map(|m| m.command.clone())
            .ok_or_else(|| CliToolError::NotFound(format!("Install method {:?} for {}", method, tool_key)))?;

        let needs_shell = command.contains('|')
            || command.contains("&&")
            || command.contains("||")
            || command.contains('>')
            || command.starts_with("bash -c")
            || command.starts_with("sh -c");

        let timeout = Duration::from_secs(tool_config.install_timeout_secs.unwrap_or(300));

        let output = if needs_shell {
            run_command_with_timeout_sync("sh", &["-c", &command], timeout)
        } else {
            let parts = shell_words::split(&command).unwrap_or_else(|_| vec![command.clone()]);
            if parts.is_empty() {
                return Err(CliToolError::CommandFailed("Empty command".to_string()));
            }
            let program = &parts[0];
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            run_command_with_timeout_sync(program, &args, timeout)
        };

        let output = output.ok_or_else(|| CliToolError::CommandFailed("Command timed out".to_string()))?;

        if output.status.success() {
            let binary_name = self.get_binary_name(tool_key);
            let new_version = self.get_version_from_binary_sync(tool_key, &binary_name);

            Ok(UpgradeResult {
                success: true,
                message: format!("Successfully upgraded {} to latest version", tool_key),
                new_version,
                method,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_msg = if stderr.is_empty() { stdout.to_string() } else { stderr.to_string() };
            Err(CliToolError::CommandFailed(error_msg))
        }
    }

    // ============== Helper Methods ==============

    /// Resolve the candidate binary names for a tool, in priority order.
    ///
    /// Most tools expose a single executable (`claude`, `gemini`, `codex`).
    /// Some official installers rename the binary (`cursor-agent` instead of
    /// `cursor`); for these we must probe both the canonical name and any
    /// known aliases the installer may have created, otherwise the detection
    /// falsely reports the tool as not installed.
    fn binary_name_candidates(&self, tool_key: &str) -> Vec<String> {
        let mut candidates: Vec<String> = match tool_key {
            "claude-code" => vec!["claude".to_string()],
            "gemini-cli" => vec!["gemini".to_string()],
            // Cursor's official installer (https://cursor.com/install) drops a
            // `cursor-agent` binary into ~/.local/bin and also creates an
            // `agent` symlink for it. The `cursor` binary is the editor app,
            // not the CLI, so it must NOT be a candidate.
            "cursor" => vec!["cursor-agent".to_string(), "agent".to_string()],
            "codex" => vec!["codex".to_string()],
            "opencode" => vec!["opencode".to_string()],
            "openclaw" => vec!["openclaw".to_string()],
            "hermes" => vec!["hermes".to_string()],
            "deepseek-reasonix" => vec!["reasonix".to_string()],
            "mimo-code" => vec!["mimo".to_string()],
            "qwen-code" => vec!["qwen-code".to_string()],
            "copilot" => vec!["copilot".to_string()],
            _ => vec![tool_key.to_string()],
        };

        // Always fall back to the tool_key itself, which lets users with
        // custom aliases (e.g. an explicit `cursor` -> `cursor-agent` shim)
        // still be detected correctly.
        let raw = tool_key.to_string();
        if !candidates.iter().any(|c| c == &raw) {
            candidates.push(raw);
        }

        candidates
    }

    /// First-resolved binary name for a tool. Kept for callers that just need
    /// a single "best-effort" name to read the version from after install
    /// (e.g. running `cursor-agent --version` after the upgrade).
    fn get_binary_name(&self, tool_key: &str) -> String {
        self.binary_name_candidates(tool_key)
            .into_iter()
            .next()
            .unwrap_or_else(|| tool_key.to_string())
    }

    fn parse_npm_version(&self, output: &str, tool_key: &str) -> Option<String> {
        for line in output.lines() {
            if line.contains("├──") || line.contains("└──") {
                let full_line = line.replace("├──", "").replace("└──", "").replace("│", "").replace(" ", "");
                if full_line.contains('@') {
                    if let Some(at_idx) = full_line.rfind('@') {
                        let version = &full_line[at_idx + 1..];
                        let package = &full_line[..at_idx];
                        if package.contains(tool_key) ||
                           package.contains(&tool_key.replace("-", "")) ||
                           (tool_key == "claude-code" && package.contains("claude")) {
                            if !version.is_empty() && version.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                                return Some(version.trim_end_matches(' ').to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn get_npm_global_path_sync(&self, package: &str) -> String {
        if let Some(output) = run_command_with_timeout_sync("npm", &["root", "-g"], Duration::from_secs(5)) {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return format!("{}/{}", path, package);
            }
        }
        format!("/usr/local/lib/node_modules/{}", package)
    }
}

impl Default for CliToolManager {
    fn default() -> Self {
        Self::new()
    }
}
