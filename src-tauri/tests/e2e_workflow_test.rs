//! End-to-end workflow tests
//!
//! Tests complete user workflows following TESTING_PLAN.md.

#[cfg(test)]
mod tests {
    use stats_io_lib::core::domain::{
        HardwareConfig, Session, Run, WorkloadProfile, WorkloadType,
        MetricSample, MetricType, BottleneckAnalysisResult,
    };
    use stats_io_lib::analysis::rules::analyze_bottlenecks;
    use stats_io_lib::persistence::export_import::export_session;
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio;

    fn create_test_metrics() -> Vec<MetricSample> {
        let mut metrics = Vec::new();
        
        // Simulate 30 seconds of metrics
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::CpuUtilization,
                value: 90.0, // High CPU
                unit: "%".to_string(),
                source_component: "CPU".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::GpuUtilization,
                value: 50.0,
                unit: "%".to_string(),
                source_component: "GPU".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::MemoryUsage,
                value: 60.0,
                unit: "%".to_string(),
                source_component: "Memory".to_string(),
            });
        }
        
        metrics
    }

    fn create_test_session() -> Session {
        let cpu = stats_io_lib::core::domain::CPUInfo {
            model: "Test CPU".to_string(),
            vendor: "Test Vendor".to_string(),
            architecture: Some("x86_64".to_string()),
            cores: 8,
            threads: 16,
            base_clock_mhz: Some(3000.0),
            boost_clock_mhz: Some(4500.0),
        };

        let memory = stats_io_lib::core::domain::MemoryInfo {
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
            metadata: stats_io_lib::core::domain::DetectionMetadata {
                detection_time: Utc::now(),
                platform: "Test".to_string(),
                warnings: vec![],
                schema_version: 1,
            },
        };

        let profile = WorkloadProfile {
            id: "gaming-profile".to_string(),
            name: "Gaming".to_string(),
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

    #[test]
    fn test_complete_session_workflow() {
        // 1. Create session
        let mut session = create_test_session();
        
        // 2. Collect metrics
        let metrics = create_test_metrics();
        
        // 3. Analyze bottlenecks
        let profile = Some(&session.profile);
        let analysis_result = analyze_bottlenecks(&metrics, 30, profile);
        
        // Verify analysis found bottlenecks
        assert!(!analysis_result.bottlenecks.is_empty());
        
        // 4. Create run with analysis
        let run = Run {
            id: Uuid::new_v4(),
            name: "Test Run".to_string(),
            metrics_streams: {
                let mut streams = HashMap::new();
                streams.insert("cpu".to_string(), 
                    metrics.iter().filter(|m| matches!(m.metric_type, MetricType::CpuUtilization))
                        .cloned().collect());
                streams.insert("gpu".to_string(),
                    metrics.iter().filter(|m| matches!(m.metric_type, MetricType::GpuUtilization))
                        .cloned().collect());
                streams.insert("memory".to_string(),
                    metrics.iter().filter(|m| matches!(m.metric_type, MetricType::MemoryUsage))
                        .cloned().collect());
                streams
            },
            analysis_result: Some(analysis_result),
            notes: Some("Test run notes".to_string()),
        };
        
        // 5. Add run to session
        session.runs.push(run);
        
        // 6. End session
        session.end_time = Some(Utc::now());
        
        // Verify session is complete
        assert!(session.end_time.is_some());
        assert_eq!(session.runs.len(), 1);
        assert!(!session.runs[0].analysis_result.as_ref().unwrap().bottlenecks.is_empty());
    }

    #[tokio::test]
    async fn test_export_workflow() {
        let session = create_test_session();
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("exported.json");
        
        // Export session
        let result = export_session(&session, &export_path).await;
        assert!(result.is_ok());
        assert!(export_path.exists());
    }

    #[test]
    fn test_analysis_with_workload_profile() {
        let metrics = create_test_metrics();
        let session = create_test_session();
        
        // Analyze with gaming profile
        let analysis = analyze_bottlenecks(&metrics, 30, Some(&session.profile));
        
        // Should detect CPU bottleneck (high CPU, low GPU in gaming scenario)
        let cpu_bottleneck = analysis.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Cpu));
        
        assert!(cpu_bottleneck.is_some());
    }
}

