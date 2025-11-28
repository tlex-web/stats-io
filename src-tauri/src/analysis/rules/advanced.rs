//! Advanced bottleneck detection rules
//!
//! This module contains advanced bottleneck detection for Phase 4.2:
//! - PCIe saturation detection
//! - Memory bus bandwidth analysis
//! - Enhanced thermal analysis with predictive warnings
//! - Multi-GPU scenarios (SLI/CrossFire)

use crate::core::domain::{
    Bottleneck, BottleneckType, EvidenceItem, MetricSample, MetricType,
};

/// PCIe bandwidth thresholds (in MB/s)
/// These are theoretical maximums for common PCIe generations
pub const PCIE_3_0_X16_MAX: f64 = 15760.0; // ~15.76 GB/s
pub const PCIE_4_0_X16_MAX: f64 = 31520.0; // ~31.52 GB/s
pub const PCIE_5_0_X16_MAX: f64 = 63040.0; // ~63.04 GB/s

/// PCIe saturation threshold (percentage of theoretical max)
pub const PCIE_SATURATION_THRESHOLD: f64 = 85.0; // 85% of theoretical max

/// Memory bus bandwidth thresholds (in MB/s)
/// These vary by platform and memory type
pub const DDR4_3200_DUAL_CHANNEL_MAX: f64 = 51200.0; // ~51.2 GB/s
pub const DDR5_4800_DUAL_CHANNEL_MAX: f64 = 76800.0; // ~76.8 GB/s

/// Memory bus saturation threshold
pub const MEMORY_BUS_SATURATION_THRESHOLD: f64 = 80.0; // 80% of theoretical max

/// Thermal throttling prediction thresholds
pub const TEMP_WARNING_THRESHOLD: f64 = 75.0; // °C - warning level
pub const TEMP_CRITICAL_THRESHOLD: f64 = 85.0; // °C - critical/throttling level
pub const TEMP_PREDICTIVE_THRESHOLD: f64 = 70.0; // °C - predictive warning

/// Temperature rise rate threshold (degrees per minute)
pub const TEMP_RISE_RATE_THRESHOLD: f64 = 2.0; // °C/minute - rapid rise indicates potential throttling

/// Detect PCIe bandwidth saturation
///
/// Checks if PCIe bandwidth is approaching saturation, which can indicate
/// a bottleneck in data transfer between CPU and GPU or other PCIe devices.
pub fn detect_pcie_saturation(metrics: &[MetricSample]) -> Option<Bottleneck> {
    // Look for PCIe-related metrics
    // Note: Actual PCIe bandwidth metrics would need to be collected from platform-specific APIs
    // For now, we infer from GPU utilization and data transfer patterns
    
    let pcie_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| {
            matches!(
                m.metric_type,
                MetricType::GpuUtilization
                    | MetricType::GpuMemoryTransfer
                    | MetricType::StorageReadThroughput
                    | MetricType::StorageWriteThroughput
            )
        })
        .collect();
    
    if pcie_metrics.is_empty() {
        return None;
    }
    
    // Calculate average GPU utilization and storage throughput
    let _avg_gpu_util: f64 = pcie_metrics
        .iter()
        .filter(|m| matches!(m.metric_type, MetricType::GpuUtilization))
        .map(|m| m.value)
        .sum::<f64>()
        / pcie_metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::GpuUtilization))
            .count()
            .max(1) as f64;
    
    let avg_storage_read: f64 = pcie_metrics
        .iter()
        .filter(|m| matches!(m.metric_type, MetricType::StorageReadThroughput))
        .map(|m| m.value)
        .last()
        .unwrap_or(0.0);
    
    let avg_storage_write: f64 = pcie_metrics
        .iter()
        .filter(|m| matches!(m.metric_type, MetricType::StorageWriteThroughput))
        .map(|m| m.value)
        .last()
        .unwrap_or(0.0);
    
    // Estimate PCIe bandwidth usage
    // This is a simplified heuristic - real implementation would need actual PCIe counters
    let estimated_pcie_usage_mb_s = avg_storage_read + avg_storage_write;
    
    // Assume PCIe 3.0 x16 as baseline (can be enhanced with hardware detection)
    let pcie_max_bandwidth = PCIE_3_0_X16_MAX;
    let pcie_utilization_percent = (estimated_pcie_usage_mb_s / pcie_max_bandwidth) * 100.0;
    
    if pcie_utilization_percent >= PCIE_SATURATION_THRESHOLD {
        let severity = if pcie_utilization_percent >= 95.0 {
            90
        } else if pcie_utilization_percent >= 90.0 {
            75
        } else {
            60
        };
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::StorageReadThroughput,
            threshold: pcie_max_bandwidth * (PCIE_SATURATION_THRESHOLD / 100.0),
            actual_value: estimated_pcie_usage_mb_s,
            time_range_start: pcie_metrics.first().unwrap().timestamp,
            time_range_end: pcie_metrics.last().unwrap().timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Bandwidth,
            severity,
            evidence,
            summary: "PCIe bandwidth saturation detected".to_string(),
            details: format!(
                "Estimated PCIe bandwidth usage: {:.1}% ({:.1} MB/s of {:.1} MB/s max). \
                 This may limit data transfer between CPU and GPU or storage devices.",
                pcie_utilization_percent,
                estimated_pcie_usage_mb_s,
                pcie_max_bandwidth
            ),
        });
    }
    
    None
}

/// Detect memory bus bandwidth saturation
///
/// Checks if memory bus bandwidth is approaching saturation, which can indicate
/// a bottleneck in memory access patterns.
pub fn detect_memory_bus_saturation(metrics: &[MetricSample]) -> Option<Bottleneck> {
    // Look for memory-related metrics
    let memory_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| {
            matches!(
                m.metric_type,
                MetricType::MemoryUsage
                    | MetricType::MemoryReadThroughput
                    | MetricType::MemoryWriteThroughput
            )
        })
        .collect();
    
    if memory_metrics.is_empty() {
        return None;
    }
    
    // Calculate average memory throughput
    let avg_read: f64 = memory_metrics
        .iter()
        .filter(|m| matches!(m.metric_type, MetricType::MemoryReadThroughput))
        .map(|m| m.value)
        .sum::<f64>()
        / memory_metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::MemoryReadThroughput))
            .count()
            .max(1) as f64;
    
    let avg_write: f64 = memory_metrics
        .iter()
        .filter(|m| matches!(m.metric_type, MetricType::MemoryWriteThroughput))
        .map(|m| m.value)
        .sum::<f64>()
        / memory_metrics
            .iter()
            .filter(|m| matches!(m.metric_type, MetricType::MemoryWriteThroughput))
            .count()
            .max(1) as f64;
    
    let total_memory_bandwidth = avg_read + avg_write;
    
    // Assume DDR4 3200 dual channel as baseline (can be enhanced with hardware detection)
    let memory_max_bandwidth = DDR4_3200_DUAL_CHANNEL_MAX;
    let memory_utilization_percent = (total_memory_bandwidth / memory_max_bandwidth) * 100.0;
    
    if memory_utilization_percent >= MEMORY_BUS_SATURATION_THRESHOLD {
        let severity = if memory_utilization_percent >= 95.0 {
            85
        } else if memory_utilization_percent >= 90.0 {
            70
        } else {
            55
        };
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::MemoryReadThroughput,
            threshold: memory_max_bandwidth * (MEMORY_BUS_SATURATION_THRESHOLD / 100.0),
            actual_value: total_memory_bandwidth,
            time_range_start: memory_metrics.first().unwrap().timestamp,
            time_range_end: memory_metrics.last().unwrap().timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Bandwidth,
            severity,
            evidence,
            summary: "Memory bus bandwidth saturation detected".to_string(),
            details: format!(
                "Memory bus bandwidth usage: {:.1}% ({:.1} MB/s of {:.1} MB/s max). \
                 This may limit memory access performance.",
                memory_utilization_percent,
                total_memory_bandwidth,
                memory_max_bandwidth
            ),
        });
    }
    
    None
}

/// Enhanced thermal analysis with predictive warnings
///
/// Detects thermal throttling and predicts potential throttling based on
/// temperature trends and cooling efficiency.
pub fn detect_enhanced_thermal_bottleneck(metrics: &[MetricSample]) -> Option<Bottleneck> {
    let temp_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| matches!(m.metric_type, MetricType::Temperature))
        .collect();
    
    if temp_metrics.len() < 2 {
        return None; // Need at least 2 samples for trend analysis
    }
    
    // Sort by timestamp
    let mut sorted_temps: Vec<&MetricSample> = temp_metrics.iter().cloned().collect();
    sorted_temps.sort_by_key(|m| m.timestamp);
    
    // Get latest temperature
    let latest_temp = sorted_temps.last().unwrap().value;
    
    // Calculate temperature rise rate
    let first_temp = sorted_temps.first().unwrap();
    let last_temp = sorted_temps.last().unwrap();
    let time_diff_minutes = (last_temp.timestamp - first_temp.timestamp)
        .num_seconds() as f64
        / 60.0;
    
    let temp_rise_rate = if time_diff_minutes > 0.0 {
        (last_temp.value - first_temp.value) / time_diff_minutes
    } else {
        0.0
    };
    
    // Check for critical temperature
    if latest_temp >= TEMP_CRITICAL_THRESHOLD {
        let severity = if latest_temp >= 95.0 {
            95
        } else if latest_temp >= 90.0 {
            85
        } else {
            75
        };
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::Temperature,
            threshold: TEMP_CRITICAL_THRESHOLD,
            actual_value: latest_temp,
            time_range_start: first_temp.timestamp,
            time_range_end: last_temp.timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Thermal,
            severity,
            evidence,
            summary: "Critical thermal throttling detected".to_string(),
            details: format!(
                "Temperature: {:.1}°C (critical threshold: {:.1}°C). \
                 System is likely throttling performance to prevent damage.",
                latest_temp,
                TEMP_CRITICAL_THRESHOLD
            ),
        });
    }
    
    // Check for predictive warning (rapid temperature rise)
    if latest_temp >= TEMP_PREDICTIVE_THRESHOLD
        && temp_rise_rate >= TEMP_RISE_RATE_THRESHOLD
    {
        let predicted_time_to_throttle = if temp_rise_rate > 0.0 {
            (TEMP_CRITICAL_THRESHOLD - latest_temp) / temp_rise_rate
        } else {
            f64::INFINITY
        };
        
        let severity = if predicted_time_to_throttle < 5.0 {
            70 // Will throttle soon
        } else if predicted_time_to_throttle < 10.0 {
            55 // May throttle soon
        } else {
            40 // Potential issue
        };
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::Temperature,
            threshold: TEMP_PREDICTIVE_THRESHOLD,
            actual_value: latest_temp,
            time_range_start: first_temp.timestamp,
            time_range_end: last_temp.timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Thermal,
            severity,
            evidence,
            summary: "Predictive thermal warning".to_string(),
            details: format!(
                "Temperature: {:.1}°C, rising at {:.1}°C/min. \
                 Predicted time to throttling: {:.1} minutes. \
                 Consider improving cooling or reducing workload.",
                latest_temp,
                temp_rise_rate,
                predicted_time_to_throttle
            ),
        });
    }
    
    // Check for warning level
    if latest_temp >= TEMP_WARNING_THRESHOLD {
        let severity = 50;
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::Temperature,
            threshold: TEMP_WARNING_THRESHOLD,
            actual_value: latest_temp,
            time_range_start: first_temp.timestamp,
            time_range_end: last_temp.timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Thermal,
            severity,
            evidence,
            summary: "High temperature warning".to_string(),
            details: format!(
                "Temperature: {:.1}°C (warning threshold: {:.1}°C). \
                 Monitor temperature trends to prevent throttling.",
                latest_temp,
                TEMP_WARNING_THRESHOLD
            ),
        });
    }
    
    None
}

/// Detect multi-GPU scenarios and workload distribution
///
/// Analyzes GPU utilization across multiple GPUs to detect
/// SLI/CrossFire configurations and workload distribution issues.
pub fn detect_multi_gpu_bottleneck(metrics: &[MetricSample]) -> Option<Bottleneck> {
    // Look for per-GPU metrics
    let gpu_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| {
            matches!(
                m.metric_type,
                MetricType::GpuUtilization | MetricType::GpuVramUsage
            )
        })
        .collect();
    
    if gpu_metrics.is_empty() {
        return None;
    }
    
    // Group by GPU (if source_component contains GPU identifier)
    // For now, we'll check if there are multiple distinct GPU sources
    let gpu_sources: std::collections::HashSet<String> = gpu_metrics
        .iter()
        .filter(|m| m.source_component.contains("GPU"))
        .map(|m| m.source_component.clone())
        .collect();
    
    if gpu_sources.len() < 2 {
        return None; // Single GPU or no GPU metrics
    }
    
    // Calculate utilization per GPU
    let mut gpu_utilizations: Vec<(String, f64)> = Vec::new();
    for source in &gpu_sources {
        let utilizations: Vec<f64> = gpu_metrics
            .iter()
            .filter(|m| {
                m.source_component == *source
                    && matches!(m.metric_type, MetricType::GpuUtilization)
            })
            .map(|m| m.value)
            .collect();
        
        if !utilizations.is_empty() {
            let avg_util = utilizations.iter().sum::<f64>() / utilizations.len() as f64;
            gpu_utilizations.push((source.clone(), avg_util));
        }
    }
    
    if gpu_utilizations.len() < 2 {
        return None;
    }
    
    // Check for workload imbalance
    let max_util = gpu_utilizations
        .iter()
        .map(|(_, util)| *util)
        .fold(0.0, f64::max);
    
    let min_util = gpu_utilizations
        .iter()
        .map(|(_, util)| *util)
        .fold(100.0, f64::min);
    
    let utilization_spread = max_util - min_util;
    
    // If one GPU is heavily utilized and others are not, it's a workload distribution issue
    if max_util >= 80.0 && utilization_spread >= 30.0 {
        let severity = if utilization_spread >= 50.0 {
            75
        } else if utilization_spread >= 40.0 {
            60
        } else {
            45
        };
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::GpuUtilization,
            threshold: 80.0,
            actual_value: max_util,
            time_range_start: gpu_metrics.first().unwrap().timestamp,
            time_range_end: gpu_metrics.last().unwrap().timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Gpu,
            severity,
            evidence,
            summary: "Multi-GPU workload imbalance detected".to_string(),
            details: format!(
                "GPU utilization spread: {:.1}% (max: {:.1}%, min: {:.1}%). \
                 Workload is not evenly distributed across GPUs. \
                 This may indicate SLI/CrossFire configuration issues or application not utilizing multiple GPUs.",
                utilization_spread,
                max_util,
                min_util
            ),
        });
    }
    
    // Check if all GPUs are saturated (potential scaling issue)
    if gpu_utilizations.iter().all(|(_, util)| *util >= 90.0) {
        let avg_util = gpu_utilizations
            .iter()
            .map(|(_, util)| *util)
            .sum::<f64>()
            / gpu_utilizations.len() as f64;
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::GpuUtilization,
            threshold: 90.0,
            actual_value: avg_util,
            time_range_start: gpu_metrics.first().unwrap().timestamp,
            time_range_end: gpu_metrics.last().unwrap().timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Gpu,
            severity: 85,
            evidence,
            summary: "All GPUs saturated in multi-GPU setup".to_string(),
            details: format!(
                "All {} GPUs are at {:.1}% average utilization. \
                 System is GPU-bound. Consider reducing quality settings or upgrading GPUs.",
                gpu_utilizations.len(),
                avg_util
            ),
        });
    }
    
    None
}

