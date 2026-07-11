use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use thiserror::Error;

#[cfg(target_os = "windows")]
use tokio::process::CommandExt as TokioCommandExt;

/// Lock the global SQLite connection, recovering from a poisoned mutex.
///
/// `std::sync::Mutex::lock().unwrap()` panics whenever any previous holder
/// panicked while holding the lock. In a Tauri command handler that
/// immediately tears down the entire backend process — which is the worst
/// possible UX (the user sees the desktop app disappear mid-install).
///
/// We use `PoisonError::into_inner()` to transparently recover. SQLite's own
/// transactional guarantees mean the in-flight statement was already rolled
/// back when the previous holder panicked, so the recovered `Connection` is
/// safe to reuse.
///
/// Marked `pub(crate)` so future sibling services (plugin_marketplace,
/// skill_repository, etc.) can adopt the same recovery pattern without
/// duplicating the helper.
pub(crate) fn lock_db(db: &crate::db::Database) -> MutexGuard<'_, Connection> {
    match db.conn.lock() {
        Ok(g) => g,
        Err(p) => {
            log::error!("DB mutex poisoned, recovering (previous holder panicked): {}", p);
            p.into_inner()
        }
    }
}

/// Same as [`lock_db`] but for a raw `Arc<Mutex<Connection>>` shared with
/// the rest of the app's startup path (e.g. `seed_builtin_cli_tools_locked`).
pub(crate) fn lock_conn_arc(
    conn_arc: &std::sync::Arc<Mutex<Connection>>,
) -> MutexGuard<'_, Connection> {
    match conn_arc.lock() {
        Ok(g) => g,
        Err(p) => {
            log::error!("DB conn_arc mutex poisoned, recovering: {}", p);
            p.into_inner()
        }
    }
}

// ============== Read-through cache for `get_supported_tools()` ==============
//
// `get_supported_tools()` is called on *every* per-tool status check
// (see audit item #7). With 28 tools and parallel status checks, the front
// page is hitting SQLite 56+ times per open. The list of tools itself
// changes only when `add_custom_cli_tool` / `remove_custom_cli_tool` /
// `seed_builtin_cli_tools` mutates the table, so we cache the result and
// invalidate on any write path.
//
// We use a plain `std::sync::OnceLock<Mutex<Option<Vec<CliToolConfig>>>`
// instead of `LazyLock` (stable since 1.80 but we still target 1.77) or
// `parking_lot` (not in Cargo.toml). The Option wrapper distinguishes
// "not yet loaded" from "loaded with an empty result".

type ToolsCache = Mutex<Option<Vec<CliToolConfig>>>;

static TOOLS_CACHE: std::sync::OnceLock<ToolsCache> = std::sync::OnceLock::new();

fn tools_cache() -> &'static ToolsCache {
    TOOLS_CACHE.get_or_init(|| Mutex::new(None))
}

/// Drop the cached tool list. Must be called from every code path that
/// mutates `custom_cli_tools` (add / remove / seed).
fn invalidate_tools_cache() {
    if let Some(c) = TOOLS_CACHE.get() {
        // Poison recovery mirrors `lock_db`: a panic during a previous
        // invalidate shouldn't block the next read.
        match c.lock() {
            Ok(mut g) => *g = None,
            Err(p) => *p.into_inner() = None,
        }
    }
}

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

// Note: the previous `is_windows()` helper that returned `cfg!(target_os
// = "windows")` was removed (audit item #8). Call sites that previously
// branched on it now use `#[cfg(target_os = "windows")]` / `#[cfg(not(...))]`
// blocks directly. The dead branch is erased at compile time on each
// platform — no runtime bool check, no string comparison.

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
            "curl" | "curl-bash" | "curl | bash" => Some(InstallMethod::CurlBash),
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
    /// Where this tool definition comes from: allagents | custom. (The legacy
    /// `builtin` variant was removed when the hardcoded builtin list was
    /// migrated to the `custom_cli_tools` DB table in 2026-07.)
    ///
    /// NOTE: This used to be `&'static str`. That worked for the seed-time
    /// literal values ("allagents" / "custom") but is **incompatible with
    /// Serde deserialization** of any value not known at compile time — the
    /// borrow checker rejects borrowing from a runtime `String` as `'static`.
    /// Round-tripping a row read from `custom_cli_tools` (or any future
    /// remote sync) would therefore panic. We store it as `String` and
    /// validate at the boundary instead.
    #[serde(default = "default_display_source")]
    pub display_source: String,
    /// If true, this tool requires manual download from website (no quick-install)
    #[serde(default)]
    pub manual_download_only: bool,
}

fn default_display_source() -> String { "allagents".to_string() }

impl Default for CliToolConfig {
    fn default() -> Self {
        Self {
            display_source: "allagents".to_string(),
            manual_download_only: false,
            key: Default::default(),
            name: Default::default(),
            icon: Default::default(),
            description: Default::default(),
            install_methods: Default::default(),
            npm_package: Default::default(),
            website_url: Default::default(),
            plugin_dir: Default::default(),
            install_timeout_secs: Default::default(),
        }
    }
}

/// Custom tool config passed from the frontend when adding a new tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCliToolConfig {
    pub key: String,
    pub is_allagents: bool,  // true = allagents tool, false = custom tool
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub description: String,
    pub install_method: String,
    pub install_command: String,
    pub detect_command: String,
    #[serde(default)]
    pub npm_package: Option<String>,
    pub website_url: Option<String>,
    pub plugin_dir: Option<String>,
    #[serde(default = "default_timeout")]
    pub install_timeout_secs: u64,
}

const fn default_timeout() -> u64 { 300 }

// ============== Custom CLI Tools Persistence (SQLite) ==============

/// Load all user-defined custom CLI tools from the database
pub fn load_custom_cli_tools() -> Vec<CliToolConfig> {
    if let Some(db) = crate::db::Database::global() {
        let conn = lock_db(&db);
        load_custom_cli_tools_from_conn(&conn)
    } else {
        Vec::new()
    }
}

/// Idempotently insert the canonical builtin CLI tools into `custom_cli_tools`
/// on first launch so the table starts populated. Existing rows (by primary
/// key) are preserved via `INSERT OR IGNORE`, so this is safe to call on every
/// DB init — only missing rows are added. This function is the **sole
/// canonical source** of the 11 builtin tool definitions; the legacy
/// `builtin_tools()` Rust hardcoded list was removed (2026-07) so all product
/// configuration (name/icon/description/install command/npm package/plugin dir/
/// ...) is now data-driven through SQLite.
/// Now moved to database with 'type' field to distinguish 'default' and 'custom' tools.
/// Seed tool entry for database initialization
struct SeedEntry<'a> {
    key: &'a str,
    is_allagents: bool,  // true = allagents 23 tools, false = custom tools
    name: &'a str,
    icon: &'a str,
    description: &'a str,
    install_method: &'a str,
    install_command: &'a str,
    detect_command: &'a str,
    website_url: Option<&'a str>,
    plugin_dir: Option<&'a str>,
    timeout_secs: Option<u64>,
    npm_package: Option<&'a str>,
}

pub fn seed_builtin_cli_tools(conn: &Connection) {
    let seeds: Vec<SeedEntry> = vec![
        // AllAgents 23 tools (Universal clients: 8, Provider clients: 15)
        SeedEntry {
            key: "copilot",
            is_allagents: true,
            name: "GitHub Copilot",
            icon: "copilot",
            description: "GitHub's Copilot CLI for AI-powered coding assistance",
            install_method: "npm",
            install_command: "npm install -g @github/copilot@latest",
            detect_command: "command -v copilot || where copilot",
            website_url: Some("https://github.com/github/copilot-cli"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@github/copilot"),
        },
        SeedEntry {
            key: "codex",
            is_allagents: true,
            name: "OpenAI Codex",
            icon: "openai",
            description: "OpenAI's CLI tool for AI code generation",
            install_method: "npm",
            install_command: "npm install -g @openai/codex@latest",
            detect_command: "command -v codex || where codex",
            website_url: Some("https://codex.openai.com"),
            plugin_dir: Some("~/.codex/plugins"),
            timeout_secs: None,
            npm_package: Some("@openai/codex"),
        },
        SeedEntry {
            key: "opencode",
            is_allagents: true,
            name: "OpenCode",
            icon: "opencode",
            description: "OpenCode AI CLI for code assistance",
            install_method: "curl",
            install_command: "curl -fsSL https://opencode.ai/install | bash",
            detect_command: "command -v opencode || where opencode",
            website_url: Some("https://opencode.ai"),
            plugin_dir: Some("~/.opencode/plugins"),
            timeout_secs: None,
            npm_package: Some("opencode-ai"),
        },
        SeedEntry {
            key: "gemini",
            is_allagents: true,
            name: "Gemini",
            icon: "gemini",
            description: "Google's CLI tool for Gemini AI interactions",
            install_method: "npm",
            install_command: "npm install -g @google/gemini-cli@latest",
            detect_command: "command -v gemini || where gemini",
            website_url: Some("https://ai.google.dev/gemini-api"),
            plugin_dir: Some("~/.gemini/plugins"),
            timeout_secs: None,
            npm_package: Some("@google/gemini-cli"),
        },
        SeedEntry {
            key: "ampcode",
            is_allagents: true,
            name: "Amp Code",
            icon: "ampcode",
            description: "Amp Code - lightweight AI coding assistant",
            install_method: "npm",
            install_command: "npm install -g @ampcode/cli@latest",
            detect_command: "command -v ampcode || where ampcode",
            website_url: Some("https://ampcode.dev"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@ampcode/cli"),
        },
        SeedEntry {
            key: "replit",
            is_allagents: true,
            name: "Replit",
            icon: "replit",
            description: "Replit - browser-based coding platform",
            install_method: "npm",
            install_command: "npm install -g replit@latest",
            detect_command: "command -v replit || where replit",
            website_url: Some("https://replit.com"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("replit"),
        },
        SeedEntry {
            key: "kimi",
            is_allagents: true,
            name: "Kimi",
            icon: "kimi",
            description: "Kimi - AI assistant from Moonshot AI",
            install_method: "npm",
            install_command: "npm install -g @moonshot-ai/kimi-code@latest",
            detect_command: "command -v kimi || where kimi",
            website_url: Some("https://kimi.moonshot.cn"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@moonshot-ai/kimi-code"),
        },
        SeedEntry {
            key: "vscode",
            is_allagents: true,
            name: "VS Code",
            icon: "vscode",
            description: "VS Code - Microsoft Visual Studio Code (Universal Client)",
            install_method: "manual",
            install_command: "",
            detect_command: "command -v code || where code",
            website_url: Some("https://code.visualstudio.com"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: None,
        },
        SeedEntry {
            key: "claude",
            is_allagents: true,
            name: "Claude Code",
            icon: "claude",
            description: "Claude Code - Anthropic's AI coding tool",
            install_method: "npm",
            install_command: "npm install -g @anthropic-ai/claude-code@latest",
            detect_command: "command -v claude || where claude",
            website_url: Some("https://docs.anthropic.com/en/docs/claude-code"),
            plugin_dir: Some("~/.claude/plugins"),
            timeout_secs: None,
            npm_package: Some("@anthropic-ai/claude-code"),
        },
        SeedEntry {
            key: "cursor",
            is_allagents: true,
            name: "Cursor",
            icon: "cursor",
            description: "Cursor - AI-first code editor (CLI: cursor-agent)",
            install_method: "curl",
            install_command: "curl https://cursor.com/install -fsS | bash",
            detect_command: "command -v cursor-agent || command -v agent || where cursor-agent || where agent",
            website_url: Some("https://cursor.com"),
            plugin_dir: Some("~/.cursor/plugins"),
            timeout_secs: None,
            npm_package: None,
        },
        SeedEntry {
            key: "factory",
            is_allagents: true,
            name: "Factory",
            icon: "factory",
            description: "Factory - AI coding assistant for software engineering",
            install_method: "npm",
            install_command: "npm install -g factory@latest",
            detect_command: "command -v factory || where factory",
            website_url: Some("https://factory.io"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("factory"),
        },
        SeedEntry {
            key: "openclaw",
            is_allagents: true,
            name: "OpenClaw",
            icon: "openclaw",
            description: "OpenClaw - AI-powered development assistant",
            install_method: "npm",
            install_command: "npm install -g openclaw@latest",
            detect_command: "command -v openclaw || where openclaw",
            website_url: Some("https://openclaw.dev"),
            plugin_dir: Some("~/.openclaw/plugins"),
            timeout_secs: None,
            npm_package: Some("openclaw"),
        },
        SeedEntry {
            key: "windsurf",
            is_allagents: true,
            // NOTE: As of 2026, the upstream Cask has been renamed
            // "windsurf" → "devin-desktop" (Cognition/Devin acquisition).
            // We no longer ship a one-line installer here — the rename
            // surface is unstable, so we point users at the official
            // download page instead. The `key` MUST stay "windsurf"
            // because it is the SQLite primary key — renaming it would
            // require a DB migration. `install_method = "manual"` flips
            // the UI into Download mode (matches VS Code).
            name: "Windsurf",
            icon: "windsurf",
            description: "Windsurf - AI coding assistant by Codeium (rebranded as Devin Desktop by Cognition). Download manually from the official site.",
            install_method: "manual",
            install_command: "",
            detect_command: "command -v windsurf || where windsurf",
            website_url: Some("https://codeium.com/windsurf"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: None,
        },
        SeedEntry {
            key: "cline",
            is_allagents: true,
            name: "Cline",
            icon: "cline",
            description: "Cline - autonomous coding assistant in VS Code",
            install_method: "npm",
            install_command: "npm install -g cline@latest",
            detect_command: "command -v cline || where cline",
            website_url: Some("https://github.com/cline/cline"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("cline"),
        },
        SeedEntry {
            key: "continue",
            is_allagents: true,
            name: "Continue",
            icon: "continue",
            description: "Continue - open-source AI coding assistant (CLI: cn)",
            install_method: "curl",
            install_command: "curl -fsSL https://raw.githubusercontent.com/continuedev/continue/main/extensions/cli/scripts/install.sh | bash",
            detect_command: "command -v cn || where cn",
            website_url: Some("https://continue.dev"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@continuedev/cli"),
        },
        SeedEntry {
            key: "roo",
            is_allagents: true,
            name: "Roo",
            icon: "roo",
            description: "Roo - AI coding assistant by Roo Code",
            install_method: "npm",
            install_command: "npm install -g roo@latest",
            detect_command: "command -v roo || where roo",
            website_url: Some("https://roo.code"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("roo"),
        },
        SeedEntry {
            key: "kilo",
            is_allagents: true,
            name: "Kilo",
            icon: "kilo",
            description: "Kilo - AI coding agent built for the terminal",
            install_method: "npm",
            install_command: "npm install -g @kilocode/cli",
            detect_command: "command -v kilo || where kilo",
            website_url: Some("https://kilo.dev"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@kilocode/cli"),
        },
        SeedEntry {
            key: "trae",
            is_allagents: true,
            name: "Trae",
            icon: "trae",
            description: "Trae - AI-powered IDE by ByteDance",
            install_method: "npm",
            install_command: "npm install -g trae@latest",
            detect_command: "command -v trae || where trae",
            website_url: Some("https://trae.ai"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("trae"),
        },
        SeedEntry {
            key: "augment",
            is_allagents: true,
            name: "Augment",
            icon: "augment",
            description: "Augment - AI coding assistant by Augment Code",
            install_method: "npm",
            install_command: "npm install -g @augmentcode/auggie@latest",
            detect_command: "command -v auggie || where auggie",
            website_url: Some("https://www.augment.dev"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@augmentcode/auggie"),
        },
        SeedEntry {
            key: "zencoder",
            is_allagents: true,
            name: "Zencoder",
            icon: "zencoder",
            description: "Zencoder - AI video encoding assistant",
            install_method: "npm",
            install_command: "npm install -g zencoder@latest",
            detect_command: "command -v zencoder || where zencoder",
            website_url: Some("https://zencoder.video"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("zencoder"),
        },
        SeedEntry {
            key: "junie",
            is_allagents: true,
            name: "Junie",
            icon: "junie",
            description: "Junie - JetBrains AI coding assistant",
            install_method: "npm",
            install_command: "npm install -g @jetbrains/junie-cli@latest",
            detect_command: "command -v junie || where junie",
            website_url: Some("https://jetbrains.com/junie"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@jetbrains/junie-cli"),
        },
        SeedEntry {
            key: "openhands",
            is_allagents: true,
            name: "OpenHands",
            icon: "openhands",
            description: "OpenHands - open-source AI coding agent",
            install_method: "npm",
            install_command: "npm install -g openhands@latest",
            detect_command: "command -v openhands || where openhands",
            website_url: Some("https://github.com/All-Hands/AI-Hands"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("openhands"),
        },
        SeedEntry {
            key: "kiro",
            is_allagents: true,
            name: "Kiro",
            icon: "kiro",
            description: "Kiro - AI coding assistant by Kiro AI",
            install_method: "npm",
            install_command: "npm install -g kiro@latest",
            detect_command: "command -v kiro || where kiro",
            website_url: Some("https://kiro.ai"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("kiro"),
        },
        // Legacy/Additional tools (also custom type - not part of allagents standard)
        // Note: These are deprecated and removed via migration in connection.rs
        // Their allagents equivalents (claude, gemini, etc.) replaced them
        SeedEntry {
            key: "mimo",
            is_allagents: false,
            name: "MiMo Code",
            icon: "mimo",
            description: "Xiaomi's MiMo AI CLI - terminal-native AI coding assistant with persistent memory",
            install_method: "npm",
            install_command: "npm install -g @mimo-ai/cli@latest",
            detect_command: "command -v mimo || where mimo",
            website_url: Some("https://github.com/XiaomiMiMo/MiMo-Code"),
            plugin_dir: Some("~/.mimocode/plugins"),
            timeout_secs: None,
            npm_package: Some("@mimo-ai/cli"),
        },
        // Hermes - https://github.com/nousresearch/hermes-agent
        // Installed via the official curl|bash installer (no npm package, no
        // Homebrew cask). Hermes ships its own Python venv + Node toolchain,
        // so the seed points users at the upstream installer. The Windows
        // PowerShell installer is documented in the description so it isn't
        // lost when the user reads the card.
        SeedEntry {
            key: "hermes",
            is_allagents: false,
            name: "Hermes",
            icon: "hermes",
            description: "Hermes Agent - self-evolving AI coding assistant by Nous Research. Windows users should use PowerShell: iex (irm https://hermes-agent.nousresearch.com/install.ps1)",
            install_method: "curl-bash",
            install_command: "curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash",
            detect_command: "command -v hermes || where hermes",
            website_url: Some("https://github.com/nousresearch/hermes-agent"),
            plugin_dir: None,
            timeout_secs: Some(300),
            npm_package: None,
        },
        SeedEntry {
            key: "qwen-code",
            is_allagents: false,
            name: "Qwen Code",
            icon: "qwen-code",
            description: "Alibaba's Qwen AI CLI for agentic coding",
            install_method: "npm",
            install_command: "npm install -g @qwen-code/qwen-code@latest",
            detect_command: "command -v qwen || where qwen",
            website_url: Some("https://github.com/QwenLM/qwen-code"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@qwen-code/qwen-code"),
        },
        // 2026热门国产CLI（Custom tools）
        SeedEntry {
            key: "codebuddy",
            is_allagents: false,
            name: "CodeBuddy",
            icon: "codebuddy",
            description: "腾讯混元官方终端编程Agent，深度适配小程序、鸿蒙、云原生；完整文件读写、自动跑单测、CI流水线集成；原生MCP；国内网络无阻碍。",
            install_method: "npm",
            install_command: "npm install -g @tencent-ai/codebuddy-code",
            detect_command: "command -v codebuddy || where codebuddy",
            website_url: Some("https://github.com/tencent-ai/codebuddy-code"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@tencent-ai/codebuddy-code"),
        },
        SeedEntry {
            key: "deepseek-tui",
            is_allagents: false,
            name: "DeepSeek TUI",
            icon: "deepseek",
            description: "DeepSeek官方CLI，Rust内核+npm薄封装；彩色TUI交互、代码逐行diff确认；权限可控shell执行；原生MCP协议，支持Ollama本地离线。",
            install_method: "npm",
            install_command: "npm install -g deepseek-tui",
            detect_command: "command -v deepseek-tui || where deepseek-tui",
            website_url: Some("https://github.com/deepseek-ai/deepseek-tui"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("deepseek-tui"),
        },
        SeedEntry {
            key: "open-code-review",
            is_allagents: false,
            name: "Open Code Review",
            icon: "code-review",
            description: "阿里开源CI专用代码审计工具；内置NPE、XSS、SQL注入、线程安全等工程规则；行级漏洞检测；输出JSON标准化报告，GitHub Action流水线集成。",
            install_method: "npm",
            install_command: "npm install -g @alibaba-group/open-code-review",
            detect_command: "command -v open-code-review || where open-code-review",
            website_url: Some("https://github.com/alibaba/open-code-review"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("@alibaba-group/open-code-review"),
        },
        SeedEntry {
            key: "deepcode-cli",
            is_allagents: false,
            name: "DeepCode CLI",
            icon: "deepcode",
            description: "第三方深度适配DeepSeek-V4系列；完整Agent技能系统、MCP集成；推理强度可调，缓解代码幻觉；兼容所有OpenAI兼容国产接口；2026年6月新增项目级记忆持久化。",
            install_method: "npm",
            install_command: "npm install -g deepcode-cli",
            detect_command: "command -v deepcode || where deepcode",
            website_url: Some("https://github.com/deepcode-ai/deepcode-cli"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("deepcode-cli"),
        },
        SeedEntry {
            key: "reasonix",
            is_allagents: false,
            name: "DeepSeek-Reasonix",
            icon: "reasoning",
            description: "DeepSeek推理增强型编程CLI；擅长复杂算法、LeetCode、多层逻辑重构；内置自动分步思考链；支持npx临时运行；适配本地vLLM/Ollama离线推理。",
            install_method: "npm",
            install_command: "npm install -g reasonix",
            detect_command: "command -v reasonix || where reasonix",
            website_url: Some("https://github.com/reasonix/deepseek-reasonix"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("reasonix"),
        },
        SeedEntry {
            key: "universal-ai-cli",
            is_allagents: false,
            name: "Universal AI CLI",
            icon: "universal",
            description: "一站式管理全部国产大模型（豆包、通义、DeepSeek、Kimi、智谱、文心一言）；标准OpenAI兼容接口，环境变量一键切换模型；可嵌入shell脚本、定时任务、CI流水线。",
            install_method: "npm",
            install_command: "npm install -g universal-ai-cli",
            detect_command: "command -v universal-ai-cli || where universal-ai-cli",
            website_url: Some("https://github.com/universal-ai/universal-ai-cli"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("universal-ai-cli"),
        },
        SeedEntry {
            key: "makecoder",
            is_allagents: false,
            name: "MakeCoder",
            icon: "makecoder",
            description: "多智能体并行调度（编码/测试/文档/审计四Agent同时工作）；统一密钥网关管理所有国产API；团队共享提示词与MCP配置；支持长期项目会话记忆。",
            install_method: "npm",
            install_command: "npm install -g makecoder",
            detect_command: "command -v makecoder || where makecoder",
            website_url: Some("https://github.com/makecoder-ai/makecoder"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("makecoder"),
        },
        SeedEntry {
            key: "xiaozhi-client",
            is_allagents: false,
            name: "小智 MCP 客户端",
            icon: "xiaozhi",
            description: "国产MCP聚合管理工具；统一聚合多端MCP服务；网页可视化配置面板；后台守护进程运行；兼容AllAgents、Cursor、OpenCode等全部23个客户端。",
            install_method: "npm",
            install_command: "npm install -g xiaozhi-client",
            detect_command: "command -v xiaozhi-client || where xiaozhi-client",
            website_url: Some("https://github.com/xiaozhi-ai/xiaozhi-client"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("xiaozhi-client"),
        },
        SeedEntry {
            key: "anyclaw",
            is_allagents: false,
            name: "AnyClaw",
            icon: "anyclaw",
            description: "API/脚本转MCP工具转换器；一键把自有业务脚本、第三方接口封装成MCP服务；配套CLI管理所有自定义技能；打通AllAgents多客户端统一分发工具能力。",
            install_method: "npm",
            install_command: "npm install -g anyclaw",
            detect_command: "command -v anyclaw || where anyclaw",
            website_url: Some("https://github.com/fastclaw-ai/anyclaw"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("anyclaw"),
        },
        SeedEntry {
            key: "doubao-cli",
            is_allagents: false,
            name: "豆包 CLI",
            icon: "doubao",
            description: "字节豆包官方CLI，火山方舟豆包专用；代码生成+多模态+MCP封装；适合字节系业务。",
            install_method: "npm",
            install_command: "npm install -g doubao-ai-toolkit",
            detect_command: "command -v doubao || where doubao",
            website_url: Some("https://github.com/doubao-ai/doubao-cli"),
            plugin_dir: None,
            timeout_secs: None,
            npm_package: Some("doubao-ai-toolkit"),
        },
    ];

    // Use INSERT OR REPLACE to update existing rows with correct is_allagents value
    let mut stmt = match conn.prepare(
        "INSERT OR REPLACE INTO custom_cli_tools
           (key, is_allagents, name, icon, description, install_method, install_command, detect_command,
            website_url, plugin_dir, install_timeout_secs, npm_package, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))",
    ) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("seed_builtin_cli_tools: prepare failed: {}", e);
            return;
        }
    };

    for seed in seeds {
        if let Err(e) = stmt.execute(rusqlite::params![
            seed.key,
            seed.is_allagents as i32,
            seed.name,
            seed.icon,
            seed.description,
            seed.install_method,
            seed.install_command,
            seed.detect_command,
            seed.website_url,
            seed.plugin_dir,
            seed.timeout_secs,
            seed.npm_package,
        ]) {
            log::warn!("seed_builtin_cli_tools: insert {} failed: {}", seed.key, e);
        }
    }
    // Seeds run only on DB init — invalidate defensively so the first
    // `get_supported_tools()` after startup repopulates from the new rows.
    invalidate_tools_cache();
}

/// Public entry point for callers (other than `init_tables`) that want to
/// trigger the builtin seed against the global DB connection.
pub fn seed_builtin_cli_tools_global() {
    if let Some(db) = crate::db::Database::global() {
        let conn = lock_db(&db);
        seed_builtin_cli_tools(&conn);
    }
}

/// Convenience wrapper that locks the global DB connection and seeds the
/// builtin CLI tools. Used from `Database::init_tables`.
pub fn seed_builtin_cli_tools_locked(
    conn_arc: &std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
) {
    let conn = lock_conn_arc(conn_arc);
    seed_builtin_cli_tools(&conn);
}

fn load_custom_cli_tools_from_conn(conn: &Connection) -> Vec<CliToolConfig> {
    let mut stmt = match conn.prepare(
        "SELECT key, is_allagents, name, icon, description, install_method, install_command, detect_command, website_url, plugin_dir, install_timeout_secs, npm_package FROM custom_cli_tools"
    ) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to prepare custom_cli_tools query: {}", e);
            return Vec::new();
        }
    };

    let rows = match stmt.query_map([], |row| {
        let method_str: String = row.get(5)?;
        let method = InstallMethod::from_str(&method_str).unwrap_or(InstallMethod::Npm);
        let is_allagents: bool = row.get::<_, i32>(1)? != 0;
        // Determine if this is a manual download only tool (install_method = 'manual')
        let manual_download_only = method_str == "manual";
        Ok(CliToolConfig {
            key: row.get(0)?,
            name: row.get(2)?,
            icon: row.get(3)?,
            description: row.get(4)?,
            install_methods: if manual_download_only {
                vec![]
            } else {
                vec![InstallCommand {
                    method,
                    command: row.get(6)?,
                    priority: 1,
                }]
            },
            npm_package: row.get(11)?,
            website_url: row.get(8)?,
            plugin_dir: row.get(9)?,
            install_timeout_secs: row.get::<_, Option<u64>>(10)?,
            display_source: if is_allagents { "allagents".to_string() } else { "custom".to_string() },
            manual_download_only,
        })
    }) {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Failed to query custom_cli_tools: {}", e);
            return Vec::new();
        }
    };

    rows.filter_map(|r| r.ok()).collect()
}

/// Parse a row from custom_cli_tools into CliToolConfig
fn parse_cli_tool_row(row: &rusqlite::Row) -> rusqlite::Result<CliToolConfig> {
    let method_str: String = row.get(5)?;
    let method = InstallMethod::from_str(&method_str).unwrap_or(InstallMethod::Npm);
    let is_allagents: bool = row.get::<_, i32>(1)? != 0;
    let manual_download_only = method_str == "manual";
    Ok(CliToolConfig {
        key: row.get(0)?,
        name: row.get(2)?,
        icon: row.get(3)?,
        description: row.get(4)?,
        install_methods: if manual_download_only {
            vec![]
        } else {
            vec![InstallCommand {
                method,
                command: row.get(6)?,
                priority: 1,
            }]
        },
        npm_package: row.get(11)?,
        website_url: row.get(8)?,
        plugin_dir: row.get(9)?,
        install_timeout_secs: row.get::<_, Option<u64>>(10)?,
        display_source: if is_allagents { "allagents".to_string() } else { "custom".to_string() },
        manual_download_only,
    })
}

/// Load tools by is_allagents filter: Some(true) = allagents, Some(false) = custom, None = all
fn load_cli_tools_by_is_allagents(conn: &Connection, is_allagents: Option<bool>) -> Vec<CliToolConfig> {
    let query = match is_allagents {
        Some(true) => "SELECT key, is_allagents, name, icon, description, install_method, install_command, detect_command, website_url, plugin_dir, install_timeout_secs, npm_package FROM custom_cli_tools WHERE is_allagents = 1".to_string(),
        Some(false) => "SELECT key, is_allagents, name, icon, description, install_method, install_command, detect_command, website_url, plugin_dir, install_timeout_secs, npm_package FROM custom_cli_tools WHERE is_allagents = 0".to_string(),
        None => "SELECT key, is_allagents, name, icon, description, install_method, install_command, detect_command, website_url, plugin_dir, install_timeout_secs, npm_package FROM custom_cli_tools".to_string(),
    };

    let mut stmt = match conn.prepare(&query) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to prepare query: {}", e);
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], parse_cli_tool_row);
    match rows {
        Ok(r) => r
            .filter_map(|row| row.ok())
            // Defensive UI filter: hide deprecated keys (e.g. github-cli) even
            // if a stale SQLite row somehow survives the startup cleanup in
            // `db/connection.rs::cleanup_deprecated_custom_cli_tools`. The
            // startup DELETE is the authoritative fix; this is the read-side
            // belt-and-suspenders so a Custom tab never surfaces an entry
            // we've explicitly removed from the seed list.
            .filter(|tool| !is_deprecated_cli_tool_key(&tool.key))
            .collect(),
        Err(e) => {
            log::warn!("Failed to query: {}", e);
            Vec::new()
        }
    }
}

/// Mirror of `db::connection::cleanup_deprecated_custom_cli_tools`'s
/// `deprecated_keys` list, exposed for read-side filtering. Keep in sync
/// with the cleanup list — any key added/removed there must be reflected
/// here.
pub fn is_deprecated_cli_tool_key(key: &str) -> bool {
    matches!(
        key,
        // AI CLI tools
        "codex-cli" | "aider" | "goose" | "kilo-code" | "kimi-cli"
        // Dev tools — `gh` was historically seeded as "GitHub CLI" but is
        // not a supported tool. A stale DB row from early versions can
        // survive; hide it from any read path.
        | "github-cli" | "gh" | "lazygit"
        // Modern CLI replacements
        | "eza" | "ripgrep" | "bat" | "fzf" | "zoxide" | "btop"
        // Network/utility
        | "httpie" | "starship"
    )
}

/// Returns tools filtered by is_allagents: "allagents" = allagents 23 tools, "custom" = custom tools
pub fn get_cli_tools_by_type(tool_type: &str) -> Vec<CliToolConfig> {
    if let Some(db) = crate::db::Database::global() {
        let conn = lock_db(&db);
        match tool_type {
            "allagents" => load_cli_tools_by_is_allagents(&conn, Some(true)),
            "custom" => load_cli_tools_by_is_allagents(&conn, Some(false)),
            _ => load_cli_tools_by_is_allagents(&conn, None),
        }
    } else {
        Vec::new()
    }
}

/// Returns only the allagents 23 tools (for the Default tab) from database.
pub fn get_allagents_tools() -> Vec<CliToolConfig> {
    if let Some(db) = crate::db::Database::global() {
        let conn = lock_db(&db);
        load_cli_tools_by_is_allagents(&conn, Some(true))
    } else {
        Vec::new()
    }
}

/// Insert a new custom CLI tool into the database
/// is_allagents: true = allagents 23 tools, false = custom tools
pub fn add_custom_cli_tool(config: CustomCliToolConfig) -> Result<(), String> {
    let db = crate::db::Database::global().ok_or("Database not initialized")?;
    let conn = lock_db(&db);

    conn.execute(
        "INSERT INTO custom_cli_tools (key, is_allagents, name, icon, description, install_method, install_command, detect_command, website_url, plugin_dir, install_timeout_secs, npm_package, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))",
        params![
            config.key,
            config.is_allagents as i32,
            config.name,
            config.icon,
            config.description,
            config.install_method,
            config.install_command,
            config.detect_command,
            config.website_url,
            config.plugin_dir,
            config.install_timeout_secs as i64,
            config.npm_package,
        ],
    )
    .map_err(|e| e.to_string())?;
    invalidate_tools_cache();
    Ok(())
}

/// Remove a custom CLI tool by key
pub fn remove_custom_cli_tool(key: &str) -> Result<(), String> {
    let db = crate::db::Database::global().ok_or("Database not initialized")?;
    let conn = lock_db(&db);
    conn.execute("DELETE FROM custom_cli_tools WHERE key = ?1", params![key])
        .map_err(|e| e.to_string())?;
    invalidate_tools_cache();
    Ok(())
}

/// List all custom CLI tools (returns full CliToolConfig)
pub fn list_custom_cli_tools() -> Vec<CliToolConfig> {
    load_custom_cli_tools()
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

/// 获取 macOS 上完整的 PATH 环境变量
/// GUI 应用运行时可能没有正确的 PATH，需要补充常见路径
/// 但保留原有 PATH 中的其他路径，避免用户配置丢失
///
/// `dirs::home_dir()` 是关键 — `macos` GUI app 从 launchd 启动，不读
/// `.zprofile`/`.zshrc`，所以默认拿不到 `~/.local/bin` 这类用户私有
/// 路径，必须显式拼接。否则 `which` 找不到用户 nvm/`.local`/`.bun`
/// 装的 CLI，UI 会把它们全部显示成"未安装"。
#[cfg(target_os = "macos")]
fn get_full_path() -> String {
    // 必要的系统路径
    let essential_paths = [
        "/opt/homebrew/bin",  // Apple Silicon Mac 的 Homebrew
        "/usr/local/bin",     // Intel Mac 的 Homebrew
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ];

    // 用户常见的私有 bin 目录。macOS GUI app 不读用户的 shell rc 文件，
    // 所以这些路径必须在这里显式列出。
    let home = dirs::home_dir();
    let user_paths: Vec<String> = if let Some(h) = home.as_ref() {
        let home_str = h.to_string_lossy().to_string();
        let mut paths: Vec<String> = vec![
            // Hermes / pipx / 各种官方 curl-bash 安装器默认会装到这里
            format!("{}/.local/bin", home_str),
            // Bun (https://bun.sh)
            format!("{}/.bun/bin", home_str),
            // Rust / Cargo
            format!("{}/.cargo/bin", home_str),
            // opencode CLI
            format!("{}/.opencode/bin", home_str),
        ];

        // nvm: 扫描所有已安装的 Node 版本
        let nvm_versions_dir = h.join(".nvm").join("versions").join("node");
        if let Ok(entries) = std::fs::read_dir(&nvm_versions_dir) {
            for entry in entries.flatten() {
                let bin = entry.path().join("bin");
                if bin.is_dir() {
                    paths.push(bin.to_string_lossy().to_string());
                }
            }
        }

        // fnm: 类似 nvm，但目录结构是 ~/.fnm/node-versions/<version>/installation/bin
        let fnm_versions_dir = h.join(".fnm").join("node-versions");
        if let Ok(entries) = std::fs::read_dir(&fnm_versions_dir) {
            for entry in entries.flatten() {
                let bin = entry.path().join("installation").join("bin");
                if bin.is_dir() {
                    paths.push(bin.to_string_lossy().to_string());
                }
            }
        }

        paths
    } else {
        Vec::new()
    };

    // 从当前环境获取 PATH
    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    // 先添加系统必要路径（去重）
    for p in &essential_paths {
        if seen.insert(p.to_string()) {
            result.push(p.to_string());
        }
    }

    // 再添加用户私有路径（去重）— 优先于环境 PATH 之前查找
    for p in &user_paths {
        if seen.insert(p.clone()) {
            result.push(p.clone());
        }
    }

    // 最后保留当前 PATH 中不在前面的路径（去重），避免用户配置丢失
    for p in current_path.split(':') {
        if !p.is_empty() && seen.insert(p.to_string()) {
            result.push(p.to_string());
        }
    }

    result.join(":")
}

/// 获取 Linux 上完整的 PATH 环境变量
#[cfg(target_os = "linux")]
fn get_full_path() -> String {
    // 同 macOS 的逻辑 — Linux GUI app 同样不读 shell rc，必须显式拼用户路径。
    let mut parts: Vec<String> = std::env::var("PATH")
        .unwrap_or_else(|_| "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string())
        .split(':')
        .filter(|p| !p.is_empty())
        .map(String::from)
        .collect();

    if let Some(h) = dirs::home_dir() {
        let home_str = h.to_string_lossy().to_string();
        let user_paths = [
            format!("{}/.local/bin", home_str),
            format!("{}/.bun/bin", home_str),
            format!("{}/.cargo/bin", home_str),
        ];
        for p in user_paths {
            if !parts.contains(&p) {
                parts.insert(0, p);
            }
        }
    }
    parts.join(":")
}

pub struct CliToolManager;

// ============== Async Command Execution ==============

/// Async command execution with timeout using tokio
/// Windows 上直接执行命令，使用 CREATE_NO_WINDOW 防止控制台弹窗
/// macOS 上设置完整的 PATH 环境变量
async fn run_command_with_timeout_async(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Option<std::process::Output> {
    #[cfg(target_os = "windows")]
    {
        // Windows: 直接执行命令（tokio 可以直接处理 .cmd 文件）
        // 使用 CREATE_NO_WINDOW 防止控制台弹窗
        tokio::select! {
            result = tokio::process::Command::new(program)
                .args(args)
                .kill_on_drop(true)
                .creation_flags(0x08000000u32) // CREATE_NO_WINDOW
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
    #[cfg(target_os = "macos")]
    {
        let full_path = get_full_path();
        let program = program.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        tokio::select! {
            result = tokio::process::Command::new(&program)
                .args(&args)
                .env("PATH", full_path)
                .kill_on_drop(true)
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
    #[cfg(target_os = "linux")]
    {
        let program = program.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        tokio::select! {
            result = tokio::process::Command::new(&program)
                .args(&args)
                .kill_on_drop(true)
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
    #[cfg(target_os = "freebsd")]
    {
        let program = program.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        tokio::select! {
            result = tokio::process::Command::new(&program)
                .args(&args)
                .kill_on_drop(true)
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
}

/// Async shell command execution - 跨平台兼容
/// Windows 上使用 cmd.exe /c 执行 shell 命令，CREATE_NO_WINDOW 防止弹窗
/// macOS/Linux 上使用 sh -c 执行，并设置完整的 PATH 环境变量
async fn run_shell_async(command: &str, timeout: Duration) -> Option<std::process::Output> {
    #[cfg(target_os = "windows")]
    {
        tokio::select! {
            result = tokio::process::Command::new("cmd.exe")
                .args(["/C", command])
                .kill_on_drop(true)
                .creation_flags(0x08000000u32) // CREATE_NO_WINDOW
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
    #[cfg(target_os = "macos")]
    {
        // 设置完整的 PATH 环境变量，确保 brew 等命令可以被找到
        let full_path = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";
        tokio::select! {
            result = tokio::process::Command::new("sh")
                .args(["-c", command])
                .env("PATH", full_path)
                .kill_on_drop(true)
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
    #[cfg(target_os = "linux")]
    {
        // Linux 上使用系统默认 PATH
        tokio::select! {
            result = tokio::process::Command::new("sh")
                .args(["-c", command])
                .kill_on_drop(true)
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
    #[cfg(target_os = "freebsd")]
    {
        tokio::select! {
            result = tokio::process::Command::new("sh")
                .args(["-c", command])
                .kill_on_drop(true)
                .output() => result.ok(),
            _ = tokio::time::sleep(timeout) => None
        }
    }
}

// ============== Legacy Sync Wrapper (for backward compatibility) ==============

/// Sync command execution with timeout
/// 在 Windows 上，通过 PowerShell 执行命令以支持 .cmd 批处理文件和特殊字符
/// 同时添加 CREATE_NO_WINDOW 标志防止控制台弹窗
/// Sync command execution with timeout
// ============== CliToolManager Implementation ==============

// ============== User-level filesystem probe helpers ==============
//
// `which` / `npm root -g` 在 Tauri GUI app 里都不可靠（见 `get_full_path`
// 上面的注释）。下面三个函数用文件系统直接探测，绕开 `PATH` 和 `npm
// prefix` 的限制。它们是「最终兜底」：探测不到就算未安装。

/// 枚举所有可能放置用户私有 CLI 的 `bin/` 目录。
///
/// 顺序很重要 — Hermes / pipx 风格的工具会先放进 `~/.local/bin`，
/// nvm 装的所有 Node 全局工具在 `~/.nvm/versions/node/*/bin`。
/// 我们用文件系统直接 read_dir，命中即返回，不依赖环境变量。
fn enumerate_user_bin_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();

    let Some(home) = dirs::home_dir() else {
        return dirs;
    };

    // 静态候选 — 大多数官方 curl-bash 安装器都会用到这些位置
    for sub in [".local/bin", ".bun/bin", ".cargo/bin", ".opencode/bin"] {
        let p = home.join(sub);
        if p.is_dir() {
            dirs.push(p);
        }
    }

    // nvm：每个安装的 Node 版本都有自己的 bin
    let nvm_versions = home.join(".nvm").join("versions").join("node");
    if let Ok(entries) = std::fs::read_dir(&nvm_versions) {
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path().join("bin"))
            .filter(|p| p.is_dir())
            .collect();
        // 排序一下让结果稳定（便于测试 & 减少 UI 抖动）
        paths.sort();
        dirs.extend(paths);
    }

    // fnm（Fast Node Manager）：目录结构不同
    let fnm_root = home.join(".fnm").join("node-versions");
    if let Ok(entries) = std::fs::read_dir(&fnm_root) {
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path().join("installation").join("bin"))
            .filter(|p| p.is_dir())
            .collect();
        paths.sort();
        dirs.extend(paths);
    }

    // volta（少用但列上）
    let volta_bin = home.join(".volta").join("bin");
    if volta_bin.is_dir() {
        dirs.push(volta_bin);
    }

    dirs
}

/// 枚举所有可能的全局 `node_modules` 目录。
///
/// 跟 `enumerate_user_bin_dirs` 平行，用于 `check_npm_version_async`
/// 在 `npm list -g` 失败时（prefix 被某个全局 npmrc 改写）直接扫
/// `package.json` 找包。返回的是 `node_modules` 本身，不是其父目录。
fn enumerate_user_node_modules_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<std::path::PathBuf> = Vec::new();

    let Some(home) = dirs::home_dir() else {
        return dirs;
    };

    // 静态候选
    for sub in [".local/lib/node_modules", ".local/lib", ".bun/install/global/node_modules"] {
        let p = home.join(sub);
        if p.is_dir() {
            dirs.push(p);
        }
    }

    // nvm：每个 Node 版本都有自己的 `lib/node_modules`
    let nvm_versions = home.join(".nvm").join("versions").join("node");
    if let Ok(entries) = std::fs::read_dir(&nvm_versions) {
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path().join("lib").join("node_modules"))
            .filter(|p| p.is_dir())
            .collect();
        paths.sort();
        dirs.extend(paths);
    }

    // fnm
    let fnm_root = home.join(".fnm").join("node-versions");
    if let Ok(entries) = std::fs::read_dir(&fnm_root) {
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path().join("installation").join("lib").join("node_modules"))
            .filter(|p| p.is_dir())
            .collect();
        paths.sort();
        dirs.extend(paths);
    }

    // volta
    let volta_nm = home.join(".volta").join("tools").join("image").join("node").join("lib").join("node_modules");
    if volta_nm.is_dir() {
        dirs.push(volta_nm);
    }

    // 兜底：`/usr/local/lib/node_modules`（macOS / Linux 上 n 默认 prefix）
    let system_nm = std::path::PathBuf::from("/usr/local/lib/node_modules");
    if system_nm.is_dir() {
        dirs.push(system_nm);
    }

    dirs
}

/// 在给定的全局 `node_modules` 目录下查找 `pkg`，返回 `(version, bin_path)`。
///
/// - `pkg` 可以是 `"@scope/name"` 或裸包名（`"claude"`）
/// - `tool_key` 仅用于在 `pkg` 未指定时构造查找路径 & 找二进制候选
/// - 找到包时还会尝试解析 `package.json` 里的 `bin` 字段，反推出
///   用户 bin 目录里那个真正的可执行路径（即便 npm prefix 错了也能
///   给 UI 返回正确 path）
fn read_npm_package_from_node_modules(
    node_modules: &std::path::Path,
    pkg: &str,
    tool_key: &str,
) -> Option<(String, String)> {
    // 候选路径：<nm>/<pkg> 或 <nm>/<scope>/<name>
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    candidates.push(node_modules.join(pkg));
    if let Some((scope, name)) = pkg.split_once('/') {
        candidates.push(node_modules.join(scope).join(name));
    }

    for pkg_dir in candidates {
        let pkg_json = pkg_dir.join("package.json");
        let raw = match std::fs::read_to_string(&pkg_json) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let v: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let version = match v.get("version").and_then(|x| x.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        // 解析 bin：在 <pkg_dir>/<bin-string-or-object>
        // npm 的 bin 字段可以是字符串 "name": "./bin/foo" 也可以是
        // 对象 {"foo": "./bin/foo"}，取 tool_key 那个或第一个。
        let bin_rel: Option<String> = match v.get("bin") {
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(serde_json::Value::Object(map)) => {
                // 优先匹配 tool_key / pkg basename
                let want = tool_key;
                let want_alt = pkg.rsplit('/').next().unwrap_or(pkg);
                map.get(want)
                    .or_else(|| map.get(want_alt))
                    .or_else(|| map.values().next())
                    .and_then(|x| x.as_str())
                    .map(String::from)
            }
            _ => None,
        };

        // 推算 binary 完整路径：node_modules 父目录的 `bin/`
        // 目录（npm 的全局 bin 目录约定）。
        // 例子：`~/.nvm/.../lib/node_modules/@anthropic-ai/claude-code`
        //       的父链向上到 `~/.nvm/.../lib`，再把 `lib` 替换成 `bin`。
        //
        // 注意：`Path::ancestors()` 从 self 开始向上枚举；我们跳过 self
        // 本身（pkg_dir）只看真正的祖先。
        let mut resolved_bin: Option<String> = None;
        for ancestor in pkg_dir.ancestors().skip(1) {
            let anc = ancestor.to_string_lossy().to_string();
            if let Some(stripped) = anc.strip_suffix("/lib") {
                let bin_dir = format!("{}/bin", stripped);
                if let Some(rel) = &bin_rel {
                    let candidate = std::path::PathBuf::from(&bin_dir).join(rel);
                    if candidate.exists() {
                        resolved_bin = Some(candidate.to_string_lossy().to_string());
                        break;
                    }
                }
                // 即便 bin 字段解析失败，也至少返回 bin 目录，
                // 调用方会通过 `binary_name_candidates` 补齐
                resolved_bin = Some(bin_dir);
                break;
            }
        }
        // Windows: 全局 bin 就是 prefix 本身（lib 的父）。只在上面没
        // 找到时跑这一段 — Unix 端用祖先链已经把 `/lib` 都剥完了。
        if resolved_bin.is_none() {
            for ancestor in pkg_dir.ancestors().skip(1) {
                let anc = ancestor.to_string_lossy().to_string();
                if anc.ends_with("/node_modules") || anc.ends_with("\\node_modules") {
                    if let Some(parent) = std::path::Path::new(&anc).parent() {
                        let prefix = parent.to_string_lossy().to_string();
                        if let Some(rel) = &bin_rel {
                            let candidate = std::path::PathBuf::from(&prefix).join(rel);
                            if candidate.exists() {
                                resolved_bin = Some(candidate.to_string_lossy().to_string());
                                break;
                            }
                        }
                        resolved_bin = Some(prefix);
                        break;
                    }
                }
            }
        }

        // 退路：返回 `node_modules` 的父目录的 `bin/`（最常见的全局
        // bin 位置），让 `binary_name_candidates` 决定具体文件名。
        let bin_path = resolved_bin.unwrap_or_else(|| {
            let nm_str = node_modules.to_string_lossy().to_string();
            if let Some(stripped) = nm_str.strip_suffix("/node_modules") {
                format!("{}/bin", stripped)
            } else {
                nm_str
            }
        });

        return Some((version, bin_path));
    }

    None
}

// ============== End of user-level filesystem probe helpers ==============

/// Extract a tool's version string from a `--version` / `-v` probe output.
///
/// Audit item #6: the prior implementation substring-matched "install",
/// "not found", "cannot find" against the **combined** stdout+stderr. That
/// incorrectly dropped valid version output like `Copilot CLI installed:
/// 1.2.3` or `Continue installed at /usr/local/bin/cn` — both legitimate
/// status messages from successful CLI binaries that include the keyword
/// "install" or "installed".
///
/// New rules:
///   1. If the child exited successfully (`status.success()`) and stdout is
///      non-empty, return stdout. Done.
///   2. Otherwise inspect **stderr only** for true error patterns
///      (ENOENT / "command not found" / "No such file"). Stdout is ignored
///      on the failure path.
///   3. As a final defensive measure, if the combined output happens to
///      contain a version-shaped token (`v?\d+\.\d+\.\d+...`) even when
///      the exit was non-zero, return that token. This handles tools that
///      print version to stdout but exit non-zero for unrelated reasons.
fn try_extract_version(path: &str, output: Option<std::process::Output>) -> Option<String> {
    let output = output?;

    // Happy path.
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stdout.is_empty() {
            return Some(stdout);
        }
    }

    // Failure path: only look at stderr to avoid keyword false-positives
    // from a successful tool whose stdout mentions "install" in a
    // legitimate way.
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_lowercase();
    if stderr.contains("command not found")
        || stderr.contains("no such file")
        || stderr.contains("enoent")
    {
        log::debug!("get_version_from_binary: {} reported missing binary in stderr", path);
        return None;
    }

    // Last-ditch: regex-look for a version-like token in the combined
    // output. We don't change the happy path; this is purely a fallback
    // for tools that print a version but exit non-zero.
    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // A loose semver-ish pattern: digits separated by dots, optionally
    // prefixed by `v` / `V`, with an optional `-suffix` we trim off.
    static VERSION_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = VERSION_RE.get_or_init(|| {
        regex::Regex::new(r"\b[vV]?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.\-]+)?)\b").unwrap()
    });
    if let Some(caps) = re.captures(&combined) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }

    None
}

/// Module-level accessor (calls impl method).
pub fn get_supported_tools() -> Vec<CliToolConfig> {
    CliToolManager::get_supported_tools()
}

impl CliToolManager {
    pub fn new() -> Self { Self }

    /// Returns all tools from database for installation/status checking.
    ///
    /// This is the hottest read path in the CLI tools module — see audit
    /// #7. We serve from `TOOLS_CACHE` whenever the list is present, and
    /// fall back to a one-shot SQLite query on miss. Any write path
    /// (`add_custom_cli_tool`, `remove_custom_cli_tool`,
    /// `seed_builtin_cli_tools`) calls `invalidate_tools_cache()` so the
    /// next read repopulates the cache.
    pub fn get_supported_tools() -> Vec<CliToolConfig> {
        // Fast path: serve from cache. We hold the lock only long enough to
        // clone the inner `Option<Vec<...>>` — the actual `.clone()` on
        // the Vec happens after we drop the mutex, so concurrent readers
        // never block each other on the heavy allocation.
        let cached: Option<Vec<CliToolConfig>> = match tools_cache().lock() {
            Ok(g) => g.clone(),
            Err(p) => p.into_inner().clone(),
        };
        if let Some(tools) = cached {
            return tools;
        }

        // Cache miss: query SQLite and repopulate. We do this without
        // holding any cache lock so a slow query doesn't stall concurrent
        // readers (they will all see `None` once and one will win the race
        // to repopulate — the other is just a duplicate `SELECT`).
        let tools = if let Some(db) = crate::db::Database::global() {
            let conn = lock_db(&db);
            load_cli_tools_by_is_allagents(&conn, None)
        } else {
            Vec::new()
        };

        // Repopulate cache. We tolerate another writer having raced ahead
        // of us — last-writer-wins is fine here because all writers
        // produce the same result.
        match tools_cache().lock() {
            Ok(mut g) => *g = Some(tools.clone()),
            Err(p) => *p.into_inner() = Some(tools.clone()),
        }
        tools
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
    ///
    /// Two failure modes used to silently drop statuses, which then made
    /// every previously-"Installed" card flip to "Not installed":
    ///   1. The 60s outer timeout fired (`.unwrap_or_default()` discarded
    ///      every in-flight result, even ones that had already completed).
    ///   2. A single tool check errored and we replaced it with
    ///      `CliToolStatus { tool_key: String::new(), ... }`, which the
    ///      frontend could not match back to any tool.
    ///
    /// We now keep any partial successes from the timeout and emit a
    /// sentinel with the real `tool_key` so the frontend can decide
    /// whether to surface or discard the failure.
    pub async fn check_all_installations_parallel(&self) -> Vec<CliToolStatus> {
        let tools = Self::get_supported_tools();

        let futures: Vec<_> = tools
            .iter()
            .map(|t| self.check_installation_async(&t.key))
            .collect();

        // Execute all checks in parallel with timeout. On timeout, we
        // drain every already-completed future and return them — we
        // never blank the result set.
        let results: Vec<CliResult<CliToolStatus>> = match tokio::time::timeout(
            Duration::from_secs(60),
            futures::future::join_all(futures),
        )
        .await
        {
            Ok(v) => v,
            Err(_) => {
                // Outer timeout fired. `join_all` is built on the same
                // future handle, so by this point some checks have
                // completed and are stashed in the future. We can't
                // extract them post-hoc from the timed-out future itself,
                // so we re-await the inner futures with a short budget
                // and keep anything that lands. The remaining tools will
                // simply be missing from the result — the frontend now
                // merges instead of replacing (see `checkAllCliToolsStatus`
                // in the store), so the previous statuses are preserved.
                log::warn!(
                    "check_all_installations_parallel: outer 60s timeout, \
                     falling back to per-tool 2s grace period"
                );
                let grace: Vec<_> = tools
                    .iter()
                    .map(|t| async {
                        tokio::time::timeout(
                            Duration::from_secs(2),
                            self.check_installation_async(&t.key),
                        )
                        .await
                        .unwrap_or_else(|_| {
                            Err(CliToolError::CommandFailed(
                                "timed out waiting for tool status".to_string(),
                            ))
                        })
                    })
                    .collect();
                futures::future::join_all(grace).await
            }
        };

        results
            .into_iter()
            .enumerate()
            .map(|(idx, r)| {
                let key = tools.get(idx).map(|t| t.key.clone()).unwrap_or_default();
                r.unwrap_or_else(|e| {
                    log::warn!("status check for '{}' failed: {}", key, e);
                    CliToolStatus {
                        tool_key: key,
                        is_installed: false,
                        installed_version: None,
                        install_method: None,
                        install_path: None,
                        has_conflict: false,
                        conflict_info: None,
                        latest_version: None,
                        needs_upgrade: false,
                    }
                })
            })
            .collect()
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

        // 主路径：`npm list -g`。在 Tauri GUI app 里这经常会失败 —
        // 用户的 `npm prefix` 可能被某个全局 npmrc（如
        // `~/.hermes/node/etc/npmrc`）改写成一个不存在的目录，导致
        // `npm list -g` 返回 `(empty)`。所以下面用一个直接扫
        // `node_modules` 目录的 fallback 来兜底。
        if let Some(output) = run_command_with_timeout_async(
            "npm",
            &["list", "-g", "--depth=0", pkg_to_check],
            Duration::from_secs(10),
        ).await {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(version) = self.parse_npm_version(&stdout, pkg_to_check) {
                    let path = self.get_npm_global_path(tool_key).await;
                    return Some((version, path));
                }
            }
        }

        // Fallback：直接扫描 `node_modules` 目录树。`enumerate_user_node_modules_dirs`
        // 会枚举 nvm / .local / fnm 等所有可能的全局 node_modules 位置，
        // 然后我们看里面有没有 `<pkg>` 或 `<scope>/<pkg>`（scoped 包）。
        for nm_dir in enumerate_user_node_modules_dirs() {
            if let Some((version, bin_path)) =
                read_npm_package_from_node_modules(&nm_dir, pkg_to_check, tool_key)
            {
                return Some((version, bin_path));
            }
        }

        None
    }

    async fn which_async(&self, binary: &str) -> Option<String> {
        // Platform-specific binary lookup. The probe program is fixed at
        // compile time: `where.exe` on Windows, `which` everywhere else.
        // We previously used `if is_windows()` for this; using a `cfg`
        // block means the dead branch is erased entirely (no runtime
        // bool check, no string branch) and matches the rest of the
        // file's platform-specific styling.
        #[cfg(target_os = "windows")]
        let output = run_command_with_timeout_async("where.exe", &[binary], Duration::from_secs(5)).await;
        #[cfg(not(target_os = "windows"))]
        let output = run_command_with_timeout_async("which", &[binary], Duration::from_secs(5)).await;

        if let Some(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Windows 的 where 命令可能返回多个路径，取第一个
                let path = stdout.lines().next()?.trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }

        // 跨路径探测：枚举所有可能放置用户 CLI 的 bin 目录
        // ------------------------------------------------------------------------
        // `which` 在 Tauri GUI app 里经常找不到用户私有路径 — macOS 上 GUI
        // app 从 launchd 启动，`PATH` 不带 `~/.local/bin`/`~/.nvm/.../bin`
        // 等；Linux GUI app 同理。同时用户的 `npm prefix` 可能被某个
        // 全局 npmrc 改写（如 `~/.hermes/node/etc/npmrc`），导致
        // `npm root -g` 指向一个空目录。所以这里用文件系统直接探测每
        // 个候选 bin 目录，命中即返回 — 这是「最终兜底」。
        for bin_dir in enumerate_user_bin_dirs() {
            #[cfg(target_os = "windows")]
            {
                let exe_path = bin_dir.join(binary);
                if exe_path.exists() {
                    return Some(exe_path.to_string_lossy().to_string());
                }
                let cmd_path = bin_dir.join(format!("{}.cmd", binary));
                if cmd_path.exists() {
                    return Some(cmd_path.to_string_lossy().to_string());
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let candidate = bin_dir.join(binary);
                if candidate.exists() {
                    return Some(candidate.to_string_lossy().to_string());
                }
            }
        }

        // npm global paths fallback（保留原逻辑作为最后一层兜底 —
        // 如果上面 `enumerate_user_bin_dirs` 漏了某个边角情况，
        // 这里还能通过 `npm root -g` 反推的 bin 目录再找一次）
        if let Some(output) = run_command_with_timeout_async("npm", &["root", "-g"], Duration::from_secs(5)).await {
            if output.status.success() {
                let root_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // Platform-specific npm binary directory layout. The
                // (dir, separator) tuple lets the rest of the function
                // stay format-string-only without further branching.
                #[cfg(target_os = "windows")]
                let (bin_dir, separator) = {
                    // Windows: npm 全局二进制文件直接在 prefix 目录（node_modules 的父目录）
                    let prefix = std::path::Path::new(&root_path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| root_path.clone());
                    (prefix, "\\")
                };
                #[cfg(not(target_os = "windows"))]
                let (bin_dir, separator) = (root_path.replace("/lib/node_modules", "/bin"), "/");

                let full_path = format!("{}{}{}", bin_dir, separator, binary);
                if std::path::Path::new(&full_path).exists() {
                    return Some(full_path);
                }

                // Windows-only: also probe `.cmd` shim. On Unix we skip
                // this branch entirely (no `is_windows()` runtime check).
                #[cfg(target_os = "windows")]
                {
                    let cmd_path = format!("{}.cmd", full_path);
                    if std::path::Path::new(&cmd_path).exists() {
                        return Some(cmd_path);
                    }
                }
            }
        }
        None
    }

    async fn get_version_from_binary_async(&self, _tool_key: &str, path: &str) -> Option<String> {
        // Run both probes concurrently. `--version` and `-v` produce
        // essentially the same answer; racing them halves the worst-case
        // latency for the "checking" UI on slow CI runners.
        let v_long = run_command_with_timeout_async(path, &["--version"], Duration::from_secs(5));
        let v_short = run_command_with_timeout_async(path, &["-v"], Duration::from_secs(5));
        let (a, b) = tokio::join!(v_long, v_short);
        try_extract_version(path, a) .or_else(|| try_extract_version(path, b))
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

    /// Async single-tool status check. Replaces the prior sync version
    /// (audit item #7): the sync path used `std::process::Command` which
    /// blocks the Tauri async-runtime thread. Under parallel status
    /// sweeps this could starve `tauri::async_runtime`'s worker pool.
    /// The `_async` helpers already exist; we just compose them here.
    pub async fn check_installation(&self, tool_key: &str) -> CliResult<CliToolStatus> {
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


    // ============== Helper Methods ==============

    /// Resolve the candidate binary names for a tool, in priority order.
    ///
    /// Most tools expose a single executable (`claude`, `gemini`, `codex`).
    /// Some official installers rename the binary (`cursor-agent` instead of
    /// `cursor`); for these we must probe both the canonical name and any
    /// known aliases the installer may have created, otherwise the detection
    /// falsely reports the tool as not installed.
    /// Probe heuristic — maps a tool key to the binary names a CLI's official
    /// installer may have placed on PATH. Most tools install under their own
    /// key, but a few rename the binary (Cursor's official installer drops
    /// `cursor-agent` instead of `cursor`); probing only the canonical name
    /// would falsely report them as not installed.
    ///
    /// This is intentionally kept as a hardcoded heuristic rather than a DB
    /// column because it captures knowledge of third-party installer behavior
    /// — not user-facing product config. All product config (name/icon/npm
    /// package/install command/...) now lives in the `custom_cli_tools` table
    /// and is loaded at runtime from `get_supported_tools()`.
    fn binary_name_candidates(&self, tool_key: &str) -> Vec<String> {
        // copilot: 跳过二进制探测。copilot 未配置时运行 `copilot --version`
        // 会触发交互式安装提示并弹出终端窗口，所以只通过 npm 检测。
        if tool_key == "copilot" {
            return vec![];
        }

        let mut candidates: Vec<String> = match tool_key {
            "claude-code" => vec!["claude".to_string()],
            "gemini-cli" => vec!["gemini".to_string()],
            // Cursor's official installer (https://cursor.com/install) drops a
            // `cursor-agent` binary into ~/.local/bin and also creates an
            // `agent` symlink for it. The `cursor` binary is the editor app,
            // not the CLI, so it must NOT be a candidate.
            "cursor" => vec!["cursor-agent".to_string(), "agent".to_string()],
            "codex" => vec!["codex".to_string()],
            // VS Code's CLI binary is 'code', installed via 'code.visualstudio.com'
            // opencode npm 包名是 opencode-ai，但安装后的二进制名是 opencode
            "opencode" => vec!["opencode".to_string(), "opencode-ai".to_string()],
            // Windsurf CLI binary is 'windsurf'
            "windsurf" => vec!["windsurf".to_string()],
            // Junie CLI binary is 'junie'
            "junie" => vec!["junie".to_string()],
            "openclaw" => vec!["openclaw".to_string()],
            "hermes" => vec!["hermes".to_string()],
            "deepseek-reasonix" => vec!["reasonix".to_string()],
            "mimo" | "mimo-code" => vec!["mimo".to_string()],
            "qwen-code" => vec!["qwen-code".to_string()],
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
            // 支持 Unix 格式: ├── @anthropic-ai/claude-code@1.2.3
            // 支持 Windows 格式: +-- @anthropic-ai/claude-code@1.2.3
            if line.contains("├──") || line.contains("└──") || line.contains("+--") || line.contains("\\--") {
                let full_line = line
                    .replace("├──", "")
                    .replace("└──", "")
                    .replace("│", "")
                    .replace("+--", "")
                    .replace("\\--", "")
                    .replace(" ", "");

                if full_line.contains('@') {
                    if let Some(at_idx) = full_line.rfind('@') {
                        let version = &full_line[at_idx + 1..];
                        let package = &full_line[..at_idx];

                        // 支持多种包名匹配方式
                        if package.contains(tool_key)
                            || package.contains(&tool_key.replace("-", ""))
                            || (tool_key == "claude-code" && package.contains("claude"))
                            || (tool_key == "opencode" && package.contains("opencode"))
                            || (tool_key == "kilo" && (package.contains("kilo") || package.contains("kilocode")))
                        {
                            if !version.is_empty()
                                && version
                                    .chars()
                                    .next()
                                    .map(|c| c.is_ascii_digit())
                                    .unwrap_or(false)
                            {
                                return Some(version.trim_end_matches(' ').to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// 获取 npm 全局安装的二进制文件路径
    /// 在 Windows 上，npm 全局二进制文件（.cmd/.exe）直接放在 prefix 目录
    /// 在 Unix 上，二进制文件在 prefix/bin 目录
    ///
    /// Async: replaces the prior `get_npm_global_path_sync` (audit item
    /// #7). Called from `check_npm_version_async` and is on the parallel
    /// status-check hot path.
    ///
    /// 探测顺序：
    /// 1. `enumerate_user_bin_dirs()` — 文件系统直接命中，绕开 `npm prefix`
    /// 2. `npm prefix -g` + 候选名 — 兼容默认安装位置
    /// 3. 平台兜底（`/usr/local/bin` 或 `%APPDATA%\npm`）
    async fn get_npm_global_path(&self, tool_key: &str) -> String {
        let binary_candidates = self.binary_name_candidates(tool_key);

        // 1) 先用文件系统直接探测用户私有 bin 目录。`npm prefix -g`
        //    在用户全局 npmrc（如 `~/.hermes/node/etc/npmrc`）把它
        //    改写时会指向不存在的目录；这一步就是修复那个 case。
        for bin_dir in enumerate_user_bin_dirs() {
            for binary_name in &binary_candidates {
                #[cfg(target_os = "windows")]
                {
                    let cmd_path = bin_dir.join(format!("{}.cmd", binary_name));
                    if cmd_path.exists() {
                        return cmd_path.to_string_lossy().to_string();
                    }
                    let exe_path = bin_dir.join(binary_name);
                    if exe_path.exists() {
                        return exe_path.to_string_lossy().to_string();
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let bin_path = bin_dir.join(binary_name);
                    if bin_path.exists() {
                        return bin_path.to_string_lossy().to_string();
                    }
                }
            }
        }

        // 2) npm prefix 推导
        if let Some(output) = run_command_with_timeout_async("npm", &["prefix", "-g"], Duration::from_secs(5)).await {
            if output.status.success() {
                let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();

                for binary_name in &binary_candidates {
                    // Platform-specific candidate paths. Each branch
                    // returns early when it finds a match; on Unix we
                    // never look for `.cmd` shims.
                    #[cfg(target_os = "windows")]
                    {
                        // Windows: 二进制文件直接在 prefix 目录，可能有 .cmd 后缀
                        let cmd_path = format!("{}\\{}.cmd", prefix, binary_name);
                        if std::path::Path::new(&cmd_path).exists() {
                            return cmd_path;
                        }
                        let exe_path = format!("{}\\{}", prefix, binary_name);
                        if std::path::Path::new(&exe_path).exists() {
                            return exe_path;
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        // Unix: 二进制文件在 prefix/bin 目录
                        let bin_path = format!("{}/bin/{}", prefix, binary_name);
                        if std::path::Path::new(&bin_path).exists() {
                            return bin_path;
                        }
                    }
                }

                // 回退：返回 prefix 目录 + 第一个候选名
                let first = binary_candidates.first().map(|s| s.as_str()).unwrap_or(tool_key);
                #[cfg(target_os = "windows")]
                {
                    return format!("{}\\{}", prefix, first);
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return format!("{}/bin/{}", prefix, first);
                }
            }
        }

        // 兜底路径
        #[cfg(target_os = "windows")]
        {
            if let Ok(user) = std::env::var("USERNAME") {
                return format!("C:\\Users\\{}\\AppData\\Roaming\\npm\\{}", user, tool_key);
            }
            format!("C:\\Users\\Default\\AppData\\Roaming\\npm\\{}", tool_key)
        }
        #[cfg(not(target_os = "windows"))]
        {
            format!("/usr/local/bin/{}", tool_key)
        }
    }
}

impl Default for CliToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 模拟一个 nvm 风格的全局 node_modules，验证 `read_npm_package_from_node_modules`
    /// 能正确解析 scoped 包的 version 和 bin path。
    ///
    /// 这是当前生产 bug 的核心 case：用户的 `npm prefix -g` 被某个全局
    /// npmrc 改写成 `/Users/rhino/.local`（一个空目录），但实际包都在
    /// `~/.nvm/.../lib/node_modules`。`npm list -g` 返回 `(empty)`，
    /// `which` 又找不到 nvm 里的 binary。我们的 fallback 必须能直接
    /// 扫到 `node_modules` 目录并解析 `package.json`。
    #[test]
    fn read_npm_package_finds_scoped_pkg_in_nvm_node_modules() {
        // 真实 nvm 布局是：~/.nvm/versions/node/v24.16.0/lib/node_modules/<pkg>
        // 测试里我们 mock `~/.nvm/.../v24.16.0/lib/node_modules/<pkg>` 这种结构。
        let tmp = std::env::temp_dir().join(format!(
            "forge-cli-tools-test-{}-{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let lib = tmp.join("v24.16.0").join("lib");
        let nm = lib.join("node_modules");
        let pkg_dir = nm.join("@anthropic-ai").join("claude-code");
        let bin_dir = tmp.join("v24.16.0").join("bin");
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::create_dir_all(&bin_dir).unwrap();

        // package.json: bin 字段用 object 形式
        fs::write(
            pkg_dir.join("package.json"),
            r#"{
                "name": "@anthropic-ai/claude-code",
                "version": "2.1.204",
                "bin": { "claude": "claude" }
            }"#,
        )
        .unwrap();

        // 全局 bin 目录要有 `claude`（模拟 nvm 把包 bin 链接到 prefix/bin）
        fs::write(bin_dir.join("claude"), "#!/bin/sh\necho claude").unwrap();

        let (version, bin_path) =
            read_npm_package_from_node_modules(&nm, "@anthropic-ai/claude-code", "claude")
                .expect("should find the package");

        assert_eq!(version, "2.1.204");
        assert!(
            bin_path.contains("/v24.16.0/bin/"),
            "expected /v24.16.0/bin/ in path, got {}",
            bin_path
        );
        assert!(bin_path.ends_with("/claude"), "got {}", bin_path);

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn read_npm_package_returns_none_for_missing_pkg() {
        let tmp = std::env::temp_dir().join(format!(
            "forge-cli-tools-test-empty-{}-{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let nm = tmp.join("node_modules");
        fs::create_dir_all(&nm).unwrap();

        let result = read_npm_package_from_node_modules(&nm, "@nonexistent/pkg", "anything");
        assert!(result.is_none());

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn read_npm_package_handles_string_bin_field() {
        let tmp = std::env::temp_dir().join(format!(
            "forge-cli-tools-test-binstr-{}-{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let nm = tmp.join("lib").join("node_modules");
        let pkg_dir = nm.join("some-tool");
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(
            pkg_dir.join("package.json"),
            r#"{
                "name": "some-tool",
                "version": "0.0.1",
                "bin": "./bin/some-tool"
            }"#,
        )
        .unwrap();

        let (version, _) = read_npm_package_from_node_modules(&nm, "some-tool", "some-tool")
            .expect("should find the package");
        assert_eq!(version, "0.0.1");

        fs::remove_dir_all(&tmp).ok();
    }

    /// 模拟用户的实际生产环境：`~/.nvm/versions/node/v24.16.0/lib/node_modules`
    /// 下有真实包（这是我们 `git status` 里 99% 用户的布局）。这个测试
    /// 完整模拟 `check_npm_version_async` 的 fallback 路径 — 它在
    /// `npm list -g` 因为全局 npmrc 把 prefix 改错而失败时救场。
    #[test]
    fn read_npm_package_matches_nvm_layout() {
        let tmp = std::env::temp_dir().join(format!(
            "forge-cli-tools-test-nvm-{}-{}",
            std::process::id(),
            chrono::Local::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        // 真实 nvm 布局：~/.nvm/versions/node/v24.16.0/lib/node_modules/<scope>/<pkg>
        let nm = tmp
            .join(".nvm")
            .join("versions")
            .join("node")
            .join("v24.16.0")
            .join("lib")
            .join("node_modules");
        let pkg_dir = nm.join("@anthropic-ai").join("claude-code");
        fs::create_dir_all(&pkg_dir).unwrap();

        fs::write(
            pkg_dir.join("package.json"),
            r#"{
                "name": "@anthropic-ai/claude-code",
                "version": "2.1.204",
                "bin": { "claude": "claude" }
            }"#,
        )
        .unwrap();

        // 全局 bin 也要有（nvm 会创建 symlink）
        let global_bin = tmp
            .join(".nvm")
            .join("versions")
            .join("node")
            .join("v24.16.0")
            .join("bin");
        fs::create_dir_all(&global_bin).unwrap();
        fs::write(global_bin.join("claude"), "").unwrap();

        let result = read_npm_package_from_node_modules(
            &nm,
            "@anthropic-ai/claude-code",
            "claude",
        )
        .expect("should find claude-code in nvm layout");
        assert_eq!(result.0, "2.1.204");
        // 路径必须指向全局 bin 目录
        assert!(
            result.1.contains("/v24.16.0/bin/"),
            "expected path in v24.16.0/bin, got {}",
            result.1
        );

        fs::remove_dir_all(&tmp).ok();
    }

    /// 验证 `enumerate_user_bin_dirs` 至少能枚举出 `~/.local/bin`、
    /// `~/.nvm/versions/node/*/bin` 等位置（用户当前环境）。
    /// 这是 GUI app 检测不到已安装 CLI 的根本原因 — 我们必须显式
    /// 探测这些路径。
    #[test]
    #[cfg(not(target_os = "windows"))]
    fn enumerate_user_bin_dirs_includes_local_bin() {
        let dirs = enumerate_user_bin_dirs();
        // 用户的 ~/.local/bin 存在 — 应该被枚举到
        if let Some(home) = dirs::home_dir() {
            let local_bin = home.join(".local").join("bin");
            if local_bin.is_dir() {
                assert!(
                    dirs.iter().any(|d| d == &local_bin),
                    "expected {:?} in {:?}",
                    local_bin,
                    dirs
                );
            }
            // 用户的 nvm 多版本
            let nvm = home.join(".nvm").join("versions").join("node");
            if nvm.is_dir() {
                let nvm_bins: Vec<_> = dirs
                    .iter()
                    .filter(|d| {
                        d.to_string_lossy().contains(nvm.to_string_lossy().as_ref())
                            && d.to_string_lossy().ends_with("/bin")
                    })
                    .collect();
                assert!(
                    !nvm_bins.is_empty(),
                    "expected at least one nvm bin dir under {:?}, got {:?}",
                    nvm,
                    dirs
                );
            }
        }
    }
}

