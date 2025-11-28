//! Test fixtures
//!
//! This module contains test data and helper functions for testing.

use stats_io_lib::core::domain::*;
use chrono::{Duration, Utc};

/// Create a mock hardware configuration for testing
pub fn mock_hardware_config() -> HardwareConfig {
    HardwareConfig {
        cpu: CPUInfo {
            model: "Intel Core i7-12700K".to_string(),
            vendor: "Intel".to_string(),
            cores: 8,
            threads: 16,
            base_clock_mhz: Some(3600.0),
            boost_clock_mhz: Some(5000.0),
            architecture: Some("x86_64".to_string()),
        },
        gpus: vec![GPUInfo {
            model: "NVIDIA GeForce RTX 3080".to_string(),
            vendor: "NVIDIA".to_string(),
            vram_total_mb: Some(10240),
            driver_version: Some("537.58".to_string()),
            pci_id: Some("10DE:2206".to_string()),
        }],
        memory: MemoryInfo {
            total_mb: 32768,
            channels: Some(2),
            speed_mhz: Some(3600),
            modules: vec![
                MemoryModule {
                    size_mb: 16384,
                    speed_mhz: Some(3600),
                    manufacturer: Some("Corsair".to_string()),
                },
                MemoryModule {
                    size_mb: 16384,
                    speed_mhz: Some(3600),
                    manufacturer: Some("Corsair".to_string()),
                },
            ],
        },
        storage_devices: vec![StorageInfo {
            model: "Samsung 980 PRO".to_string(),
            capacity_mb: 1_000_000, // 1TB
            storage_type: StorageType::NVMe,
            interface: Some("PCIe 4.0 x4".to_string()),
        }],
        motherboard: Some(MotherboardInfo {
            model: "ASUS ROG STRIX Z690-E".to_string(),
            manufacturer: "ASUS".to_string(),
            chipset: Some("Intel Z690".to_string()),
            bios_version: Some("2801".to_string()),
        }),
        psu: Some(PSUInfo {
            wattage: 850,
            efficiency_rating: Some("80+ Gold".to_string()),
        }),
        cooling: Some(CoolingInfo {
            cpu_cooler_type: Some("AIO".to_string()),
            case_fans: Some(6),
        }),
        displays: vec![DisplayInfo {
            name: "LG 27GL850".to_string(),
            resolution_width: 2560,
            resolution_height: 1440,
            refresh_rate_hz: Some(144),
            gpu_attachment: Some("NVIDIA GeForce RTX 3080".to_string()),
        }],
        metadata: DetectionMetadata {
            detection_time: Utc::now(),
            platform: "windows".to_string(),
            warnings: vec![],
            schema_version: 1,
        },
    }
}

/// Create a mock workload profile for testing
pub fn mock_workload_profile() -> WorkloadProfile {
    use std::collections::HashMap;
    
    WorkloadProfile {
        id: "gaming-1080p-60fps".to_string(),
        name: "1080p 60 FPS Gaming".to_string(),
        workload_type: WorkloadType::Gaming,
        parameters: {
            let mut params = HashMap::new();
            params.insert("target_fps".to_string(), serde_json::json!(60));
            params.insert("resolution".to_string(), serde_json::json!("1920x1080"));
            params
        },
        threshold_overrides: None,
    }
}

/// Create mock CPU-bound metrics for testing
pub fn mock_cpu_bound_metrics() -> Vec<MetricSample> {
    use stats_io_lib::core::domain::{MetricSample, MetricType};
    
    let mut samples = Vec::new();
    let start_time = Utc::now();
    
    for i in 0..30 {
        samples.push(MetricSample {
            timestamp: start_time + Duration::seconds(i),
            metric_type: MetricType::CpuUtilization,
            value: 0.95, // 95% CPU utilization
            unit: "percent".to_string(),
            source_component: "CPU".to_string(),
        });
        
        samples.push(MetricSample {
            timestamp: start_time + Duration::seconds(i),
            metric_type: MetricType::GpuUtilization,
            value: 0.45, // 45% GPU utilization (underutilized)
            unit: "percent".to_string(),
            source_component: "GPU".to_string(),
        });
    }
    
    samples
}

/// Create mock GPU-bound metrics for testing
pub fn mock_gpu_bound_metrics() -> Vec<MetricSample> {
    use stats_io_lib::core::domain::{MetricSample, MetricType};
    
    let mut samples = Vec::new();
    let start_time = Utc::now();
    
    for i in 0..30 {
        samples.push(MetricSample {
            timestamp: start_time + Duration::seconds(i),
            metric_type: MetricType::CpuUtilization,
            value: 0.65, // 65% CPU utilization
            unit: "percent".to_string(),
            source_component: "CPU".to_string(),
        });
        
        samples.push(MetricSample {
            timestamp: start_time + Duration::seconds(i),
            metric_type: MetricType::GpuUtilization,
            value: 0.98, // 98% GPU utilization
            unit: "percent".to_string(),
            source_component: "GPU".to_string(),
        });
    }
    
    samples
}
