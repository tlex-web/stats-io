//! Workload profile definitions and presets
//!
//! This module provides preset workload profiles and profile management
//! following AGENT.md Section 3.5 and IMPLEMENTATION_PLAN.md Phase 2.1.

use crate::core::domain::{WorkloadProfile, WorkloadType, ThresholdOverrides};
use std::collections::HashMap;

/// Preset workload profiles
pub struct WorkloadProfiles;

impl WorkloadProfiles {
    /// Get all preset profiles
    pub fn get_presets() -> Vec<WorkloadProfile> {
        vec![
            Self::gaming_1080p_60fps(),
            Self::gaming_1440p_60fps(),
            Self::gaming_4k_60fps(),
            Self::video_editing_4k(),
            Self::rendering_3d(),
            Self::ai_ml_small(),
            Self::ai_ml_large(),
            Self::productivity_general(),
        ]
    }
    
    /// 1080p 60 FPS Gaming profile
    pub fn gaming_1080p_60fps() -> WorkloadProfile {
        WorkloadProfile {
            id: "gaming_1080p_60fps".to_string(),
            name: "1080p 60 FPS Gaming".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: {
                let mut params = HashMap::new();
                params.insert("resolution".to_string(), serde_json::Value::String("1920x1080".to_string()));
                params.insert("target_fps".to_string(), serde_json::Value::Number(60.into()));
                params
            },
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(85.0),
                gpu_high: Some(90.0),
                ram_high: Some(80.0),
                vram_high: Some(85.0),
            }),
        }
    }
    
    /// 1440p 60 FPS Gaming profile
    pub fn gaming_1440p_60fps() -> WorkloadProfile {
        WorkloadProfile {
            id: "gaming_1440p_60fps".to_string(),
            name: "1440p 60 FPS Gaming".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: {
                let mut params = HashMap::new();
                params.insert("resolution".to_string(), serde_json::Value::String("2560x1440".to_string()));
                params.insert("target_fps".to_string(), serde_json::Value::Number(60.into()));
                params
            },
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(80.0),
                gpu_high: Some(95.0),
                ram_high: Some(80.0),
                vram_high: Some(90.0),
            }),
        }
    }
    
    /// 4K 60 FPS Gaming profile
    pub fn gaming_4k_60fps() -> WorkloadProfile {
        WorkloadProfile {
            id: "gaming_4k_60fps".to_string(),
            name: "4K 60 FPS Gaming".to_string(),
            workload_type: WorkloadType::Gaming,
            parameters: {
                let mut params = HashMap::new();
                params.insert("resolution".to_string(), serde_json::Value::String("3840x2160".to_string()));
                params.insert("target_fps".to_string(), serde_json::Value::Number(60.into()));
                params
            },
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(75.0),
                gpu_high: Some(98.0),
                ram_high: Some(75.0),
                vram_high: Some(95.0),
            }),
        }
    }
    
    /// 4K Video Editing profile
    pub fn video_editing_4k() -> WorkloadProfile {
        WorkloadProfile {
            id: "video_editing_4k".to_string(),
            name: "4K Video Editing".to_string(),
            workload_type: WorkloadType::Rendering,
            parameters: {
                let mut params = HashMap::new();
                params.insert("resolution".to_string(), serde_json::Value::String("3840x2160".to_string()));
                params.insert("codec".to_string(), serde_json::Value::String("H.264/H.265".to_string()));
                params
            },
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(90.0),
                gpu_high: Some(85.0),
                ram_high: Some(85.0),
                vram_high: Some(80.0),
            }),
        }
    }
    
    /// 3D Rendering profile
    pub fn rendering_3d() -> WorkloadProfile {
        WorkloadProfile {
            id: "rendering_3d".to_string(),
            name: "3D Rendering".to_string(),
            workload_type: WorkloadType::Rendering,
            parameters: {
                let mut params = HashMap::new();
                params.insert("render_type".to_string(), serde_json::Value::String("CPU/GPU".to_string()));
                params
            },
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(95.0),
                gpu_high: Some(95.0),
                ram_high: Some(90.0),
                vram_high: Some(90.0),
            }),
        }
    }
    
    /// AI/ML Small Model profile
    pub fn ai_ml_small() -> WorkloadProfile {
        WorkloadProfile {
            id: "ai_ml_small".to_string(),
            name: "AI/ML Inference (Small Model)".to_string(),
            workload_type: WorkloadType::AI,
            parameters: {
                let mut params = HashMap::new();
                params.insert("model_size".to_string(), serde_json::Value::String("Small (<4GB)".to_string()));
                params.insert("batch_size".to_string(), serde_json::Value::String("Medium".to_string()));
                params
            },
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(70.0),
                gpu_high: Some(85.0),
                ram_high: Some(70.0),
                vram_high: Some(85.0),
            }),
        }
    }
    
    /// AI/ML Large Model profile
    pub fn ai_ml_large() -> WorkloadProfile {
        WorkloadProfile {
            id: "ai_ml_large".to_string(),
            name: "AI/ML Inference (Large Model)".to_string(),
            workload_type: WorkloadType::AI,
            parameters: {
                let mut params = HashMap::new();
                params.insert("model_size".to_string(), serde_json::Value::String("Large (>8GB)".to_string()));
                params.insert("batch_size".to_string(), serde_json::Value::String("Small".to_string()));
                params
            },
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(60.0),
                gpu_high: Some(90.0),
                ram_high: Some(80.0),
                vram_high: Some(95.0),
            }),
        }
    }
    
    /// Productivity/General profile
    pub fn productivity_general() -> WorkloadProfile {
        WorkloadProfile {
            id: "productivity_general".to_string(),
            name: "Productivity/General".to_string(),
            workload_type: WorkloadType::Productivity,
            parameters: HashMap::new(),
            threshold_overrides: Some(ThresholdOverrides {
                cpu_high: Some(70.0),
                gpu_high: Some(50.0),
                ram_high: Some(85.0),
                vram_high: None,
            }),
        }
    }
    
    /// Get a profile by ID
    pub fn get_by_id(id: &str) -> Option<WorkloadProfile> {
        Self::get_presets().into_iter().find(|p| p.id == id)
    }
}

