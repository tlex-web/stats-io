//! Metrics collection subsystem
//!
//! This module provides metrics collection functionality following AGENT.md Section 6.3.

pub mod providers;
pub mod collector;
pub mod models;
pub mod utils;

pub use collector::{MetricsCollector, MetricsCollectorConfig};
pub use utils::{aggregate_metrics, percentile, MetricAggregation};

use std::sync::Arc;
use std::sync::OnceLock;

/// Global metrics collector instance
static METRICS_COLLECTOR: OnceLock<Arc<MetricsCollector>> = OnceLock::new();

/// Initialize the metrics collector
pub fn init_metrics_collector(config: MetricsCollectorConfig) -> Arc<MetricsCollector> {
    METRICS_COLLECTOR.get_or_init(|| {
        Arc::new(MetricsCollector::new(config))
    }).clone()
}

/// Get the global metrics collector instance
pub fn get_metrics_collector() -> Option<Arc<MetricsCollector>> {
    METRICS_COLLECTOR.get().cloned()
}
