//! Trait definitions for the application
//!
//! This module defines all trait interfaces used throughout the application,
//! following the interface contracts outlined in AGENT.md Section 5.3.

use crate::core::domain::HardwareConfig;
use crate::core::error::{HardwareError, MetricsError};
use async_trait::async_trait;

/// Hardware detection trait
///
/// Implementations should provide hardware configuration information
/// by querying platform-specific APIs.
#[async_trait]
pub trait HardwareDetector: Send + Sync {
    /// Get the current hardware configuration
    ///
    /// Returns a `HardwareConfig` containing all detected hardware components.
    /// May return partial data if some components cannot be detected.
    async fn get_hardware_config(&self) -> Result<HardwareConfig, HardwareError>;
    
    /// Refresh the hardware configuration
    ///
    /// Forces a new detection, bypassing any cache.
    async fn refresh(&self) -> Result<HardwareConfig, HardwareError>;
}

/// CPU metrics provider trait
#[async_trait]
pub trait CpuMetricsProvider: Send + Sync {
    /// Get current CPU metrics
    async fn get_cpu_metrics(&self) -> Result<crate::metrics::models::CpuMetrics, MetricsError>;
}

/// GPU metrics provider trait
#[async_trait]
pub trait GpuMetricsProvider: Send + Sync {
    /// Get current GPU metrics
    async fn get_gpu_metrics(&self) -> Result<crate::metrics::models::GpuMetrics, MetricsError>;
}

/// Memory metrics provider trait
#[async_trait]
pub trait MemoryMetricsProvider: Send + Sync {
    /// Get current memory metrics
    async fn get_memory_metrics(&self) -> Result<crate::metrics::models::MemoryMetrics, MetricsError>;
}

/// Storage metrics provider trait
#[async_trait]
pub trait StorageMetricsProvider: Send + Sync {
    /// Get current storage metrics
    async fn get_storage_metrics(&self) -> Result<crate::metrics::models::StorageMetrics, MetricsError>;
}

/// Workload KPI provider trait
///
/// Provides workload-specific key performance indicators (FPS, render times, etc.)
#[async_trait]
pub trait WorkloadKPIProvider: Send + Sync {
    /// Get current workload KPIs
    async fn get_kpis(&self) -> Result<crate::metrics::models::WorkloadKPIs, MetricsError>;
}

