//! CPU metrics provider using sysinfo

use crate::core::error::MetricsError;
use crate::core::interfaces::CpuMetricsProvider;
use crate::metrics::models::CpuMetrics;
use async_trait::async_trait;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;

/// CPU metrics provider using sysinfo
pub struct SysInfoCpuMetricsProvider {
    system: Arc<Mutex<System>>,
}

impl SysInfoCpuMetricsProvider {
    pub fn new(system: Arc<Mutex<System>>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl CpuMetricsProvider for SysInfoCpuMetricsProvider {
    async fn get_cpu_metrics(&self) -> Result<CpuMetrics, MetricsError> {
        let mut system = self.system.lock().await;
        system.refresh_cpu();
        
        let cpus = system.cpus();
        let per_core_utilization: Vec<f64> = cpus
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64 / 100.0)
            .collect();
        
        // Overall utilization is average of all cores
        let overall_utilization = if !per_core_utilization.is_empty() {
            per_core_utilization.iter().sum::<f64>() / per_core_utilization.len() as f64
        } else {
            0.0
        };
        
        // Get CPU temperature using platform-specific methods
        let temperature = get_cpu_temperature().await;
        
        Ok(CpuMetrics {
            overall_utilization,
            per_core_utilization,
            temperature,
        })
    }
    
}

/// Get CPU temperature using platform-specific APIs
#[cfg(target_os = "windows")]
async fn get_cpu_temperature() -> Option<f64> {
        // Use WMI COM interface directly - no process spawning, no windows
        use wmi::WMIConnection;
        
        let wmi_con = WMIConnection::new().ok()?;
        
        // Try MSAcpi_ThermalZoneTemperature first (more accurate)
        let query = "SELECT CurrentTemperature FROM MSAcpi_ThermalZoneTemperature";
        if let Ok(results) = wmi_con.raw_query::<serde_json::Value>(query) {
            for temp_obj in results {
                if let Some(temp_value) = temp_obj.get("CurrentTemperature") {
                    if let Some(temp_kelvin_10th) = temp_value.as_u64() {
                        // Convert from tenths of Kelvin to Celsius
                        let temp_kelvin = temp_kelvin_10th as f64 / 10.0;
                        let temp_celsius = temp_kelvin - 273.15;
                        
                        // Sanity check: CPU temps should be between 0-150°C
                        if temp_celsius >= 0.0 && temp_celsius <= 150.0 {
                            return Some(temp_celsius);
                        }
                    }
                }
            }
        }
        
        // Fallback to Win32_TemperatureProbe
        let query = "SELECT CurrentReading FROM Win32_TemperatureProbe WHERE Description LIKE '%CPU%'";
        if let Ok(results) = wmi_con.raw_query::<serde_json::Value>(query) {
            for temp_obj in results {
                if let Some(temp_value) = temp_obj.get("CurrentReading") {
                    if let Some(temp_tenths_celsius) = temp_value.as_u64() {
                        let temp_celsius = temp_tenths_celsius as f64 / 10.0;
                        
                        // Sanity check: CPU temps should be between 0-150°C
                        if temp_celsius >= 0.0 && temp_celsius <= 150.0 {
                            return Some(temp_celsius);
                        }
                    }
                }
            }
        }
        
        None
    }
    
#[cfg(target_os = "linux")]
async fn get_cpu_temperature() -> Option<f64> {
        // Try to read from /sys/class/thermal/thermal_zone*/temp
        use tokio::fs;
        
        // Look for CPU thermal zone (usually thermal_zone0 or thermal_zone1)
        for zone_id in 0..10 {
            let path = format!("/sys/class/thermal/thermal_zone{}/temp", zone_id);
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Ok(temp_millidegrees) = content.trim().parse::<i32>() {
                    let temp_celsius = temp_millidegrees as f64 / 1000.0;
                    // Sanity check
                    if temp_celsius >= 0.0 && temp_celsius <= 150.0 {
                        return Some(temp_celsius);
                    }
                }
            }
        }
        
        None
    }
    
#[cfg(target_os = "macos")]
async fn get_cpu_temperature() -> Option<f64> {
        // Try to use smcutil or system_profiler
        // For now, return None - requires SMC library
        None
    }
    
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
async fn get_cpu_temperature() -> Option<f64> {
    None
}

