//! Unit tests for session management
//!
//! Tests session creation, persistence, and management following TESTING_PLAN.md.

#[cfg(test)]
mod tests {
    use stats_io_lib::persistence::storage::SessionStorage;
    use stats_io_lib::core::domain::{
        Session, WorkloadProfile, WorkloadType, HardwareConfig,
        CPUInfo, MemoryInfo, DetectionMetadata, Run, MetricSample, MetricType,
    };
    use chrono::Utc;
    use uuid::Uuid;
    use tempfile::TempDir;
    use std::collections::HashMap;
    use tokio;

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
            parameters: HashMap::new(),
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

    #[tokio::test]
    async fn test_session_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path().to_path_buf());
        
        // Storage should be created (verify by using it)
        let session = create_test_session();
        let save_result = storage.save_session(&session).await;
        assert!(save_result.is_ok());
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path().to_path_buf());
        
        let session = create_test_session();
        
        // Save session
        let save_result = storage.save_session(&session).await;
        assert!(save_result.is_ok());
        
        // Load session
        let loaded_session = storage.load_session(&session.id).await;
        assert!(loaded_session.is_ok());
        
        let loaded = loaded_session.unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.profile.name, session.profile.name);
        assert_eq!(loaded.hardware_config_snapshot.cpu.model, session.hardware_config_snapshot.cpu.model);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path().to_path_buf());
        
        // Create and save multiple sessions
        let session1 = create_test_session();
        let session2 = create_test_session();
        
        storage.save_session(&session1).await.unwrap();
        storage.save_session(&session2).await.unwrap();
        
        // List sessions
        let sessions = storage.list_sessions().await.unwrap();
        assert!(sessions.len() >= 2);
        assert!(sessions.contains(&session1.id));
        assert!(sessions.contains(&session2.id));
    }

    #[tokio::test]
    async fn test_delete_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path().to_path_buf());
        
        let session = create_test_session();
        storage.save_session(&session).await.unwrap();
        
        // Verify session exists
        let load_result = storage.load_session(&session.id).await;
        assert!(load_result.is_ok());
        
        // Delete session
        let delete_result = storage.delete_session(&session.id).await;
        assert!(delete_result.is_ok());
        
        // Verify session is deleted
        let load_result = storage.load_session(&session.id).await;
        assert!(load_result.is_err());
    }

    #[tokio::test]
    async fn test_session_with_runs() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path().to_path_buf());
        
        let mut session = create_test_session();
        
        // Add a run to the session
        let run = Run {
            id: Uuid::new_v4(),
            name: "Test Run".to_string(),
            metrics_streams: {
                let mut streams = HashMap::new();
                streams.insert("cpu".to_string(), vec![
                    MetricSample {
                        timestamp: Utc::now(),
                        metric_type: MetricType::CpuUtilization,
                        value: 80.0,
                        unit: "%".to_string(),
                        source_component: "CPU".to_string(),
                    }
                ]);
                streams
            },
            analysis_result: None,
            notes: Some("Test notes".to_string()),
        };
        
        session.runs.push(run);
        session.end_time = Some(Utc::now());
        
        // Save and load
        storage.save_session(&session).await.unwrap();
        let loaded = storage.load_session(&session.id).await.unwrap();
        
        assert_eq!(loaded.runs.len(), 1);
        assert_eq!(loaded.runs[0].name, "Test Run");
        assert!(loaded.end_time.is_some());
    }

    #[tokio::test]
    async fn test_load_nonexistent_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path().to_path_buf());
        
        let nonexistent_id = Uuid::new_v4();
        let result = storage.load_session(&nonexistent_id).await;
        
        assert!(result.is_err());
    }
}

