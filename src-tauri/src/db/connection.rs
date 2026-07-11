use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use crate::models::{Software, Plugin, Skill, McpService, Rule, BackupRecord, Agent};

static GLOBAL_DB: OnceLock<Database> = OnceLock::new();

/// Database handle. Multiple instances can share the same underlying
/// `Connection` (via `Arc<Mutex<Connection>>`) — the global `KvStore`
/// and the Tauri `AppState` both hold a clone of this struct, so all
/// reads and writes serialise on the same lock instead of racing two
/// separate connections against the same SQLite file.
#[derive(Clone)]
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        // Enable WAL mode + a 5s busy timeout. WAL allows readers to
        // proceed without blocking the writer, which is essential now
        // that the same `Connection` is shared between the global KV
        // store and the Tauri `AppState`. The busy timeout prevents
        // immediate `SQLITE_BUSY` errors under concurrent access.
        // Errors here are non-fatal: the DB still works without WAL.
        if let Err(e) = conn.pragma_update(None, "journal_mode", "WAL") {
            log::warn!("Failed to enable WAL mode: {}", e);
        }
        if let Err(e) = conn.pragma_update(None, "synchronous", "NORMAL") {
            log::warn!("Failed to set synchronous=NORMAL: {}", e);
        }
        if let Err(e) = conn.pragma_update(None, "busy_timeout", "5000") {
            log::warn!("Failed to set busy_timeout: {}", e);
        }
        if let Err(e) = conn.pragma_update(None, "foreign_keys", "ON") {
            log::warn!("Failed to enable foreign_keys: {}", e);
        }
        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// Set the global database instance (call once at app startup) and
    /// return a *clone* that points at the same underlying connection.
    /// The caller typically hands this clone to `AppState::manage`, so
    /// both the global `KvStore` and the Tauri `AppState` share one
    /// `Connection` — no more `SQLITE_BUSY` from two connections
    /// competing on the same file.
    pub fn set_global(db: Database) -> Database {
        let _ = GLOBAL_DB.set(db.clone());
        db
    }

    /// Get a reference to the global database, if initialized.
    pub fn global() -> Option<&'static Database> {
        GLOBAL_DB.get()
    }

    /// Acquire the connection lock, recovering from a poisoned mutex.
    /// All `Database` operations go through this helper so a panic in
    /// one thread cannot permanently kill the application's ability to
    /// talk to SQLite.
    pub fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        match self.conn.lock() {
            Ok(g) => g,
            Err(poisoned) => {
                log::warn!("Recovering from poisoned SQLite connection in Database::lock");
                poisoned.into_inner()
            }
        }
    }

    /// Like `lock` but returns a `String` error instead of recovering
    /// silently. Used by call sites that want to surface the failure to
    /// the Tauri command's `Result<_, String>` channel.
    pub fn lock_or_err(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        match self.conn.lock() {
            Ok(g) => Ok(g),
            Err(poisoned) => {
                log::warn!("Recovering from poisoned SQLite connection in Database::lock_or_err");
                Ok(poisoned.into_inner())
            }
        }
    }

    fn init_tables(&self) -> Result<()> {
        self.init_tables_inner()?;
        // Run one-time JSON → KV migration (releases lock between table creation and migration)
        Self::migrate_json_to_kv(&self.conn);
        // Seed builtin CLI tools into custom_cli_tools on first launch so the
        // table starts populated (idempotent — uses INSERT OR REPLACE).
        crate::services::cli_tools::seed_builtin_cli_tools_locked(&self.conn);
        // Cleanup: remove deprecated tools every launch. Previously this lived
        // inside `init_tables_inner` under `CREATE TABLE IF NOT EXISTS`, so it
        // only ran on first-ever DB creation. For existing users who already
        // had a stale `github-cli` row, it never executed. Moving it here
        // guarantees cleanup even for pre-existing databases.
        self.cleanup_deprecated_custom_cli_tools()?;
        Ok(())
    }

    /// Delete explicitly deprecated custom CLI tool entries.
    /// Called on every startup to ensure stale rows don't resurface.
    fn cleanup_deprecated_custom_cli_tools(&self) -> Result<()> {
        let conn = self.lock();
        let deprecated_keys = [
            // AI CLI tools
            "codex-cli", "aider", "goose", "kilo-code", "kimi-cli",
            // Dev tools — `gh` (GitHub CLI) was historically seeded but is not
            // a supported tool. We also keep `"github-cli"` for completeness in
            // case an even older row used that key.
            "github-cli", "gh", "lazygit",
            // Modern CLI replacements
            "eza", "ripgrep", "bat", "fzf", "zoxide", "btop",
            // Network/utility
            "httpie", "starship",
        ];
        let list = deprecated_keys.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(",");
        conn.execute(
            &format!("DELETE FROM custom_cli_tools WHERE key IN ({})", list),
            [],
        )?;
        log::info!("Cleanup: removed deprecated custom CLI tools");
        Ok(())
    }

    fn init_tables_inner(&self) -> Result<()> {
        let conn = self.lock();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS software (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                key TEXT NOT NULL UNIQUE,
                version TEXT DEFAULT '',
                install_path TEXT DEFAULT '',
                config_path TEXT DEFAULT '',
                is_installed INTEGER DEFAULT 0,
                last_checked TEXT,
                website_url TEXT,
                platform TEXT
            )",
            [],
        )?;

        // Migration: add website_url and platform columns if they don't exist
        let _ = conn.execute("ALTER TABLE software ADD COLUMN website_url TEXT", []);
        let _ = conn.execute("ALTER TABLE software ADD COLUMN platform TEXT", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                software_id TEXT NOT NULL,
                name TEXT NOT NULL,
                version TEXT DEFAULT '',
                author TEXT DEFAULT '',
                description TEXT DEFAULT '',
                installed_path TEXT DEFAULT '',
                enabled INTEGER DEFAULT 1,
                installed_at TEXT,
                last_updated TEXT,
                FOREIGN KEY (software_id) REFERENCES software(id)
            )",
            [],
        )?;

        // Create index for plugins.software_id foreign key
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_plugins_software_id ON plugins(software_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                software_id TEXT NOT NULL,
                name TEXT NOT NULL,
                skill_type TEXT NOT NULL,
                config TEXT DEFAULT '{}',
                file_path TEXT DEFAULT '',
                installed_at TEXT,
                FOREIGN KEY (software_id) REFERENCES software(id)
            )",
            [],
        )?;

        // Create index for skills.software_id foreign key
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_skills_software_id ON skills(software_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcp_services (
                id TEXT PRIMARY KEY,
                software_id TEXT NOT NULL,
                name TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                auth_type TEXT DEFAULT 'none',
                config TEXT DEFAULT '{}',
                is_healthy INTEGER DEFAULT 0,
                last_checked TEXT,
                FOREIGN KEY (software_id) REFERENCES software(id)
            )",
            [],
        )?;

        // Create index for mcp_services.software_id foreign key
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mcp_services_software_id ON mcp_services(software_id)",
            [],
        )?;

        // FEAT-022: MCP Health Log table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcp_health_log (
                id TEXT PRIMARY KEY,
                service_id TEXT NOT NULL,
                status TEXT NOT NULL,
                latency_ms INTEGER,
                error_message TEXT,
                checked_at TEXT NOT NULL
            )",
            [],
        )?;

        // FEAT-022: MCP Groups table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcp_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#F59E0B',
                is_visible INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // FEAT-022: MCP Service Groups junction table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcp_service_groups (
                service_id TEXT NOT NULL,
                group_id TEXT NOT NULL,
                PRIMARY KEY (service_id, group_id),
                FOREIGN KEY (service_id) REFERENCES mcp_services(id) ON DELETE CASCADE,
                FOREIGN KEY (group_id) REFERENCES mcp_groups(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // FEAT-022: MCP Audit Log table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcp_audit_log (
                id TEXT PRIMARY KEY,
                actor TEXT NOT NULL DEFAULT 'user',
                action TEXT NOT NULL,
                service_id TEXT,
                service_name TEXT,
                details TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // FEAT-022: Indexes for new tables
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mcp_health_log_service_checked ON mcp_health_log(service_id, checked_at DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_created ON mcp_audit_log(created_at DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_service ON mcp_audit_log(service_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mcp_service_groups_group ON mcp_service_groups(group_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY,
                software_id TEXT NOT NULL,
                name TEXT NOT NULL,
                rule_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                content TEXT DEFAULT '',
                is_active INTEGER DEFAULT 1,
                created_at TEXT,
                updated_at TEXT,
                FOREIGN KEY (software_id) REFERENCES software(id)
            )",
            [],
        )?;

        // Create index for rules.software_id foreign key
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rules_software_id ON rules(software_id)",
            [],
        )?;

        // Custom CLI tools table (user-defined)
        // is_allagents: 1 = allagents 23 tools (Default tab), 0 = custom tools (Custom tab)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS custom_cli_tools (
                key TEXT PRIMARY KEY,
                is_allagents INTEGER NOT NULL DEFAULT 0,
                name TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                install_method TEXT NOT NULL,
                install_command TEXT NOT NULL,
                detect_command TEXT NOT NULL,
                website_url TEXT,
                plugin_dir TEXT,
                install_timeout_secs INTEGER NOT NULL DEFAULT 300,
                npm_package TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Migration: add 'is_allagents' column if it doesn't exist (for existing databases)
        let has_is_allagents_column: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('custom_cli_tools') WHERE name = 'is_allagents'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0) > 0;

        if !has_is_allagents_column {
            // Add is_allagents column with default value 0 (custom)
            conn.execute(
                "ALTER TABLE custom_cli_tools ADD COLUMN is_allagents INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
            log::info!("Migration: added 'is_allagents' column to custom_cli_tools");
        }

        // Migration: mark all allagents 23 tools as is_allagents=1
        // These are the standard allagents tools (copilot, codex, opencode, etc.)
        let allagents_keys = [
            "copilot", "codex", "opencode", "gemini", "ampcode", "replit", "kimi", "vscode",
            "claude", "cursor", "factory", "openclaw", "windsurf", "cline", "continue",
            "roo", "kilo", "trae", "augment", "zencoder", "junie", "openhands", "kiro",
        ];
        let allagents_list = allagents_keys.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(",");
        conn.execute(
            &format!("UPDATE custom_cli_tools SET is_allagents = 1 WHERE key IN ({})", allagents_list),
            [],
        )?;
        log::info!("Migration: marked allagents tools as is_allagents=1");

        // Migration: remove duplicate legacy tools from custom_cli_tools
        // These are deprecated - their allagents equivalents replaced them
        // Note: mimo is NOT removed here since we now have it as a valid custom tool.
        // `mimo-code` is the historical alias for the `mimo` seed and is removed
        // so it doesn't appear as a duplicate of MiMo Code in the Custom tab.
        // `hermes` is removed from this list because hermes now has its own
        // SeedEntry in cli_tools.rs; previously the seed list dropped it
        // which left stale rows stranded in the DB.
        let legacy_keys = [
            "claude-code", "gemini-cli", "deepseek-reasonix", "qwen-code",
            "mimo-code",
        ];
        let legacy_list = legacy_keys.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(",");
        conn.execute(
            &format!("DELETE FROM custom_cli_tools WHERE key IN ({})", legacy_list),
            [],
        )?;
        log::info!("Migration: removed duplicate legacy tools from custom_cli_tools");

        // NOTE: The per-startup deprecated-tools cleanup (including github-cli removal)
        // lives in `cleanup_deprecated_custom_cli_tools()` called from `init_tables()`
        // on every app launch. The code below only runs on first-ever DB creation
        // (it is inside `CREATE TABLE IF NOT EXISTS`), so it is kept here as a
        // belt-and-suspenders measure but the authoritative cleanup is in the
        // dedicated function above.

        conn.execute(
            "CREATE TABLE IF NOT EXISTS backups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                size INTEGER DEFAULT 0,
                file_count INTEGER DEFAULT 0,
                created_at TEXT,
                includes TEXT DEFAULT '[]'
            )",
            [],
        )?;

        // Agents table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                emoji TEXT,
                color TEXT,
                department TEXT NOT NULL,
                content TEXT NOT NULL,
                source TEXT NOT NULL DEFAULT 'builtin',
                tags TEXT,
                installed_targets TEXT,
                is_custom INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_agents_department ON agents(department)",
            [],
        )?;

        // KV store — unified storage for former JSON config files
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    // Software CRUD operations
    pub fn get_all_software(&self) -> Result<Vec<Software>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, key, version, install_path, config_path, is_installed, last_checked, website_url, platform FROM software"
        )?;

        let software_iter = stmt.query_map([], |row| {
            Ok(Software {
                id: row.get(0)?,
                name: row.get(1)?,
                key: row.get(2)?,
                version: row.get(3)?,
                install_path: row.get(4)?,
                config_path: row.get(5)?,
                is_installed: row.get::<_, i32>(6)? != 0,
                last_checked: row.get(7)?,
                latest_version: None,
                is_upgradable: false,
                status: crate::models::SoftwareStatus::default(),
                website_url: row.get(8)?,
                platform: row.get(9)?,
            })
        })?;

        let mut results = Vec::new();
        for software in software_iter {
            results.push(software?);
        }
        Ok(results)
    }

    pub fn get_software_by_key(&self, key: &str) -> Result<Option<Software>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, key, version, install_path, config_path, is_installed, last_checked, website_url, platform FROM software WHERE key = ?"
        )?;

        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Software {
                id: row.get(0)?,
                name: row.get(1)?,
                key: row.get(2)?,
                version: row.get(3)?,
                install_path: row.get(4)?,
                config_path: row.get(5)?,
                is_installed: row.get::<_, i32>(6)? != 0,
                last_checked: row.get(7)?,
                latest_version: None,
                is_upgradable: false,
                status: crate::models::SoftwareStatus::default(),
                website_url: row.get(8)?,
                platform: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn upsert_software(&self, software: &Software) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO software (id, name, key, version, install_path, config_path, is_installed, last_checked, website_url, platform)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                software.id,
                software.name,
                software.key,
                software.version,
                software.install_path,
                software.config_path,
                if software.is_installed { 1 } else { 0 },
                software.last_checked,
                software.website_url,
                software.platform
            ],
        )?;
        Ok(())
    }

    // Plugin CRUD operations
    pub fn get_plugins_by_software(&self, software_id: &str) -> Result<Vec<Plugin>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, version, author, description, installed_path, enabled, installed_at, last_updated FROM plugins WHERE software_id = ?"
        )?;
        
        let plugin_iter = stmt.query_map(params![software_id], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                author: row.get(4)?,
                description: row.get(5)?,
                installed_path: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                installed_at: row.get(8)?,
                last_updated: row.get(9)?,
            })
        })?;

        let mut results = Vec::new();
        for plugin in plugin_iter {
            results.push(plugin?);
        }
        Ok(results)
    }

    pub fn get_all_plugins(&self) -> Result<Vec<Plugin>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, version, author, description, installed_path, enabled, installed_at, last_updated FROM plugins"
        )?;
        
        let plugin_iter = stmt.query_map([], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                author: row.get(4)?,
                description: row.get(5)?,
                installed_path: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                installed_at: row.get(8)?,
                last_updated: row.get(9)?,
            })
        })?;

        let mut results = Vec::new();
        for plugin in plugin_iter {
            results.push(plugin?);
        }
        Ok(results)
    }

    pub fn upsert_plugin(&self, plugin: &Plugin) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO plugins (id, software_id, name, version, author, description, installed_path, enabled, installed_at, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                plugin.id,
                plugin.software_id,
                plugin.name,
                plugin.version,
                plugin.author,
                plugin.description,
                plugin.installed_path,
                if plugin.enabled { 1 } else { 0 },
                plugin.installed_at,
                plugin.last_updated
            ],
        )?;
        Ok(())
    }

    pub fn delete_plugin(&self, plugin_id: &str) -> Result<()> {
        let conn = self.lock();
        conn.execute("DELETE FROM plugins WHERE id = ?", params![plugin_id])?;
        Ok(())
    }

    // Skill CRUD operations
    pub fn get_skills_by_software(&self, software_id: &str) -> Result<Vec<Skill>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, type, config, file_path, installed_at FROM skills WHERE software_id = ?"
        )?;
        
        let skill_iter = stmt.query_map(params![software_id], |row| {
            Ok(Skill {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                skill_type: row.get(3)?,
                config: row.get(4)?,
                file_path: row.get(5)?,
                installed_at: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for skill in skill_iter {
            results.push(skill?);
        }
        Ok(results)
    }

    pub fn get_all_skills(&self) -> Result<Vec<Skill>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, type, config, file_path, installed_at FROM skills"
        )?;
        
        let skill_iter = stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                skill_type: row.get(3)?,
                config: row.get(4)?,
                file_path: row.get(5)?,
                installed_at: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for skill in skill_iter {
            results.push(skill?);
        }
        Ok(results)
    }

    pub fn upsert_skill(&self, skill: &Skill) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO skills (id, software_id, name, type, config, file_path, installed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                skill.id,
                skill.software_id,
                skill.name,
                skill.skill_type,
                skill.config,
                skill.file_path,
                skill.installed_at
            ],
        )?;
        Ok(())
    }

    pub fn delete_skill(&self, skill_id: &str) -> Result<()> {
        let conn = self.lock();
        conn.execute("DELETE FROM skills WHERE id = ?", params![skill_id])?;
        Ok(())
    }

    // MCP Service CRUD operations
    pub fn get_mcp_services_by_software(&self, software_id: &str) -> Result<Vec<McpService>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, endpoint, auth_type, config, is_healthy, last_checked FROM mcp_services WHERE software_id = ?"
        )?;
        
        let mcp_iter = stmt.query_map(params![software_id], |row| {
            Ok(McpService {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                endpoint: row.get(3)?,
                auth_type: row.get(4)?,
                config: row.get(5)?,
                is_healthy: row.get::<_, i32>(6)? != 0,
                last_checked: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for mcp in mcp_iter {
            results.push(mcp?);
        }
        Ok(results)
    }

    pub fn get_all_mcp_services(&self) -> Result<Vec<McpService>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, endpoint, auth_type, config, is_healthy, last_checked FROM mcp_services"
        )?;
        
        let mcp_iter = stmt.query_map([], |row| {
            Ok(McpService {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                endpoint: row.get(3)?,
                auth_type: row.get(4)?,
                config: row.get(5)?,
                is_healthy: row.get::<_, i32>(6)? != 0,
                last_checked: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for mcp in mcp_iter {
            results.push(mcp?);
        }
        Ok(results)
    }

    pub fn upsert_mcp_service(&self, service: &McpService) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO mcp_services (id, software_id, name, endpoint, auth_type, config, is_healthy, last_checked)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                service.id,
                service.software_id,
                service.name,
                service.endpoint,
                service.auth_type,
                service.config,
                if service.is_healthy { 1 } else { 0 },
                service.last_checked
            ],
        )?;
        Ok(())
    }

    pub fn delete_mcp_service(&self, service_id: &str) -> Result<()> {
        let conn = self.lock();
        conn.execute("DELETE FROM mcp_services WHERE id = ?", params![service_id])?;
        Ok(())
    }

    // Rule CRUD operations
    pub fn get_rules_by_software(&self, software_id: &str) -> Result<Vec<Rule>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, type, file_path, content, is_active, created_at, updated_at FROM rules WHERE software_id = ?"
        )?;
        
        let rule_iter = stmt.query_map(params![software_id], |row| {
            Ok(Rule {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                rule_type: row.get(3)?,
                file_path: row.get(4)?,
                content: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for rule in rule_iter {
            results.push(rule?);
        }
        Ok(results)
    }

    pub fn get_all_rules(&self) -> Result<Vec<Rule>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, software_id, name, type, file_path, content, is_active, created_at, updated_at FROM rules"
        )?;
        
        let rule_iter = stmt.query_map([], |row| {
            Ok(Rule {
                id: row.get(0)?,
                software_id: row.get(1)?,
                name: row.get(2)?,
                rule_type: row.get(3)?,
                file_path: row.get(4)?,
                content: row.get(5)?,
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for rule in rule_iter {
            results.push(rule?);
        }
        Ok(results)
    }

    pub fn upsert_rule(&self, rule: &Rule) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO rules (id, software_id, name, type, file_path, content, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                rule.id,
                rule.software_id,
                rule.name,
                rule.rule_type,
                rule.file_path,
                rule.content,
                if rule.is_active { 1 } else { 0 },
                rule.created_at,
                rule.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn delete_rule(&self, rule_id: &str) -> Result<()> {
        let conn = self.lock();
        conn.execute("DELETE FROM rules WHERE id = ?", params![rule_id])?;
        Ok(())
    }

    // Backup CRUD operations
    pub fn get_all_backup_records(&self) -> Result<Vec<BackupRecord>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, path, size, file_count, created_at, includes FROM backups"
        )?;
        
        let backup_iter = stmt.query_map([], |row| {
            Ok(BackupRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                size: row.get(3)?,
                file_count: row.get(4)?,
                created_at: row.get(5)?,
                includes: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for backup in backup_iter {
            results.push(backup?);
        }
        Ok(results)
    }

    pub fn upsert_backup_record(&self, backup: &BackupRecord) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO backups (id, name, path, size, file_count, created_at, includes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                backup.id,
                backup.name,
                backup.path,
                backup.size,
                backup.file_count,
                backup.created_at,
                backup.includes
            ],
        )?;
        Ok(())
    }

    pub fn delete_backup_record(&self, backup_id: &str) -> Result<()> {
        let conn = self.lock();
        conn.execute("DELETE FROM backups WHERE id = ?", params![backup_id])?;
        Ok(())
    }

    // Agent CRUD operations
    pub fn get_all_agents(&self) -> Result<Vec<Agent>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, emoji, color, department, content, source, tags, installed_targets, is_custom, created_at, updated_at FROM agents"
        )?;

        let agent_iter = stmt.query_map([], |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                emoji: row.get(3)?,
                color: row.get(4)?,
                department: row.get(5)?,
                content: row.get(6)?,
                source: row.get(7)?,
                tags: row.get(8)?,
                installed_targets: row.get(9)?,
                is_custom: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;

        let mut results = Vec::new();
        for agent in agent_iter {
            results.push(agent?);
        }
        Ok(results)
    }

    pub fn get_agents_by_department(&self, dept: &str) -> Result<Vec<Agent>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, emoji, color, department, content, source, tags, installed_targets, is_custom, created_at, updated_at FROM agents WHERE department = ?"
        )?;

        let agent_iter = stmt.query_map(params![dept], |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                emoji: row.get(3)?,
                color: row.get(4)?,
                department: row.get(5)?,
                content: row.get(6)?,
                source: row.get(7)?,
                tags: row.get(8)?,
                installed_targets: row.get(9)?,
                is_custom: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;

        let mut results = Vec::new();
        for agent in agent_iter {
            results.push(agent?);
        }
        Ok(results)
    }

    pub fn search_agents(&self, query: &str) -> Result<Vec<Agent>> {
        let conn = self.lock();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, name, description, emoji, color, department, content, source, tags, installed_targets, is_custom, created_at, updated_at FROM agents WHERE name LIKE ?1 OR description LIKE ?1 OR tags LIKE ?1"
        )?;

        let agent_iter = stmt.query_map(params![pattern], |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                emoji: row.get(3)?,
                color: row.get(4)?,
                department: row.get(5)?,
                content: row.get(6)?,
                source: row.get(7)?,
                tags: row.get(8)?,
                installed_targets: row.get(9)?,
                is_custom: row.get::<_, i32>(10)? != 0,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;

        let mut results = Vec::new();
        for agent in agent_iter {
            results.push(agent?);
        }
        Ok(results)
    }

    pub fn upsert_agent(&self, agent: &Agent) -> Result<()> {
        let conn = self.lock();
        conn.execute(
            "INSERT OR REPLACE INTO agents (id, name, description, emoji, color, department, content, source, tags, installed_targets, is_custom, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                agent.id,
                agent.name,
                agent.description,
                agent.emoji,
                agent.color,
                agent.department,
                agent.content,
                agent.source,
                agent.tags,
                agent.installed_targets,
                if agent.is_custom { 1 } else { 0 },
                agent.created_at,
                agent.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn delete_agent(&self, agent_id: &str) -> Result<()> {
        let conn = self.lock();
        conn.execute("DELETE FROM agents WHERE id = ?", params![agent_id])?;
        Ok(())
    }

    /// Migrate legacy JSON config files into the `kv_store` table.
    ///
    /// Each file is checked independently — if it exists and parses, its
    /// content is written to KV and the original is renamed to `.json.bak`.
    /// Failures for one file do not block the others.
    fn migrate_json_to_kv(conn: &std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>) {
        use crate::db::kv_store::KvStore;
        use crate::services::plugin_marketplace::forge_home;

        let kv = KvStore::new(conn.clone());
        let home = forge_home();

        let data_local = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let migrations: &[(&str, std::path::PathBuf)] = &[
            ("marketplace_manifest",      home.join("plugins").join("marketplace.json")),
            ("plugin_user_sources",       home.join("plugins").join("user_sources.json")),
            ("plugin_source_notes",       home.join("plugins").join("source_notes.json")),
            ("installed_plugin_registry", home.join("plugins").join("installed_plugins.json")),
            ("plugin_sync_records",       home.join("plugins").join("sync_records.json")),
            ("skill_repositories",        data_local.join("forge").join("repositories.json")),
        ];

        for (key, path) in migrations {
            if !path.exists() {
                continue;
            }
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Migration: cannot read {}: {}", path.display(), e);
                    continue;
                }
            };
            let value: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Migration: cannot parse {}: {}", path.display(), e);
                    continue;
                }
            };
            if let Err(e) = kv.put(key, &value) {
                log::warn!("Migration: failed to write key '{}': {}", key, e);
                continue;
            }
            let bak = path.with_extension("json.bak");
            if let Err(e) = std::fs::rename(path, &bak) {
                log::warn!(
                    "Migration: wrote key '{}' but failed to rename {} → {}: {}",
                    key,
                    path.display(),
                    bak.display(),
                    e
                );
            } else {
                log::info!("Migration: '{}' ← {} (old file → .bak)", key, path.display());
            }
        }
    }
}
