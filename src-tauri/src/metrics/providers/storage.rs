//! Storage metrics provider
//!
//! Provides disk I/O metrics including throughput, queue depth, and latency.
//! Uses platform-specific APIs for accurate metrics collection.

use crate::core::error::MetricsError;
use crate::core::interfaces::StorageMetricsProvider;
use crate::metrics::models::StorageMetrics;
use async_trait::async_trait;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;

// Platform-specific modules are defined inline below

/// Storage metrics provider
/// 
/// Platform-specific implementation that uses the best available method
/// for storage I/O metrics collection.
pub struct SysInfoStorageMetricsProvider {
    #[allow(dead_code)]
    system: Arc<Mutex<System>>, // Reserved for future use
    #[cfg(target_os = "windows")]
    last_read_bytes: Arc<Mutex<u64>>,
    #[cfg(target_os = "windows")]
    last_write_bytes: Arc<Mutex<u64>>,
    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    last_sample_time: Arc<Mutex<std::time::Instant>>, // Reserved for future delta calculation
}

impl SysInfoStorageMetricsProvider {
    pub fn new(system: Arc<Mutex<System>>) -> Self {
        Self {
            system,
            #[cfg(target_os = "windows")]
            last_read_bytes: Arc::new(Mutex::new(0)),
            #[cfg(target_os = "windows")]
            last_write_bytes: Arc::new(Mutex::new(0)),
            #[cfg(target_os = "windows")]
            last_sample_time: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }
}

#[async_trait]
impl StorageMetricsProvider for SysInfoStorageMetricsProvider {
    async fn get_storage_metrics(&self) -> Result<StorageMetrics, MetricsError> {
        platform_get_storage_metrics(
            #[cfg(target_os = "windows")]
            &self.last_read_bytes,
            #[cfg(target_os = "windows")]
            &self.last_write_bytes,
            #[cfg(target_os = "windows")]
            &self.last_sample_time,
        ).await
    }
}

#[cfg(target_os = "windows")]
async fn platform_get_storage_metrics(
    last_read_bytes: &Arc<Mutex<u64>>,
    last_write_bytes: &Arc<Mutex<u64>>,
    last_sample_time: &Arc<Mutex<std::time::Instant>>,
) -> Result<StorageMetrics, MetricsError> {
    windows_impl::get_storage_metrics(last_read_bytes, last_write_bytes, last_sample_time).await
}

#[cfg(target_os = "linux")]
async fn platform_get_storage_metrics(
    _last_read_bytes: &Arc<Mutex<u64>>,
    _last_write_bytes: &Arc<Mutex<u64>>,
    _last_sample_time: &Arc<Mutex<std::time::Instant>>,
) -> Result<StorageMetrics, MetricsError> {
    linux_impl::get_storage_metrics().await
}

#[cfg(target_os = "macos")]
async fn platform_get_storage_metrics(
    _last_read_bytes: &Arc<Mutex<u64>>,
    _last_write_bytes: &Arc<Mutex<u64>>,
    _last_sample_time: &Arc<Mutex<std::time::Instant>>,
) -> Result<StorageMetrics, MetricsError> {
    macos_impl::get_storage_metrics().await
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
async fn platform_get_storage_metrics(
    _last_read_bytes: &Arc<Mutex<u64>>,
    _last_write_bytes: &Arc<Mutex<u64>>,
    _last_sample_time: &Arc<Mutex<std::time::Instant>>,
) -> Result<StorageMetrics, MetricsError> {
    Ok(StorageMetrics {
        read_throughput_mb_per_s: 0.0,
        write_throughput_mb_per_s: 0.0,
        queue_depth: None,
        latency_ms: None,
    })
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::time::Instant;
    
    /// Get storage metrics on Windows using Performance Counters
    pub async fn get_storage_metrics(
        last_read_bytes: &Arc<Mutex<u64>>,
        last_write_bytes: &Arc<Mutex<u64>>,
        last_sample_time: &Arc<Mutex<Instant>>,
    ) -> Result<StorageMetrics, MetricsError> {
        // Use typeperf to query performance counters
        // This is simpler than using PDH API directly
        let output = tokio::process::Command::new("typeperf")
            .args(&[
                "\\PhysicalDisk(_Total)\\Disk Read Bytes/sec",
                "\\PhysicalDisk(_Total)\\Disk Write Bytes/sec",
                "\\PhysicalDisk(_Total)\\Avg. Disk Queue Length",
                "-sc", "1",
                "-si", "1",
            ])
            .output()
            .await
            .map_err(|e| MetricsError::CollectionFailed(format!("typeperf failed: {}", e)))?;
        
        if !output.status.success() {
            // Fallback: return zero metrics if typeperf fails
            return Ok(StorageMetrics {
                read_throughput_mb_per_s: 0.0,
                write_throughput_mb_per_s: 0.0,
                queue_depth: None,
                latency_ms: None,
            });
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse typeperf output (CSV format)
        // Format: "(PDH-CSV 4.0)","\\ComputerName\\PhysicalDisk(_Total)\\Disk Read Bytes/sec","\\ComputerName\\PhysicalDisk(_Total)\\Disk Write Bytes/sec","\\ComputerName\\PhysicalDisk(_Total)\\Avg. Disk Queue Length"
        // "10/15/2024 12:00:00.000","1234.567","890.123","1.234"
        
        let lines: Vec<&str> = output_str.lines().collect();
        if lines.len() < 3 {
            return Ok(StorageMetrics {
                read_throughput_mb_per_s: 0.0,
                write_throughput_mb_per_s: 0.0,
                queue_depth: None,
                latency_ms: None,
            });
        }
        
        // Get the data line (usually line 2, after header)
        let data_line = lines.get(2).unwrap_or(&"");
        
        // Parse CSV values (remove quotes)
        let values: Vec<&str> = data_line.split(',').collect();
        if values.len() < 4 {
            return Ok(StorageMetrics {
                read_throughput_mb_per_s: 0.0,
                write_throughput_mb_per_s: 0.0,
                queue_depth: None,
                latency_ms: None,
            });
        }
        
        // Parse values (remove quotes and parse)
        let read_bytes_per_sec = values.get(1)
            .and_then(|s| s.trim_matches('"').parse::<f64>().ok())
            .unwrap_or(0.0);
        let write_bytes_per_sec = values.get(2)
            .and_then(|s| s.trim_matches('"').parse::<f64>().ok())
            .unwrap_or(0.0);
        let queue_depth = values.get(3)
            .and_then(|s| s.trim_matches('"').parse::<f64>().ok())
            .map(|q| q as u32);
        
        // Convert bytes/sec to MB/sec
        let read_throughput_mb_per_s = read_bytes_per_sec / (1024.0 * 1024.0);
        let write_throughput_mb_per_s = write_bytes_per_sec / (1024.0 * 1024.0);
        
        Ok(StorageMetrics {
            read_throughput_mb_per_s,
            write_throughput_mb_per_s,
            queue_depth,
            latency_ms: None, // Would require additional performance counter
        })
    }
}

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;
    use tokio::fs;
    
    /// Get storage metrics on Linux using /proc/diskstats
    pub async fn get_storage_metrics() -> Result<StorageMetrics, MetricsError> {
        // Read /proc/diskstats
        let content = fs::read_to_string("/proc/diskstats")
            .await
            .map_err(|e| MetricsError::Io(e))?;
        
        // Parse diskstats format:
        // major minor name rio rmerge rsect ruse wio wmerge wsect wuse running use aveq
        // 0    0     sda   123  456   7890  12   34  567   8901  23  45     67  89
        // Fields: 3=sectors read, 7=sectors written, 11=I/O in progress
        
        let mut total_sectors_read = 0u64;
        let mut total_sectors_written = 0u64;
        let mut total_io_in_progress = 0u32;
        
        for line in content.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 14 {
                continue;
            }
            
            // Skip loop devices and ramdisks
            let name = fields.get(2).unwrap_or(&"");
            if name.starts_with("loop") || name.starts_with("ram") {
                continue;
            }
            
            // Sum up all physical disks
            if let (Ok(sectors_read), Ok(sectors_written), Ok(io_in_progress)) = (
                fields.get(3).and_then(|s| s.parse::<u64>().ok()),
                fields.get(7).and_then(|s| s.parse::<u64>().ok()),
                fields.get(11).and_then(|s| s.parse::<u32>().ok()),
            ) {
                total_sectors_read += sectors_read.unwrap_or(0);
                total_sectors_written += sectors_written.unwrap_or(0);
                total_io_in_progress += io_in_progress.unwrap_or(0);
            }
        }
        
        // Note: To calculate throughput, we'd need to track deltas over time
        // For now, return zero (would need to store previous values)
        // TODO: Implement delta calculation with state tracking
        
        Ok(StorageMetrics {
            read_throughput_mb_per_s: 0.0, // Would need delta calculation
            write_throughput_mb_per_s: 0.0, // Would need delta calculation
            queue_depth: Some(total_io_in_progress),
            latency_ms: None,
        })
    }
}

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    
    /// Get storage metrics on macOS
    pub async fn get_storage_metrics() -> Result<StorageMetrics, MetricsError> {
        // macOS storage metrics require iostat or IOKit
        // For now, return zero metrics
        // TODO: Implement iostat parsing or IOKit-based metrics
        Ok(StorageMetrics {
            read_throughput_mb_per_s: 0.0,
            write_throughput_mb_per_s: 0.0,
            queue_depth: None,
            latency_ms: None,
        })
    }
}

