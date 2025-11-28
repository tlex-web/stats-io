//! Overview Dashboard component
//!
//! Displays hardware summary, key metrics, and bottleneck analysis results.

import { useHardware } from "../hooks/useHardware";
import { useMetrics } from "../hooks/useMetrics";
import { useAnalysis } from "../hooks/useAnalysis";
import { useSessions } from "../hooks/useSessions";
import { useProfiles } from "../hooks/useProfiles";
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import { useEffect, useMemo, useState } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from "recharts";
import { CpuCoreHeatmap, MemoryStackedChart, CorrelationScatterPlot, MultiMetricChart } from "./AdvancedCharts";
import { exportReportInFormat } from "../utils/reports";

export function Dashboard() {
  const { hardware, loading: hardwareLoading, error: hardwareError } = useHardware();
  const { metrics, isCollecting, startCollection, stopCollection } = useMetrics();
  const { analysisResult, insights, analyze } = useAnalysis();
  const { currentSession, createSession, endSession, addRun } = useSessions();
  const { profiles, selectedProfile, setSelectedProfile } = useProfiles();
  const [recording, setRecording] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(["hardware", "metrics"]));
  const containerRef = useKeyboardNavigation();

  // Analyze metrics when they change
  useEffect(() => {
    if (metrics.length > 0 && isCollecting && selectedProfile) {
      // Analyze last 30 seconds of metrics
      const thirtySecondsAgo = new Date(Date.now() - 30000).toISOString();
      const recentMetrics = metrics.filter(
        (m) => m.timestamp >= thirtySecondsAgo
      );
      if (recentMetrics.length > 0) {
        analyze(recentMetrics, 30, selectedProfile);
      }
    }
  }, [metrics, isCollecting, selectedProfile, analyze]);

  // Prepare chart data from metrics
  const chartData = useMemo(() => {
    const cpuData = metrics
      .filter((m) => m.metric_type === "cpu_utilization")
      .slice(-60) // Last 60 samples
      .map((m) => ({
        time: new Date(m.timestamp).toLocaleTimeString(),
        cpu: m.value,
      }));

    const memoryData = metrics
      .filter((m) => m.metric_type === "memory_usage")
      .slice(-60)
      .map((m) => ({
        time: new Date(m.timestamp).toLocaleTimeString(),
        memory: m.value,
      }));

    const gpuData = metrics
      .filter((m) => m.metric_type === "gpu_utilization")
      .slice(-60)
      .map((m) => ({
        time: new Date(m.timestamp).toLocaleTimeString(),
        gpu: m.value,
      }));

    // Merge data by time
    const merged = new Map<string, { time: string; cpu?: number; memory?: number; gpu?: number }>();
    cpuData.forEach((d) => {
      merged.set(d.time, { ...merged.get(d.time), time: d.time, cpu: d.cpu });
    });
    memoryData.forEach((d) => {
      merged.set(d.time, { ...merged.get(d.time), time: d.time, memory: d.memory });
    });
    gpuData.forEach((d) => {
      merged.set(d.time, { ...merged.get(d.time), time: d.time, gpu: d.gpu });
    });

    return Array.from(merged.values());
  }, [metrics]);

  // Get current metric values for gauges
  const currentMetrics = useMemo(() => {
    if (metrics.length === 0) {
      return {
        cpu: 0,
        memory: 0,
        gpu: 0,
        temperature: null as number | null,
      };
    }

    const cpuMetrics = metrics.filter((m) => m.metric_type === "cpu_utilization");
    const memoryMetrics = metrics.filter((m) => m.metric_type === "memory_usage");
    const gpuMetrics = metrics.filter((m) => m.metric_type === "gpu_utilization");
    const tempMetrics = metrics.filter((m) => m.metric_type === "temperature");

    return {
      cpu: cpuMetrics.length > 0 ? cpuMetrics[cpuMetrics.length - 1].value : 0,
      memory: memoryMetrics.length > 0 ? memoryMetrics[memoryMetrics.length - 1].value : 0,
      gpu: gpuMetrics.length > 0 ? gpuMetrics[gpuMetrics.length - 1].value : 0,
      temperature: tempMetrics.length > 0 ? tempMetrics[tempMetrics.length - 1].value : null,
    };
  }, [metrics]);

  if (hardwareLoading) {
    return (
      <div className="dashboard">
        <div className="loading">Loading hardware information...</div>
      </div>
    );
  }

  if (hardwareError) {
    return (
      <div className="dashboard">
        <div className="error">Error: {hardwareError}</div>
      </div>
    );
  }

  if (!hardware) {
    return (
      <div className="dashboard">
        <div className="error">No hardware information available</div>
      </div>
    );
  }

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  return (
    <div className="dashboard" ref={containerRef}>
      <h1>PC Rig Hardware & Bottleneck Analyzer</h1>

      {/* Hardware Summary */}
      <section className="hardware-summary collapsible-section">
        <div className="section-header">
          <h2>Hardware Summary</h2>
          <button
            className="collapse-button"
            onClick={() => toggleSection("hardware")}
            aria-expanded={expandedSections.has("hardware")}
            aria-controls="hardware-content"
            aria-label={expandedSections.has("hardware") ? "Collapse hardware section" : "Expand hardware section"}
          >
            {expandedSections.has("hardware") ? "−" : "+"}
          </button>
        </div>
        {expandedSections.has("hardware") && (
          <div id="hardware-content" className="section-content">
            <div className="hardware-grid">
          <div className="hardware-card">
            <h3>CPU</h3>
            <p>{hardware.cpu.model}</p>
            <p>
              {hardware.cpu.cores} cores, {hardware.cpu.threads} threads
            </p>
            {hardware.cpu.base_clock_mhz && (
              <p>Base: {hardware.cpu.base_clock_mhz} MHz</p>
            )}
          </div>

          <div className="hardware-card">
            <h3>Memory</h3>
            <p>{(hardware.memory.total_mb / 1024).toFixed(1)} GB</p>
            {hardware.memory.speed_mhz && (
              <p>{hardware.memory.speed_mhz} MHz</p>
            )}
          </div>

          <div className="hardware-card">
            <h3>GPU</h3>
            {hardware.gpus.length > 0 ? (
              hardware.gpus.map((gpu, idx) => (
                <div key={idx}>
                  <p>{gpu.model}</p>
                  {gpu.vram_total_mb && (
                    <p>VRAM: {(gpu.vram_total_mb / 1024).toFixed(1)} GB</p>
                  )}
                </div>
              ))
            ) : (
              <p>No GPU detected</p>
            )}
          </div>
            </div>
          </div>
        )}
      </section>

      {/* Workload Profile Selection */}
      <section className="workload-profile">
        <h2>Workload Profile</h2>
        <div className="profile-selector">
          <label htmlFor="profile-select">Select Workload Profile:</label>
          <select
            id="profile-select"
            value={selectedProfile?.id || ""}
            onChange={(e) => {
              const profile = profiles.find((p) => p.id === e.target.value);
              if (profile) {
                setSelectedProfile(profile);
              }
            }}
            disabled={recording}
          >
            {profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.name}
              </option>
            ))}
          </select>
          {selectedProfile && (
            <div className="profile-info">
              <p>
                <strong>Type:</strong> {selectedProfile.workload_type}
              </p>
              {selectedProfile.parameters && Object.keys(selectedProfile.parameters).length > 0 && (
                <div>
                  <strong>Parameters:</strong>
                  <ul>
                    {Object.entries(selectedProfile.parameters).map(([key, value]) => (
                      <li key={key}>
                        {key}: {String(value)}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      </section>

      {/* Metrics Collection Controls */}
      <section className="metrics-controls collapsible-section">
        <div className="section-header">
          <h2>Metrics Collection & Session Recording</h2>
          <button
            className="collapse-button"
            onClick={() => toggleSection("metrics")}
            aria-expanded={expandedSections.has("metrics")}
            aria-controls="metrics-content"
            aria-label={expandedSections.has("metrics") ? "Collapse metrics section" : "Expand metrics section"}
          >
            {expandedSections.has("metrics") ? "−" : "+"}
          </button>
        </div>
        {expandedSections.has("metrics") && (
          <div id="metrics-content" className="section-content">
            <div className="controls">
          {!isCollecting ? (
            <button onClick={() => startCollection()}>Start Monitoring</button>
          ) : (
            <button onClick={stopCollection}>Stop Monitoring</button>
          )}
          
          {isCollecting && !recording && selectedProfile && (
            <button
              onClick={async () => {
                if (hardware && selectedProfile) {
                  try {
                    await createSession("Session", selectedProfile, hardware);
                    setRecording(true);
                  } catch (err) {
                    console.error("Failed to create session:", err);
                  }
                }
              }}
            >
              Start Recording Session
            </button>
          )}
          
          {recording && currentSession && (
            <button
              onClick={async () => {
                try {
                  // Create a run from current metrics and analysis
                  const run = {
                    id: crypto.randomUUID(),
                    name: `Run ${currentSession.runs.length + 1}`,
                    metrics_streams: {
                      cpu: metrics.filter((m) => m.metric_type === "cpu_utilization"),
                      memory: metrics.filter((m) => m.metric_type === "memory_usage"),
                    },
                    analysis_result: analysisResult,
                    notes: null,
                  };
                  
                  await addRun(currentSession.id, run);
                } catch (err) {
                  console.error("Failed to add run:", err);
                }
              }}
            >
              Save Run
            </button>
          )}
          
          {recording && currentSession && (
            <button
              onClick={async () => {
                try {
                  await endSession(currentSession.id);
                  setRecording(false);
                } catch (err) {
                  console.error("Failed to end session:", err);
                }
              }}
            >
              End Session
            </button>
          )}
          
          {currentSession && insights && (
            <>
              <button
                onClick={async () => {
                  try {
                    await exportReportInFormat(currentSession, hardware, "text");
                  } catch (err) {
                    console.error("Failed to export report:", err);
                  }
                }}
              >
                Export Text Report
              </button>
              <button
                onClick={async () => {
                  try {
                    await exportReportInFormat(currentSession, hardware, "html");
                  } catch (err) {
                    console.error("Failed to export report:", err);
                  }
                }}
              >
                Export HTML Report
              </button>
              <button
                onClick={async () => {
                  try {
                    await exportReportInFormat(currentSession, hardware, "json");
                  } catch (err) {
                    console.error("Failed to export report:", err);
                  }
                }}
              >
                Export JSON Report
              </button>
            </>
          )}
            </div>
          </div>
        )}
      </section>

      {/* Current Metrics Gauges */}
      {isCollecting && (
        <section className="current-metrics">
          <h2>Current Metrics</h2>
          <div className="metrics-gauges">
            <div className="gauge-card">
              <h3>CPU Utilization</h3>
              <div className="gauge">
                <div className="gauge-fill" style={{ width: `${currentMetrics.cpu}%` }}></div>
                <span className="gauge-value">{currentMetrics.cpu.toFixed(1)}%</span>
              </div>
            </div>
            <div className="gauge-card">
              <h3>Memory Usage</h3>
              <div className="gauge">
                <div className="gauge-fill" style={{ width: `${currentMetrics.memory}%` }}></div>
                <span className="gauge-value">{currentMetrics.memory.toFixed(1)}%</span>
              </div>
            </div>
            {currentMetrics.gpu > 0 && (
              <div className="gauge-card">
                <h3>GPU Utilization</h3>
                <div className="gauge">
                  <div className="gauge-fill" style={{ width: `${currentMetrics.gpu}%` }}></div>
                  <span className="gauge-value">{currentMetrics.gpu.toFixed(1)}%</span>
                </div>
              </div>
            )}
            {currentMetrics.temperature !== null && (
              <div className="gauge-card">
                <h3>Temperature</h3>
                <div className="temperature-display">
                  <span className="temp-value">{currentMetrics.temperature.toFixed(1)}°C</span>
                  <span className="temp-status">
                    {currentMetrics.temperature > 90
                      ? "⚠️ Critical"
                      : currentMetrics.temperature > 80
                      ? "⚠️ High"
                      : "✓ Normal"}
                  </span>
                </div>
              </div>
            )}
          </div>
        </section>
      )}

      {/* Real-time Metrics Charts */}
      {isCollecting && chartData.length > 0 && (
        <>
          <section className="metrics-charts">
            <h2>Real-time Metrics</h2>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis domain={[0, 100]} />
                <Tooltip />
                <Legend />
                <Line
                  type="monotone"
                  dataKey="cpu"
                  stroke="#8884d8"
                  name="CPU %"
                  dot={false}
                  strokeWidth={2}
                />
                <Line
                  type="monotone"
                  dataKey="memory"
                  stroke="#82ca9d"
                  name="Memory %"
                  dot={false}
                  strokeWidth={2}
                />
                {chartData.some((d) => d.gpu !== undefined) && (
                  <Line
                    type="monotone"
                    dataKey="gpu"
                    stroke="#ffc658"
                    name="GPU %"
                    dot={false}
                    strokeWidth={2}
                  />
                )}
              </LineChart>
            </ResponsiveContainer>
          </section>

          {/* Advanced Visualizations */}
          <section className="advanced-charts">
            <h2>Advanced Visualizations</h2>
            <div className="charts-grid">
              <div className="chart-item">
                <MultiMetricChart metrics={metrics} />
              </div>
              <div className="chart-item">
                <MemoryStackedChart metrics={metrics} />
              </div>
              <div className="chart-item">
                <CpuCoreHeatmap metrics={metrics} />
              </div>
              {metrics.some((m) => m.metric_type === "gpu_utilization") && (
                <div className="chart-item">
                  <CorrelationScatterPlot metrics={metrics} />
                </div>
              )}
            </div>
          </section>
        </>
      )}

      {/* Bottleneck Analysis Results */}
      {insights && (
        <section className="bottleneck-analysis">
          <h2>Bottleneck Analysis</h2>
          <div className="insights">
            <div className="summary">
              <h3>Summary</h3>
              <p>{insights.summary}</p>
            </div>
            {insights.recommendations.length > 0 && (
              <div className="recommendations">
                <h3>Recommendations</h3>
                <ul>
                  {insights.recommendations.map((rec, idx) => (
                    <li key={idx}>{rec}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
          {analysisResult && analysisResult.bottlenecks.length > 0 && (
            <div className="bottlenecks">
              <h3>Detected Bottlenecks</h3>
              {analysisResult.bottlenecks.map((bottleneck, idx) => (
                <div key={idx} className="bottleneck-card">
                  <h4>
                    {bottleneck.bottleneck_type.toUpperCase()} (Severity:{" "}
                    {bottleneck.severity})
                  </h4>
                  <p>{bottleneck.summary}</p>
                  <details>
                    <summary>Details</summary>
                    <p>{bottleneck.details}</p>
                  </details>
                </div>
              ))}
            </div>
          )}
        </section>
      )}
    </div>
  );
}

