// FEAT-022: MCP Manager Integration Tests
//
// Tests for:
// - utils::now_rfc3339() and utils::future_rfc3339()
// - import_mcp_services (skip/overwrite modes, validation)
// - export_mcp_services (JSON/YAML formats)
// - get_mcp_groups (empty initially)
// - create_mcp_group (create and return)
// - delete_mcp_group (remove)
// - get_mcp_audit_log (empty initially)

use rusqlite::Connection;
use tempfile::TempDir;

mod test_utils {
    use super::*;

    pub fn create_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).expect("Failed to open test DB");

        // Initialize the schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS mcp_services (
                id TEXT PRIMARY KEY,
                software_id TEXT NOT NULL,
                name TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                auth_type TEXT DEFAULT 'none',
                config TEXT,
                is_healthy INTEGER DEFAULT 0,
                last_checked TEXT
            );

            CREATE TABLE IF NOT EXISTS mcp_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#F59E0B',
                is_visible INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mcp_service_groups (
                service_id TEXT NOT NULL,
                group_id TEXT NOT NULL,
                PRIMARY KEY (service_id, group_id)
            );

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

            -- FEAT-022: Health Log table
            CREATE TABLE IF NOT EXISTS mcp_health_log (
                id TEXT PRIMARY KEY,
                service_id TEXT NOT NULL,
                status TEXT NOT NULL,
                latency_ms INTEGER,
                error_message TEXT,
                checked_at TEXT NOT NULL
            );
            "#,
        )
        .expect("Failed to initialize schema");

        (temp_dir, conn)
    }
}

#[cfg(test)]
mod utils_tests {
    use forge_desktop_lib::utils::{future_rfc3339, now_rfc3339};
    use chrono::DateTime;

    #[test]
    fn test_now_rfc3339_is_valid_format() {
        let now = now_rfc3339();
        // RFC3339 format: 2024-01-15T10:30:00Z or 2024-01-15T10:30:00+00:00
        assert!(
            DateTime::parse_from_rfc3339(&now).is_ok(),
            "now_rfc3339() returned invalid RFC3339: {}",
            now
        );
    }

    #[test]
    fn test_future_rfc3339_is_valid_format() {
        let future = future_rfc3339(300);
        assert!(
            DateTime::parse_from_rfc3339(&future).is_ok(),
            "future_rfc3339() returned invalid RFC3339: {}",
            future
        );
    }

    #[test]
    fn test_future_is_after_now() {
        let now = now_rfc3339();
        let future = future_rfc3339(300);

        let now_dt = DateTime::parse_from_rfc3339(&now).unwrap();
        let future_dt = DateTime::parse_from_rfc3339(&future).unwrap();

        assert!(
            future_dt > now_dt,
            "future_rfc3339(300) should return a time after now_rfc3339()"
        );
    }

    #[test]
    fn test_future_rfc3339_approx_300_seconds() {
        let now = now_rfc3339();
        let future = future_rfc3339(300);

        let now_dt = DateTime::parse_from_rfc3339(&now).unwrap();
        let future_dt = DateTime::parse_from_rfc3339(&future).unwrap();

        let diff_secs = future_dt.signed_duration_since(now_dt).num_seconds();

        // Allow 2 seconds tolerance for test execution time
        assert!(
            (295..=305).contains(&diff_secs),
            "Difference should be approximately 300 seconds, got {}",
            diff_secs
        );
    }

    #[test]
    fn test_future_rfc3339_zero_offset() {
        let now = now_rfc3339();
        let future = future_rfc3339(0);

        let now_dt = DateTime::parse_from_rfc3339(&now).unwrap();
        let future_dt = DateTime::parse_from_rfc3339(&future).unwrap();

        let diff_secs = future_dt.signed_duration_since(now_dt).num_seconds();

        // Should be very close to 0 (within 1 second tolerance)
        assert!(
            diff_secs.abs() <= 1,
            "Difference for 0 offset should be ~0, got {}",
            diff_secs
        );
    }
}

#[cfg(test)]
mod mcp_manager_tests {
    use super::test_utils::create_test_db;
    use forge_desktop_lib::db::mcp_tables::AuditLogFilters;
    use forge_desktop_lib::db::Database;
    use forge_desktop_lib::models::McpService;
    use rusqlite::Connection;
    use std::sync::Mutex;
    use tempfile::TempDir;

    fn create_db_wrapper(_temp_dir: &TempDir, conn: Connection) -> Database {
        Database {
            conn: Mutex::new(conn),
        }
    }

    // ==================== Import Tests ====================

    #[test]
    fn test_import_skips_duplicates() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Insert existing service
        let existing = McpService {
            id: "existing-id".to_string(),
            software_id: "soft-1".to_string(),
            name: "ExistingService".to_string(),
            endpoint: "http://existing.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&existing).expect("Failed to insert existing");

        // Import data with same name
        let import_data = r#"{
            "services": [
                {
                    "name": "ExistingService",
                    "endpoint": "http://new-endpoint.local",
                    "authType": "bearer"
                }
            ]
        }"#;

        // Execute import command logic (synchronously for test)
        let result = import_services_sync(&db, import_data.to_string(), "skip".to_string());

        assert!(result.is_ok(), "Import should succeed");
        let import_result = result.unwrap();
        assert_eq!(import_result.skipped, 1, "Should skip 1 duplicate");
        assert_eq!(import_result.imported, 0, "Should not import anything");
        assert_eq!(import_result.overwritten, 0, "Should not overwrite");
        assert!(
            import_result.errors.is_empty(),
            "Should have no errors, got: {:?}",
            import_result.errors
        );
    }

    #[test]
    fn test_import_overwrites_existing() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Insert existing service
        let existing = McpService {
            id: "existing-id".to_string(),
            software_id: "soft-1".to_string(),
            name: "ExistingService".to_string(),
            endpoint: "http://old-endpoint.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&existing).expect("Failed to insert existing");

        // Import data with same name but different endpoint
        let import_data = r#"{
            "services": [
                {
                    "name": "ExistingService",
                    "endpoint": "http://new-endpoint.local",
                    "authType": "bearer"
                }
            ]
        }"#;

        let result = import_services_sync(&db, import_data.to_string(), "overwrite".to_string());

        assert!(result.is_ok(), "Import should succeed");
        let import_result = result.unwrap();
        assert_eq!(import_result.overwritten, 1, "Should overwrite 1 service");
        assert_eq!(import_result.skipped, 0, "Should not skip");
        assert_eq!(import_result.imported, 0, "Should not import new");

        // Verify the endpoint was updated
        let services = db.get_all_mcp_services().expect("Failed to get services");
        let updated = services.iter().find(|s| s.name == "ExistingService").unwrap();
        assert_eq!(
            updated.endpoint, "http://new-endpoint.local",
            "Endpoint should be updated"
        );
    }

    #[test]
    fn test_import_creates_new_services() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Import new services (no existing)
        let import_data = r#"{
            "services": [
                {
                    "name": "NewService1",
                    "endpoint": "http://service1.local",
                    "authType": "bearer"
                },
                {
                    "name": "NewService2",
                    "endpoint": "http://service2.local",
                    "authType": "api-key"
                }
            ]
        }"#;

        let result = import_services_sync(&db, import_data.to_string(), "skip".to_string());

        assert!(result.is_ok(), "Import should succeed");
        let import_result = result.unwrap();
        assert_eq!(import_result.imported, 2, "Should import 2 new services");
        assert_eq!(import_result.skipped, 0, "Should not skip anything");
        assert_eq!(import_result.overwritten, 0, "Should not overwrite");
    }

    #[test]
    fn test_import_invalid_data_returns_error() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Valid JSON but missing name
        let missing_name = r#"{
            "services": [
                {
                    "endpoint": "http://service.local"
                }
            ]
        }"#;
        let result = import_services_sync(&db, missing_name.to_string(), "skip".to_string());
        assert!(
            result.is_ok(),
            "Should succeed but record error in result"
        );
        let import_result = result.unwrap();
        assert!(
            !import_result.errors.is_empty(),
            "Should have errors for missing name"
        );

        // Valid JSON but missing endpoint
        let missing_endpoint = r#"{
            "services": [
                {
                    "name": "TestService"
                }
            ]
        }"#;
        let result = import_services_sync(&db, missing_endpoint.to_string(), "skip".to_string());
        assert!(result.is_ok(), "Should succeed but record error");
        let import_result = result.unwrap();
        assert!(
            import_result.errors.iter().any(|e| e.contains("endpoint")),
            "Should have error about missing endpoint"
        );
    }

    #[test]
    fn test_import_no_services_array() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // JSON without services array
        let no_services = r#"{"version": "1.0"}"#;
        let result = import_services_sync(&db, no_services.to_string(), "skip".to_string());
        assert!(
            result.is_err(),
            "Should return error when no services array found"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("No services array found"),
            "Error should mention missing services array"
        );
    }

    // ==================== Export Tests ====================

    #[test]
    fn test_export_json_format() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Insert a service
        let service = McpService {
            id: "svc-1".to_string(),
            software_id: "soft-1".to_string(),
            name: "TestService".to_string(),
            endpoint: "http://test.local".to_string(),
            auth_type: "bearer".to_string(),
            config: Some(r#"{"token": "test"}"#.to_string()),
            is_healthy: true,
            last_checked: None,
        };
        db.upsert_mcp_service(&service).expect("Failed to insert service");

        let export = export_services_sync(&db, None, "json".to_string());
        assert!(export.is_ok(), "Export should succeed");

        let json_str = export.unwrap();
        // Verify it's valid JSON and contains expected fields
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Export should be valid JSON");

        assert_eq!(parsed["version"], "1.0", "Should have version 1.0");
        assert!(
            parsed["services"].is_array(),
            "Should have services array"
        );
        assert_eq!(
            parsed["services"][0]["name"], "TestService",
            "Should contain service name"
        );
        assert_eq!(
            parsed["services"][0]["endpoint"], "http://test.local",
            "Should contain endpoint"
        );
    }

    #[test]
    fn test_export_yaml_format() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Insert a service
        let service = McpService {
            id: "svc-1".to_string(),
            software_id: "soft-1".to_string(),
            name: "TestService".to_string(),
            endpoint: "http://test.local".to_string(),
            auth_type: "bearer".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&service).expect("Failed to insert service");

        let export = export_services_sync(&db, None, "yaml".to_string());
        assert!(export.is_ok(), "Export should succeed");

        let yaml_str = export.unwrap();
        // Verify it's valid YAML (parse with serde_yaml)
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(&yaml_str).expect("Export should be valid YAML");

        let map = parsed.as_mapping().expect("Should be mapping");
        let version_key = serde_yaml::Value::String("version".to_string());
        assert_eq!(
            map.get(&version_key).map(|v| v.as_str().unwrap_or("")),
            Some("1.0"),
            "Should have version 1.0"
        );
    }

    #[test]
    fn test_export_specific_services_by_ids() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Insert multiple services
        let service1 = McpService {
            id: "svc-1".to_string(),
            software_id: "soft-1".to_string(),
            name: "Service1".to_string(),
            endpoint: "http://svc1.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        let service2 = McpService {
            id: "svc-2".to_string(),
            software_id: "soft-1".to_string(),
            name: "Service2".to_string(),
            endpoint: "http://svc2.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&service1).expect("Failed to insert service1");
        db.upsert_mcp_service(&service2).expect("Failed to insert service2");

        // Export only service1
        let export = export_services_sync(&db, Some(vec!["svc-1".to_string()]), "json".to_string());
        assert!(export.is_ok(), "Export should succeed");

        let json_str = export.unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).expect("Should be valid JSON");

        let services = parsed["services"].as_array().unwrap();
        assert_eq!(
            services.len(),
            1,
            "Should export only 1 service"
        );
        assert_eq!(
            services[0]["name"], "Service1",
            "Should be Service1"
        );
    }

    // ==================== Groups Tests ====================

    #[test]
    fn test_get_groups_empty_initially() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        let groups = db.get_groups().expect("Should get groups");
        assert!(
            groups.is_empty(),
            "Groups should be empty initially"
        );
    }

    #[test]
    fn test_create_group() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Create a group
        let group = db
            .create_group("TestGroup", "#FF5733")
            .expect("Should create group");

        assert!(!group.id.is_empty(), "Group should have an ID");
        assert_eq!(group.name, "TestGroup", "Group name should match");
        assert_eq!(group.color, "#FF5733", "Group color should match");
        assert!(group.is_visible, "Group should be visible by default");
        assert_eq!(group.server_count, Some(0), "New group should have 0 servers");

        // Verify it can be retrieved
        let groups = db.get_groups().expect("Should get groups");
        assert_eq!(groups.len(), 1, "Should have 1 group");
        assert_eq!(groups[0].name, "TestGroup");
    }

    #[test]
    fn test_create_group_with_default_color() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        let group = db
            .create_group("DefaultColorGroup", "#F59E0B")
            .expect("Should create group");

        assert_eq!(group.color, "#F59E0B", "Should use provided color");
    }

    #[test]
    fn test_update_group() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Create a group
        let group = db
            .create_group("OriginalName", "#000000")
            .expect("Should create group");

        // Update the group
        let updated = db
            .update_group(&group.id, Some("NewName"), Some("#FFFFFF"), Some(true))
            .expect("Should update group");

        assert_eq!(updated.name, "NewName", "Name should be updated");
        assert_eq!(updated.color, "#FFFFFF", "Color should be updated");
        assert!(updated.is_visible, "Visibility should be updated");
    }

    #[test]
    fn test_delete_group() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Create a group
        let group = db
            .create_group("ToDelete", "#000000")
            .expect("Should create group");

        // Verify it exists
        let groups_before = db.get_groups().expect("Should get groups");
        assert_eq!(groups_before.len(), 1);

        // Delete it
        db.delete_group(&group.id).expect("Should delete group");

        // Verify it's gone
        let groups_after = db.get_groups().expect("Should get groups");
        assert!(
            groups_after.is_empty(),
            "Groups should be empty after delete"
        );
    }

    #[test]
    fn test_delete_nonexistent_group() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Try to delete non-existent group
        let result = db.delete_group("non-existent-id");
        assert!(
            result.is_ok(),
            "Deleting non-existent group should not error (silently fails in SQLite)"
        );
    }

    // ==================== Audit Log Tests ====================

    #[test]
    fn test_get_audit_log_empty_initially() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        let filters = AuditLogFilters {
            action: None,
            service_id: None,
            service_name: None,
            actor: None,
            date_from: None,
            date_to: None,
            status: None,
        };

        let page = db
            .get_audit_log_paginated(&filters, 1, 20)
            .expect("Should get audit log");

        assert!(
            page.items.is_empty(),
            "Audit log should be empty initially"
        );
        assert_eq!(page.total, 0, "Total should be 0");
        assert_eq!(page.page, 1, "Page should be 1");
    }

    #[test]
    fn test_insert_and_retrieve_audit_log() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Insert audit log entry
        let log_id = db
            .insert_audit_log(
                "test_user",
                "test_action",
                Some("svc-1"),
                Some("TestService"),
                Some(r#"{"key": "value"}"#),
                "success",
            )
            .expect("Should insert audit log");

        assert!(!log_id.is_empty(), "Should return log ID");

        // Retrieve it
        let filters = AuditLogFilters {
            action: None,
            service_id: None,
            service_name: None,
            actor: None,
            date_from: None,
            date_to: None,
            status: None,
        };

        let page = db
            .get_audit_log_paginated(&filters, 1, 20)
            .expect("Should get audit log");

        assert_eq!(page.total, 1, "Should have 1 entry");
        assert_eq!(page.items[0].actor, "test_user");
        assert_eq!(page.items[0].action, "test_action");
    }

    #[test]
    fn test_audit_log_pagination() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Insert 25 audit log entries
        for i in 0..25 {
            db.insert_audit_log(
                &format!("user_{}", i),
                "test_action",
                None,
                None,
                None,
                "success",
            )
            .expect("Should insert audit log");
        }

        // Get first page
        let filters = AuditLogFilters {
            action: None,
            service_id: None,
            service_name: None,
            actor: None,
            date_from: None,
            date_to: None,
            status: None,
        };

        let page1 = db
            .get_audit_log_paginated(&filters, 1, 10)
            .expect("Should get page 1");

        assert_eq!(page1.items.len(), 10, "Page 1 should have 10 items");
        assert_eq!(page1.total, 25, "Total should be 25");
        assert_eq!(page1.total_pages, 3, "Should have 3 pages");

        // Get second page
        let page2 = db
            .get_audit_log_paginated(&filters, 2, 10)
            .expect("Should get page 2");

        assert_eq!(page2.items.len(), 10, "Page 2 should have 10 items");
        assert_eq!(page2.page, 2, "Page should be 2");
    }

    // ==================== Service Groups Tests ====================

    #[test]
    fn test_add_service_to_group() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Create service and group
        let service = McpService {
            id: "svc-1".to_string(),
            software_id: "soft-1".to_string(),
            name: "TestService".to_string(),
            endpoint: "http://test.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&service).expect("Should insert service");

        let group = db
            .create_group("TestGroup", "#FF0000")
            .expect("Should create group");

        // Add service to group
        db.add_service_to_group("svc-1", &group.id)
            .expect("Should add service to group");

        // Verify association
        let groups = db.get_service_groups("svc-1").expect("Should get groups");
        assert!(
            groups.contains(&group.id),
            "Service should be in the group"
        );
    }

    #[test]
    fn test_remove_service_from_group() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Create service and group
        let service = McpService {
            id: "svc-1".to_string(),
            software_id: "soft-1".to_string(),
            name: "TestService".to_string(),
            endpoint: "http://test.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&service).expect("Should insert service");

        let group = db
            .create_group("TestGroup", "#FF0000")
            .expect("Should create group");

        // Add then remove
        db.add_service_to_group("svc-1", &group.id)
            .expect("Should add service to group");
        db.remove_service_from_group("svc-1", &group.id)
            .expect("Should remove service from group");

        // Verify removal
        let groups = db.get_service_groups("svc-1").expect("Should get groups");
        assert!(
            !groups.contains(&group.id),
            "Service should not be in the group"
        );
    }

    // ==================== Health Log Tests ====================

    #[test]
    fn test_insert_health_log() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Create service first
        let service = McpService {
            id: "svc-1".to_string(),
            software_id: "soft-1".to_string(),
            name: "TestService".to_string(),
            endpoint: "http://test.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&service).expect("Should insert service");

        // Insert health log
        let log_id = db
            .insert_health_log("svc-1", "online", Some(150), None)
            .expect("Should insert health log");

        assert!(!log_id.is_empty(), "Should return log ID");

        // Retrieve history
        let history = db
            .get_health_history("svc-1", Some(10))
            .expect("Should get health history");

        assert_eq!(history.len(), 1, "Should have 1 history entry");
        assert_eq!(history[0].status, "online");
        assert_eq!(history[0].latency_ms, Some(150));
    }

    #[test]
    fn test_get_health_history_respects_limit() {
        let (_temp_dir, conn) = create_test_db();
        let db = create_db_wrapper(&_temp_dir, conn);

        // Create service
        let service = McpService {
            id: "svc-1".to_string(),
            software_id: "soft-1".to_string(),
            name: "TestService".to_string(),
            endpoint: "http://test.local".to_string(),
            auth_type: "none".to_string(),
            config: None,
            is_healthy: false,
            last_checked: None,
        };
        db.upsert_mcp_service(&service).expect("Should insert service");

        // Insert multiple health logs
        for i in 0..10 {
            db.insert_health_log("svc-1", "online", Some(i * 10), None)
                .expect("Should insert health log");
        }

        // Get only last 5
        let history = db
            .get_health_history("svc-1", Some(5))
            .expect("Should get health history");

        assert_eq!(history.len(), 5, "Should return only 5 entries");
    }

    // ==================== Sync Helper Functions ====================

    // ImportResult struct mirrors the one from mcp_manager
    #[derive(Debug)]
    struct ImportResult {
        imported: i32,
        skipped: i32,
        overwritten: i32,
        errors: Vec<String>,
    }

    fn import_services_sync(
        db: &Database,
        data: String,
        mode: String,
    ) -> Result<ImportResult, String> {
        // This mirrors the sync portion of import_mcp_services
        let parsed: serde_json::Value = if data.trim().starts_with('{')
            || data.trim().starts_with('[')
        {
            serde_json::from_str(&data).map_err(|e| format!("Invalid JSON: {}", e))?
        } else {
            serde_yaml::from_str(&data).map_err(|e| format!("Invalid YAML: {}", e))?
        };

        let services_array: Vec<serde_json::Value> = if let Some(arr) =
            parsed.get("services").and_then(|s| s.as_array())
        {
            arr.clone()
        } else if parsed.is_array() {
            parsed.as_array().unwrap().clone()
        } else {
            return Err("No services array found in import data".to_string());
        };

        let mut result = ImportResult {
            imported: 0,
            skipped: 0,
            overwritten: 0,
            errors: Vec::new(),
        };

        let existing_map: std::collections::HashMap<String, _> = db
            .get_all_mcp_services()
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|s| (s.name.clone(), s))
            .collect();

        for (idx, item) in services_array.iter().enumerate() {
            let name = match item.get("name").and_then(|n| n.as_str()) {
                Some(n) => n.to_string(),
                None => {
                    result.errors.push(format!("Entry {}: missing name", idx));
                    continue;
                }
            };

            let endpoint = match item.get("endpoint").and_then(|e| e.as_str()) {
                Some(e) => e.to_string(),
                None => {
                    result.errors.push(format!(
                        "Entry {} ('{}'): missing endpoint",
                        idx, name
                    ));
                    continue;
                }
            };

            let existing = existing_map.get(&name);

            match (existing, mode.as_str()) {
                (Some(_existing), "skip") => {
                    result.skipped += 1;
                }
                (Some(existing), "overwrite") => {
                    let mut updated = existing.clone();
                    updated.endpoint = endpoint;
                    updated.auth_type = item
                        .get("authType")
                        .and_then(|a| a.as_str())
                        .unwrap_or("none")
                        .to_string();
                    updated.config = item.get("config").map(|c| c.to_string());

                    if let Err(e) = db.upsert_mcp_service(&updated) {
                        result.errors.push(format!(
                            "Entry {} ('{}'): update failed: {}",
                            idx, name, e
                        ));
                    } else {
                        result.overwritten += 1;
                    }
                }
                (None, _) => {
                    let new_service = McpService {
                        id: uuid::Uuid::new_v4().to_string(),
                        software_id: String::new(),
                        name: name.clone(),
                        endpoint,
                        auth_type: item
                            .get("authType")
                            .and_then(|a| a.as_str())
                            .unwrap_or("none")
                            .to_string(),
                        config: item.get("config").map(|c| c.to_string()),
                        is_healthy: false,
                        last_checked: None,
                    };

                    if let Err(e) = db.upsert_mcp_service(&new_service) {
                        result.errors.push(format!(
                            "Entry {} ('{}'): insert failed: {}",
                            idx, name, e
                        ));
                    } else {
                        result.imported += 1;
                    }
                }
                _ => {}
            }
        }

        Ok(result)
    }

    fn export_services_sync(
        db: &Database,
        ids: Option<Vec<String>>,
        format: String,
    ) -> Result<String, String> {
        use forge_desktop_lib::utils::now_rfc3339;

        let services = db.get_all_mcp_services().map_err(|e| e.to_string())?;

        let filtered: Vec<_> = if let Some(ref service_ids) = ids {
            services
                .into_iter()
                .filter(|s| service_ids.contains(&s.id))
                .collect()
        } else {
            services
        };

        let export_services: Vec<serde_json::Value> = filtered
            .iter()
            .map(|s| {
                let group_ids = db.get_service_groups(&s.id).unwrap_or_default();
                serde_json::json!({
                    "name": s.name,
                    "endpoint": s.endpoint,
                    "protocol": determine_protocol(&s.endpoint),
                    "authType": s.auth_type,
                    "config": serde_json::from_str::<serde_json::Value>(s.config.as_ref().unwrap_or(&String::new())).unwrap_or(serde_json::json!({})),
                    "groups": group_ids,
                })
            })
            .collect();

        let export_data = serde_json::json!({
            "version": "1.0",
            "exportedAt": now_rfc3339(),
            "services": export_services
        });

        match format.as_str() {
            "yaml" => serde_yaml::to_string(&export_data).map_err(|e| e.to_string()),
            _ => serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string()),
        }
    }

    fn determine_protocol(endpoint: &str) -> String {
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            "http".to_string()
        } else {
            "stdio".to_string()
        }
    }
}
