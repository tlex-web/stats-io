//! Comparison utilities
//!
//! This module provides functionality for comparing runs and sessions
//! following AGENT.md Section 6.5.1 and IMPLEMENTATION_PLAN.md Phase 3.1.

use crate::core::domain::{BottleneckAnalysisResult, MetricSample, Run};
use std::collections::HashMap;

/// Comparison result between two runs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComparisonResult {
    pub run1_id: String,
    pub run2_id: String,
    pub metric_deltas: HashMap<String, MetricDelta>,
    pub bottleneck_changes: Vec<BottleneckChange>,
    pub summary: String,
}

/// Delta for a specific metric
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricDelta {
    pub metric_type: String,
    pub run1_avg: f64,
    pub run2_avg: f64,
    pub delta: f64,
    pub delta_percent: f64,
    pub unit: String,
}

/// Change in bottleneck between runs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BottleneckChange {
    pub bottleneck_type: String,
    pub run1_severity: Option<u8>,
    pub run2_severity: Option<u8>,
    pub severity_delta: i16, // Can be negative
    pub status: BottleneckStatus,
}

/// Bottleneck status change
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BottleneckStatus {
    New,        // Appeared in run2
    Resolved,   // Was in run1, not in run2
    Improved,   // Severity decreased
    Worsened,   // Severity increased
    Unchanged,  // Same severity
}

/// Compare two runs
pub fn compare_runs(run1: &Run, run2: &Run) -> ComparisonResult {
    let mut metric_deltas = HashMap::new();
    
    // Compare metrics by type
    let run1_metrics = flatten_metrics(&run1.metrics_streams);
    let run2_metrics = flatten_metrics(&run2.metrics_streams);
    
    // Group metrics by type
    let run1_by_type = group_metrics_by_type(&run1_metrics);
    let run2_by_type = group_metrics_by_type(&run2_metrics);
    
    // Calculate deltas for common metric types
    for (metric_type, run1_values) in &run1_by_type {
        if let Some(run2_values) = run2_by_type.get(metric_type) {
            let run1_avg = run1_values.iter().sum::<f64>() / run1_values.len() as f64;
            let run2_avg = run2_values.iter().sum::<f64>() / run2_values.len() as f64;
            let delta = run2_avg - run1_avg;
            let delta_percent = if run1_avg != 0.0 {
                (delta / run1_avg) * 100.0
            } else {
                0.0
            };
            
            // Get unit from first metric sample
            let unit = run1_metrics
                .iter()
                .find(|m| format!("{:?}", m.metric_type) == *metric_type)
                .map(|m| m.unit.clone())
                .unwrap_or_else(|| "".to_string());
            
            metric_deltas.insert(
                metric_type.clone(),
                MetricDelta {
                    metric_type: metric_type.clone(),
                    run1_avg,
                    run2_avg,
                    delta,
                    delta_percent,
                    unit,
                },
            );
        }
    }
    
    // Compare bottlenecks
    let bottleneck_changes = compare_bottlenecks(
        run1.analysis_result.as_ref(),
        run2.analysis_result.as_ref(),
    );
    
    // Generate summary
    let summary = generate_comparison_summary(&metric_deltas, &bottleneck_changes);
    
    ComparisonResult {
        run1_id: run1.id.to_string(),
        run2_id: run2.id.to_string(),
        metric_deltas,
        bottleneck_changes,
        summary,
    }
}

/// Flatten metrics streams into a single vector
fn flatten_metrics(streams: &HashMap<String, Vec<MetricSample>>) -> Vec<MetricSample> {
    streams.values().flatten().cloned().collect()
}

/// Group metrics by type and extract values
fn group_metrics_by_type(metrics: &[MetricSample]) -> HashMap<String, Vec<f64>> {
    let mut grouped: HashMap<String, Vec<f64>> = HashMap::new();
    
    for metric in metrics {
        let key = format!("{:?}", metric.metric_type);
        grouped.entry(key).or_insert_with(Vec::new).push(metric.value);
    }
    
    grouped
}

/// Compare bottlenecks between two analysis results
fn compare_bottlenecks(
    result1: Option<&BottleneckAnalysisResult>,
    result2: Option<&BottleneckAnalysisResult>,
) -> Vec<BottleneckChange> {
    let mut changes = Vec::new();
    
    let bottlenecks1: HashMap<String, u8> = result1
        .map(|r| {
            r.bottlenecks
                .iter()
                .map(|b| (format!("{:?}", b.bottleneck_type), b.severity))
                .collect()
        })
        .unwrap_or_default();
    
    let bottlenecks2: HashMap<String, u8> = result2
        .map(|r| {
            r.bottlenecks
                .iter()
                .map(|b| (format!("{:?}", b.bottleneck_type), b.severity))
                .collect()
        })
        .unwrap_or_default();
    
    // Check all bottleneck types
    let all_types: std::collections::HashSet<String> = bottlenecks1
        .keys()
        .chain(bottlenecks2.keys())
        .cloned()
        .collect();
    
    for bottleneck_type in all_types {
        let severity1 = bottlenecks1.get(&bottleneck_type).copied();
        let severity2 = bottlenecks2.get(&bottleneck_type).copied();
        
        let status = match (severity1, severity2) {
            (None, Some(_)) => BottleneckStatus::New,
            (Some(_), None) => BottleneckStatus::Resolved,
            (Some(s1), Some(s2)) => {
                if s2 < s1 {
                    BottleneckStatus::Improved
                } else if s2 > s1 {
                    BottleneckStatus::Worsened
                } else {
                    BottleneckStatus::Unchanged
                }
            }
            (None, None) => continue,
        };
        
        let severity_delta = match (severity1, severity2) {
            (Some(s1), Some(s2)) => s2 as i16 - s1 as i16,
            (None, Some(s2)) => s2 as i16,
            (Some(s1), None) => -(s1 as i16),
            (None, None) => 0,
        };
        
        changes.push(BottleneckChange {
            bottleneck_type: bottleneck_type.clone(),
            run1_severity: severity1,
            run2_severity: severity2,
            severity_delta,
            status,
        });
    }
    
    changes
}

/// Generate a human-readable comparison summary
fn generate_comparison_summary(
    metric_deltas: &HashMap<String, MetricDelta>,
    bottleneck_changes: &[BottleneckChange],
) -> String {
    let mut parts = Vec::new();
    
    // Summarize metric changes
    let significant_deltas: Vec<&MetricDelta> = metric_deltas
        .values()
        .filter(|d| d.delta_percent.abs() > 5.0) // >5% change
        .collect();
    
    if !significant_deltas.is_empty() {
        parts.push(format!("{} metric(s) changed significantly", significant_deltas.len()));
    }
    
    // Summarize bottleneck changes
    let new_bottlenecks = bottleneck_changes
        .iter()
        .filter(|c| matches!(c.status, BottleneckStatus::New))
        .count();
    let resolved_bottlenecks = bottleneck_changes
        .iter()
        .filter(|c| matches!(c.status, BottleneckStatus::Resolved))
        .count();
    let improved_bottlenecks = bottleneck_changes
        .iter()
        .filter(|c| matches!(c.status, BottleneckStatus::Improved))
        .count();
    
    if new_bottlenecks > 0 {
        parts.push(format!("{} new bottleneck(s) detected", new_bottlenecks));
    }
    if resolved_bottlenecks > 0 {
        parts.push(format!("{} bottleneck(s) resolved", resolved_bottlenecks));
    }
    if improved_bottlenecks > 0 {
        parts.push(format!("{} bottleneck(s) improved", improved_bottlenecks));
    }
    
    if parts.is_empty() {
        "No significant changes detected between runs.".to_string()
    } else {
        parts.join(". ")
    }
}

