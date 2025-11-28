//! Tests for GPU metrics provider functionality
//!
//! Tests GPU metrics collection on Windows and Linux

#[cfg(test)]
mod tests {
    use stats_io_lib::metrics::providers::gpu::GpuMetricsProviderImpl;
    use stats_io_lib::core::interfaces::GpuMetricsProvider;

    #[tokio::test]
    async fn test_gpu_metrics_provider_creation() {
        let provider = GpuMetricsProviderImpl::new();
        
        // Should create without panicking
        let metrics = provider.get_gpu_metrics().await;
        
        // Should return Ok (even if metrics are zero/unavailable)
        assert!(metrics.is_ok());
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_windows_gpu_metrics_nvidia_smi() {
        // This test requires nvidia-smi to be available
        // If not available, should fallback gracefully
        
        let provider = GpuMetricsProviderImpl::new();
        let result = provider.get_gpu_metrics().await;
        
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        
        // Verify metric ranges
        assert!(metrics.utilization >= 0.0 && metrics.utilization <= 1.0,
                "GPU utilization should be between 0.0 and 1.0");
        
        // If VRAM is available, verify it's reasonable
        if let (Some(used), Some(total)) = (metrics.vram_used_mb, metrics.vram_total_mb) {
            assert!(used <= total, "Used VRAM should not exceed total VRAM");
            assert!(total > 0, "Total VRAM should be positive");
            assert!(total < 100_000, "Total VRAM should be reasonable (< 100GB)");
        }
        
        // If temperature is available, verify it's reasonable
        if let Some(temp) = metrics.temperature {
            assert!(temp >= 0.0 && temp <= 150.0,
                    "GPU temperature should be between 0°C and 150°C");
        }
        
        // If clock speeds are available, verify they're reasonable
        if let Some(clock) = metrics.clock_core_mhz {
            assert!(clock > 0.0 && clock < 10_000.0,
                    "GPU core clock should be reasonable (0-10GHz)");
        }
        
        if let Some(clock) = metrics.clock_memory_mhz {
            assert!(clock > 0.0 && clock < 10_000.0,
                    "GPU memory clock should be reasonable (0-10GHz)");
        }
        
        // If power is available, verify it's reasonable
        if let Some(power) = metrics.power_watts {
            assert!(power >= 0.0 && power < 2000.0,
                    "GPU power should be reasonable (0-2000W)");
        }
    }

    #[tokio::test]
    async fn test_gpu_metrics_serialization() {
        let provider = GpuMetricsProviderImpl::new();
        let metrics = provider.get_gpu_metrics().await.unwrap();
        
        // Verify metrics can be serialized
        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok(), "GPU metrics should be serializable");
        
        // Verify deserialization
        let json_str = json.unwrap();
        let deserialized: stats_io_lib::metrics::models::GpuMetrics = 
            serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(deserialized.utilization, metrics.utilization);
        assert_eq!(deserialized.vram_used_mb, metrics.vram_used_mb);
        assert_eq!(deserialized.vram_total_mb, metrics.vram_total_mb);
    }

    #[tokio::test]
    async fn test_gpu_metrics_no_gpu_graceful() {
        // Test that metrics provider handles no GPU gracefully
        let provider = GpuMetricsProviderImpl::new();
        let result = provider.get_gpu_metrics().await;
        
        // Should not panic even if no GPU is available
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        
        // Should return zero metrics (indicating unavailable)
        // This is acceptable behavior
        assert_eq!(metrics.utilization, 0.0);
    }

    #[tokio::test]
    async fn test_gpu_metrics_multiple_calls() {
        // Test that multiple calls work correctly
        let provider = GpuMetricsProviderImpl::new();
        
        let metrics1 = provider.get_gpu_metrics().await.unwrap();
        let metrics2 = provider.get_gpu_metrics().await.unwrap();
        
        // Both calls should succeed
        // Values may differ (if GPU is active), but structure should be consistent
        assert_eq!(metrics1.utilization >= 0.0, metrics2.utilization >= 0.0);
    }
}

