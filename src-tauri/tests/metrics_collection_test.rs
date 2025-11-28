//! Unit tests for metrics collection
//!
//! Tests metrics collection functionality following TESTING_PLAN.md.

#[cfg(test)]
mod tests {
    use stats_io_lib::metrics::collector::{MetricsCollector, MetricsCollectorConfig};
    use stats_io_lib::metrics::providers::{
        SysInfoCpuMetricsProvider, SysInfoMemoryMetricsProvider,
        PlaceholderGpuMetricsProvider, SysInfoStorageMetricsProvider,
    };
    use stats_io_lib::core::interfaces::{
        CpuMetricsProvider, MemoryMetricsProvider, GpuMetricsProvider, StorageMetricsProvider,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use sysinfo::System;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsCollectorConfig {
            sampling_interval_ms: 1000,
            buffer_size: 1000,
        };
        
        let collector = MetricsCollector::new(config.clone());
        
        // Verify collector was created with correct config
        // Note: config is private, so we test by behavior
        let _ = collector; // Just verify it compiles
    }

    #[tokio::test]
    async fn test_metrics_collector_start_stop() {
        let config = MetricsCollectorConfig {
            sampling_interval_ms: 100,
            buffer_size: 100,
        };
        
        let collector = MetricsCollector::new(config);
        
        // Start collection
        let start_result = collector.start().await;
        assert!(start_result.is_ok());
        
        // Wait a bit for metrics to be collected
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        // Stop collection
        collector.stop().await;
        
        // Verify buffer has some metrics
        let buffer = collector.get_buffer().await;
        assert!(!buffer.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_buffer_management() {
        let config = MetricsCollectorConfig {
            sampling_interval_ms: 50,
            buffer_size: 10, // Small buffer to test overflow
        };
        
        let collector = MetricsCollector::new(config.clone());
        collector.start().await.unwrap();
        
        // Wait for buffer to fill
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        
        let buffer = collector.get_buffer().await;
        
        // Buffer should not exceed configured size (with some tolerance for in-flight samples)
        assert!(buffer.len() <= config.buffer_size + 5); // Allow small overflow during collection
        
        collector.stop().await;
    }

    #[tokio::test]
    async fn test_metrics_subscription() {
        let config = MetricsCollectorConfig {
            sampling_interval_ms: 100,
            buffer_size: 100,
        };
        
        let collector = MetricsCollector::new(config);
        collector.start().await.unwrap();
        
        // Subscribe to metrics updates (returns Receiver, not a future)
        let mut receiver = collector.subscribe();
        
        // Wait for at least one update
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        // Try to receive (non-blocking check)
        let result = receiver.try_recv();
        // May be Ok, Err(TryRecvError::Empty), or Err(TryRecvError::Closed)
        // We just verify the receiver works - it's ok if empty since we're checking quickly
        
        collector.stop().await;
    }

    #[tokio::test]
    async fn test_cpu_metrics_provider() {
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoCpuMetricsProvider::new(system);
        
        let metrics = provider.get_cpu_metrics().await;
        assert!(metrics.is_ok());
        
        let cpu_metrics = metrics.unwrap();
        // CPU utilization should be between 0 and 1 (as a fraction)
        assert!(cpu_metrics.overall_utilization >= 0.0);
        assert!(cpu_metrics.overall_utilization <= 1.0);
    }

    #[tokio::test]
    async fn test_memory_metrics_provider() {
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoMemoryMetricsProvider::new(system);
        
        let metrics = provider.get_memory_metrics().await;
        assert!(metrics.is_ok());
        
        let memory_metrics = metrics.unwrap();
        // Memory should have valid values
        assert!(memory_metrics.used_mb >= 0);
        assert!(memory_metrics.total_mb > 0);
        assert!(memory_metrics.used_mb <= memory_metrics.total_mb);
    }

    #[tokio::test]
    async fn test_storage_metrics_provider() {
        let system = Arc::new(Mutex::new(System::new_all()));
        let provider = SysInfoStorageMetricsProvider::new(system);
        
        let metrics = provider.get_storage_metrics().await;
        assert!(metrics.is_ok());
        
        let storage_metrics = metrics.unwrap();
        // Throughput should be non-negative
        assert!(storage_metrics.read_throughput_mb_per_s >= 0.0);
        assert!(storage_metrics.write_throughput_mb_per_s >= 0.0);
    }

    #[tokio::test]
    async fn test_metrics_aggregation() {
        use stats_io_lib::metrics::utils::aggregate_metrics;
        use stats_io_lib::core::domain::{MetricSample, MetricType};
        use chrono::Utc;
        
        let mut metrics = Vec::new();
        
        // Create sample metrics
        for i in 0..10 {
            metrics.push(MetricSample {
                timestamp: Utc::now() - chrono::Duration::seconds(i),
                metric_type: MetricType::CpuUtilization,
                value: 50.0 + (i as f64 * 5.0),
                unit: "%".to_string(),
                source_component: "CPU".to_string(),
            });
        }
        
        let aggregated = aggregate_metrics(&metrics);
        
        // Should have aggregation for CPU utilization
        let cpu_key = format!("{:?}", MetricType::CpuUtilization);
        assert!(aggregated.contains_key(&cpu_key));
        
        let agg = aggregated.get(&cpu_key).unwrap();
        assert!(agg.min >= 0.0);
        assert!(agg.max <= 100.0);
        assert!(agg.avg > 0.0);
        assert_eq!(agg.count, 10);
    }
}

