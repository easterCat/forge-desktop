// src-tauri/src/db/kv_store.rs
//
// Unified key-value storage backed by the `kv_store` table in forge.db.
// Replaces per-module JSON file read/write patterns with a single
// generic interface. Each former JSON file maps to a string key whose
// value is the serialised JSON content.

use rusqlite::{Connection, params};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex, MutexGuard};

/// Lightweight handle to the `kv_store` table.
///
/// Holds an `Arc<Mutex<Connection>>` so the same `Connection` can be
/// shared with the rest of the application without opening a second
/// SQLite connection (which would race against itself).
pub struct KvStore {
    conn: Arc<Mutex<Connection>>,
}

/// Acquire a `Mutex<Connection>`, recovering automatically if the
/// previous holder panicked mid-write. The previous holder's partial
/// transaction is left intact; SQLite will roll it back on the next
/// operation because the connection is still in WAL mode with
/// `busy_timeout` set.
fn lock_conn(conn: &Arc<Mutex<Connection>>) -> MutexGuard<'_, Connection> {
    match conn.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            log::warn!("Recovering from poisoned SQLite connection");
            poisoned.into_inner()
        }
    }
}

impl KvStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Read a key and deserialize the JSON value into `T`.
    /// Returns `None` when the key does not exist or deserialization fails.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let conn = lock_conn(&self.conn);
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
        let conn = lock_conn(&self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, json, now],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Delete a single key. No-op if the key does not exist.
    pub fn delete(&self, key: &str) -> Result<(), String> {
        let conn = lock_conn(&self.conn);
        conn.execute("DELETE FROM kv_store WHERE key = ?1", params![key])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Export every row as a JSON array of `{key, value, updated_at}`.
    /// `value` is the *parsed* JSON object (not a double-encoded string).
    pub fn export_all(&self) -> Result<String, String> {
        let conn = lock_conn(&self.conn);
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
    ///
    /// The `value` field of each entry is the *parsed* JSON object (not a
    /// double-encoded string). We serialise it with `serde_json::to_string`
    /// so it round-trips byte-for-byte with what `export_all` produced.
    ///
    /// All inserts run inside a single transaction so a partial import can
    /// never leave the store in a half-updated state. If anything fails the
    /// whole batch is rolled back.
    pub fn import_all(&self, json: &str) -> Result<usize, String> {
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();
        let conn = lock_conn(&self.conn);

        conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

        let result = (|| -> Result<usize, String> {
            let mut count = 0usize;
            for entry in &entries {
                let key = entry["key"].as_str().unwrap_or("");
                let value = match &entry["value"] {
                    serde_json::Value::Null => "null".to_string(),
                    other => serde_json::to_string(other)
                        .map_err(|e| format!("Serialise value for key '{}': {}", key, e))?,
                };
                let updated_at = entry["updated_at"].as_str().unwrap_or(&now);
                conn.execute(
                    "INSERT OR REPLACE INTO kv_store (key, value, updated_at) VALUES (?1, ?2, ?3)",
                    params![key, value, updated_at],
                )
                .map_err(|e| e.to_string())?;
                count += 1;
            }
            Ok(count)
        })();

        match result {
            Ok(count) => {
                conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                Ok(count)
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }
}
