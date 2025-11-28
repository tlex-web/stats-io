//! Regression tests
//!
//! Tests to ensure existing functionality still works after new changes.

#[cfg(test)]
mod tests {
    use stats_io_lib::core::domain::{
        HardwareConfig, Session, Run, WorkloadProfile, WorkloadType,
        MetricSample, MetricType, BottleneckAnalysisResult,
    };
    use stats_io_lib::analysis::rules::analyze_bottlenecks;
    use stats_io_lib::core::profiles::WorkloadProfiles;
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_preset_profiles_available() {
        let profiles = WorkloadProfiles::get_presets();
        
        // Should have at least some preset profiles
        assert!(!profiles.is_empty());
        
        // Verify profile structure
        for profile in &profiles {
            assert!(!profile.id.is_empty());
            assert!(!profile.name.is_empty());
        }
    }

    #[test]
    fn test_basic_cpu_detection_still_works() {
        let mut metrics = Vec::new();
        
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::CpuUtilization,
                value: 90.0,
                unit: "%".to_string(),
                source_component: "CPU".to_string(),
            });
        }
        
        let result = analyze_bottlenecks(&metrics, 30, None);
        
        // Should detect CPU bottleneck
        let cpu_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Cpu));
        
        assert!(cpu_bottleneck.is_some());
    }

    #[test]
    fn test_basic_gpu_detection_still_works() {
        let mut metrics = Vec::new();
        
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::GpuUtilization,
                value: 95.0,
                unit: "%".to_string(),
                source_component: "GPU".to_string(),
            });
        }
        
        let result = analyze_bottlenecks(&metrics, 30, None);
        
        // Should detect GPU bottleneck
        let gpu_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Gpu));
        
        assert!(gpu_bottleneck.is_some());
    }

    #[test]
    fn test_session_structure_compatibility() {
        // Verify Session structure matches expected format
        let session = Session {
            id: Uuid::new_v4(),
            start_time: Utc::now(),
            end_time: None,
            hardware_config_snapshot: HardwareConfig {
                cpu: stats_io_lib::core::domain::CPUInfo {
                    model: "Test".to_string(),
                    vendor: "Test".to_string(),
                    architecture: None,
                    cores: 4,
                    threads: 8,
                    base_clock_mhz: None,
                    boost_clock_mhz: None,
                },
                gpus: vec![],
                memory: stats_io_lib::core::domain::MemoryInfo {
                    total_mb: 8192,
                    channels: None,
                    speed_mhz: None,
                    modules: vec![],
                },
                storage_devices: vec![],
                motherboard: None,
                psu: None,
                cooling: None,
                displays: vec![],
                metadata: stats_io_lib::core::domain::DetectionMetadata {
                    detection_time: Utc::now(),
                    platform: "Test".to_string(),
                    warnings: vec![],
                    schema_version: 1,
                },
            },
            profile: WorkloadProfile {
                id: "test".to_string(),
                name: "Test".to_string(),
                workload_type: WorkloadType::General,
                parameters: HashMap::new(),
                threshold_overrides: None,
            },
            runs: vec![],
        };
        
        // Verify session can be serialized (for persistence)
        let json = serde_json::to_string(&session);
        assert!(json.is_ok());
        
        // Verify session can be deserialized
        let json_str = json.unwrap();
        let deserialized: Result<Session, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        
        let deserialized_session = deserialized.unwrap();
        assert_eq!(deserialized_session.id, session.id);
    }

    #[test]
    fn test_run_structure_compatibility() {
        let run = Run {
            id: Uuid::new_v4(),
            name: "Test Run".to_string(),
            metrics_streams: HashMap::new(),
            analysis_result: Some(BottleneckAnalysisResult {
                bottlenecks: vec![],
                timestamp: Utc::now(),
            }),
            notes: Some("Test".to_string()),
        };
        
        // Verify run can be serialized
        let json = serde_json::to_string(&run);
        assert!(json.is_ok());
        
        // Verify run can be deserialized
        let json_str = json.unwrap();
        let deserialized: Result<Run, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_workload_profile_serialization() {
        let profile = WorkloadProfile {
            id: "test".to_string(),
            name: "Test Profile".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: {
                let mut params = HashMap::new();
                params.insert("target_fps".to_string(), serde_json::json!(60));
                params
            },
            threshold_overrides: None,
        };
        
        // Verify profile can be serialized
        let json = serde_json::to_string(&profile);
        assert!(json.is_ok());
        
        // Verify profile can be deserialized
        let json_str = json.unwrap();
        let deserialized: Result<WorkloadProfile, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        
        let deserialized_profile = deserialized.unwrap();
        assert_eq!(deserialized_profile.id, profile.id);
        assert_eq!(deserialized_profile.workload_type, profile.workload_type);
    }
}

