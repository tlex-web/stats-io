//! Central metrics collector
//!
//! This module implements the central metrics collector that runs in a background
//! Tokio task and collects metrics from all providers at configurable intervals.

use crate::core::domain::{MetricSample, MetricType};
use crate::core::error::MetricsError;
use crate::core::interfaces::{
    CpuMetricsProvider, GpuMetricsProvider, MemoryMetricsProvider, StorageMetricsProvider,
};
use crate::metrics::providers::{
        GpuMetricsProviderImpl, SysInfoCpuMetricsProvider, SysInfoMemoryMetricsProvider,
    SysInfoStorageMetricsProvider,
};
use chrono::Utc;
use std::collections::VecDeque;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::{interval, Duration};

/// Metrics collector configuration
#[derive(Debug, Clone)]
pub struct MetricsCollectorConfig {
    /// Sampling interval in milliseconds
    pub sampling_interval_ms: u64,
    /// Maximum number of samples to keep in buffer
    pub buffer_size: usize,
}

impl Default for MetricsCollectorConfig {
    fn default() -> Self {
        Self {
            sampling_interval_ms: 1000, // 1 second default
            buffer_size: 600,           // 10 minutes at 1s intervals
        }
    }
}

/// Central metrics collector
pub struct MetricsCollector {
    config: MetricsCollectorConfig,
    buffer: Arc<RwLock<VecDeque<MetricSample>>>,
    cpu_provider: Arc<dyn CpuMetricsProvider>,
    gpu_provider: Arc<dyn GpuMetricsProvider>,
    memory_provider: Arc<dyn MemoryMetricsProvider>,
    storage_provider: Arc<dyn StorageMetricsProvider>,
    sender: broadcast::Sender<Vec<MetricSample>>,
    running: Arc<Mutex<bool>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsCollectorConfig) -> Self {
        // Create shared system instance for providers
        let system = Arc::new(Mutex::new(System::new_all()));
        let buffer_size = config.buffer_size;
        
        let cpu_provider = Arc::new(SysInfoCpuMetricsProvider::new(system.clone()))
            as Arc<dyn CpuMetricsProvider>;
        let gpu_provider = Arc::new(GpuMetricsProviderImpl::new())
            as Arc<dyn GpuMetricsProvider>;
        let memory_provider = Arc::new(SysInfoMemoryMetricsProvider::new(system.clone()))
            as Arc<dyn MemoryMetricsProvider>;
        let storage_provider = Arc::new(SysInfoStorageMetricsProvider::new(system))
            as Arc<dyn StorageMetricsProvider>;
        
        let (sender, _) = broadcast::channel(100);
        
        Self {
            config,
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(buffer_size))),
            cpu_provider,
            gpu_provider,
            memory_provider,
            storage_provider,
            sender,
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the metrics collection loop
    pub async fn start(&self) -> Result<(), MetricsError> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(MetricsError::Unknown("Collector already running".to_string()));
        }
        *running = true;
        drop(running);
        
        let sampling_interval_ms = self.config.sampling_interval_ms;
        let buffer_size = self.config.buffer_size;
        let buffer = self.buffer.clone();
        let cpu_provider = self.cpu_provider.clone();
        let gpu_provider = self.gpu_provider.clone();
        let memory_provider = self.memory_provider.clone();
        let storage_provider = self.storage_provider.clone();
        let sender = self.sender.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(sampling_interval_ms));
            
            loop {
                interval.tick().await;
                
                // Check if we should stop
                {
                    let r = running.lock().await;
                    if !*r {
                        break;
                    }
                }
                
                // Collect metrics from all providers
                let mut samples = Vec::new();
                let timestamp = Utc::now();
                
                // CPU metrics
                if let Ok(cpu_metrics) = cpu_provider.get_cpu_metrics().await {
                    samples.push(MetricSample {
                        timestamp,
                        metric_type: MetricType::CpuUtilization,
                        value: cpu_metrics.overall_utilization * 100.0, // Convert to percentage
                        unit: "percent".to_string(),
                        source_component: "CPU".to_string(),
                    });
                    
                    // Per-core utilization
                    for (idx, util) in cpu_metrics.per_core_utilization.iter().enumerate() {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::CpuUtilizationPerCore,
                            value: *util * 100.0,
                            unit: "percent".to_string(),
                            source_component: format!("CPU Core {}", idx),
                        });
                    }
                }
                
                // GPU metrics
                if let Ok(gpu_metrics) = gpu_provider.get_gpu_metrics().await {
                    if gpu_metrics.utilization > 0.0 {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::GpuUtilization,
                            value: gpu_metrics.utilization * 100.0,
                            unit: "percent".to_string(),
                            source_component: "GPU".to_string(),
                        });
                    }
                    
                    if let Some(vram_used) = gpu_metrics.vram_used_mb {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::GpuVramUsage,
                            value: vram_used as f64,
                            unit: "MB".to_string(),
                            source_component: "GPU".to_string(),
                        });
                    }
                }
                
                // Memory metrics
                if let Ok(memory_metrics) = memory_provider.get_memory_metrics().await {
                    let usage_percent = (memory_metrics.used_mb as f64 / memory_metrics.total_mb as f64) * 100.0;
                    samples.push(MetricSample {
                        timestamp,
                        metric_type: MetricType::MemoryUsage,
                        value: usage_percent,
                        unit: "percent".to_string(),
                        source_component: "Memory".to_string(),
                    });
                    
                    if let Some(swap_used) = memory_metrics.swap_used_mb {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::MemorySwapUsage,
                            value: swap_used as f64,
                            unit: "MB".to_string(),
                            source_component: "Memory".to_string(),
                        });
                    }
                }
                
                // Storage metrics
                if let Ok(storage_metrics) = storage_provider.get_storage_metrics().await {
                    if storage_metrics.read_throughput_mb_per_s > 0.0 {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::StorageReadThroughput,
                            value: storage_metrics.read_throughput_mb_per_s,
                            unit: "MB/s".to_string(),
                            source_component: "Storage".to_string(),
                        });
                    }
                    
                    if storage_metrics.write_throughput_mb_per_s > 0.0 {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::StorageWriteThroughput,
                            value: storage_metrics.write_throughput_mb_per_s,
                            unit: "MB/s".to_string(),
                            source_component: "Storage".to_string(),
                        });
                    }
                    
                    if let Some(queue_depth) = storage_metrics.queue_depth {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::StorageQueueDepth,
                            value: queue_depth as f64,
                            unit: "requests".to_string(),
                            source_component: "Storage".to_string(),
                        });
                    }
                }
                
                // CPU temperature (if available)
                if let Ok(cpu_metrics) = cpu_provider.get_cpu_metrics().await {
                    if let Some(temp) = cpu_metrics.temperature {
                        samples.push(MetricSample {
                            timestamp,
                            metric_type: MetricType::Temperature,
                            value: temp,
                            unit: "Celsius".to_string(),
                            source_component: "CPU".to_string(),
                        });
                    }
                }
                
                // Add samples to buffer
                {
                    let mut buf = buffer.write().await;
                    for sample in &samples {
                        buf.push_back(sample.clone());
                        if buf.len() > buffer_size {
                            buf.pop_front();
                        }
                    }
                }
                
                // Broadcast to subscribers (for internal use)
                let _ = sender.send(samples.clone());
                
                // Note: Tauri events will be emitted from the Tauri command layer
                // to avoid coupling the collector with Tauri directly
            }
        });
        
        Ok(())
    }
    
    /// Stop the metrics collection loop
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
    
    /// Get a receiver for metrics updates
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<MetricSample>> {
        self.sender.subscribe()
    }
    
    /// Get current metrics buffer
    pub async fn get_buffer(&self) -> Vec<MetricSample> {
        let buffer = self.buffer.read().await;
        buffer.iter().cloned().collect()
    }
    
    /// Get metrics for a specific time range
    pub async fn get_metrics_in_range(
        &self,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
    ) -> Vec<MetricSample> {
        let buffer = self.buffer.read().await;
        buffer
            .iter()
            .filter(|sample| sample.timestamp >= start && sample.timestamp <= end)
            .cloned()
            .collect()
    }
}
