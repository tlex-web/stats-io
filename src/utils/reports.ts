//! Report generation utilities
//!
//! This module provides functions to generate and export reports
//! using the backend report generation system.

// Note: Tauri v2 uses different API structure
// For now, we'll use the invoke command to handle file operations
// The backend will handle file dialogs and writing
import { invoke } from "@tauri-apps/api/core";
import type {
  Session,
  HardwareConfig,
  ComparisonResult,
  Run,
} from "../types/index";

/// Export report in a specific format
export async function exportReportInFormat(
  session: Session,
  hardware: HardwareConfig,
  format: "text" | "html" | "json" | "pdf",
  options?: {
    includeHardware?: boolean;
    includeMetrics?: boolean;
    includeAnalysis?: boolean;
    includeRecommendations?: boolean;
  }
): Promise<void> {
  try {
    const report = await invoke<string>("generate_report", {
      session,
      hardware,
      format,
      includeHardware: options?.includeHardware ?? true,
      includeMetrics: options?.includeMetrics ?? true,
      includeAnalysis: options?.includeAnalysis ?? true,
      includeRecommendations: options?.includeRecommendations ?? true,
    });

    // For MVP, use browser download
    // Tauri file dialog integration can be added later
    const extension = format === "pdf" ? "html" : format;
    const blob = new Blob([report], {
      type: format === "html" ? "text/html" : format === "json" ? "application/json" : "text/plain",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `report-${session.id}-${Date.now()}.${extension}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error("Failed to export report:", error);
    alert("Failed to export report");
  }
}

/// Export comparison report
export async function exportComparisonReport(
  comparison: ComparisonResult,
  run1: Run,
  run2: Run,
  format: "text" | "html" | "json" | "pdf"
): Promise<void> {
  try {
    const report = await invoke<string>("generate_comparison_report_command", {
      comparison,
      run1,
      run2,
      format,
    });

    // For MVP, use browser download
    const extension = format === "pdf" ? "html" : format;
    const blob = new Blob([report], {
      type: format === "html" ? "text/html" : format === "json" ? "application/json" : "text/plain",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `comparison-${run1.id}-${run2.id}-${Date.now()}.${extension}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error("Failed to export comparison report:", error);
    alert("Failed to export comparison report");
  }
}

/// Legacy function for backward compatibility
export async function exportReport(
  session: Session,
  hardware: HardwareConfig
): Promise<void> {
  await exportReportInFormat(session, hardware, "html");
}
