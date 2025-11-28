//! Insights and recommendations engine
//!
//! This module converts bottleneck analysis results into human-readable
//! insights and actionable recommendations following AGENT.md Section 6.4.4.

use crate::core::domain::{Bottleneck, BottleneckAnalysisResult, WorkloadProfile, WorkloadType};
use serde::{Deserialize, Serialize};

/// User-facing insights generated from analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFacingInsights {
    pub summary: String,
    pub recommendations: Vec<String>,
    pub severity: u8, // 0-100, highest severity from bottlenecks
}

/// Generate user-facing insights from analysis results
pub fn generate_insights(
    result: &BottleneckAnalysisResult,
    profile: Option<&WorkloadProfile>,
) -> UserFacingInsights {
    if result.bottlenecks.is_empty() {
        return UserFacingInsights {
            summary: "No significant bottlenecks detected. System appears to be performing well.".to_string(),
            recommendations: vec!["Continue monitoring to identify any performance issues.".to_string()],
            severity: 0,
        };
    }
    
    let highest_severity = result.bottlenecks.iter()
        .map(|b| b.severity)
        .max()
        .unwrap_or(0);
    
    let mut summary_parts = Vec::new();
    let mut recommendations = Vec::new();
    
    for bottleneck in &result.bottlenecks {
        summary_parts.push(bottleneck.summary.clone());
        recommendations.extend(generate_recommendations(bottleneck, profile));
    }
    
    let summary = if summary_parts.len() == 1 {
        summary_parts[0].clone()
    } else {
        format!("Multiple bottlenecks detected: {}", summary_parts.join("; "))
    };
    
    UserFacingInsights {
        summary,
        recommendations,
        severity: highest_severity,
    }
}

/// Generate recommendations for a specific bottleneck
fn generate_recommendations(
    bottleneck: &Bottleneck,
    profile: Option<&WorkloadProfile>,
) -> Vec<String> {
    let workload_type = profile.map(|p| &p.workload_type);
    
    match bottleneck.bottleneck_type {
        crate::core::domain::BottleneckType::Cpu => {
            match workload_type {
                Some(WorkloadType::Gaming) => vec![
                    "For gaming: Consider upgrading to a CPU with higher single-core performance.".to_string(),
                    "Close background applications and browser tabs while gaming.".to_string(),
                    "Check if your game is CPU-limited by monitoring per-core utilization.".to_string(),
                    "Consider overclocking if your CPU and cooling allow (advanced users only).".to_string(),
                ],
                Some(WorkloadType::Rendering) => vec![
                    "For rendering: Consider upgrading to a CPU with more cores (e.g., Ryzen 9, Threadripper, or Intel Xeon).".to_string(),
                    "Ensure your rendering software is using all available CPU cores.".to_string(),
                    "Consider using GPU-accelerated rendering if available (e.g., CUDA, OpenCL).".to_string(),
                ],
                Some(WorkloadType::AI) => vec![
                    "For AI/ML: Consider upgrading to a CPU with more cores for data preprocessing.".to_string(),
                    "Optimize data loading pipeline to reduce CPU bottleneck.".to_string(),
                    "Consider using a faster storage solution (NVMe SSD) for dataset access.".to_string(),
                ],
                _ => vec![
                    "Consider upgrading to a faster CPU with more cores.".to_string(),
                    "Close background applications to free CPU resources.".to_string(),
                    "Check for CPU-intensive processes and optimize them.".to_string(),
                ],
            }
        }
        crate::core::domain::BottleneckType::Gpu => {
            match workload_type {
                Some(WorkloadType::Gaming) => {
                    let mut recs = vec![
                        "For gaming: Consider upgrading to a more powerful GPU.".to_string(),
                        "Lower graphics settings: Reduce texture quality, shadows, and anti-aliasing.".to_string(),
                        "Reduce resolution or use upscaling (DLSS/FSR) if available.".to_string(),
                    ];
                    
                    // Check profile parameters for resolution-specific advice
                    if let Some(profile) = profile {
                        if let Some(resolution) = profile.parameters.get("resolution") {
                            if let Some(res_str) = resolution.as_str() {
                                if res_str.contains("3840x2160") || res_str.contains("4K") {
                                    recs.push("For 4K gaming, a high-end GPU (RTX 3080/4080 or RX 6800 XT/7800 XT) is recommended.".to_string());
                                } else if res_str.contains("2560x1440") || res_str.contains("1440p") {
                                    recs.push("For 1440p gaming, a mid-to-high-end GPU (RTX 3070/4070 or RX 6700 XT/7700 XT) is recommended.".to_string());
                                }
                            }
                        }
                    }
                    
                    recs
                }
                Some(WorkloadType::Rendering) => vec![
                    "For rendering: Consider upgrading to a professional GPU (Quadro, Radeon Pro) or high-end consumer GPU.".to_string(),
                    "Use GPU-accelerated rendering engines (e.g., Cycles GPU, Octane, Redshift).".to_string(),
                    "Reduce scene complexity or use proxy objects for complex geometry.".to_string(),
                    "Optimize texture sizes and use compression where appropriate.".to_string(),
                ],
                Some(WorkloadType::AI) => vec![
                    "For AI/ML: Consider upgrading to a GPU with more CUDA cores and VRAM (e.g., RTX 3090/4090, A100).".to_string(),
                    "Reduce batch size to fit within available VRAM.".to_string(),
                    "Use mixed precision training (FP16) to reduce VRAM usage.".to_string(),
                    "Consider using model quantization or pruning to reduce model size.".to_string(),
                ],
                _ => vec![
                    "Consider upgrading to a more powerful GPU.".to_string(),
                    "Lower graphics settings in games or rendering applications.".to_string(),
                    "Reduce resolution or disable resource-intensive visual effects.".to_string(),
                ],
            }
        }
        crate::core::domain::BottleneckType::Ram => {
            match workload_type {
                Some(WorkloadType::Gaming) => vec![
                    "For gaming: Consider adding more RAM (16GB+ recommended for modern games).".to_string(),
                    "Close unnecessary applications and browser tabs while gaming.".to_string(),
                    "Check if your game has memory leaks or high memory requirements.".to_string(),
                ],
                Some(WorkloadType::Rendering) => vec![
                    "For rendering: Consider adding more RAM (32GB+ recommended for 4K/8K projects).".to_string(),
                    "Use proxy files or lower resolution previews during editing.".to_string(),
                    "Close other applications to free up RAM for rendering.".to_string(),
                ],
                Some(WorkloadType::AI) => vec![
                    "For AI/ML: Consider adding more RAM (32GB+ recommended for large datasets).".to_string(),
                    "Use data streaming or batch loading instead of loading entire datasets into memory.".to_string(),
                    "Optimize data preprocessing to reduce memory footprint.".to_string(),
                ],
                Some(WorkloadType::Productivity) => vec![
                    "For productivity: Consider adding more RAM (16GB+ recommended for multitasking).".to_string(),
                    "Close unused browser tabs and applications.".to_string(),
                    "Check for memory leaks in frequently used applications.".to_string(),
                ],
                _ => vec![
                    "Consider adding more RAM to your system.".to_string(),
                    "Close unnecessary applications to free memory.".to_string(),
                    "Check for memory leaks in running applications.".to_string(),
                ],
            }
        }
        crate::core::domain::BottleneckType::Vram => {
            match workload_type {
                Some(WorkloadType::Gaming) => vec![
                    "For gaming: Consider upgrading to a GPU with more VRAM (8GB+ recommended for modern games).".to_string(),
                    "Lower texture quality settings in games (e.g., High → Medium).".to_string(),
                    "Reduce resolution or disable high-resolution texture packs.".to_string(),
                    "Close other GPU-intensive applications.".to_string(),
                ],
                Some(WorkloadType::Rendering) => vec![
                    "For rendering: Consider upgrading to a GPU with more VRAM (12GB+ recommended).".to_string(),
                    "Reduce texture resolution and use compression.".to_string(),
                    "Use out-of-core rendering or render in passes if available.".to_string(),
                    "Optimize scene geometry and reduce polygon count.".to_string(),
                ],
                Some(WorkloadType::AI) => vec![
                    "For AI/ML: Consider upgrading to a GPU with more VRAM (24GB+ recommended for large models).".to_string(),
                    "Reduce batch size to fit within available VRAM.".to_string(),
                    "Use gradient checkpointing to reduce memory usage.".to_string(),
                    "Consider using model sharding or distributed training.".to_string(),
                ],
                _ => vec![
                    "Consider upgrading to a GPU with more VRAM.".to_string(),
                    "Lower texture quality and resolution in games.".to_string(),
                    "Reduce model complexity in rendering/AI workloads.".to_string(),
                ],
            }
        }
        crate::core::domain::BottleneckType::Storage => {
            match workload_type {
                Some(WorkloadType::Rendering) => vec![
                    "For rendering: Consider upgrading to a faster NVMe SSD for project files and cache.".to_string(),
                    "Use separate drives for OS, projects, and cache to improve I/O performance.".to_string(),
                    "Free up disk space on your project drive (keep 20%+ free).".to_string(),
                ],
                Some(WorkloadType::AI) => vec![
                    "For AI/ML: Consider using a fast NVMe SSD for dataset storage.".to_string(),
                    "Use data prefetching and caching to reduce I/O wait times.".to_string(),
                    "Consider using RAM disk for frequently accessed small datasets.".to_string(),
                ],
                Some(WorkloadType::Productivity) => vec![
                    "For productivity: Consider upgrading to an SSD if using an HDD.".to_string(),
                    "Free up disk space (keep 15%+ free for optimal performance).".to_string(),
                    "Defragment HDD if applicable (not needed for SSDs).".to_string(),
                ],
                _ => vec![
                    "Consider upgrading to a faster SSD or NVMe drive.".to_string(),
                    "Free up disk space to improve performance.".to_string(),
                    "Check for disk fragmentation and defragment if needed.".to_string(),
                ],
            }
        }
        crate::core::domain::BottleneckType::Thermal => vec![
            "Improve system cooling: Add case fans, upgrade CPU cooler, or improve case airflow.".to_string(),
            "Clean dust from system components (CPU heatsink, GPU fans, case filters).".to_string(),
            "Check thermal paste on CPU/GPU - consider reapplying if temperatures are very high.".to_string(),
            "Ensure proper case ventilation and cable management for better airflow.".to_string(),
            "Consider undervolting CPU/GPU (advanced users only) to reduce heat generation.".to_string(),
        ],
        crate::core::domain::BottleneckType::Bandwidth => vec![
            "Check PCIe slot configuration - ensure GPU is in the fastest available slot (usually x16).".to_string(),
            "Verify PCIe generation (PCIe 4.0/5.0) and ensure components support it.".to_string(),
            "Check for loose connections or damaged PCIe slots.".to_string(),
            "Consider upgrading motherboard if PCIe bandwidth is limiting performance.".to_string(),
        ],
    }
}
