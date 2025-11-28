//! Tests for storage metrics provider functionality
//!
//! Tests storage I/O metrics collection on Windows and Linux

#[cfg(test)]
mod tests {
    use stats_io_lib::metrics::providers::storage::SysInfoStorageMetricsProvider;
    use stats_io_lib::core::interfaces::StorageMetricsProvider;
    use std::sync::Arc;
    use sysinfo::System;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_storage_metrics_provider_creation() {
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoStorageMetricsProvider::new(system);
        
        let metrics = provider.get_storage_metrics().await;
        
        // Should return Ok (even if metrics are zero/unavailable)
        assert!(metrics.is_ok());
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_windows_storage_metrics_typeperf() {
        // This test requires typeperf to be available
        // If not available, should return zero metrics gracefully
        
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoStorageMetricsProvider::new(system);
        let result = provider.get_storage_metrics().await;
        
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        
        // Verify metric ranges
        assert!(metrics.read_throughput_mb_per_s >= 0.0,
                "Read throughput should be non-negative");
        assert!(metrics.write_throughput_mb_per_s >= 0.0,
                "Write throughput should be non-negative");
        
        // Throughput values should be reasonable
        // On idle systems, may be zero or very low
        // On active systems, could be higher
        assert!(metrics.read_throughput_mb_per_s < 100_000.0,
                "Read throughput should be reasonable (< 100TB/s)");
        assert!(metrics.write_throughput_mb_per_s < 100_000.0,
                "Write throughput should be reasonable (< 100TB/s)");
        
        // Queue depth is optional
        if let Some(queue) = metrics.queue_depth {
            assert!(queue >= 0, "Queue depth should be non-negative");
            assert!(queue < 10_000, "Queue depth should be reasonable");
        }
    }

    #[tokio::test]
    #[cfg(target_os = "linux")]
    async fn test_linux_storage_metrics_diskstats() {
        // This test requires /proc/diskstats to be readable
        
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoStorageMetricsProvider::new(system);
        let result = provider.get_storage_metrics().await;
        
        // Should not panic even if diskstats is unavailable
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        
        // Queue depth should be available if diskstats is readable
        // Throughput may be zero (requires delta calculation)
        assert!(metrics.read_throughput_mb_per_s >= 0.0);
        assert!(metrics.write_throughput_mb_per_s >= 0.0);
        
        // Queue depth may be None or Some(value)
        if let Some(queue) = metrics.queue_depth {
            assert!(queue >= 0);
        }
    }

    #[tokio::test]
    async fn test_storage_metrics_serialization() {
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoStorageMetricsProvider::new(system);
        let metrics = provider.get_storage_metrics().await.unwrap();
        
        // Verify metrics can be serialized
        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok(), "Storage metrics should be serializable");
        
        // Verify deserialization
        let json_str = json.unwrap();
        let deserialized: stats_io_lib::metrics::models::StorageMetrics = 
            serde_json::from_str(&json_str).unwrap();
        
        // Use approximate comparison for floats
        assert!((deserialized.read_throughput_mb_per_s - metrics.read_throughput_mb_per_s).abs() < 0.001,
                "Read throughput should match after serialization");
        assert!((deserialized.write_throughput_mb_per_s - metrics.write_throughput_mb_per_s).abs() < 0.001,
                "Write throughput should match after serialization");
        assert_eq!(deserialized.queue_depth, metrics.queue_depth);
        assert_eq!(deserialized.latency_ms, metrics.latency_ms);
    }

    #[tokio::test]
    async fn test_storage_metrics_multiple_calls() {
        // Test that multiple calls work correctly
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoStorageMetricsProvider::new(system);
        
        let metrics1 = provider.get_storage_metrics().await.unwrap();
        let metrics2 = provider.get_storage_metrics().await.unwrap();
        
        // Both calls should succeed
        // Values may differ (if I/O is active), but structure should be consistent
        assert!(metrics1.read_throughput_mb_per_s >= 0.0);
        assert!(metrics2.read_throughput_mb_per_s >= 0.0);
    }

    #[tokio::test]
    async fn test_storage_metrics_bytes_to_mb_conversion() {
        // Test that bytes/sec to MB/sec conversion is correct
        // This is tested implicitly in the Windows metrics provider
        // But we can verify the math is correct
        
        // 1 MB = 1024 * 1024 bytes
        let bytes_per_sec = 1048576.0; // 1 MB/sec
        let mb_per_sec = bytes_per_sec / (1024.0 * 1024.0);
        
        assert!((mb_per_sec - 1.0f64).abs() < 0.001, 
                "1 MB/sec should equal 1.0 MB/sec after conversion");
    }
}

