# JSON to KV Store 迁移实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 6 个 Forge 内部 JSON 文件迁移到 SQLite KV 存储，消除代码重复，提供统一抽象和导出/导入能力。

**Architecture:** 在 `forge.db` 中新增 `kv_store` 表（key/value/updated_at），创建 `KvStore` 抽象层提供泛型 `get`/`put`/`delete`/`export_all`/`import_all` 方法。启动时自动迁移旧 JSON 文件。各模块的 read/write 函数改为调用 KvStore。

**Tech Stack:** Rust, rusqlite, serde/serde_json, chrono（均已在 Cargo.toml 中）

---

## 文件结构

| 文件 | 操作 | 职责 |
|---|---|---|
| `src-tauri/src/db/kv_store.rs` | **新增** | KvStore 抽象层 |
| `src-tauri/src/db/connection.rs` | **修改** | 建表 + 迁移调用 |
| `src-tauri/src/db/mod.rs` | **修改** | 导出 kv_store 模块 |
| `src-tauri/src/services/plugin_marketplace.rs` | **修改** | 替换 4 组 read/write |
| `src-tauri/src/commands/plugin_sync.rs` | **修改** | 替换 load/save_sync_records |
| `src-tauri/src/commands/skill_repository.rs` | **修改** | 替换 load/save_repositories |
| `src-tauri/src/commands/data_transfer.rs` | **新增** | 导出/导入 Tauri command |

---

### Task 1: 创建 KvStore 模块

**Files:**
- Create: `src-tauri/src/db/kv_store.rs`

- [ ] **Step 1: 创建 kv_store.rs 文件**

```rust
// src-tauri/src/db/kv_store.rs
//
// Unified key-value storage backed by the `kv_store` table in forge.db.
// Replaces per-module JSON file read/write patterns with a single
// generic interface. Each former JSON file maps to a string key whose
// value is the serialised JSON content.

use rusqlite::{Connection, params};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Mutex;

/// Lightweight handle to the `kv_store` table.
///
/// Holds a reference to the shared `Mutex<Connection>` so callers never
/// need to touch raw SQL for simple get/put operations.
pub struct KvStore<'a> {
    conn: &'a Mutex<Connection>,
}

impl<'a> KvStore<'a> {
    pub fn new(conn: &'a Mutex<Connection>) -> Self {
        Self { conn }
    }

    /// Read a key and deserialize the JSON value into `T`.
    /// Returns `None` when the key does not exist or deserialization fails.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let conn = self.conn.lock().unwrap();
        let value: String = conn
            .query_row(
                "SELECT value FROM kv_store WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok()?;
        serde_json::from_str(&value).ok()
    }

    /// Serialize `value` to JSON and upsert it under `key`.
    /// Also writes the current UTC timestamp to `updated_at`.
    pub fn put<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, json, now],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Delete a single key. No-op if the key does not exist.
    pub fn delete(&self, key: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM kv_store WHERE key = ?1", params![key])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Export every row as a JSON array of `{key, value, updated_at}`.
    /// `value` is the *parsed* JSON object (not a double-encoded string).
    pub fn export_all(&self) -> Result<String, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT key, value, updated_at FROM kv_store")
            .map_err(|e| e.to_string())?;
        let entries: Vec<serde_json::Value> = stmt
            .query_map([], |row| {
                let raw: String = row.get(1)?;
                let parsed: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
                Ok(serde_json::json!({
                    "key": row.get::<_, String>(0)?,
                    "value": parsed,
                    "updated_at": row.get::<_, String>(2)?
                }))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())
    }

    /// Import rows from a JSON array previously produced by `export_all`.
    /// Returns the number of rows imported.
    pub fn import_all(&self, json: &str) -> Result<usize, String> {
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        let mut count = 0usize;
        for entry in &entries {
            let key = entry["key"].as_str().unwrap_or("");
            let value = entry["value"].to_string();
            let updated_at = entry["updated_at"].as_str().unwrap_or(&now);
            conn.execute(
                "INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?1, ?2, ?3)",
                params![key, value, updated_at],
            )
            .map_err(|e| e.to_string())?;
            count += 1;
        }
        Ok(count)
    }
}
```

- [ ] **Step 2: 验证编译通过**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -5
```

Expected: 编译通过（此时 mod.rs 还未导出，但文件本身无语法错误）

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/kv_store.rs
git commit -m "feat(db): add KvStore abstraction for unified key-value storage"
```

---

### Task 2: 注册 KvStore 模块 + 建表

**Files:**
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/db/connection.rs`

- [ ] **Step 1: 在 mod.rs 中导出 kv_store 模块**

将 `src-tauri/src/db/mod.rs` 修改为：

```rust
mod connection;
pub use connection::Database;
pub mod mcp_tables;
pub mod kv_store;
pub use kv_store::KvStore;
```

- [ ] **Step 2: 在 connection.rs 的 init_tables() 中添加 kv_store 建表语句**

在 `src-tauri/src/db/connection.rs` 的 `init_tables()` 方法中，在 `Ok(())` 之前添加：

```rust
        // KV store — unified storage for former JSON config files
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
```

- [ ] **Step 3: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -5
```

Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/mod.rs src-tauri/src/db/connection.rs
git commit -m "feat(db): register KvStore module and create kv_store table"
```

---

### Task 3: 实现 JSON → KV 迁移逻辑

**Files:**
- Modify: `src-tauri/src/db/connection.rs`

- [ ] **Step 1: 在 connection.rs 中添加迁移函数**

在 `Database` impl 块外部（文件末尾附近）添加：

```rust
/// Migrate legacy JSON config files into the `kv_store` table.
///
/// Each file is checked independently — if it exists and parses, its
/// content is written to KV and the original is renamed to `.json.bak`.
/// Failures for one file do not block the others.
pub fn migrate_json_to_kv(conn: &std::sync::Mutex<rusqlite::Connection>) {
    use crate::db::kv_store::KvStore;
    use crate::services::plugin_marketplace::forge_home;

    let kv = KvStore::new(conn);
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
```

- [ ] **Step 2: 在 init_tables() 末尾调用迁移**

在 `connection.rs` 的 `init_tables()` 方法中，在刚添加的 `kv_store` 建表语句之后、`Ok(())` 之前添加：

```rust
        // Run one-time JSON → KV migration
        drop(conn); // release the lock before migration (it re-acquires internally)
        Self::migrate_json_to_kv(&self.conn);
        let conn = self.conn.lock().unwrap(); // re-acquire for any code after
```

**注意:** 由于 `init_tables` 在函数开头已经 `let conn = self.conn.lock().unwrap()`，需要在迁移调用前 drop 掉这个锁。具体做法：

将 `init_tables` 方法重构为两部分 — `init_tables` 调用 `init_tables_inner` 然后调用迁移：

```rust
    fn init_tables(&self) -> Result<()> {
        self.init_tables_inner()?;
        // Run one-time JSON → KV migration
        Self::migrate_json_to_kv(&self.conn);
        Ok(())
    }

    fn init_tables_inner(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // ... 原有的所有 CREATE TABLE 语句保持不变 ...
        // （在最后的 kv_store 建表之后）
        Ok(())
    }
```

- [ ] **Step 3: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -5
```

Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/connection.rs
git commit -m "feat(db): add JSON-to-KV migration on startup"
```

---

### Task 4: 改造 plugin_marketplace.rs — read/write_manifest

**Files:**
- Modify: `src-tauri/src/services/plugin_marketplace.rs`

- [ ] **Step 1: 替换 read_manifest 和 write_manifest**

将 `read_manifest()` 函数（约第 354-370 行）替换为：

```rust
pub fn read_manifest() -> MarketplaceManifest {
    // Try KV store first; fall back to legacy JSON file during migration window
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        if let Some(manifest) = kv.get::<MarketplaceManifest>("marketplace_manifest") {
            return manifest;
        }
    }
    // Fallback: read from JSON file (handles first-launch before DB is ready)
    let path = manifest_path();
    if !path.exists() {
        log::warn!("marketplace.json not found at {}", path.display());
        return MarketplaceManifest::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            log::error!("Failed to parse marketplace.json: {}", e);
            MarketplaceManifest::default()
        }),
        Err(e) => {
            log::error!("Failed to read marketplace.json: {}", e);
            MarketplaceManifest::default()
        }
    }
}

pub fn write_manifest(manifest: &MarketplaceManifest) -> Result<(), String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.put("marketplace_manifest", manifest);
    }
    // Fallback: write to JSON file
    let path = manifest_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude-plugin dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;
    Ok(())
}
```

**重要:** 这个方案需要在 `Database` 上添加一个全局访问点。在 `connection.rs` 中添加：

```rust
use std::sync::OnceLock;

static GLOBAL_DB: OnceLock<Database> = OnceLock::new();

impl Database {
    /// Set the global database instance (call once at app startup).
    pub fn set_global(db: Database) {
        GLOBAL_DB.set(db).ok();
    }

    /// Get a reference to the global database, if initialized.
    pub fn global() -> Option<&'static Database> {
        GLOBAL_DB.get()
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -10
```

Expected: 编译通过（可能有 unused import 警告，忽略）

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/connection.rs src-tauri/src/services/plugin_marketplace.rs
git commit -m "refactor(marketplace): switch read/write_manifest to KvStore"
```

---

### Task 5: 改造 plugin_marketplace.rs — 其余 3 组 read/write

**Files:**
- Modify: `src-tauri/src/services/plugin_marketplace.rs`

- [ ] **Step 1: 替换 read_source_notes / write_source_notes**

将 `read_source_notes()` 函数（约第 209-227 行）替换为：

```rust
pub fn read_source_notes() -> HashMap<String, String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        if let Some(reg) = kv.get::<SourceNotesRegistry>("plugin_source_notes") {
            return reg.notes;
        }
    }
    // Fallback
    let path = source_notes_path();
    if !path.exists() {
        return HashMap::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<SourceNotesRegistry>(&content) {
            Ok(reg) => reg.notes,
            Err(e) => {
                log::warn!("Failed to parse source_notes.json: {}", e);
                HashMap::new()
            }
        },
        Err(e) => {
            log::warn!("Failed to read source_notes.json: {}", e);
            HashMap::new()
        }
    }
}
```

将 `write_source_notes()` 函数（约第 230-245 行）替换为：

```rust
pub fn write_source_notes(notes: &HashMap<String, String>) -> Result<(), String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        let reg = SourceNotesRegistry {
            version: "1".to_string(),
            notes: notes.clone(),
        };
        return kv.put("plugin_source_notes", &reg);
    }
    // Fallback
    let path = source_notes_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create plugins dir: {}", e))?;
    }
    let reg = SourceNotesRegistry {
        version: "1".to_string(),
        notes: notes.clone(),
    };
    let json = serde_json::to_string_pretty(&reg)
        .map_err(|e| format!("Failed to serialize source_notes.json: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write source_notes.json: {}", e))?;
    Ok(())
}
```

- [ ] **Step 2: 替换 read_user_sources / write_user_sources**

将 `read_user_sources()` 函数（约第 255-293 行）替换为：

```rust
pub fn read_user_sources() -> Vec<PluginSource> {
    let mut sources: Vec<PluginSource> = if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        kv.get::<UserSourceRegistry>("plugin_user_sources")
            .map(|reg| reg.sources)
            .unwrap_or_default()
    } else {
        // Fallback
        let path = user_sources_path();
        if !path.exists() {
            return Vec::new();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<UserSourceRegistry>(&content) {
                Ok(reg) => reg.sources,
                Err(e) => {
                    log::error!("Failed to parse user_sources.json: {}", e);
                    return Vec::new();
                }
            },
            Err(e) => {
                log::error!("Failed to read user_sources.json: {}", e);
                return Vec::new();
            }
        }
    };
    // Compute plugin counts for user sources using their repoName
    for s in &mut sources {
        if s.repo_type.as_deref() == Some("res") {
            s.plugin_count = Some(1);
            continue;
        }
        let repo_name = s.repo_name.clone().unwrap_or_else(|| s.id.clone());
        let source_path = marketplace_sources_dir().join(&repo_name);
        if source_path.exists() && is_git_repo(&source_path) {
            if let Some(count) = read_source_marketplace_json(&source_path) {
                s.plugin_count = Some(count);
            } else {
                let count = count_plugin_dirs(&source_path);
                s.plugin_count = Some(count);
            }
        }
    }
    sources
}
```

将 `write_user_sources()` 函数（约第 295-310 行）替换为：

```rust
pub fn write_user_sources(sources: &[PluginSource]) -> Result<(), String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        let reg = UserSourceRegistry {
            version: "1".to_string(),
            sources: sources.to_vec(),
        };
        return kv.put("plugin_user_sources", &reg);
    }
    // Fallback
    let path = user_sources_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create marketplace dir: {}", e))?;
    }
    let reg = UserSourceRegistry {
        version: "1".to_string(),
        sources: sources.to_vec(),
    };
    let json = serde_json::to_string_pretty(&reg)
        .map_err(|e| format!("Failed to serialize user_sources.json: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write user_sources.json: {}", e))?;
    Ok(())
}
```

- [ ] **Step 3: 替换 read_installed_registry / write_installed_registry**

将 `read_installed_registry()` 函数（约第 2831-2846 行）替换为：

```rust
pub fn read_installed_registry() -> InstalledRegistry {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        if let Some(reg) = kv.get::<InstalledRegistry>("installed_plugin_registry") {
            return reg;
        }
    }
    // Fallback
    let path = installed_plugins_path();
    if !path.exists() {
        return InstalledRegistry::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            log::error!("Failed to parse installed_plugins.json: {}", e);
            InstalledRegistry::default()
        }),
        Err(e) => {
            log::error!("Failed to read installed_plugins.json: {}", e);
            InstalledRegistry::default()
        }
    }
}
```

将 `write_installed_registry()` 函数（约第 2848-2859 行）替换为：

```rust
pub fn write_installed_registry(reg: &InstalledRegistry) -> Result<(), String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.put("installed_plugin_registry", reg);
    }
    // Fallback
    let path = installed_plugins_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create marketplace dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(reg)
        .map_err(|e| format!("Failed to serialize registry: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write installed_plugins.json: {}", e))?;
    Ok(())
}
```

- [ ] **Step 4: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -10
```

Expected: 编译通过

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/plugin_marketplace.rs
git commit -m "refactor(marketplace): switch remaining JSON read/write to KvStore"
```

---

### Task 6: 改造 plugin_sync.rs — load/save_sync_records

**Files:**
- Modify: `src-tauri/src/commands/plugin_sync.rs`

- [ ] **Step 1: 替换 load_sync_records 和 save_sync_records**

将 `load_sync_records()` 函数（约第 52-61 行）替换为：

```rust
fn load_sync_records() -> HashMap<String, PluginSyncRecord> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.get("plugin_sync_records").unwrap_or_default();
    }
    // Fallback
    let path = sync_records_path();
    if !path.exists() {
        return HashMap::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}
```

将 `save_sync_records()` 函数（约第 63-71 行）替换为：

```rust
fn save_sync_records(records: &HashMap<String, PluginSyncRecord>) -> Result<(), String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        return kv.put("plugin_sync_records", records);
    }
    // Fallback
    let path = sync_records_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create sync records dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize sync records: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write sync records: {}", e))
}
```

- [ ] **Step 2: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -10
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/plugin_sync.rs
git commit -m "refactor(sync): switch load/save_sync_records to KvStore"
```

---

### Task 7: 改造 skill_repository.rs — load/save_repositories

**Files:**
- Modify: `src-tauri/src/commands/skill_repository.rs`

- [ ] **Step 1: 替换 load_repositories 和 save_repositories**

将 `load_repositories()` 函数（约第 63-86 行）替换为：

```rust
fn load_repositories() -> Result<Vec<SkillRepository>, String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        if let Some(data) = kv.get::<serde_json::Value>("skill_repositories") {
            let repos = data.get("repositories")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| serde_json::from_value(item.clone()).ok())
                        .collect()
                })
                .unwrap_or_default();
            return Ok(repos);
        }
        return Ok(vec![]);
    }
    // Fallback
    let config_path = get_config_path();
    if !config_path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("配置文件格式错误: {}", e))?;
    let repos = data.get("repositories")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| serde_json::from_value(item.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    Ok(repos)
}
```

将 `save_repositories()` 函数（约第 89-105 行）替换为：

```rust
fn save_repositories(repos: &[SkillRepository]) -> Result<(), String> {
    if let Some(db) = crate::db::connection::Database::global() {
        let kv = crate::db::KvStore::new(&db.conn);
        let data = serde_json::json!({
            "version": "1.0",
            "repositories": repos
        });
        return kv.put("skill_repositories", &data);
    }
    // Fallback
    let config_path = ensure_config_dir()
        .map_err(|e| format!("无法创建配置目录: {}", e))?;
    let data = serde_json::json!({
        "version": "1.0",
        "repositories": repos
    });
    let content = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}
```

- [ ] **Step 2: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -10
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/skill_repository.rs
git commit -m "refactor(repositories): switch load/save_repositories to KvStore"
```

---

### Task 8: 在 main.rs/lib.rs 中初始化全局 Database

**Files:**
- Modify: `src-tauri/src/lib.rs` 或 `src-tauri/src/main.rs`（取决于哪个文件执行 `Database::new`）

- [ ] **Step 1: 找到 Database 初始化位置并设置全局实例**

搜索 `Database::new` 的调用位置。在创建 Database 实例后添加：

```rust
// After: let db = Database::new(&db_path).expect("...");
Database::set_global(db);
// Then continue using db via Database::global() or the Tauri state
```

**注意:** 如果 `Database` 被 `manage()` 注册到 Tauri state，需要在 `manage()` 之前调用 `set_global`，因为 `manage()` 会 move db。

具体模式：

```rust
let db = Database::new(&db_path).expect("Failed to initialize database");
Database::set_global(db); // set global first
let db = Database::new(&db_path).expect("..."); // re-create for Tauri state
app.manage(db);
```

或者更好的方式 — 修改 `set_global` 不 consume db，改为在 `Database::new` 内部自动设置：

```rust
impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        // Auto-register as global (first-wins)
        GLOBAL_DB.set(db).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
        Ok(GLOBAL_DB.get().unwrap()) // 不太好，因为 set 消耗了 db
    }
}
```

**更好的方案:** 使用 `OnceLock` + 内部可变性，或者直接在 `new` 之后手动调用。找到实际代码后确定最佳模式。

- [ ] **Step 2: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -10
```

Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(app): set global Database instance for KvStore access"
```

---

### Task 9: 添加导出/导入 Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/data_transfer.rs`
- Modify: `src-tauri/src/commands/mod.rs`（或 lib.rs 中的模块声明）

- [ ] **Step 1: 创建 data_transfer.rs**

```rust
// Data export/import commands — expose KvStore.export_all / import_all
// to the frontend for backup and migration workflows.

use tauri::State;
use crate::db::Database;

/// Export all KV store data as a JSON string.
/// Returns a JSON array of `{key, value, updated_at}` objects.
#[tauri::command]
pub async fn export_all_data(db: State<'_, Database>) -> Result<String, String> {
    let kv = crate::db::KvStore::new(&db.conn);
    kv.export_all()
}

/// Import data from a JSON string previously produced by `export_all_data`.
/// Returns the number of rows imported.
#[tauri::command]
pub async fn import_all_data(db: State<'_, Database>, json: String) -> Result<usize, String> {
    let kv = crate::db::KvStore::new(&db.conn);
    kv.import_all(&json)
}
```

- [ ] **Step 2: 注册模块和 commands**

在 `src-tauri/src/commands/mod.rs` 中添加：

```rust
pub mod data_transfer;
```

在 `lib.rs` 或 `main.rs` 的 `invoke_handler` 中添加：

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::data_transfer::export_all_data,
    commands::data_transfer::import_all_data,
])
```

- [ ] **Step 3: 编译验证**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo check 2>&1 | tail -10
```

Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(commands): add export_all_data and import_all_data Tauri commands"
```

---

### Task 10: 端到端验证

- [ ] **Step 1: 完整编译**

```bash
cd /Users/rhino/Desktop/AI/env-manager/src-tauri && cargo build 2>&1 | tail -10
```

Expected: 编译成功

- [ ] **Step 2: 运行应用测试迁移**

```bash
cd /Users/rhino/Desktop/AI/env-manager && cargo tauri dev
```

验证：
1. 应用启动无 panic
2. 控制台输出包含 "Migration:" 日志
3. 旧 JSON 文件被重命名为 `.json.bak`
4. Marketplace、Plugins、Skills 页面数据正常加载
5. 新增/删除操作正常工作

- [ ] **Step 3: 测试导出/导入**

在前端控制台或 Tauri 调用：
```javascript
const data = await invoke('export_all_data');
console.log(data); // 应显示 JSON 数组
```

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete JSON-to-KV store migration"
```
