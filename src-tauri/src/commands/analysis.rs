//! Tauri commands for bottleneck analysis
//!
//! This module exposes bottleneck analysis functionality to the frontend.

use crate::analysis::{AnalysisEngine, insights};
use crate::core::domain::{BottleneckAnalysisResult, MetricSample, WorkloadProfile};

/// Analyze metrics and detect bottlenecks
#[tauri::command]
pub async fn analyze_bottlenecks(
    metrics: Vec<MetricSample>,
    time_window_seconds: Option<i64>,
    profile: Option<WorkloadProfile>,
) -> Result<BottleneckAnalysisResult, String> {
    let engine = AnalysisEngine::new();
    Ok(engine.analyze(
        &metrics,
        time_window_seconds,
        profile.as_ref(),
    ))
}

/// Generate user-facing insights from analysis results
#[tauri::command]
pub fn generate_insights(
    result: BottleneckAnalysisResult,
    profile: Option<WorkloadProfile>,
) -> Result<insights::UserFacingInsights, String> {
    Ok(insights::generate_insights(&result, profile.as_ref()))
}
