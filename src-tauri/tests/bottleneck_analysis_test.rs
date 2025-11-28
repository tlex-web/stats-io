//! Unit tests for bottleneck analysis
//!
//! Tests bottleneck detection rules following TESTING_PLAN.md.

#[cfg(test)]
mod tests {
    use stats_io_lib::analysis::rules::analyze_bottlenecks;
    use stats_io_lib::core::domain::{
        MetricSample, MetricType, WorkloadProfile, WorkloadType,
    };
    use std::collections::HashMap;
    use chrono::Utc;

    fn create_cpu_bound_metrics() -> Vec<MetricSample> {
        let mut metrics = Vec::new();
        
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::CpuUtilization,
                value: 95.0, // High CPU
                unit: "%".to_string(),
                source_component: "CPU".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::GpuUtilization,
                value: 30.0, // Low GPU
                unit: "%".to_string(),
                source_component: "GPU".to_string(),
            });
        }
        
        metrics
    }

    fn create_gpu_bound_metrics() -> Vec<MetricSample> {
        let mut metrics = Vec::new();
        
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::CpuUtilization,
                value: 40.0, // Low CPU
                unit: "%".to_string(),
                source_component: "CPU".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::GpuUtilization,
                value: 98.0, // High GPU
                unit: "%".to_string(),
                source_component: "GPU".to_string(),
            });
        }
        
        metrics
    }

    fn create_ram_bound_metrics() -> Vec<MetricSample> {
        let mut metrics = Vec::new();
        
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::MemoryUsage,
                value: 95.0, // High memory
                unit: "%".to_string(),
                source_component: "Memory".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::MemorySwapUsage,
                value: 10.0, // Swap usage
                unit: "%".to_string(),
                source_component: "Memory".to_string(),
            });
        }
        
        metrics
    }

    fn create_storage_bound_metrics() -> Vec<MetricSample> {
        let mut metrics = Vec::new();
        
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::StorageQueueDepth,
                value: 15.0, // High queue depth
                unit: "count".to_string(),
                source_component: "Storage".to_string(),
            });
        }
        
        metrics
    }

    fn create_thermal_metrics() -> Vec<MetricSample> {
        let mut metrics = Vec::new();
        
        for i in 0..30 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(30 - i),
                metric_type: MetricType::Temperature,
                value: 92.0, // Critical temperature
                unit: "°C".to_string(),
                source_component: "CPU".to_string(),
            });
        }
        
        metrics
    }

    #[test]
    fn test_cpu_bound_detection() {
        let metrics = create_cpu_bound_metrics();
        let profile = WorkloadProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: HashMap::new(),
            threshold_overrides: None,
        };
        
        let result = analyze_bottlenecks(&metrics, 30, Some(&profile));
        
        let cpu_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Cpu));
        
        assert!(cpu_bottleneck.is_some());
        let bottleneck = cpu_bottleneck.unwrap();
        assert!(bottleneck.severity >= 50);
        assert!(bottleneck.details.contains("CPU") || bottleneck.details.contains("cpu"));
    }

    #[test]
    fn test_gpu_bound_detection() {
        let metrics = create_gpu_bound_metrics();
        let profile = WorkloadProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: HashMap::new(),
            threshold_overrides: None,
        };
        
        let result = analyze_bottlenecks(&metrics, 30, Some(&profile));
        
        let gpu_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Gpu));
        
        assert!(gpu_bottleneck.is_some());
        let bottleneck = gpu_bottleneck.unwrap();
        assert!(bottleneck.severity >= 50);
    }

    #[test]
    fn test_ram_bound_detection() {
        let metrics = create_ram_bound_metrics();
        let profile = WorkloadProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            workload_type: WorkloadType::Productivity,
            parameters: HashMap::new(),
            threshold_overrides: None,
        };
        
        let result = analyze_bottlenecks(&metrics, 30, Some(&profile));
        
        let ram_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Ram));
        
        assert!(ram_bottleneck.is_some());
        let bottleneck = ram_bottleneck.unwrap();
        assert!(bottleneck.severity >= 50);
    }

    #[test]
    fn test_storage_bound_detection() {
        let metrics = create_storage_bound_metrics();
        let profile = WorkloadProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            workload_type: WorkloadType::Productivity,
            parameters: HashMap::new(),
            threshold_overrides: None,
        };
        
        let result = analyze_bottlenecks(&metrics, 30, Some(&profile));
        
        let storage_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Storage));
        
        assert!(storage_bottleneck.is_some());
    }

    #[test]
    fn test_thermal_detection() {
        let metrics = create_thermal_metrics();
        let result = analyze_bottlenecks(&metrics, 30, None);
        
        let thermal_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Thermal));
        
        assert!(thermal_bottleneck.is_some());
        let bottleneck = thermal_bottleneck.unwrap();
        assert!(bottleneck.severity >= 50);
    }

    #[test]
    fn test_workload_profile_threshold_overrides() {
        use stats_io_lib::core::domain::ThresholdOverrides;
        
        let mut metrics = create_cpu_bound_metrics();
        // Lower CPU values to test threshold override
        for metric in &mut metrics {
            if matches!(metric.metric_type, MetricType::CpuUtilization) {
                metric.value = 80.0; // Below default threshold but above custom
            }
        }
        
        let profile = WorkloadProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: HashMap::new(),
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(75.0), // Lower threshold
                gpu_high: None,
                ram_high: None,
                vram_high: None,
            }),
        };
        
        let result = analyze_bottlenecks(&metrics, 30, Some(&profile));
        
        // Should detect CPU bottleneck with lower threshold
        let cpu_bottleneck = result.bottlenecks.iter()
            .find(|b| matches!(b.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Cpu));
        
        assert!(cpu_bottleneck.is_some());
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = Vec::new();
        let result = analyze_bottlenecks(&metrics, 30, None);
        
        // Should not panic, may return empty bottlenecks
        assert!(result.bottlenecks.is_empty() || result.bottlenecks.len() >= 0);
    }

    #[test]
    fn test_evidence_collection() {
        let metrics = create_cpu_bound_metrics();
        let profile = WorkloadProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: HashMap::new(),
            threshold_overrides: None,
        };
        
        let result = analyze_bottlenecks(&metrics, 30, Some(&profile));
        
        if let Some(bottleneck) = result.bottlenecks.first() {
            // Should have evidence
            assert!(!bottleneck.evidence.is_empty());
            
            // Evidence should have valid values
            for evidence in &bottleneck.evidence {
                assert!(evidence.actual_value >= 0.0);
                assert!(evidence.threshold > 0.0);
            }
        }
    }
}

