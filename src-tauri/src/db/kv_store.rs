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
