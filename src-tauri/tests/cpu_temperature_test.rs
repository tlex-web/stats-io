//! Tests for CPU temperature detection functionality
//!
//! Tests CPU temperature detection on Windows, Linux, and macOS

#[cfg(test)]
mod tests {
    use stats_io_lib::metrics::providers::cpu::SysInfoCpuMetricsProvider;
    use stats_io_lib::core::interfaces::CpuMetricsProvider;
    use std::sync::Arc;
    use sysinfo::System;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_cpu_metrics_with_temperature() {
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoCpuMetricsProvider::new(system);
        
        let result = provider.get_cpu_metrics().await;
        assert!(result.is_ok(), "CPU metrics should be retrievable");
        
        let metrics = result.unwrap();
        
        // Temperature may or may not be available
        // If available, verify it's reasonable
        if let Some(temp) = metrics.temperature {
            assert!(temp >= 0.0 && temp <= 150.0,
                    "CPU temperature should be between 0°C and 150°C, got {}°C", temp);
        }
        
        // Other metrics should always be available
        assert!(metrics.overall_utilization >= 0.0 && metrics.overall_utilization <= 1.0);
        assert!(!metrics.per_core_utilization.is_empty());
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_windows_cpu_temperature_wmi() {
        // Test WMI temperature query
        // This may return None if thermal sensors aren't available
        // That's acceptable - we just verify it doesn't crash
        
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoCpuMetricsProvider::new(system);
        
        let metrics = provider.get_cpu_metrics().await.unwrap();
        
        // Temperature may be None (if sensors unavailable)
        // If present, verify it's reasonable
        if let Some(temp) = metrics.temperature {
            assert!(temp >= 0.0 && temp <= 150.0);
        }
    }

    #[tokio::test]
    #[cfg(target_os = "linux")]
    async fn test_linux_cpu_temperature_thermal_zones() {
        // Test reading from /sys/class/thermal
        // May return None if thermal zones don't exist
        // That's acceptable
        
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoCpuMetricsProvider::new(system);
        
        let metrics = provider.get_cpu_metrics().await.unwrap();
        
        // Temperature may be None (if thermal zones unavailable)
        // If present, verify it's reasonable
        if let Some(temp) = metrics.temperature {
            assert!(temp >= 0.0 && temp <= 150.0);
        }
    }

    #[tokio::test]
    async fn test_cpu_temperature_doesnt_affect_other_metrics() {
        // Verify that temperature detection doesn't break other CPU metrics
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoCpuMetricsProvider::new(system);
        
        let metrics = provider.get_cpu_metrics().await.unwrap();
        
        // Other metrics should always be available
        assert!(metrics.overall_utilization >= 0.0);
        assert!(!metrics.per_core_utilization.is_empty());
        
        // Temperature is optional
        // If None, that's fine - just verify it doesn't break anything
    }

    #[tokio::test]
    async fn test_cpu_temperature_serialization() {
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoCpuMetricsProvider::new(system);
        let metrics = provider.get_cpu_metrics().await.unwrap();
        
        // Verify temperature (if present) is serializable
        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok(), "CPU metrics with temperature should be serializable");
        
        let json_str = json.unwrap();
        let deserialized: stats_io_lib::metrics::models::CpuMetrics = 
            serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(deserialized.temperature, metrics.temperature);
    }
}

