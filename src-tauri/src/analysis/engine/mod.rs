//! Bottleneck analysis engine
//!
//! This module implements the main bottleneck analysis engine following
//! AGENT.md Section 6.4.

use crate::core::domain::{BottleneckAnalysisResult, MetricSample, WorkloadProfile};
use crate::analysis::rules;

/// Bottleneck analysis engine
pub struct AnalysisEngine;

impl AnalysisEngine {
    /// Create a new analysis engine
    pub fn new() -> Self {
        Self
    }
    
    /// Analyze metrics and detect bottlenecks
    ///
    /// Uses a configurable time window (default: 30 seconds) to analyze
    /// recent metrics and identify performance bottlenecks.
    /// If a workload profile is provided, uses workload-specific heuristics.
    pub fn analyze(
        &self,
        metrics: &[MetricSample],
        time_window_seconds: Option<i64>,
        profile: Option<&WorkloadProfile>,
    ) -> BottleneckAnalysisResult {
        let window = time_window_seconds.unwrap_or(rules::SUSTAINED_WINDOW_SECONDS);
        rules::analyze_bottlenecks(metrics, window, profile)
    }
}

impl Default for AnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}
