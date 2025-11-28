//! Integration tests for comparison functionality
//!
//! Tests the comparison workflow end-to-end following IMPLEMENTATION_PLAN.md Phase 3.6.

#[cfg(test)]
mod tests {
    use stats_io_lib::analysis::comparison::{compare_runs, BottleneckStatus};
    use stats_io_lib::core::domain::{
        Bottleneck, BottleneckAnalysisResult, BottleneckType, HardwareConfig, MetricSample,
        MetricType, Run, WorkloadProfile, WorkloadType,
    };
    use std::collections::HashMap;

    fn create_test_run(
        id: &str,
        name: &str,
        cpu_avg: f64,
        gpu_avg: f64,
        memory_avg: f64,
        bottlenecks: Vec<Bottleneck>,
    ) -> Run {
        let mut metrics = HashMap::new();
        metrics.insert(
            "cpu".to_string(),
            vec![MetricSample {
                timestamp: chrono::Utc::now(),
                metric_type: MetricType::CpuUtilization,
                value: cpu_avg,
                unit: "%".to_string(),
                source_component: "CPU".to_string(),
            }],
        );
        metrics.insert(
            "gpu".to_string(),
            vec![MetricSample {
                timestamp: chrono::Utc::now(),
                metric_type: MetricType::GpuUtilization,
                value: gpu_avg,
                unit: "%".to_string(),
                source_component: "GPU".to_string(),
            }],
        );
        metrics.insert(
            "memory".to_string(),
            vec![MetricSample {
                timestamp: chrono::Utc::now(),
                metric_type: MetricType::MemoryUsage,
                value: memory_avg,
                unit: "%".to_string(),
                source_component: "Memory".to_string(),
            }],
        );

        Run {
            id: uuid::Uuid::parse_str(id).unwrap(),
            name: name.to_string(),
            metrics_streams: metrics,
            analysis_result: Some(BottleneckAnalysisResult {
                bottlenecks,
                analysis_timestamp: chrono::Utc::now(),
            }),
            notes: None,
        }
    }

    #[test]
    fn test_compare_runs_basic() {
        let run1 = create_test_run(
            "00000000-0000-0000-0000-000000000001",
            "Run 1",
            90.0,
            50.0,
            60.0,
            vec![Bottleneck {
                bottleneck_type: BottleneckType::Cpu,
                severity: 85,
                evidence: vec![],
                summary: "CPU-bound".to_string(),
                details: "High CPU usage".to_string(),
            }],
        );

        let run2 = create_test_run(
            "00000000-0000-0000-0000-000000000002",
            "Run 2",
            50.0,
            90.0,
            60.0,
            vec![Bottleneck {
                bottleneck_type: BottleneckType::Gpu,
                severity: 80,
                evidence: vec![],
                summary: "GPU-bound".to_string(),
                details: "High GPU usage".to_string(),
            }],
        );

        let comparison = compare_runs(&run1, &run2);

        assert_eq!(comparison.run1_id, run1.id.to_string());
        assert_eq!(comparison.run2_id, run2.id.to_string());
        assert!(!comparison.metric_deltas.is_empty());
        assert!(!comparison.bottleneck_changes.is_empty());
    }

    #[test]
    fn test_compare_runs_bottleneck_resolved() {
        let run1 = create_test_run(
            "00000000-0000-0000-0000-000000000003",
            "Run 1",
            95.0,
            50.0,
            60.0,
            vec![Bottleneck {
                bottleneck_type: BottleneckType::Cpu,
                severity: 90,
                evidence: vec![],
                summary: "CPU-bound".to_string(),
                details: "High CPU usage".to_string(),
            }],
        );

        let run2 = create_test_run(
            "00000000-0000-0000-0000-000000000004",
            "Run 2",
            50.0,
            50.0,
            60.0,
            vec![], // No bottlenecks
        );

        let comparison = compare_runs(&run1, &run2);

        // Should detect that CPU bottleneck was resolved
        let cpu_change = comparison
            .bottleneck_changes
            .iter()
            .find(|c| c.bottleneck_type.contains("Cpu"));
        assert!(cpu_change.is_some());
        assert!(matches!(cpu_change.unwrap().status, BottleneckStatus::Resolved));
    }

    #[test]
    fn test_compare_runs_metric_deltas() {
        let run1 = create_test_run(
            "00000000-0000-0000-0000-000000000005",
            "Run 1",
            80.0,
            70.0,
            60.0,
            vec![],
        );

        let run2 = create_test_run(
            "00000000-0000-0000-0000-000000000006",
            "Run 2",
            90.0,
            50.0,
            60.0,
            vec![],
        );

        let comparison = compare_runs(&run1, &run2);

        // CPU should increase by 10%
        let cpu_delta = comparison
            .metric_deltas
            .values()
            .find(|d| d.metric_type.contains("Cpu"));
        assert!(cpu_delta.is_some());
        let delta = cpu_delta.unwrap();
        assert_eq!(delta.run1_avg, 80.0);
        assert_eq!(delta.run2_avg, 90.0);
        assert_eq!(delta.delta, 10.0);
        assert!((delta.delta_percent - 12.5).abs() < 0.1); // 10/80 * 100 = 12.5%
    }
}

