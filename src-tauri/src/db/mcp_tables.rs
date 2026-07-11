// FEAT-022: MCP Tables Helper Functions
// Provides database operations for MCP health log, groups, service groups, and audit log

use crate::db::Database;
use crate::utils::now_rfc3339;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthRecord {
    pub id: String,
    pub service_id: String,
    pub status: String,
    pub latency_ms: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: String,
    pub is_visible: bool,
    pub created_at: String,
    pub server_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: String,
    pub actor: String,
    pub action: String,
    pub service_id: Option<String>,
    pub service_name: Option<String>,
    pub details: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPage {
    pub items: Vec<AuditEntry>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogFilters {
    pub action: Option<String>,
    pub service_id: Option<String>,
    pub service_name: Option<String>,
    pub actor: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub status: Option<String>,
}

impl Database {
    // ==================== Health Log Functions ====================

    pub fn insert_health_log(
        &self,
        service_id: &str,
        status: &str,
        latency_ms: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<String, String> {
        let conn = self.lock_or_err()?;
        let id = uuid::Uuid::new_v4().to_string();
        let checked_at = now_rfc3339();

        conn.execute(
            "INSERT INTO mcp_health_log (id, service_id, status, latency_ms, error_message, checked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, service_id, status, latency_ms, error_message, checked_at],
        )
        .map_err(|e| e.to_string())?;

        Ok(id)
    }

    pub fn get_health_history(
        &self,
        service_id: &str,
        limit: Option<i32>,
    ) -> Result<Vec<HealthRecord>, String> {
        let conn = self.lock_or_err()?;
        let limit = limit.unwrap_or(50);
        let mut stmt = conn
            .prepare(
                "SELECT id, service_id, status, latency_ms, error_message, checked_at
                 FROM mcp_health_log
                 WHERE service_id = ?1
                 ORDER BY checked_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;

        let records = stmt
            .query_map(params![service_id, limit], |row| {
                Ok(HealthRecord {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    status: row.get(2)?,
                    latency_ms: row.get(3)?,
                    error_message: row.get(4)?,
                    checked_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(records)
    }

    pub fn get_latest_health(&self, service_id: &str) -> Result<Option<HealthRecord>, String> {
        let conn = self.lock_or_err()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, service_id, status, latency_ms, error_message, checked_at
                 FROM mcp_health_log
                 WHERE service_id = ?1
                 ORDER BY checked_at DESC
                 LIMIT 1",
            )
            .map_err(|e| e.to_string())?;

        let result = stmt
            .query_row(params![service_id], |row| {
                Ok(HealthRecord {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    status: row.get(2)?,
                    latency_ms: row.get(3)?,
                    error_message: row.get(4)?,
                    checked_at: row.get(5)?,
                })
            })
            .ok();

        Ok(result)
    }

    // ==================== Groups Functions ====================

    pub fn get_groups(&self) -> Result<Vec<Group>, String> {
        let conn = self.lock_or_err()?;
        let mut stmt = conn
            .prepare(
                "SELECT g.id, g.name, g.color, g.is_visible, g.created_at,
                        (SELECT COUNT(*) FROM mcp_service_groups sg WHERE sg.group_id = g.id) as server_count
                 FROM mcp_groups g
                 ORDER BY g.name",
            )
            .map_err(|e| e.to_string())?;

        let groups = stmt
            .query_map([], |row| {
                Ok(Group {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    is_visible: row.get::<_, i32>(3)? != 0,
                    created_at: row.get(4)?,
                    server_count: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(groups)
    }

    pub fn create_group(&self, name: &str, color: &str) -> Result<Group, String> {
        let conn = self.lock_or_err()?;
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = now_rfc3339();

        conn.execute(
            "INSERT INTO mcp_groups (id, name, color, is_visible, created_at)
             VALUES (?1, ?2, ?3, 1, ?4)",
            params![id, name, color, created_at],
        )
        .map_err(|e| e.to_string())?;

        Ok(Group {
            id,
            name: name.to_string(),
            color: color.to_string(),
            is_visible: true,
            created_at,
            server_count: Some(0),
        })
    }

    pub fn update_group(
        &self,
        id: &str,
        name: Option<&str>,
        color: Option<&str>,
        is_visible: Option<bool>,
    ) -> Result<Group, String> {
        let conn = self.lock_or_err()?;

        // Build dynamic update query
        let mut updates = Vec::new();
        if name.is_some() {
            updates.push("name = ?");
        }
        if color.is_some() {
            updates.push("color = ?");
        }
        if is_visible.is_some() {
            updates.push("is_visible = ?");
        }

        if updates.is_empty() {
            return Err("No fields to update".to_string());
        }

        let query = format!("UPDATE mcp_groups SET {} WHERE id = ?", updates.join(", "));

        // Build params dynamically
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(n) = name {
            params_vec.push(Box::new(n.to_string()));
        }
        if let Some(c) = color {
            params_vec.push(Box::new(c.to_string()));
        }
        if let Some(v) = is_visible {
            params_vec.push(Box::new(if v { 1i32 } else { 0i32 }));
        }
        params_vec.push(Box::new(id.to_string()));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        conn.execute(&query, params_refs.as_slice())
            .map_err(|e| e.to_string())?;

        // Fetch updated group
        let mut stmt = conn
            .prepare(
                "SELECT id, name, color, is_visible, created_at,
                        (SELECT COUNT(*) FROM mcp_service_groups sg WHERE sg.group_id = g.id) as server_count
                 FROM mcp_groups g WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;

        stmt.query_row(params![id], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                is_visible: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                server_count: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())
    }

    pub fn delete_group(&self, id: &str) -> Result<(), String> {
        let conn = self.lock_or_err()?;
        conn.execute("DELETE FROM mcp_groups WHERE id = ?", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ==================== Service Groups Functions ====================

    pub fn get_service_groups(&self, service_id: &str) -> Result<Vec<String>, String> {
        let conn = self.lock_or_err()?;
        let mut stmt = conn
            .prepare("SELECT group_id FROM mcp_service_groups WHERE service_id = ?1")
            .map_err(|e| e.to_string())?;

        let groups = stmt
            .query_map(params![service_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(groups)
    }

    /// Batch variant of `get_service_groups`. Returns a `HashMap` keyed by
    /// `service_id` so callers that need group membership for many services
    /// (e.g. `export_mcp_services`) can do it in a single SQL query rather
    /// than N+1 round-trips. Services that have no group membership map to
    /// an empty `Vec`.
    pub fn get_service_groups_batch(
        &self,
        service_ids: &[String],
    ) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
        use std::collections::HashMap;
        if service_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let conn = self.lock_or_err()?;

        // Build the IN clause with the right number of `?` placeholders. We
        // build `?1, ?2, ...` rather than building a string with the ids
        // spliced in so the query stays parameterised and SQL-injection safe.
        let placeholders: Vec<String> = (1..=service_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "SELECT service_id, group_id FROM mcp_service_groups WHERE service_id IN ({})",
            placeholders.join(", ")
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let params_iter: Vec<&dyn rusqlite::ToSql> = service_ids
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let mut out: HashMap<String, Vec<String>> = HashMap::new();
        // Pre-seed with empty vecs so callers can index unconditionally.
        for id in service_ids {
            out.entry(id.clone()).or_default();
        }

        let rows = stmt
            .query_map(params_iter.as_slice(), |row| {
                let svc: String = row.get(0)?;
                let grp: String = row.get(1)?;
                Ok((svc, grp))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (svc, grp) = row.map_err(|e| e.to_string())?;
            out.entry(svc).or_default().push(grp);
        }

        Ok(out)
    }

    pub fn add_service_to_group(&self, service_id: &str, group_id: &str) -> Result<(), String> {
        let conn = self.lock_or_err()?;
        conn.execute(
            "INSERT OR IGNORE INTO mcp_service_groups (service_id, group_id) VALUES (?1, ?2)",
            params![service_id, group_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_service_from_group(&self, service_id: &str, group_id: &str) -> Result<(), String> {
        let conn = self.lock_or_err()?;
        conn.execute(
            "DELETE FROM mcp_service_groups WHERE service_id = ?1 AND group_id = ?2",
            params![service_id, group_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_service_groups(&self, service_id: &str, group_ids: &[String]) -> Result<(), String> {
        let conn = self.lock_or_err()?;

        // The DELETE followed by N×INSERT must be atomic: if the INSERT
        // for one group fails after we've already wiped the prior state,
        // the service is left with a partial membership set. Wrap the
        // whole sequence in a transaction so the WAL/rollback guarantees
        // either all of the new associations land, or none do.
        conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            conn.execute(
                "DELETE FROM mcp_service_groups WHERE service_id = ?",
                params![service_id],
            )
            .map_err(|e| e.to_string())?;

            for group_id in group_ids {
                conn.execute(
                    "INSERT INTO mcp_service_groups (service_id, group_id) VALUES (?1, ?2)",
                    params![service_id, group_id],
                )
                .map_err(|e| e.to_string())?;
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => {
                // Best-effort rollback. If `COMMIT` itself failed we cannot
                // roll back the same connection; surface the original error.
                let _ = conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    // ==================== Audit Log Functions ====================

    pub fn insert_audit_log(
        &self,
        actor: &str,
        action: &str,
        service_id: Option<&str>,
        service_name: Option<&str>,
        details: Option<&str>,
        status: &str,
    ) -> Result<String, String> {
        let conn = self.lock_or_err()?;
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = now_rfc3339();

        conn.execute(
            "INSERT INTO mcp_audit_log (id, actor, action, service_id, service_name, details, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, actor, action, service_id, service_name, details, status, created_at],
        )
        .map_err(|e| e.to_string())?;

        Ok(id)
    }

    pub fn get_audit_log_paginated(
        &self,
        filters: &AuditLogFilters,
        page: i32,
        page_size: i32,
    ) -> Result<AuditPage, String> {
        let conn = self.lock_or_err()?;

        // Build WHERE clause
        let mut conditions = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref action) = filters.action {
            if !action.is_empty() {
                conditions.push("action = ?");
                params_vec.push(Box::new(action.clone()));
            }
        }
        if let Some(ref service_id) = filters.service_id {
            if !service_id.is_empty() {
                conditions.push("service_id = ?");
                params_vec.push(Box::new(service_id.clone()));
            }
        }
        if let Some(ref service_name) = filters.service_name {
            if !service_name.is_empty() {
                conditions.push("service_name LIKE ?");
                params_vec.push(Box::new(format!("%{}%", service_name)));
            }
        }
        if let Some(ref actor) = filters.actor {
            if !actor.is_empty() {
                conditions.push("actor = ?");
                params_vec.push(Box::new(actor.clone()));
            }
        }
        if let Some(ref date_from) = filters.date_from {
            if !date_from.is_empty() {
                conditions.push("created_at >= ?");
                params_vec.push(Box::new(date_from.clone()));
            }
        }
        if let Some(ref date_to) = filters.date_to {
            if !date_to.is_empty() {
                conditions.push("created_at <= ?");
                params_vec.push(Box::new(format!("{}T23:59:59Z", date_to)));
            }
        }
        if let Some(ref status) = filters.status {
            if !status.is_empty() {
                conditions.push("status = ?");
                params_vec.push(Box::new(status.clone()));
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count total
        let count_query = format!("SELECT COUNT(*) FROM mcp_audit_log {}", where_clause);
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let total: i64 = conn
            .query_row(&count_query, params_refs.as_slice(), |row| row.get(0))
            .map_err(|e| e.to_string())?;

        // Fetch paginated results
        let offset = (page - 1) * page_size;
        let data_query = format!(
            "SELECT id, actor, action, service_id, service_name, details, status, created_at
             FROM mcp_audit_log {}
             ORDER BY created_at DESC
             LIMIT ? OFFSET ?",
            where_clause
        );

        let mut data_params: Vec<&dyn rusqlite::ToSql> = params_refs.clone();
        data_params.push(&page_size);
        data_params.push(&offset);

        let mut stmt = conn
            .prepare(&data_query)
            .map_err(|e| e.to_string())?;

        let items = stmt
            .query_map(data_params.as_slice(), |row| {
                Ok(AuditEntry {
                    id: row.get(0)?,
                    actor: row.get(1)?,
                    action: row.get(2)?,
                    service_id: row.get(3)?,
                    service_name: row.get(4)?,
                    details: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i32;

        Ok(AuditPage {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
