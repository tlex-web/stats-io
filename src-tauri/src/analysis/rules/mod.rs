//! Bottleneck analysis rules
//!
//! This module contains rule-based heuristics for detecting bottlenecks
//! following AGENT.md Section 6.4.3.

pub mod advanced;

use crate::core::domain::{
    Bottleneck, BottleneckAnalysisResult, BottleneckType, EvidenceItem, MetricSample, MetricType,
    WorkloadProfile, WorkloadType,
};
use chrono::{Duration, Utc};
pub use advanced::{
    detect_enhanced_thermal_bottleneck, detect_memory_bus_saturation, detect_multi_gpu_bottleneck,
    detect_pcie_saturation,
};

/// Threshold constants for bottleneck detection
pub const CPU_HIGH_THRESHOLD: f64 = 85.0; // 85% utilization
pub const GPU_HIGH_THRESHOLD: f64 = 90.0; // 90% utilization
pub const RAM_HIGH_THRESHOLD: f64 = 90.0; // 90% usage
pub const VRAM_HIGH_THRESHOLD: f64 = 90.0; // 90% usage

/// Time window for sustained threshold violations (in seconds)
pub const SUSTAINED_WINDOW_SECONDS: i64 = 30;

/// Analyze metrics to detect bottlenecks
pub fn analyze_bottlenecks(
    metrics: &[MetricSample],
    time_window_seconds: i64,
    profile: Option<&WorkloadProfile>,
) -> BottleneckAnalysisResult {
    let now = Utc::now();
    let window_start = now - Duration::seconds(time_window_seconds);
    
    // Filter metrics to the time window
    let recent_metrics: Vec<MetricSample> = metrics
        .iter()
        .filter(|m| m.timestamp >= window_start && m.timestamp <= now)
        .cloned()
        .collect();
    
    let mut bottlenecks = Vec::new();
    
    // Check for enhanced thermal throttling (applies to all workloads)
    // Use enhanced thermal detection if available, fallback to basic
    if let Some(thermal_bottleneck) = detect_enhanced_thermal_bottleneck(&recent_metrics) {
        bottlenecks.push(thermal_bottleneck);
    } else if let Some(thermal_bottleneck) = detect_thermal_throttling(&recent_metrics) {
        bottlenecks.push(thermal_bottleneck);
    }
    
    // Check for bandwidth bottlenecks (PCIe and memory bus)
    if let Some(pcie_bottleneck) = detect_pcie_saturation(&recent_metrics) {
        bottlenecks.push(pcie_bottleneck);
    }
    
    if let Some(memory_bus_bottleneck) = detect_memory_bus_saturation(&recent_metrics) {
        bottlenecks.push(memory_bus_bottleneck);
    }
    
    // Check for multi-GPU bottlenecks
    if let Some(multi_gpu_bottleneck) = detect_multi_gpu_bottleneck(&recent_metrics) {
        bottlenecks.push(multi_gpu_bottleneck);
    }
    
    // Use workload-specific analysis if profile is provided
    if let Some(profile) = profile {
        match profile.workload_type {
            WorkloadType::Gaming => {
                if let Some(b) = detect_gaming_bottlenecks(&recent_metrics, profile) {
                    bottlenecks.extend(b);
                }
            }
            WorkloadType::Rendering => {
                if let Some(b) = detect_rendering_bottlenecks(&recent_metrics, profile) {
                    bottlenecks.extend(b);
                }
            }
            WorkloadType::AI => {
                if let Some(b) = detect_ai_ml_bottlenecks(&recent_metrics, profile) {
                    bottlenecks.extend(b);
                }
            }
            WorkloadType::Productivity | WorkloadType::General => {
                if let Some(b) = detect_productivity_bottlenecks(&recent_metrics, profile) {
                    bottlenecks.extend(b);
                }
            }
        }
    } else {
        // Fallback to generic analysis
        if let Some(cpu_bottleneck) = detect_cpu_bottleneck(&recent_metrics, None) {
            bottlenecks.push(cpu_bottleneck);
        }
        
        if let Some(gpu_bottleneck) = detect_gpu_bottleneck(&recent_metrics, None) {
            bottlenecks.push(gpu_bottleneck);
        }
        
        if let Some(ram_bottleneck) = detect_ram_bottleneck(&recent_metrics, None) {
            bottlenecks.push(ram_bottleneck);
        }
    }
    
    BottleneckAnalysisResult {
        bottlenecks,
        timestamp: now,
    }
}

/// Detect gaming-specific bottlenecks
fn detect_gaming_bottlenecks(
    metrics: &[MetricSample],
    profile: &WorkloadProfile,
) -> Option<Vec<Bottleneck>> {
    let mut bottlenecks = Vec::new();
    
    // Get thresholds from profile or use defaults
    let cpu_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.cpu_high)
        .unwrap_or(CPU_HIGH_THRESHOLD);
    let gpu_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.gpu_high)
        .unwrap_or(GPU_HIGH_THRESHOLD);
    let vram_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.vram_high)
        .unwrap_or(VRAM_HIGH_THRESHOLD);
    
    // Check for GPU-bound (most common in gaming)
    if let Some(gpu_bottleneck) = detect_gpu_bottleneck(metrics, Some(gpu_threshold)) {
        bottlenecks.push(gpu_bottleneck);
    }
    
    // Check for CPU-bound (less common but possible)
    if let Some(cpu_bottleneck) = detect_cpu_bottleneck(metrics, Some(cpu_threshold)) {
        bottlenecks.push(cpu_bottleneck);
    }
    
    // Check for VRAM-bound
    if let Some(vram_bottleneck) = detect_vram_bottleneck(metrics, Some(vram_threshold)) {
        bottlenecks.push(vram_bottleneck);
    }
    
    if bottlenecks.is_empty() {
        None
    } else {
        Some(bottlenecks)
    }
}

/// Detect rendering/content creation bottlenecks
fn detect_rendering_bottlenecks(
    metrics: &[MetricSample],
    profile: &WorkloadProfile,
) -> Option<Vec<Bottleneck>> {
    let mut bottlenecks = Vec::new();
    
    let cpu_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.cpu_high)
        .unwrap_or(95.0); // Higher threshold for rendering
    let gpu_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.gpu_high)
        .unwrap_or(95.0);
    let vram_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.vram_high)
        .unwrap_or(90.0);
    
    // CPU-bound render (CPU pegged, GPU idle)
    if let Some(cpu_bottleneck) = detect_cpu_bottleneck(metrics, Some(cpu_threshold)) {
        bottlenecks.push(cpu_bottleneck);
    }
    
    // GPU-bound render
    if let Some(gpu_bottleneck) = detect_gpu_bottleneck(metrics, Some(gpu_threshold)) {
        bottlenecks.push(gpu_bottleneck);
    }
    
    // VRAM-limited
    if let Some(vram_bottleneck) = detect_vram_bottleneck(metrics, Some(vram_threshold)) {
        bottlenecks.push(vram_bottleneck);
    }
    
    if bottlenecks.is_empty() {
        None
    } else {
        Some(bottlenecks)
    }
}

/// Detect AI/ML workload bottlenecks
fn detect_ai_ml_bottlenecks(
    metrics: &[MetricSample],
    profile: &WorkloadProfile,
) -> Option<Vec<Bottleneck>> {
    let mut bottlenecks = Vec::new();
    
    let _gpu_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.gpu_high)
        .unwrap_or(85.0);
    let vram_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.vram_high)
        .unwrap_or(95.0);
    
    // GPU-starved: GPU usage fluctuating low, CPU/disk pegged
    let gpu_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::GpuUtilization)
        .collect();
    
    if !gpu_metrics.is_empty() {
        let avg_gpu = gpu_metrics.iter().map(|m| m.value).sum::<f64>() / gpu_metrics.len() as f64;
        let min_gpu = gpu_metrics.iter().map(|m| m.value).fold(f64::INFINITY, f64::min);
        let max_gpu = gpu_metrics.iter().map(|m| m.value).fold(f64::NEG_INFINITY, f64::max);
        let variance = max_gpu - min_gpu;
        
        // GPU-starved: Low average utilization with high variance
        if avg_gpu < 50.0 && variance > 30.0 {
            let severity = ((50.0 - avg_gpu) / 50.0 * 100.0) as u8;
            bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::Gpu,
                severity,
                evidence: vec![EvidenceItem {
                    metric_type: MetricType::GpuUtilization,
                    threshold: 50.0,
                    actual_value: avg_gpu,
                    time_range_start: gpu_metrics.first().unwrap().timestamp,
                    time_range_end: gpu_metrics.last().unwrap().timestamp,
                }],
                summary: format!("GPU-starved: Average GPU utilization is {:.1}% with high variance ({:.1}%), indicating GPU is waiting for CPU/disk", avg_gpu, variance),
                details: format!(
                    "GPU utilization averaged {:.1}% with variance of {:.1}%, suggesting the GPU is frequently idle while waiting for data from CPU or disk. This is common in AI/ML workloads when data preprocessing or I/O is the bottleneck.",
                    avg_gpu, variance
                ),
            });
        }
    }
    
    // VRAM-limited
    if let Some(vram_bottleneck) = detect_vram_bottleneck(metrics, Some(vram_threshold)) {
        bottlenecks.push(vram_bottleneck);
    }
    
    if bottlenecks.is_empty() {
        None
    } else {
        Some(bottlenecks)
    }
}

/// Detect productivity/general workload bottlenecks
fn detect_productivity_bottlenecks(
    metrics: &[MetricSample],
    profile: &WorkloadProfile,
) -> Option<Vec<Bottleneck>> {
    let mut bottlenecks = Vec::new();
    
    let ram_threshold = profile
        .threshold_overrides
        .as_ref()
        .and_then(|t| t.ram_high)
        .unwrap_or(RAM_HIGH_THRESHOLD);
    
    // Memory-bound
    if let Some(ram_bottleneck) = detect_ram_bottleneck(metrics, Some(ram_threshold)) {
        bottlenecks.push(ram_bottleneck);
    }
    
    // Storage-bound
    if let Some(storage_bottleneck) = detect_storage_bottleneck(metrics) {
        bottlenecks.push(storage_bottleneck);
    }
    
    if bottlenecks.is_empty() {
        None
    } else {
        Some(bottlenecks)
    }
}

/// Detect CPU-bound bottleneck
fn detect_cpu_bottleneck(
    metrics: &[MetricSample],
    threshold_override: Option<f64>,
) -> Option<Bottleneck> {
    let cpu_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::CpuUtilization)
        .collect();
    
    if cpu_metrics.is_empty() {
        return None;
    }
    
    // Check if CPU utilization is consistently high
    let avg_cpu = cpu_metrics.iter().map(|m| m.value).sum::<f64>() / cpu_metrics.len() as f64;
    let max_cpu = cpu_metrics.iter().map(|m| m.value).fold(0.0, f64::max);
    
    // Check GPU utilization to confirm CPU-bound (GPU should be lower)
    let gpu_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::GpuUtilization)
        .collect();
    
    let avg_gpu = if !gpu_metrics.is_empty() {
        gpu_metrics.iter().map(|m| m.value).sum::<f64>() / gpu_metrics.len() as f64
    } else {
        0.0
    };
    
    let threshold = threshold_override.unwrap_or(CPU_HIGH_THRESHOLD);
    
    // CPU-bound: High CPU (above threshold), GPU not saturated (<70%)
    if avg_cpu > threshold && avg_gpu < 70.0 {
        let severity = calculate_severity(avg_cpu, CPU_HIGH_THRESHOLD);
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::CpuUtilization,
            threshold,
            actual_value: avg_cpu,
            time_range_start: cpu_metrics.first().unwrap().timestamp,
            time_range_end: cpu_metrics.last().unwrap().timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Cpu,
            severity,
            evidence,
            summary: format!("CPU-bound: Average CPU utilization is {:.1}% (threshold: {:.1}%)", avg_cpu, threshold),
            details: format!(
                "CPU utilization averaged {:.1}% over the analysis period, indicating CPU is the limiting factor. GPU utilization is {:.1}%, suggesting GPU has headroom.",
                avg_cpu, avg_gpu
            ),
        });
    }
    
    None
}

/// Detect GPU-bound bottleneck
fn detect_gpu_bottleneck(
    metrics: &[MetricSample],
    threshold_override: Option<f64>,
) -> Option<Bottleneck> {
    let gpu_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::GpuUtilization)
        .collect();
    
    if gpu_metrics.is_empty() {
        return None;
    }
    
    let avg_gpu = gpu_metrics.iter().map(|m| m.value).sum::<f64>() / gpu_metrics.len() as f64;
    let _max_gpu = gpu_metrics.iter().map(|m| m.value).fold(0.0, f64::max);
    
    // Check CPU utilization to confirm GPU-bound
    let cpu_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::CpuUtilization)
        .collect();
    
    let avg_cpu = if !cpu_metrics.is_empty() {
        cpu_metrics.iter().map(|m| m.value).sum::<f64>() / cpu_metrics.len() as f64
    } else {
        0.0
    };
    
    let threshold = threshold_override.unwrap_or(GPU_HIGH_THRESHOLD);
    
    // GPU-bound: High GPU (above threshold), CPU not saturated (<80%)
    if avg_gpu > threshold && avg_cpu < 80.0 {
        let severity = calculate_severity(avg_gpu, threshold);
        
        let evidence = vec![EvidenceItem {
            metric_type: MetricType::GpuUtilization,
            threshold,
            actual_value: avg_gpu,
            time_range_start: gpu_metrics.first().unwrap().timestamp,
            time_range_end: gpu_metrics.last().unwrap().timestamp,
        }];
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Gpu,
            severity,
            evidence,
            summary: format!("GPU-bound: Average GPU utilization is {:.1}% (threshold: {:.1}%)", avg_gpu, threshold),
            details: format!(
                "GPU utilization averaged {:.1}% over the analysis period, indicating GPU is the limiting factor. CPU utilization is {:.1}%, suggesting CPU has headroom.",
                avg_gpu, avg_cpu
            ),
        });
    }
    
    None
}

/// Detect VRAM-bound bottleneck
fn detect_vram_bottleneck(
    metrics: &[MetricSample],
    threshold_override: Option<f64>,
) -> Option<Bottleneck> {
    let vram_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::GpuVramUsage)
        .collect();
    
    if vram_metrics.is_empty() {
        return None;
    }
    
    // Get VRAM total from metrics (would need to be passed or stored)
    // For now, check if VRAM usage is consistently high
    let avg_vram = vram_metrics.iter().map(|m| m.value).sum::<f64>() / vram_metrics.len() as f64;
    let max_vram = vram_metrics.iter().map(|m| m.value).fold(0.0, f64::max);
    
    // Need VRAM total to calculate percentage - placeholder for now
    // TODO: Pass VRAM total from hardware config
    let threshold = threshold_override.unwrap_or(VRAM_HIGH_THRESHOLD);
    
    // For MVP, we'll use a simple heuristic: if VRAM usage is consistently high
    // This will be enhanced when we have VRAM total information
    if max_vram > 0.0 && avg_vram > 0.0 {
        // Placeholder: assume high if we're seeing consistent VRAM usage
        // Real implementation would compare against total VRAM
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Vram,
            severity: 70, // Placeholder
            evidence: vec![EvidenceItem {
                metric_type: MetricType::GpuVramUsage,
                threshold,
                actual_value: avg_vram,
                time_range_start: vram_metrics.first().unwrap().timestamp,
                time_range_end: vram_metrics.last().unwrap().timestamp,
            }],
            summary: format!("VRAM-bound: Average VRAM usage is {:.1} MB", avg_vram),
            details: format!(
                "VRAM usage averaged {:.1} MB over the analysis period. High VRAM usage can cause stuttering and performance degradation in games and rendering workloads.",
                avg_vram
            ),
        });
    }
    
    None
}

/// Detect storage-bound bottleneck
fn detect_storage_bottleneck(metrics: &[MetricSample]) -> Option<Bottleneck> {
    let _read_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::StorageReadThroughput)
        .collect();
    
    let _write_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::StorageWriteThroughput)
        .collect();
    
    let queue_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::StorageQueueDepth)
        .collect();
    
    // Check for high queue depth (indicates I/O saturation)
    if let Some(max_queue) = queue_metrics.iter().map(|m| m.value).max_by(|a, b| a.partial_cmp(b).unwrap()) {
        if max_queue > 10.0 {
            // High queue depth indicates storage bottleneck
            let avg_queue = queue_metrics.iter().map(|m| m.value).sum::<f64>() / queue_metrics.len() as f64;
            let severity = (avg_queue.min(100.0) as u8).max(50);
            
            return Some(Bottleneck {
                bottleneck_type: BottleneckType::Storage,
                severity,
                evidence: vec![EvidenceItem {
                    metric_type: MetricType::StorageQueueDepth,
                    threshold: 10.0,
                    actual_value: avg_queue,
                    time_range_start: queue_metrics.first().unwrap().timestamp,
                    time_range_end: queue_metrics.last().unwrap().timestamp,
                }],
                summary: format!("Storage-bound: Average I/O queue depth is {:.1} (threshold: 10.0)", avg_queue),
                details: format!(
                    "Storage I/O queue depth averaged {:.1} over the analysis period, indicating storage is saturated. This can cause application slowdowns and stuttering.",
                    avg_queue
                ),
            });
        }
    }
    
    None
}

/// Detect RAM-bound bottleneck
fn detect_ram_bottleneck(
    metrics: &[MetricSample],
    threshold_override: Option<f64>,
) -> Option<Bottleneck> {
    let memory_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::MemoryUsage)
        .collect();
    
    if memory_metrics.is_empty() {
        return None;
    }
    
    let avg_memory = memory_metrics.iter().map(|m| m.value).sum::<f64>() / memory_metrics.len() as f64;
    let _max_memory = memory_metrics.iter().map(|m| m.value).fold(0.0, f64::max);
    
    // Check for swap usage
    let swap_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::MemorySwapUsage)
        .collect();
    
    let has_swap_usage = !swap_metrics.is_empty() && swap_metrics.iter().any(|m| m.value > 0.0);
    
    let threshold = threshold_override.unwrap_or(RAM_HIGH_THRESHOLD);
    
    // RAM-bound: High RAM usage (above threshold) or significant swap usage
    if avg_memory > threshold || has_swap_usage {
        let severity = if has_swap_usage {
            // Swap usage indicates more severe memory pressure
            (avg_memory.min(100.0) as u8).max(80)
        } else {
            calculate_severity(avg_memory, threshold)
        };
        
        let mut evidence = vec![EvidenceItem {
            metric_type: MetricType::MemoryUsage,
            threshold,
            actual_value: avg_memory,
            time_range_start: memory_metrics.first().unwrap().timestamp,
            time_range_end: memory_metrics.last().unwrap().timestamp,
        }];
        
        if has_swap_usage {
            evidence.push(EvidenceItem {
                metric_type: MetricType::MemorySwapUsage,
                threshold: 0.0,
                actual_value: swap_metrics.iter().map(|m| m.value).sum::<f64>() / swap_metrics.len() as f64,
                time_range_start: swap_metrics.first().unwrap().timestamp,
                time_range_end: swap_metrics.last().unwrap().timestamp,
            });
        }
        
        let details = if has_swap_usage {
            format!(
                "Memory usage averaged {:.1}% with swap usage detected, indicating severe memory pressure. System is likely paging to disk, causing performance degradation.",
                avg_memory
            )
        } else {
            format!(
                "Memory usage averaged {:.1}% over the analysis period, indicating memory is approaching capacity.",
                avg_memory
            )
        };
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Ram,
            severity,
            evidence,
            summary: format!("RAM-bound: Average memory usage is {:.1}% (threshold: {:.1}%)", avg_memory, threshold),
            details,
        });
    }
    
    None
}

/// Detect thermal throttling
fn detect_thermal_throttling(metrics: &[MetricSample]) -> Option<Bottleneck> {
    // Check for high temperatures
    let temp_metrics: Vec<&MetricSample> = metrics
        .iter()
        .filter(|m| m.metric_type == MetricType::Temperature)
        .collect();
    
    if temp_metrics.is_empty() {
        return None;
    }
    
    let max_temp = temp_metrics.iter().map(|m| m.value).fold(f64::NEG_INFINITY, f64::max);
    let avg_temp = temp_metrics.iter().map(|m| m.value).sum::<f64>() / temp_metrics.len() as f64;
    
    // Thermal throttling thresholds (typical CPU/GPU limits)
    const CPU_THROTTLE_TEMP: f64 = 90.0; // Celsius
    const CRITICAL_TEMP: f64 = 95.0; // Celsius
    
    // Check if temperature is near or above throttling limits
    if max_temp >= CPU_THROTTLE_TEMP || avg_temp >= CPU_THROTTLE_TEMP {
        let severity = if max_temp >= CRITICAL_TEMP {
            100
        } else if max_temp >= CPU_THROTTLE_TEMP {
            ((max_temp - CPU_THROTTLE_TEMP) / (CRITICAL_TEMP - CPU_THROTTLE_TEMP) * 50.0 + 50.0) as u8
        } else {
            ((avg_temp - 80.0) / (CPU_THROTTLE_TEMP - 80.0) * 50.0) as u8
        };
        
        return Some(Bottleneck {
            bottleneck_type: BottleneckType::Thermal,
            severity: severity.min(100),
            evidence: vec![EvidenceItem {
                metric_type: MetricType::Temperature,
                threshold: CPU_THROTTLE_TEMP,
                actual_value: max_temp,
                time_range_start: temp_metrics.first().unwrap().timestamp,
                time_range_end: temp_metrics.last().unwrap().timestamp,
            }],
            summary: format!("Thermal throttling: Maximum temperature reached {:.1}°C (threshold: {:.1}°C)", max_temp, CPU_THROTTLE_TEMP),
            details: format!(
                "Temperature reached {:.1}°C (average: {:.1}°C), indicating thermal throttling. The CPU/GPU is reducing clock speeds to prevent overheating, causing performance degradation. Consider improving cooling.",
                max_temp, avg_temp
            ),
        });
    }
    
    None
}

/// Calculate severity score (0-100) based on how much the value exceeds the threshold
fn calculate_severity(actual_value: f64, threshold: f64) -> u8 {
    if actual_value <= threshold {
        return 0;
    }
    
    // Severity increases as value exceeds threshold
    // At threshold: 0, at 100%: 100, linear scaling
    let excess = actual_value - threshold;
    let max_excess = 100.0 - threshold;
    let severity_ratio = (excess / max_excess).min(1.0);
    
    (threshold + (severity_ratio * (100.0 - threshold))) as u8
}
