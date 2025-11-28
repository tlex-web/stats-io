//! Error handling tests for new features
//!
//! Tests graceful degradation when external commands fail or are unavailable.

#[cfg(test)]
mod tests {
    use stats_io_lib::hardware::adapters::windows::WindowsHardwareDetector;
    use stats_io_lib::core::interfaces::HardwareDetector;
    use stats_io_lib::metrics::providers::gpu::GpuMetricsProviderImpl;
    use stats_io_lib::metrics::providers::storage::SysInfoStorageMetricsProvider;
    use stats_io_lib::core::interfaces::{GpuMetricsProvider, StorageMetricsProvider};
    use std::sync::Arc;
    use sysinfo::System;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_gpu_detection_graceful_degradation() {
        // Test that GPU detection doesn't crash if WMI fails
        // This is tested implicitly by ensuring detection always returns Ok
        let detector = WindowsHardwareDetector::new();
        let result = detector.get_hardware_config().await;
        
        // Should never panic, even if WMI fails
        assert!(result.is_ok(), "Hardware detection should not fail even if GPU detection fails");
        
        let config = result.unwrap();
        
        // If GPUs are not detected, should return empty vector (not error)
        // This is acceptable behavior
        for gpu in &config.gpus {
            // If GPUs are detected, they should be valid
            assert!(!gpu.model.is_empty());
        }
    }

    #[tokio::test]
    async fn test_gpu_metrics_no_gpu_graceful() {
        // Test that GPU metrics provider handles no GPU gracefully
        let provider = GpuMetricsProviderImpl::new();
        let result = provider.get_gpu_metrics().await;
        
        // Should not panic even if no GPU is available
        assert!(result.is_ok(), "GPU metrics should not fail even if no GPU is present");
        
        let metrics = result.unwrap();
        
        // Should return zero metrics (indicating unavailable)
        // This is acceptable behavior
        assert_eq!(metrics.utilization, 0.0);
    }

    #[tokio::test]
    async fn test_storage_detection_graceful_degradation() {
        // Test that storage detection doesn't crash if WMI fails
        let detector = WindowsHardwareDetector::new();
        let result = detector.get_hardware_config().await;
        
        // Should never panic, even if WMI fails
        assert!(result.is_ok(), "Hardware detection should not fail even if storage detection fails");
        
        let config = result.unwrap();
        
        // If storage is not detected, should return empty vector (not error)
        // This is acceptable behavior
        // Storage devices may be empty if detection fails, that's OK
        // If storage devices are detected, verify they have reasonable data
        for device in &config.storage_devices {
            // If storage is detected, it should have at least a model name
            // Capacity might be 0 if detection failed partially, which is acceptable
            // We just verify the structure is correct
            assert!(!device.model.is_empty() || device.capacity_mb > 0,
                    "Storage device should have either a model name or capacity");
        }
    }

    #[tokio::test]
    async fn test_storage_metrics_graceful_degradation() {
        // Test that storage metrics provider handles failures gracefully
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoStorageMetricsProvider::new(system);
        let result = provider.get_storage_metrics().await;
        
        // Should not panic even if typeperf fails or is unavailable
        assert!(result.is_ok(), "Storage metrics should not fail even if typeperf is unavailable");
        
        let metrics = result.unwrap();
        
        // Should return zero metrics (indicating unavailable)
        // This is acceptable behavior
        assert!(metrics.read_throughput_mb_per_s >= 0.0);
        assert!(metrics.write_throughput_mb_per_s >= 0.0);
    }

    #[tokio::test]
    async fn test_cpu_temperature_graceful_degradation() {
        // Test that CPU temperature detection handles failures gracefully
        use stats_io_lib::core::interfaces::CpuMetricsProvider;
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = stats_io_lib::metrics::providers::cpu::SysInfoCpuMetricsProvider::new(system);
        let result = provider.get_cpu_metrics().await;
        
        // Should not panic even if temperature sensors are unavailable
        assert!(result.is_ok(), "CPU metrics should not fail even if temperature is unavailable");
        
        let metrics = result.unwrap();
        
        // Temperature may be None if unavailable - that's acceptable
        // Other metrics should still be available
        assert!(metrics.overall_utilization >= 0.0);
        assert!(!metrics.per_core_utilization.is_empty());
        
        // If temperature is None, that's fine - just verify it doesn't break anything
        if let Some(temp) = metrics.temperature {
            assert!(temp >= 0.0 && temp <= 150.0);
        }
    }

    #[tokio::test]
    async fn test_metrics_provider_multiple_failures() {
        // Test that multiple metric provider failures don't crash the system
        let system = Arc::new(Mutex::new(System::new_all()));
        
        // Test GPU metrics
        let gpu_provider = GpuMetricsProviderImpl::new();
        let gpu_result = gpu_provider.get_gpu_metrics().await;
        assert!(gpu_result.is_ok());
        
        // Test storage metrics
        let storage_provider = SysInfoStorageMetricsProvider::new(system.clone());
        let storage_result = storage_provider.get_storage_metrics().await;
        assert!(storage_result.is_ok());
        
        // Test CPU metrics (with temperature)
        use stats_io_lib::core::interfaces::CpuMetricsProvider;
        let cpu_provider = stats_io_lib::metrics::providers::cpu::SysInfoCpuMetricsProvider::new(system);
        let cpu_result = cpu_provider.get_cpu_metrics().await;
        assert!(cpu_result.is_ok());
        
        // All should succeed even if individual metrics are unavailable
    }

    #[tokio::test]
    async fn test_hardware_detection_with_missing_components() {
        // Test that hardware detection works even if some components are missing
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        assert!(config.is_ok(), "Hardware detection should succeed even with missing components");
        
        let config = config.unwrap();
        
        // Core components should always be detected
        assert!(config.cpu.cores > 0, "CPU should always be detected");
        assert!(config.memory.total_mb > 0, "Memory should always be detected");
        
        // Optional components (GPU, Storage) may be empty, but shouldn't cause errors
        // This is acceptable behavior
    }

    #[tokio::test]
    async fn test_serialization_with_missing_data() {
        // Test that serialization works even when optional fields are None
        let system = Arc::new(Mutex::new(System::new_all()));
        
        // Get metrics that may have None values
        let gpu_provider = GpuMetricsProviderImpl::new();
        let gpu_metrics = gpu_provider.get_gpu_metrics().await.unwrap();
        
        // Serialize - should work even with None values
        let json = serde_json::to_string(&gpu_metrics);
        assert!(json.is_ok(), "Serialization should work with None values");
        
        // Deserialize
        let json_str = json.unwrap();
        let deserialized: stats_io_lib::metrics::models::GpuMetrics = 
            serde_json::from_str(&json_str).unwrap();
        
        // None values should be preserved
        assert_eq!(deserialized.vram_used_mb, gpu_metrics.vram_used_mb);
        assert_eq!(deserialized.temperature, gpu_metrics.temperature);
    }
}

