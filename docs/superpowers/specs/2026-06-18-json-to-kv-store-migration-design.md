# JSON to KV Store 迁移设计

**日期**: 2026-06-18
**状态**: 待审批
**范围**: Forge 内部 JSON 文件迁移至 SQLite KV 存储

## 背景

Forge 当前使用双存储架构：SQLite (`forge.db`) 存核心数据，6 个 JSON 文件存 marketplace 注册表、用户源、同步记录等辅助配置。这些 JSON 文件存在以下问题：

1. **代码重复** — 每个模块独立实现 `read_foo()` / `write_foo()` 模式，逻辑完全相同
2. **非原子写入** — 除 GitHub token 外，所有 JSON 写入使用 `std::fs::write`，进程崩溃可能导致数据损坏
3. **无统一抽象** — 6 个文件分散在 3 个模块中，没有共享的存储层
4. **导出困难** — 没有统一的导出/导入机制

外部工具的 JSON 文件（`~/.claude/settings.json`、`~/.cursor/mcp.json` 等）不在本范围内。

## 目标

1. 消除所有 JSON 文件，数据统一存入已有的 `forge.db`
2. 提供统一的 `KvStore` 抽象，将每个模块的 read/write 代码从 ~20 行缩减到 ~2 行
3. 内置 JSON 导出/导入能力，方便迁移和调试
4. 启动时自动迁移旧 JSON 文件，对用户透明

## 方案

### 核心：纯 KV 表

在 `forge.db` 中新增一张表：

```sql
CREATE TABLE IF NOT EXISTS kv_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

所有 6 个 JSON 文件的数据以 JSON 字符串形式存储在 `value` 列中。

### Key 映射

| KV Key | 原 JSON 文件 | Value 类型 |
|---|---|---|
| `marketplace_manifest` | `$FORGE_HOME/plugins/marketplace.json` | `MarketplaceManifest` |
| `plugin_user_sources` | `$FORGE_HOME/plugins/user_sources.json` | `UserSourceRegistry` |
| `plugin_source_notes` | `$FORGE_HOME/plugins/source_notes.json` | `SourceNotesRegistry` |
| `installed_plugin_registry` | `$FORGE_HOME/plugins/installed_plugins.json` | `InstalledRegistry` |
| `plugin_sync_records` | `$FORGE_HOME/plugins/sync_records.json` | `HashMap<String, PluginSyncRecord>` |
| `skill_repositories` | `<data_local_dir>/forge/repositories.json` | `{"version","repositories": Vec<SkillRepository>}` |

### KvStore 抽象

新增 `src-tauri/src/db/kv_store.rs`：

```rust
pub struct KvStore<'a> {
    conn: &'a Mutex<Connection>,
}

impl<'a> KvStore<'a> {
    pub fn new(conn: &'a Mutex<Connection>) -> Self;

    /// 读取一个 key，反序列化为 T。key 不存在返回 None。
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;

    /// 写入一个 key，序列化 T 为 JSON。INSERT OR REPLACE 语义。
    pub fn put<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String>;

    /// 删除一个 key。
    pub fn delete(&self, key: &str) -> Result<(), String>;

    /// 导出全部数据为 JSON 数组 [{key, value, updated_at}]。
    pub fn export_all(&self) -> Result<String, String>;

    /// 从 JSON 数组导入全部数据，返回导入行数。
    pub fn import_all(&self, json: &str) -> Result<usize, String>;
}
```

### 迁移策略

在 `Database::init_tables()` 中创建 `kv_store` 表后，调用迁移函数：

```rust
fn migrate_json_to_kv(kv: &KvStore, forge_home: &Path) {
    let migrations = [
        ("marketplace_manifest",      forge_home.join("plugins/marketplace.json")),
        ("plugin_user_sources",       forge_home.join("plugins/user_sources.json")),
        ("plugin_source_notes",       forge_home.join("plugins/source_notes.json")),
        ("installed_plugin_registry", forge_home.join("plugins/installed_plugins.json")),
        ("plugin_sync_records",       forge_home.join("plugins/sync_records.json")),
        ("skill_repositories",        dirs::data_local_dir().join("forge/repositories.json")),
    ];

    for (key, path) in &migrations {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                    if kv.put(key, &value).is_ok() {
                        let bak = path.with_extension("json.bak");
                        let _ = std::fs::rename(path, bak);
                    }
                }
            }
        }
    }
}
```

迁移逻辑：
- 检测旧 JSON 文件是否存在
- 存在则读取内容，写入 KV store
- 写入成功后重命名为 `.json.bak`（不删除，安全起见）
- 每个文件独立迁移，单个失败不影响其他

### 模块改造

**改造前**（以 `plugin_sync.rs` 为例，~20 行）：

```rust
fn load_sync_records() -> HashMap<String, PluginSyncRecord> {
    let path = sync_records_path();
    if !path.exists() { return HashMap::new(); }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}
fn save_sync_records(records: &HashMap<String, PluginSyncRecord>) -> Result<(), String> {
    let json = serde_json::to_string_pretty(records).map_err(|e| e.to_string())?;
    std::fs::write(sync_records_path(), json).map_err(|e| e.to_string())?;
    Ok(())
}
```

**改造后**（~2 行）：

```rust
fn load_sync_records(kv: &KvStore) -> HashMap<String, PluginSyncRecord> {
    kv.get("plugin_sync_records").unwrap_or_default()
}
fn save_sync_records(kv: &KvStore, records: &HashMap<String, PluginSyncRecord>) -> Result<(), String> {
    kv.put("plugin_sync_records", records)
}
```

### 导出/迁移工具

Tauri command 暴露给前端：

```rust
#[tauri::command]
pub async fn export_all_data(db: State<'_, Database>) -> Result<String, String> {
    let kv = KvStore::new(&db.conn);
    kv.export_all()
}

#[tauri::command]
pub async fn import_all_data(db: State<'_, Database>, json: String) -> Result<usize, String> {
    let kv = KvStore::new(&db.conn);
    kv.import_all(&json)
}
```

导出格式：JSON 数组，每个元素包含 `key`、`value`（已解析的 JSON 对象）、`updated_at`。人类可读，可编辑后重新导入。

## 改动范围

| 文件 | 操作 | 说明 |
|---|---|---|
| `src-tauri/src/db/kv_store.rs` | 新增 | KvStore 抽象（~80 行） |
| `src-tauri/src/db/connection.rs` | 修改 | `init_tables()` 加建表 + 迁移调用 |
| `src-tauri/src/db/mod.rs` | 修改 | 导出 `kv_store` 模块 |
| `src-tauri/src/services/plugin_marketplace.rs` | 改造 | 替换 4 组 read/write 为 kv.get/put |
| `src-tauri/src/commands/plugin_sync.rs` | 改造 | 替换 2 组 load/save 为 kv.get/put |
| `src-tauri/src/commands/skill_repository.rs` | 改造 | 替换 1 组 load/save 为 kv.get/put |

## 数据流

```
应用启动
  → Database::new()
    → init_tables()
      → CREATE TABLE kv_store
      → migrate_json_to_kv()
        → 检测 ~/.forge/plugins/*.json
        → 逐个读取 → 写入 kv_store → 重命名为 .bak

运行时读写
  command 函数 → KvStore::get/put → SQLite kv_store 表

导出
  前端 invoke("export_all_data") → KvStore::export_all() → JSON 字符串 → 下载

导入
  前端上传 JSON → invoke("import_all_data", json) → KvStore::import_all() → 写入 kv_store
```

## 不在范围内

- 外部工具 JSON 文件（`~/.claude/settings.json`、`~/.cursor/mcp.json` 等）
- 现有 SQLite 表结构变更
- 前端 UI 变更（导出/导入按钮需后续单独设计）
- 备份系统改造（`manifest.json` 在备份目录中，保持原样）
