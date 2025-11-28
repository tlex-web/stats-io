//! Unit tests for advanced bottleneck detection
//!
//! Tests the advanced bottleneck detection rules following IMPLEMENTATION_PLAN.md Phase 4.2.

#[cfg(test)]
mod tests {
    use stats_io_lib::analysis::rules::advanced::{
        detect_enhanced_thermal_bottleneck, detect_memory_bus_saturation,
        detect_multi_gpu_bottleneck, detect_pcie_saturation,
    };
    use stats_io_lib::core::domain::{MetricSample, MetricType};
    use chrono::Utc;

    fn create_temp_metric(value: f64) -> MetricSample {
        MetricSample {
            timestamp: Utc::now(),
            metric_type: MetricType::Temperature,
            value,
            unit: "°C".to_string(),
            source_component: "CPU".to_string(),
        }
    }

    fn create_gpu_metric(value: f64, source: &str) -> MetricSample {
        MetricSample {
            timestamp: Utc::now(),
            metric_type: MetricType::GpuUtilization,
            value,
            unit: "%".to_string(),
            source_component: source.to_string(),
        }
    }

    #[test]
    fn test_pcie_saturation_detection() {
        // Create metrics with high storage throughput (indicating PCIe saturation)
        let mut metrics = Vec::new();
        
        // High storage read/write throughput
        for i in 0..10 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::StorageReadThroughput,
                value: 14000.0, // High throughput
                unit: "MB/s".to_string(),
                source_component: "Storage".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::StorageWriteThroughput,
                value: 5000.0,
                unit: "MB/s".to_string(),
                source_component: "Storage".to_string(),
            });
        }
        
        let bottleneck = detect_pcie_saturation(&metrics);
        assert!(bottleneck.is_some());
        let bottleneck = bottleneck.unwrap();
        assert_eq!(bottleneck.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Bandwidth);
    }

    #[test]
    fn test_memory_bus_saturation_detection() {
        // Create metrics with high memory throughput
        let mut metrics = Vec::new();
        
        for i in 0..10 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::MemoryReadThroughput,
                value: 45000.0, // High memory read
                unit: "MB/s".to_string(),
                source_component: "Memory".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::MemoryWriteThroughput,
                value: 10000.0,
                unit: "MB/s".to_string(),
                source_component: "Memory".to_string(),
            });
        }
        
        let bottleneck = detect_memory_bus_saturation(&metrics);
        assert!(bottleneck.is_some());
        let bottleneck = bottleneck.unwrap();
        assert_eq!(bottleneck.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Bandwidth);
    }

    #[test]
    fn test_enhanced_thermal_predictive_warning() {
        // Create metrics with rising temperature
        let mut metrics = Vec::new();
        
        // Temperature rising from 70°C to 85°C over 10 minutes
        for i in 0..10 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::minutes(10 - i as i64),
                metric_type: MetricType::Temperature,
                value: 70.0 + (i as f64 * 1.5), // Rising temperature
                unit: "°C".to_string(),
                source_component: "CPU".to_string(),
            });
        }
        
        let bottleneck = detect_enhanced_thermal_bottleneck(&metrics);
        assert!(bottleneck.is_some());
        let bottleneck = bottleneck.unwrap();
        assert_eq!(bottleneck.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Thermal);
        // Check for predictive warning indicators
        assert!(
            bottleneck.details.contains("rising") 
            || bottleneck.details.contains("Predicted") 
            || bottleneck.details.contains("throttling")
        );
    }

    #[test]
    fn test_enhanced_thermal_critical() {
        // Create metrics with critical temperature
        let mut metrics = Vec::new();
        
        for i in 0..5 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::Temperature,
                value: 90.0, // Critical temperature
                unit: "°C".to_string(),
                source_component: "CPU".to_string(),
            });
        }
        
        let bottleneck = detect_enhanced_thermal_bottleneck(&metrics);
        assert!(bottleneck.is_some());
        let bottleneck = bottleneck.unwrap();
        assert_eq!(bottleneck.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Thermal);
        assert!(bottleneck.details.contains("critical") || bottleneck.details.contains("throttling"));
    }

    #[test]
    fn test_multi_gpu_workload_imbalance() {
        // Create metrics with imbalanced GPU utilization
        let mut metrics = Vec::new();
        
        // GPU 0 at 90%, GPU 1 at 30% - imbalanced
        for i in 0..10 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::GpuUtilization,
                value: 90.0,
                unit: "%".to_string(),
                source_component: "GPU 0".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::GpuUtilization,
                value: 30.0,
                unit: "%".to_string(),
                source_component: "GPU 1".to_string(),
            });
        }
        
        let bottleneck = detect_multi_gpu_bottleneck(&metrics);
        assert!(bottleneck.is_some());
        let bottleneck = bottleneck.unwrap();
        assert_eq!(bottleneck.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Gpu);
        assert!(bottleneck.details.contains("imbalance") || bottleneck.details.contains("distributed"));
    }

    #[test]
    fn test_multi_gpu_all_saturated() {
        // Create metrics with all GPUs saturated
        let mut metrics = Vec::new();
        
        // Both GPUs at 95%
        for i in 0..10 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::GpuUtilization,
                value: 95.0,
                unit: "%".to_string(),
                source_component: "GPU 0".to_string(),
            });
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::GpuUtilization,
                value: 95.0,
                unit: "%".to_string(),
                source_component: "GPU 1".to_string(),
            });
        }
        
        let bottleneck = detect_multi_gpu_bottleneck(&metrics);
        assert!(bottleneck.is_some());
        let bottleneck = bottleneck.unwrap();
        assert_eq!(bottleneck.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Gpu);
        assert!(bottleneck.details.contains("saturated") || bottleneck.details.contains("GPU-bound"));
    }
}

