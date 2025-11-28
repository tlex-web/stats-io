//! Tauri commands for report generation and export
//!
//! This module exposes report generation functionality to the frontend.

use crate::core::domain::{HardwareConfig, Run, Session};
use crate::persistence::reports::{
    generate_comparison_report, generate_session_report, ReportConfig, ReportFormat,
};
use crate::analysis::comparison::ComparisonResult;

/// Generate a session report
#[tauri::command]
pub fn generate_report(
    session: Session,
    hardware: HardwareConfig,
    format: String,
    include_hardware: Option<bool>,
    include_metrics: Option<bool>,
    include_analysis: Option<bool>,
    include_recommendations: Option<bool>,
) -> Result<String, String> {
    let report_format = match format.as_str() {
        "text" => ReportFormat::Text,
        "html" => ReportFormat::Html,
        "json" => ReportFormat::Json,
        "pdf" => ReportFormat::Pdf,
        _ => return Err("Invalid format. Must be 'text', 'html', 'json', or 'pdf'".to_string()),
    };

    let config = ReportConfig {
        include_hardware: include_hardware.unwrap_or(true),
        include_metrics: include_metrics.unwrap_or(true),
        include_analysis: include_analysis.unwrap_or(true),
        include_recommendations: include_recommendations.unwrap_or(true),
        include_comparison: false,
        format: report_format,
    };

    Ok(generate_session_report(&session, &hardware, &config))
}

/// Generate a comparison report
#[tauri::command]
pub fn generate_comparison_report_command(
    comparison: ComparisonResult,
    run1: Run,
    run2: Run,
    format: String,
) -> Result<String, String> {
    let report_format = match format.as_str() {
        "text" => ReportFormat::Text,
        "html" => ReportFormat::Html,
        "json" => ReportFormat::Json,
        "pdf" => ReportFormat::Pdf,
        _ => return Err("Invalid format. Must be 'text', 'html', 'json', or 'pdf'".to_string()),
    };

    let config = ReportConfig {
        include_hardware: false,
        include_metrics: true,
        include_analysis: true,
        include_recommendations: false,
        include_comparison: true,
        format: report_format,
    };

    Ok(generate_comparison_report(&comparison, &run1, &run2, &config))
}

