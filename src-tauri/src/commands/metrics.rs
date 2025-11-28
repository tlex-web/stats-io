//! Tauri commands for metrics collection
//!
//! This module exposes metrics collection functionality to the frontend.

use crate::core::domain::MetricSample;
use crate::metrics::{get_metrics_collector, init_metrics_collector, MetricsCollectorConfig, aggregate_metrics};
use chrono::Utc;
use std::collections::HashMap;

/// Start metrics collection
#[tauri::command]
pub async fn start_metrics_collection(
    sampling_interval_ms: Option<u64>,
) -> Result<(), String> {
    let config = MetricsCollectorConfig {
        sampling_interval_ms: sampling_interval_ms.unwrap_or(1000),
        buffer_size: 600, // 10 minutes at 1s intervals
    };
    
    let collector = init_metrics_collector(config);
    collector.start().await
        .map_err(|e| e.to_string())
}

/// Stop metrics collection
#[tauri::command]
pub async fn stop_metrics_collection() -> Result<(), String> {
    if let Some(collector) = get_metrics_collector() {
        collector.stop().await;
        Ok(())
    } else {
        Err("Metrics collector not initialized".to_string())
    }
}

/// Get current metrics buffer
#[tauri::command]
pub async fn get_metrics_buffer() -> Result<Vec<MetricSample>, String> {
    if let Some(collector) = get_metrics_collector() {
        Ok(collector.get_buffer().await)
    } else {
        Err("Metrics collector not initialized".to_string())
    }
}

/// Get metrics in a time range
#[tauri::command]
pub async fn get_metrics_in_range(
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> Result<Vec<MetricSample>, String> {
    if let Some(collector) = get_metrics_collector() {
        Ok(collector.get_metrics_in_range(start, end).await)
    } else {
        Err("Metrics collector not initialized".to_string())
    }
}

/// Get aggregated metrics statistics
#[tauri::command]
pub fn get_aggregated_metrics(
    metrics: Vec<MetricSample>,
) -> Result<HashMap<String, crate::metrics::MetricAggregation>, String> {
    Ok(aggregate_metrics(&metrics))
}
