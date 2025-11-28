//! Report generation and export
//!
//! This module provides functionality for generating and exporting reports
//! following AGENT.md Section 3.4 and IMPLEMENTATION_PLAN.md Phase 3.2.

use crate::core::domain::{
    HardwareConfig, Run, Session,
};
use crate::analysis::comparison::ComparisonResult;
use serde::{Deserialize, Serialize};

/// Report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub include_hardware: bool,
    pub include_metrics: bool,
    pub include_analysis: bool,
    pub include_recommendations: bool,
    pub include_comparison: bool,
    pub format: ReportFormat,
}

/// Report format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Text,
    Html,
    Json,
    Pdf, // Note: PDF generation would require additional dependencies
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_hardware: true,
            include_metrics: true,
            include_analysis: true,
            include_recommendations: true,
            include_comparison: false,
            format: ReportFormat::Html,
        }
    }
}

/// Generate a comprehensive report for a session
pub fn generate_session_report(
    session: &Session,
    hardware: &HardwareConfig,
    config: &ReportConfig,
) -> String {
    match config.format {
        ReportFormat::Text => generate_text_report(session, hardware, config),
        ReportFormat::Html => generate_html_report(session, hardware, config),
        ReportFormat::Json => generate_json_report(session, hardware, config),
        ReportFormat::Pdf => {
            // PDF generation would require additional dependencies
            // For now, generate HTML which can be converted to PDF
            generate_html_report(session, hardware, config)
        }
    }
}

/// Generate a comparison report
pub fn generate_comparison_report(
    comparison: &ComparisonResult,
    run1: &Run,
    run2: &Run,
    config: &ReportConfig,
) -> String {
    match config.format {
        ReportFormat::Text => generate_text_comparison_report(comparison, run1, run2, config),
        ReportFormat::Html => generate_html_comparison_report(comparison, run1, run2, config),
        ReportFormat::Json => generate_json_comparison_report(comparison, run1, run2, config),
        ReportFormat::Pdf => generate_html_comparison_report(comparison, run1, run2, config),
    }
}

/// Generate text report
fn generate_text_report(
    session: &Session,
    hardware: &HardwareConfig,
    config: &ReportConfig,
) -> String {
    let mut report = String::new();
    
    report.push_str("=".repeat(80).as_str());
    report.push_str("\n");
    report.push_str(&format!("PC RIG HARDWARE & BOTTLENECK ANALYSIS REPORT\n"));
    report.push_str("=".repeat(80).as_str());
    report.push_str("\n\n");
    
    report.push_str(&format!("Session: {}\n", session.id));
    report.push_str(&format!("Profile: {}\n", session.profile.name));
    report.push_str(&format!("Started: {}\n", session.start_time));
    if let Some(end_time) = session.end_time {
        report.push_str(&format!("Ended: {}\n", end_time));
    }
    report.push_str(&format!("Runs: {}\n", session.runs.len()));
    report.push_str("\n");
    
    if config.include_hardware {
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n");
        report.push_str("HARDWARE CONFIGURATION\n");
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n\n");
        
        report.push_str(&format!("CPU: {} ({} cores, {} threads)\n", 
            hardware.cpu.model, hardware.cpu.cores, hardware.cpu.threads));
        report.push_str(&format!("Memory: {:.2} GB\n", 
            hardware.memory.total_mb as f64 / 1024.0));
        if !hardware.gpus.is_empty() {
            report.push_str(&format!("GPU: {}\n", hardware.gpus[0].model));
        }
        report.push_str("\n");
    }
    
    if config.include_analysis && !session.runs.is_empty() {
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n");
        report.push_str("BOTTLENECK ANALYSIS\n");
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n\n");
        
        for (idx, run) in session.runs.iter().enumerate() {
            if let Some(analysis) = &run.analysis_result {
                report.push_str(&format!("Run {}: {}\n", idx + 1, run.name));
                report.push_str(&format!("  Bottlenecks detected: {}\n", analysis.bottlenecks.len()));
                
                for bottleneck in &analysis.bottlenecks {
                    report.push_str(&format!("  - {} (Severity: {}/100)\n", 
                        bottleneck.summary, bottleneck.severity));
                    report.push_str(&format!("    {}\n", bottleneck.details));
                }
                report.push_str("\n");
            }
        }
    }
    
    if config.include_recommendations && !session.runs.is_empty() {
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n");
        report.push_str("RECOMMENDATIONS\n");
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n\n");
        
        for (idx, run) in session.runs.iter().enumerate() {
            if let Some(_analysis) = &run.analysis_result {
                report.push_str(&format!("Run {} Recommendations:\n", idx + 1));
                // Recommendations would come from insights generation
                report.push_str("\n");
            }
        }
    }
    
    report
}

/// Generate HTML report
fn generate_html_report(
    session: &Session,
    hardware: &HardwareConfig,
    config: &ReportConfig,
) -> String {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("  <title>PC Rig Analysis Report</title>\n");
    html.push_str("  <style>\n");
    html.push_str(include_str!("report_styles.css"));
    html.push_str("  </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    
    html.push_str("  <div class=\"report-container\">\n");
    html.push_str("    <header class=\"report-header\">\n");
    html.push_str("      <h1>PC Rig Hardware & Bottleneck Analysis Report</h1>\n");
    html.push_str("    </header>\n");
    
    html.push_str("    <section class=\"session-info\">\n");
    html.push_str("      <h2>Session Information</h2>\n");
    html.push_str(&format!("      <p><strong>Session:</strong> {}</p>\n", session.id));
    html.push_str(&format!("      <p><strong>Profile:</strong> {}</p>\n", session.profile.name));
    html.push_str(&format!("      <p><strong>Started:</strong> {}</p>\n", session.start_time));
    if let Some(end_time) = session.end_time {
        html.push_str(&format!("      <p><strong>Ended:</strong> {}</p>\n", end_time));
    }
    html.push_str(&format!("      <p><strong>Runs:</strong> {}</p>\n", session.runs.len()));
    html.push_str("    </section>\n");
    
    if config.include_hardware {
        html.push_str("    <section class=\"hardware-section\">\n");
        html.push_str("      <h2>Hardware Configuration</h2>\n");
        html.push_str("      <div class=\"hardware-grid\">\n");
        html.push_str(&format!(
            "        <div class=\"hardware-card\">\n          <h3>CPU</h3>\n          <p>{}</p>\n          <p>{} cores, {} threads</p>\n        </div>\n",
            hardware.cpu.model, hardware.cpu.cores, hardware.cpu.threads
        ));
        html.push_str(&format!(
            "        <div class=\"hardware-card\">\n          <h3>Memory</h3>\n          <p>{:.2} GB</p>\n        </div>\n",
            hardware.memory.total_mb as f64 / 1024.0
        ));
        if !hardware.gpus.is_empty() {
            html.push_str(&format!(
                "        <div class=\"hardware-card\">\n          <h3>GPU</h3>\n          <p>{}</p>\n        </div>\n",
                hardware.gpus[0].model
            ));
        }
        html.push_str("      </div>\n");
        html.push_str("    </section>\n");
    }
    
    if config.include_analysis && !session.runs.is_empty() {
        html.push_str("    <section class=\"analysis-section\">\n");
        html.push_str("      <h2>Bottleneck Analysis</h2>\n");
        
        for (idx, run) in session.runs.iter().enumerate() {
            if let Some(analysis) = &run.analysis_result {
                html.push_str(&format!("      <div class=\"run-analysis\">\n"));
                html.push_str(&format!("        <h3>Run {}: {}</h3>\n", idx + 1, run.name));
                html.push_str(&format!("        <p class=\"bottleneck-count\">Bottlenecks detected: {}</p>\n", 
                    analysis.bottlenecks.len()));
                
                for bottleneck in &analysis.bottlenecks {
                    html.push_str("        <div class=\"bottleneck-card\">\n");
                    html.push_str(&format!("          <h4>{}</h4>\n", bottleneck.summary));
                    html.push_str(&format!("          <p class=\"severity\">Severity: {}/100</p>\n", 
                        bottleneck.severity));
                    html.push_str(&format!("          <p>{}</p>\n", bottleneck.details));
                    html.push_str("        </div>\n");
                }
                html.push_str("      </div>\n");
            }
        }
        html.push_str("    </section>\n");
    }
    
    html.push_str("  </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");
    
    html
}

/// Generate JSON report
fn generate_json_report(
    session: &Session,
    hardware: &HardwareConfig,
    _config: &ReportConfig,
) -> String {
    #[derive(Serialize)]
    struct JsonReport {
        session: SessionSummary,
        hardware: HardwareSummary,
        runs: Vec<RunSummary>,
    }
    
    #[derive(Serialize)]
    struct SessionSummary {
        name: String,
        profile: String,
        start_time: String,
        end_time: Option<String>,
        run_count: usize,
    }
    
    #[derive(Serialize)]
        struct HardwareSummary {
            cpu: String,
            cores: u32,
            threads: u32,
            memory_mb: u64,
            gpu: Option<String>,
        }
    
    #[derive(Serialize)]
    struct RunSummary {
        name: String,
        bottleneck_count: usize,
        bottlenecks: Vec<BottleneckSummary>,
    }
    
    #[derive(Serialize)]
    struct BottleneckSummary {
        r#type: String,
        severity: u8,
        summary: String,
    }
    
    let report = JsonReport {
        session: SessionSummary {
            name: session.id.to_string(),
            profile: session.profile.name.clone(),
            start_time: session.start_time.to_string(),
            end_time: session.end_time.map(|t| t.to_string()),
            run_count: session.runs.len(),
        },
            hardware: HardwareSummary {
                cpu: hardware.cpu.model.clone(),
                cores: hardware.cpu.cores,
                threads: hardware.cpu.threads,
                memory_mb: hardware.memory.total_mb,
                gpu: hardware.gpus.first().map(|g| g.model.clone()),
            },
        runs: session.runs
            .iter()
            .map(|run| RunSummary {
                name: run.name.clone(),
                bottleneck_count: run.analysis_result.as_ref()
                    .map(|a| a.bottlenecks.len())
                    .unwrap_or(0),
                bottlenecks: run.analysis_result.as_ref()
                    .map(|a| a.bottlenecks.iter().map(|b| BottleneckSummary {
                        r#type: format!("{:?}", b.bottleneck_type),
                        severity: b.severity,
                        summary: b.summary.clone(),
                    }).collect())
                    .unwrap_or_default(),
            })
            .collect(),
    };
    
    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}

/// Generate text comparison report
fn generate_text_comparison_report(
    comparison: &ComparisonResult,
    _run1: &Run,
    _run2: &Run,
    _config: &ReportConfig,
) -> String {
    let mut report = String::new();
    
    report.push_str("=".repeat(80).as_str());
    report.push_str("\n");
    report.push_str("RUN COMPARISON REPORT\n");
    report.push_str("=".repeat(80).as_str());
    report.push_str("\n\n");
    
    report.push_str(&format!("Run 1: {}\n", comparison.run1_id));
    report.push_str(&format!("Run 2: {}\n", comparison.run2_id));
    report.push_str("\n");
    
    report.push_str(&format!("Summary: {}\n", comparison.summary));
    report.push_str("\n");
    
    if !comparison.metric_deltas.is_empty() {
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n");
        report.push_str("METRIC CHANGES\n");
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n\n");
        
        for delta in comparison.metric_deltas.values() {
            report.push_str(&format!("{}:\n", delta.metric_type));
            report.push_str(&format!("  Run 1 Avg: {:.2} {}\n", delta.run1_avg, delta.unit));
            report.push_str(&format!("  Run 2 Avg: {:.2} {}\n", delta.run2_avg, delta.unit));
            report.push_str(&format!("  Delta: {:.2} {} ({:+.1}%)\n", 
                delta.delta, delta.unit, delta.delta_percent));
            report.push_str("\n");
        }
    }
    
    if !comparison.bottleneck_changes.is_empty() {
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n");
        report.push_str("BOTTLENECK CHANGES\n");
        report.push_str("-".repeat(80).as_str());
        report.push_str("\n\n");
        
        for change in &comparison.bottleneck_changes {
            report.push_str(&format!("{}:\n", change.bottleneck_type));
            report.push_str(&format!("  Run 1 Severity: {}\n", 
                change.run1_severity.map(|s| s.to_string()).unwrap_or_else(|| "None".to_string())));
            report.push_str(&format!("  Run 2 Severity: {}\n", 
                change.run2_severity.map(|s| s.to_string()).unwrap_or_else(|| "None".to_string())));
            report.push_str(&format!("  Status: {:?}\n", change.status));
            report.push_str("\n");
        }
    }
    
    report
}

/// Generate HTML comparison report
fn generate_html_comparison_report(
    comparison: &ComparisonResult,
    _run1: &Run,
    _run2: &Run,
    _config: &ReportConfig,
) -> String {
    let mut html = String::new();
    
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <title>Run Comparison Report</title>\n");
    html.push_str("  <style>\n");
    html.push_str(include_str!("report_styles.css"));
    html.push_str("  </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    
    html.push_str("  <div class=\"report-container\">\n");
    html.push_str("    <header class=\"report-header\">\n");
    html.push_str("      <h1>Run Comparison Report</h1>\n");
    html.push_str("    </header>\n");
    
    html.push_str(&format!("    <p><strong>Run 1:</strong> {}</p>\n", comparison.run1_id));
    html.push_str(&format!("    <p><strong>Run 2:</strong> {}</p>\n", comparison.run2_id));
    html.push_str(&format!("    <p class=\"summary\">{}</p>\n", comparison.summary));
    
    if !comparison.metric_deltas.is_empty() {
        html.push_str("    <section class=\"metric-comparison\">\n");
        html.push_str("      <h2>Metric Changes</h2>\n");
        html.push_str("      <table>\n");
        html.push_str("        <thead><tr><th>Metric</th><th>Run 1</th><th>Run 2</th><th>Delta</th><th>Delta %</th></tr></thead>\n");
        html.push_str("        <tbody>\n");
        
        for delta in comparison.metric_deltas.values() {
            html.push_str("          <tr>\n");
            html.push_str(&format!("            <td>{}</td>\n", delta.metric_type));
            html.push_str(&format!("            <td>{:.2} {}</td>\n", delta.run1_avg, delta.unit));
            html.push_str(&format!("            <td>{:.2} {}</td>\n", delta.run2_avg, delta.unit));
            html.push_str(&format!("            <td class=\"{}\">{:+.2} {}</td>\n", 
                if delta.delta >= 0.0 { "delta-positive" } else { "delta-negative" },
                delta.delta, delta.unit));
            html.push_str(&format!("            <td class=\"{}\">{:+.1}%</td>\n", 
                if delta.delta_percent >= 0.0 { "delta-positive" } else { "delta-negative" },
                delta.delta_percent));
            html.push_str("          </tr>\n");
        }
        
        html.push_str("        </tbody>\n");
        html.push_str("      </table>\n");
        html.push_str("    </section>\n");
    }
    
    html.push_str("  </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");
    
    html
}

/// Generate JSON comparison report
fn generate_json_comparison_report(
    comparison: &ComparisonResult,
    _run1: &Run,
    _run2: &Run,
    _config: &ReportConfig,
) -> String {
    serde_json::to_string_pretty(comparison).unwrap_or_else(|_| "{}".to_string())
}

