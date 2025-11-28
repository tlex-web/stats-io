//! Comparison View component
//!
//! Displays side-by-side comparison of runs/sessions with delta visualization.

import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSessions } from "../hooks/useSessions";
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import { exportComparisonReport } from "../utils/reports";
import {
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  BarChart,
  Bar,
} from "recharts";
import type {
  Run,
  ComparisonResult,
  MetricDelta,
  BottleneckStatus,
} from "../types/index";

export function ComparisonView() {
  const { sessions, loadSessions } = useSessions();
  const containerRef = useKeyboardNavigation();
  
  useEffect(() => {
    loadSessions().catch(console.error);
  }, [loadSessions]);
  const [selectedRun1, setSelectedRun1] = useState<{ sessionId: string; runId: string } | null>(
    null
  );
  const [selectedRun2, setSelectedRun2] = useState<{ sessionId: string; runId: string } | null>(
    null
  );
  const [comparisonResult, setComparisonResult] = useState<ComparisonResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Get all available runs from sessions
  const availableRuns = useMemo(() => {
    const runs: Array<{ sessionId: string; run: Run; sessionName: string }> = [];
    sessions.forEach((session) => {
      session.runs.forEach((run) => {
        runs.push({
          sessionId: session.id,
          run,
          sessionName: session.profile.name,
        });
      });
    });
    return runs;
  }, [sessions]);

  const compare = async () => {
    if (!selectedRun1 || !selectedRun2) {
      setError("Please select two runs to compare");
      return;
    }

    try {
      setLoading(true);
      setError(null);

      // Load the runs
      const run1Data = availableRuns.find(
        (r) => r.sessionId === selectedRun1.sessionId && r.run.id === selectedRun1.runId
      );
      const run2Data = availableRuns.find(
        (r) => r.sessionId === selectedRun2.sessionId && r.run.id === selectedRun2.runId
      );

      if (!run1Data || !run2Data) {
        setError("Could not find selected runs");
        return;
      }

      const result = await invoke<ComparisonResult>("compare_runs_command", {
        run1: run1Data.run,
        run2: run2Data.run,
      });

      setComparisonResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to compare runs");
    } finally {
      setLoading(false);
    }
  };

  // Prepare chart data for metric comparison
  const metricChartData = useMemo(() => {
    if (!comparisonResult) return [];

    return (Object.values(comparisonResult.metric_deltas) as MetricDelta[]).map((delta) => ({
      metric: delta.metric_type.replace(/_/g, " "),
      run1: delta.run1_avg,
      run2: delta.run2_avg,
      delta: delta.delta,
      deltaPercent: delta.delta_percent,
    }));
  }, [comparisonResult]);

  return (
    <div className="comparison-view" ref={containerRef}>
      <h1>Run Comparison</h1>

      {/* Run Selection */}
      <section className="run-selection">
        <h2>Select Runs to Compare</h2>
        <div className="run-selectors">
          <div className="run-selector">
            <label htmlFor="run1-select">Run 1:</label>
            <select
              id="run1-select"
              value={selectedRun1 ? `${selectedRun1.sessionId}:${selectedRun1.runId}` : ""}
              onChange={(e) => {
                const [sessionId, runId] = e.target.value.split(":");
                setSelectedRun1({ sessionId, runId });
              }}
            >
              <option value="">Select a run...</option>
              {availableRuns.map((r) => (
                <option key={`${r.sessionId}:${r.run.id}`} value={`${r.sessionId}:${r.run.id}`}>
                  {r.sessionName} - {r.run.name}
                </option>
              ))}
            </select>
          </div>

          <div className="run-selector">
            <label htmlFor="run2-select">Run 2:</label>
            <select
              id="run2-select"
              value={selectedRun2 ? `${selectedRun2.sessionId}:${selectedRun2.runId}` : ""}
              onChange={(e) => {
                const [sessionId, runId] = e.target.value.split(":");
                setSelectedRun2({ sessionId, runId });
              }}
            >
              <option value="">Select a run...</option>
              {availableRuns.map((r) => (
                <option key={`${r.sessionId}:${r.run.id}`} value={`${r.sessionId}:${r.run.id}`}>
                  {r.sessionName} - {r.run.name}
                </option>
              ))}
            </select>
          </div>

          <button onClick={compare} disabled={!selectedRun1 || !selectedRun2 || loading}>
            {loading ? "Comparing..." : "Compare Runs"}
          </button>
        </div>
      </section>

      {/* Comparison Results */}
      {comparisonResult && (
        <>
          <section className="comparison-summary">
            <h2>Comparison Summary</h2>
            <p>{comparisonResult.summary}</p>
          </section>

          {/* Metric Deltas */}
          {metricChartData.length > 0 && (
            <section className="metric-comparison">
              <h2>Metric Comparison</h2>
              <ResponsiveContainer width="100%" height={400}>
                <BarChart data={metricChartData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="metric" angle={-45} textAnchor="end" height={100} />
                  <YAxis />
                  <Tooltip />
                  <Legend />
                  <Bar dataKey="run1" fill="#8884d8" name="Run 1" />
                  <Bar dataKey="run2" fill="#82ca9d" name="Run 2" />
                </BarChart>
              </ResponsiveContainer>

              {/* Delta Table */}
              <div className="delta-table">
                <h3>Metric Deltas</h3>
                <table>
                  <thead>
                    <tr>
                      <th>Metric</th>
                      <th>Run 1 Avg</th>
                      <th>Run 2 Avg</th>
                      <th>Delta</th>
                      <th>Delta %</th>
                    </tr>
                  </thead>
                  <tbody>
                    {(Object.values(comparisonResult.metric_deltas) as MetricDelta[]).map((delta) => (
                      <tr key={delta.metric_type}>
                        <td>{delta.metric_type.replace(/_/g, " ")}</td>
                        <td>
                          {delta.run1_avg.toFixed(2)} {delta.unit}
                        </td>
                        <td>
                          {delta.run2_avg.toFixed(2)} {delta.unit}
                        </td>
                        <td className={delta.delta >= 0 ? "delta-positive" : "delta-negative"}>
                          {delta.delta >= 0 ? "+" : ""}
                          {delta.delta.toFixed(2)} {delta.unit}
                        </td>
                        <td className={delta.delta_percent >= 0 ? "delta-positive" : "delta-negative"}>
                          {delta.delta_percent >= 0 ? "+" : ""}
                          {delta.delta_percent.toFixed(1)}%
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          )}

          {/* Bottleneck Changes */}
          {comparisonResult.bottleneck_changes.length > 0 && (
            <section className="bottleneck-comparison">
              <h2>Bottleneck Changes</h2>
              <div className="bottleneck-changes">
                {comparisonResult.bottleneck_changes.map((change, idx) => (
                  <div key={idx} className="bottleneck-change-card">
                    <h3>{change.bottleneck_type.replace(/_/g, " ").toUpperCase()}</h3>
                    <div className="change-details">
                      <div>
                        <strong>Run 1:</strong>{" "}
                        {change.run1_severity ? `${change.run1_severity}/100` : "None"}
                      </div>
                      <div>
                        <strong>Run 2:</strong>{" "}
                        {change.run2_severity ? `${change.run2_severity}/100` : "None"}
                      </div>
                      <div className={`status status-${change.status}`}>
                        <strong>Status:</strong> {formatStatus(change.status)}
                        {change.severity_delta !== 0 && (
                          <span>
                            {" "}
                            ({change.severity_delta >= 0 ? "+" : ""}
                            {change.severity_delta})
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </section>
          )}
        </>
      )}

      {error && <div className="error" role="alert">Error: {error}</div>}
      
      {/* Export Comparison Report */}
      {comparisonResult && selectedRun1 && selectedRun2 && (
        <section className="export-section">
          <h2>Export Comparison Report</h2>
          <div className="export-buttons">
            <button
              onClick={async () => {
                const run1Data = availableRuns.find(
                  (r) => r.sessionId === selectedRun1.sessionId && r.run.id === selectedRun1.runId
                );
                const run2Data = availableRuns.find(
                  (r) => r.sessionId === selectedRun2.sessionId && r.run.id === selectedRun2.runId
                );
                if (run1Data && run2Data) {
                  await exportComparisonReport(comparisonResult, run1Data.run, run2Data.run, "html");
                }
              }}
              aria-label="Export comparison report as HTML"
            >
              Export HTML Report
            </button>
            <button
              onClick={async () => {
                const run1Data = availableRuns.find(
                  (r) => r.sessionId === selectedRun1.sessionId && r.run.id === selectedRun1.runId
                );
                const run2Data = availableRuns.find(
                  (r) => r.sessionId === selectedRun2.sessionId && r.run.id === selectedRun2.runId
                );
                if (run1Data && run2Data) {
                  await exportComparisonReport(comparisonResult, run1Data.run, run2Data.run, "json");
                }
              }}
              aria-label="Export comparison report as JSON"
            >
              Export JSON Report
            </button>
          </div>
        </section>
      )}
    </div>
  );
}

function formatStatus(status: BottleneckStatus): string {
  switch (status) {
    case "new":
      return "New";
    case "resolved":
      return "Resolved";
    case "improved":
      return "Improved";
    case "worsened":
      return "Worsened";
    case "unchanged":
      return "Unchanged";
    default:
      return String(status);
  }
}

