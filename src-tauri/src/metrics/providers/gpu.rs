//! GPU metrics provider
//!
//! GPU metrics require platform-specific APIs (NVML, ADL, nvidia-smi, etc.)
//! This module provides platform-specific implementations.

use crate::core::error::MetricsError;
use crate::core::interfaces::GpuMetricsProvider;
use crate::metrics::models::GpuMetrics;
use async_trait::async_trait;

// Platform-specific modules are defined inline below

/// GPU metrics provider
/// 
/// Platform-specific implementation that detects and uses the best available method
/// for GPU metrics collection.
pub struct GpuMetricsProviderImpl;

impl GpuMetricsProviderImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl GpuMetricsProvider for GpuMetricsProviderImpl {
    async fn get_gpu_metrics(&self) -> Result<GpuMetrics, MetricsError> {
        platform_get_gpu_metrics().await
    }
}

#[cfg(target_os = "windows")]
async fn platform_get_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
    windows::get_gpu_metrics().await
}

#[cfg(target_os = "linux")]
async fn platform_get_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
    linux::get_gpu_metrics().await
}

#[cfg(target_os = "macos")]
async fn platform_get_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
    macos::get_gpu_metrics().await
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
async fn platform_get_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
    Ok(GpuMetrics {
        utilization: 0.0,
        vram_used_mb: None,
        vram_total_mb: None,
        temperature: None,
        clock_core_mhz: None,
        clock_memory_mhz: None,
        power_watts: None,
    })
}

impl Default for GpuMetricsProviderImpl {
    fn default() -> Self {
        Self::new()
    }
}

// Keep PlaceholderGpuMetricsProvider for backward compatibility
/// Placeholder GPU metrics provider (deprecated, use GpuMetricsProviderImpl)
pub type PlaceholderGpuMetricsProvider = GpuMetricsProviderImpl;

#[cfg(target_os = "windows")]
mod windows {
    use crate::core::error::MetricsError;
    use crate::metrics::models::GpuMetrics;
    
    /// Get GPU metrics on Windows
    /// 
    /// Tries nvidia-smi first for NVIDIA GPUs, then falls back to WMI
    pub async fn get_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
        // Try nvidia-smi first (most accurate for NVIDIA GPUs)
        if let Ok(metrics) = get_nvidia_metrics().await {
            return Ok(metrics);
        }
        
        // Fallback to WMI for basic GPU info
        get_wmi_gpu_metrics().await
    }
    
    /// Get NVIDIA GPU metrics using nvidia-smi
    async fn get_nvidia_metrics() -> Result<GpuMetrics, MetricsError> {
        let output = tokio::process::Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu,clocks.current.graphics,clocks.current.memory,power.draw",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .await
            .map_err(|e| MetricsError::CollectionFailed(format!("nvidia-smi failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(MetricsError::CollectionFailed("nvidia-smi command failed".to_string()));
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let line = output_str.lines().next()
            .ok_or_else(|| MetricsError::CollectionFailed("No output from nvidia-smi".to_string()))?;
        
        // Parse CSV: utilization.gpu,memory.used,memory.total,temperature.gpu,clocks.current.graphics,clocks.current.memory,power.draw
        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if fields.len() < 7 {
            return Err(MetricsError::CollectionFailed("Invalid nvidia-smi output format".to_string()));
        }
        
        let utilization = fields[0].parse::<f64>()
            .map_err(|_| MetricsError::CollectionFailed("Failed to parse GPU utilization".to_string()))? / 100.0;
        
        let vram_used_mb = fields[1].parse::<u64>().ok();
        let vram_total_mb = fields[2].parse::<u64>().ok();
        let temperature = fields[3].parse::<f64>().ok();
        let clock_core_mhz = fields[4].parse::<f64>().ok();
        let clock_memory_mhz = fields[5].parse::<f64>().ok();
        let power_watts = fields[6].parse::<f64>().ok();
        
        Ok(GpuMetrics {
            utilization,
            vram_used_mb,
            vram_total_mb,
            temperature,
            clock_core_mhz,
            clock_memory_mhz,
            power_watts,
        })
    }
    
    /// Get basic GPU metrics using WMI (fallback)
    async fn get_wmi_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
        // WMI doesn't provide real-time metrics, so we return basic structure
        // with utilization set to 0 (indicating metrics unavailable)
        Ok(GpuMetrics {
            utilization: 0.0,
            vram_used_mb: None,
            vram_total_mb: None,
            temperature: None,
            clock_core_mhz: None,
            clock_memory_mhz: None,
            power_watts: None,
        })
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use crate::core::error::MetricsError;
    use crate::metrics::models::GpuMetrics;
    
    /// Get GPU metrics on Linux
    pub async fn get_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
        // Try nvidia-smi first
        if let Ok(metrics) = get_nvidia_metrics().await {
            return Ok(metrics);
        }
        
        // TODO: Add AMD GPU support (radeontop, rocm-smi)
        // TODO: Add Intel GPU support
        
        // Return zero metrics if no GPU detected
        Ok(GpuMetrics {
            utilization: 0.0,
            vram_used_mb: None,
            vram_total_mb: None,
            temperature: None,
            clock_core_mhz: None,
            clock_memory_mhz: None,
            power_watts: None,
        })
    }
    
    async fn get_nvidia_metrics() -> Result<GpuMetrics, MetricsError> {
        // Same implementation as Windows
        let output = tokio::process::Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu,clocks.current.graphics,clocks.current.memory,power.draw",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .await
            .map_err(|e| MetricsError::CollectionFailed(format!("nvidia-smi failed: {}", e)))?;
        
        if !output.status.success() {
            return Err(MetricsError::CollectionFailed("nvidia-smi command failed".to_string()));
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let line = output_str.lines().next()
            .ok_or_else(|| MetricsError::CollectionFailed("No output from nvidia-smi".to_string()))?;
        
        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if fields.len() < 7 {
            return Err(MetricsError::CollectionFailed("Invalid nvidia-smi output format".to_string()));
        }
        
        let utilization = fields[0].parse::<f64>()
            .map_err(|_| MetricsError::CollectionFailed("Failed to parse GPU utilization".to_string()))? / 100.0;
        
        let vram_used_mb = fields[1].parse::<u64>().ok();
        let vram_total_mb = fields[2].parse::<u64>().ok();
        let temperature = fields[3].parse::<f64>().ok();
        let clock_core_mhz = fields[4].parse::<f64>().ok();
        let clock_memory_mhz = fields[5].parse::<f64>().ok();
        let power_watts = fields[6].parse::<f64>().ok();
        
        Ok(GpuMetrics {
            utilization,
            vram_used_mb,
            vram_total_mb,
            temperature,
            clock_core_mhz,
            clock_memory_mhz,
            power_watts,
        })
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use crate::core::error::MetricsError;
    use crate::metrics::models::GpuMetrics;
    
    /// Get GPU metrics on macOS
    pub async fn get_gpu_metrics() -> Result<GpuMetrics, MetricsError> {
        // macOS GPU metrics require IOKit or Metal APIs
        // For now, return zero metrics
        // TODO: Implement IOKit-based GPU metrics
        Ok(GpuMetrics {
            utilization: 0.0,
            vram_used_mb: None,
            vram_total_mb: None,
            temperature: None,
            clock_core_mhz: None,
            clock_memory_mhz: None,
            power_watts: None,
        })
    }
}

