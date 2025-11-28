//! Metrics data models
//!
//! This module defines the data structures for metrics collection.

use serde::{Deserialize, Serialize};

/// CPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub overall_utilization: f64, // 0.0 - 1.0
    pub per_core_utilization: Vec<f64>,
    pub temperature: Option<f64>, // Celsius
}

/// GPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub utilization: f64, // 0.0 - 1.0
    pub vram_used_mb: Option<u64>,
    pub vram_total_mb: Option<u64>,
    pub temperature: Option<f64>, // Celsius
    pub clock_core_mhz: Option<f64>,
    pub clock_memory_mhz: Option<f64>,
    pub power_watts: Option<f64>,
}

/// Memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub used_mb: u64,
    pub total_mb: u64,
    pub swap_used_mb: Option<u64>,
    pub swap_total_mb: Option<u64>,
}

/// Storage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub read_throughput_mb_per_s: f64,
    pub write_throughput_mb_per_s: f64,
    pub queue_depth: Option<u32>,
    pub latency_ms: Option<f64>,
}

/// Workload KPIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadKPIs {
    pub fps: Option<f64>,
    pub frame_time_ms: Option<f64>,
    pub render_time_ms: Option<f64>,
    pub samples_per_second: Option<f64>,
    pub tokens_per_second: Option<f64>,
}

