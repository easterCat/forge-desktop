use rusqlite::Connection;

pub fn initialize_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- Software table
        CREATE TABLE IF NOT EXISTS software (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            key TEXT UNIQUE NOT NULL,
            version TEXT,
            install_path TEXT,
            config_path TEXT NOT NULL,
            is_installed INTEGER DEFAULT 0,
            last_checked TEXT,
            website_url TEXT,
            platform TEXT
        );

        -- Plugins table
        CREATE TABLE IF NOT EXISTS plugins (
            id TEXT PRIMARY KEY,
            software_id TEXT NOT NULL,
            name TEXT NOT NULL,
            version TEXT,
            author TEXT,
            description TEXT,
            installed_path TEXT,
            enabled INTEGER DEFAULT 1,
            installed_at TEXT,
            last_updated TEXT,
            FOREIGN KEY (software_id) REFERENCES software(id) ON DELETE CASCADE
        );

        -- Skills table
        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            software_id TEXT NOT NULL,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            config TEXT,
            file_path TEXT,
            installed_at TEXT,
            FOREIGN KEY (software_id) REFERENCES software(id) ON DELETE CASCADE
        );

        -- MCP services table
        CREATE TABLE IF NOT EXISTS mcp_services (
            id TEXT PRIMARY KEY,
            software_id TEXT NOT NULL,
            name TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            auth_type TEXT DEFAULT 'none',
            config TEXT,
            is_healthy INTEGER DEFAULT 0,
            last_checked TEXT,
            FOREIGN KEY (software_id) REFERENCES software(id) ON DELETE CASCADE
        );

        -- Rules table
        CREATE TABLE IF NOT EXISTS rules (
            id TEXT PRIMARY KEY,
            software_id TEXT NOT NULL,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            file_path TEXT,
            content TEXT,
            is_active INTEGER DEFAULT 1,
            created_at TEXT,
            updated_at TEXT,
            FOREIGN KEY (software_id) REFERENCES software(id) ON DELETE CASCADE
        );

        -- Backup records table
        CREATE TABLE IF NOT EXISTS backup_records (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            size INTEGER,
            file_count INTEGER,
            created_at TEXT,
            includes TEXT
        );

        -- Config templates table
        CREATE TABLE IF NOT EXISTS config_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT,
            author TEXT,
            version TEXT,
            files TEXT,
            created_at TEXT,
            updated_at TEXT
        );

        -- Create indexes for better query performance
        CREATE INDEX IF NOT EXISTS idx_plugins_software_id ON plugins(software_id);
        CREATE INDEX IF NOT EXISTS idx_skills_software_id ON skills(software_id);
        CREATE INDEX IF NOT EXISTS idx_mcp_services_software_id ON mcp_services(software_id);
        CREATE INDEX IF NOT EXISTS idx_rules_software_id ON rules(software_id);
        CREATE INDEX IF NOT EXISTS idx_backup_records_created_at ON backup_records(created_at);

        -- FEAT-022: MCP Health Log table
        CREATE TABLE IF NOT EXISTS mcp_health_log (
            id TEXT PRIMARY KEY,
            service_id TEXT NOT NULL,
            status TEXT NOT NULL,
            latency_ms INTEGER,
            error_message TEXT,
            checked_at TEXT NOT NULL
        );

        -- FEAT-022: MCP Groups table
        CREATE TABLE IF NOT EXISTS mcp_groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#F59E0B',
            is_visible INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        -- FEAT-022: MCP Service Groups junction table
        CREATE TABLE IF NOT EXISTS mcp_service_groups (
            service_id TEXT NOT NULL,
            group_id TEXT NOT NULL,
            PRIMARY KEY (service_id, group_id),
            FOREIGN KEY (service_id) REFERENCES mcp_services(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES mcp_groups(id) ON DELETE CASCADE
        );

        -- FEAT-022: MCP Audit Log table
        CREATE TABLE IF NOT EXISTS mcp_audit_log (
            id TEXT PRIMARY KEY,
            actor TEXT NOT NULL DEFAULT 'user',
            action TEXT NOT NULL,
            service_id TEXT,
            service_name TEXT,
            details TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        -- FEAT-022: Indexes for new tables
        CREATE INDEX IF NOT EXISTS idx_mcp_health_log_service_checked
            ON mcp_health_log(service_id, checked_at DESC);
        CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_created
            ON mcp_audit_log(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_service
            ON mcp_audit_log(service_id);
        CREATE INDEX IF NOT EXISTS idx_mcp_service_groups_group
            ON mcp_service_groups(group_id);
        "#,
    )?;

    // Migration: add website_url and platform columns if they don't exist
    let _ = conn.execute("ALTER TABLE software ADD COLUMN website_url TEXT", []);
    let _ = conn.execute("ALTER TABLE software ADD COLUMN platform TEXT", []);

    Ok(())
}
