//! Unit tests for advanced bottleneck detection
//!
//! Tests advanced detection features following TESTING_PLAN.md Phase 4.2.

#[cfg(test)]
mod tests {
    use stats_io_lib::analysis::rules::advanced::{
        detect_enhanced_thermal_bottleneck, detect_memory_bus_saturation,
        detect_multi_gpu_bottleneck, detect_pcie_saturation,
    };
    use stats_io_lib::core::domain::{MetricSample, MetricType};
    use chrono::Utc;

    #[test]
    fn test_pcie_saturation_high_throughput() {
        let mut metrics = Vec::new();
        
        // High storage throughput indicating PCIe saturation
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
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::GpuUtilization,
                value: 80.0,
                unit: "%".to_string(),
                source_component: "GPU".to_string(),
            });
        }
        
        let bottleneck = detect_pcie_saturation(&metrics);
        assert!(bottleneck.is_some());
        let bottleneck = bottleneck.unwrap();
        assert_eq!(bottleneck.bottleneck_type, stats_io_lib::core::domain::BottleneckType::Bandwidth);
        assert!(bottleneck.details.contains("PCIe"));
    }

    #[test]
    fn test_memory_bus_saturation() {
        let mut metrics = Vec::new();
        
        // High memory throughput
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
        assert!(bottleneck.details.contains("Memory bus"));
    }

    #[test]
    fn test_enhanced_thermal_critical() {
        let mut metrics = Vec::new();
        
        // Critical temperature
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
        assert!(bottleneck.severity >= 75);
    }

    #[test]
    fn test_enhanced_thermal_predictive() {
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
        // Should be predictive warning
        assert!(bottleneck.details.contains("Predicted") || 
                bottleneck.details.contains("rising") ||
                bottleneck.details.contains("throttling"));
    }

    #[test]
    fn test_multi_gpu_imbalance() {
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
        assert!(bottleneck.details.contains("imbalance") || 
                bottleneck.details.contains("distributed"));
    }

    #[test]
    fn test_multi_gpu_all_saturated() {
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
        assert!(bottleneck.details.contains("saturated") || 
                bottleneck.details.contains("GPU-bound"));
    }
}

