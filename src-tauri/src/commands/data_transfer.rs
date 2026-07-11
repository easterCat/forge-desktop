// Data export/import commands — expose KvStore.export_all / import_all
// to the frontend for backup and migration workflows.

use tauri::State;
use crate::db::Database;

/// Export all KV store data as a JSON string.
/// Returns a JSON array of `{key, value, updated_at}` objects.
#[tauri::command]
pub async fn export_all_data(db: State<'_, Database>) -> Result<String, String> {
    let kv = crate::db::KvStore::new(db.conn.clone());
    kv.export_all()
}

/// Import data from a JSON string previously produced by `export_all_data`.
/// Returns the number of rows imported.
#[tauri::command]
pub async fn import_all_data(db: State<'_, Database>, json: String) -> Result<usize, String> {
    let kv = crate::db::KvStore::new(db.conn.clone());
    kv.import_all(&json)
}
