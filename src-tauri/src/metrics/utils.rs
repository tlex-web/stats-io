//! Metrics utility functions
//!
//! This module provides utility functions for metrics aggregation and analysis.

use crate::core::domain::MetricSample;
use std::collections::HashMap;

/// Calculate percentile from a sorted vector of values
pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    
    if p < 0.0 || p > 100.0 {
        return None;
    }
    
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let index = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    
    if lower == upper {
        Some(sorted[lower])
    } else {
        let weight = index - lower as f64;
        Some(sorted[lower] * (1.0 - weight) + sorted[upper] * weight)
    }
}

/// Aggregate metrics by type
pub fn aggregate_metrics(metrics: &[MetricSample]) -> HashMap<String, MetricAggregation> {
    let mut grouped: HashMap<String, Vec<f64>> = HashMap::new();
    
    for metric in metrics {
        let key = format!("{:?}", metric.metric_type);
        grouped.entry(key).or_insert_with(Vec::new).push(metric.value);
    }
    
    grouped
        .into_iter()
        .map(|(key, values)| {
            let aggregation = MetricAggregation {
                min: values.iter().cloned().fold(f64::INFINITY, f64::min),
                max: values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                avg: values.iter().sum::<f64>() / values.len() as f64,
                p50: percentile(&values, 50.0),
                p95: percentile(&values, 95.0),
                p99: percentile(&values, 99.0),
                count: values.len(),
            };
            (key, aggregation)
        })
        .collect()
}

/// Metric aggregation statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricAggregation {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
    pub count: usize,
}

