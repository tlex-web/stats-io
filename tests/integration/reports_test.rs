//! Integration tests for report generation
//!
//! Tests report generation functionality following IMPLEMENTATION_PLAN.md Phase 3.6.

#[cfg(test)]
mod tests {
    use stats_io_lib::persistence::reports::{
        generate_session_report, ReportConfig, ReportFormat,
    };
    use stats_io_lib::core::domain::{
        HardwareConfig, Session, WorkloadProfile, WorkloadType,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_session() -> Session {
        let cpu = stats_io_lib::core::domain::CPUInfo {
            model: "Test CPU".to_string(),
            vendor: "Test Vendor".to_string(),
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
    fn test_generate_text_report() {
        let session = create_test_session();
        let hardware = session.hardware_config_snapshot.clone();
        let config = ReportConfig {
            include_hardware: true,
            include_metrics: true,
            include_analysis: true,
            include_recommendations: true,
            include_comparison: false,
            format: ReportFormat::Text,
        };

        let report = generate_session_report(&session, &hardware, &config);

        assert!(report.contains("PC RIG HARDWARE"));
        assert!(report.contains("Test CPU"));
        assert!(report.contains("Session:"));
    }

    #[test]
    fn test_generate_html_report() {
        let session = create_test_session();
        let hardware = session.hardware_config_snapshot.clone();
        let config = ReportConfig {
            include_hardware: true,
            include_metrics: true,
            include_analysis: true,
            include_recommendations: true,
            include_comparison: false,
            format: ReportFormat::Html,
        };

        let report = generate_session_report(&session, &hardware, &config);

        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("<html"));
        assert!(report.contains("Test CPU"));
    }

    #[test]
    fn test_generate_json_report() {
        let session = create_test_session();
        let hardware = session.hardware_config_snapshot.clone();
        let config = ReportConfig {
            include_hardware: true,
            include_metrics: true,
            include_analysis: true,
            include_recommendations: true,
            include_comparison: false,
            format: ReportFormat::Json,
        };

        let report = generate_session_report(&session, &hardware, &config);

        // JSON should be valid
        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert!(parsed.get("session").is_some());
        assert!(parsed.get("hardware").is_some());
    }

    #[test]
    fn test_report_config_excludes() {
        let session = create_test_session();
        let hardware = session.hardware_config_snapshot.clone();
        let config = ReportConfig {
            include_hardware: false,
            include_metrics: false,
            include_analysis: true,
            include_recommendations: true,
            include_comparison: false,
            format: ReportFormat::Text,
        };

        let report = generate_session_report(&session, &hardware, &config);

        // Should not contain hardware section
        assert!(!report.contains("HARDWARE CONFIGURATION"));
    }
}

