//! Bottleneck analysis subsystem
//!
//! This module provides bottleneck analysis functionality following AGENT.md Section 6.4.

pub mod engine;
pub mod insights;
pub mod rules;
pub mod comparison;

pub use engine::AnalysisEngine;
pub use comparison::{compare_runs, ComparisonResult, MetricDelta, BottleneckChange, BottleneckStatus};
