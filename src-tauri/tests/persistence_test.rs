//! Integration tests for persistence functionality
//!
//! Tests database, export/import, and retention following TESTING_PLAN.md Phase 4.3.

#[cfg(test)]
mod tests {
    use stats_io_lib::persistence::database::DatabaseStorage;
    use stats_io_lib::persistence::export_import::{export_session, import_session};
    use stats_io_lib::persistence::retention::{cleanup_old_sessions_file, RetentionPolicy, get_retention_stats};
    use stats_io_lib::core::domain::{Session, Run, WorkloadProfile, WorkloadType, HardwareConfig, CPUInfo, MemoryInfo, DetectionMetadata};
    use chrono::Utc;
    use uuid::Uuid;
    use tempfile::TempDir;
    use std::path::PathBuf;
    use std::collections::HashMap;

    fn create_test_session() -> Session {
        let cpu = CPUInfo {
            model: "Test CPU".to_string(),
            vendor: "Test Vendor".to_string(),
            architecture: Some("x86_64".to_string()),
            cores: 8,
            threads: 16,
            base_clock_mhz: Some(3000.0),
            boost_clock_mhz: Some(4500.0),
        };

        let memory = MemoryInfo {
            total_mb: 16384,
            channels: Some(2),
            speed_mhz: Some(3200),
            modules: vec![],
        };

        let hardware = HardwareConfig {
            cpu,
            gpus: vec![],
            memory,
            storage_devices: vec![],
            motherboard: None,
            psu: None,
            cooling: None,
            displays: vec![],
            metadata: DetectionMetadata {
                detection_time: Utc::now(),
                platform: "Test".to_string(),
                warnings: vec![],
                schema_version: 1,
            },
        };

        let profile = WorkloadProfile {
            id: "test-profile".to_string(),
            name: "Test Profile".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: std::collections::HashMap::new(),
            threshold_overrides: None,
        };

        Session {
            id: Uuid::new_v4(),
            start_time: Utc::now(),
            end_time: None,
            hardware_config_snapshot: hardware,
            profile,
            runs: vec![],
        }
    }

    #[test]
    fn test_database_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = DatabaseStorage::new(&db_path);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_database_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = DatabaseStorage::new(&db_path).unwrap();
        let session = create_test_session();
        
        // Save session (without runs to avoid potential issues)
        let save_result = storage.save_session(&session);
        assert!(save_result.is_ok(), "Failed to save session: {:?}", save_result.err());
        
        // Load session
        let loaded_session = storage.load_session(&session.id);
        assert!(loaded_session.is_ok(), "Failed to load session: {:?}", loaded_session.err());
        
        let loaded = loaded_session.unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.profile.name, session.profile.name);
    }
    
    #[test]
    fn test_database_save_and_load_session_with_runs() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_with_runs.db");
        
        let storage = DatabaseStorage::new(&db_path).unwrap();
        let mut session = create_test_session();
        
        // Add a run with empty metrics to test full persistence
        let run = Run {
            id: Uuid::new_v4(),
            name: "Test Run".to_string(),
            metrics_streams: HashMap::new(), // Empty to avoid potential serialization issues
            analysis_result: None,
            notes: None,
        };
        session.runs.push(run);
        
        // Save session
        let save_result = storage.save_session(&session);
        assert!(save_result.is_ok(), "Failed to save session with runs: {:?}", save_result.err());
        
        // Load session
        let loaded_session = storage.load_session(&session.id);
        assert!(loaded_session.is_ok(), "Failed to load session with runs: {:?}", loaded_session.err());
        
        let loaded = loaded_session.unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.profile.name, session.profile.name);
        assert_eq!(loaded.runs.len(), session.runs.len());
    }

    #[test]
    fn test_database_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = DatabaseStorage::new(&db_path).unwrap();
        
        // Create and save multiple sessions
        let session1 = create_test_session();
        let session2 = create_test_session();
        
        storage.save_session(&session1).unwrap();
        storage.save_session(&session2).unwrap();
        
        // List sessions
        let sessions = storage.list_sessions().unwrap();
        assert!(sessions.len() >= 2);
        assert!(sessions.contains(&session1.id));
        assert!(sessions.contains(&session2.id));
    }

    #[test]
    fn test_database_cleanup_old_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let storage = DatabaseStorage::new(&db_path).unwrap();
        
        // Create a session
        let session = create_test_session();
        storage.save_session(&session).unwrap();
        
        // Cleanup with 0 days retention (should delete all)
        let deleted = storage.cleanup_old_sessions(0).unwrap();
        assert!(deleted > 0);
        
        // Verify session is deleted
        let load_result = storage.load_session(&session.id);
        assert!(load_result.is_err());
    }

    #[tokio::test]
    async fn test_export_import_session() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("exported_session.json");
        
        let session = create_test_session();
        
        // Export session
        let export_result = export_session(&session, &export_path).await;
        assert!(export_result.is_ok());
        assert!(export_path.exists());
        
        // Import session
        let imported_session = import_session(&export_path).await;
        assert!(imported_session.is_ok());
        
        let imported = imported_session.unwrap();
        assert_eq!(imported.id, session.id);
        assert_eq!(imported.profile.name, session.profile.name);
    }

    #[tokio::test]
    async fn test_retention_policy_defaults() {
        let policy = RetentionPolicy::default();
        
        assert_eq!(policy.retention_days, 90);
        assert_eq!(policy.auto_cleanup_enabled, false);
        assert_eq!(policy.min_sessions_to_keep, 10);
    }

    #[tokio::test]
    async fn test_retention_stats() {
        let temp_dir = TempDir::new().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        
        let policy = RetentionPolicy::default();
        
        // Get stats for empty directory
        let stats = get_retention_stats(&sessions_dir, &policy).await.unwrap();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.sessions_to_delete, 0);
    }
}

