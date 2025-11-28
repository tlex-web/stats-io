//! Advanced visualization components
//!
//! Provides heatmaps, stacked charts, scatter plots, and other advanced visualizations
//! following IMPLEMENTATION_PLAN.md Phase 3.4.

import { useMemo } from "react";
import {
  ResponsiveContainer,
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  Cell,
  ScatterChart,
  Scatter,
  ZAxis,
} from "recharts";
import type { MetricSample } from "../types/index";

interface AdvancedChartsProps {
  metrics: MetricSample[];
  timeRange?: { start: Date; end: Date };
}

/// CPU Core Heatmap Component
export function CpuCoreHeatmap({ metrics }: AdvancedChartsProps) {
  // Group metrics by core (if available)
  // For now, we'll show overall CPU utilization over time
  const chartData = useMemo(() => {
    const cpuMetrics = metrics.filter((m) => m.metric_type === "cpu_utilization");
    
    // Group by time intervals (e.g., every 5 seconds)
    const intervals = new Map<string, number[]>();
    
    cpuMetrics.forEach((metric) => {
      const date = new Date(metric.timestamp);
      const intervalKey = `${Math.floor(date.getMinutes())}:${Math.floor(date.getSeconds() / 5) * 5}`;
      
      if (!intervals.has(intervalKey)) {
        intervals.set(intervalKey, []);
      }
      intervals.get(intervalKey)!.push(metric.value);
    });
    
    return Array.from(intervals.entries()).map(([time, values]) => ({
      time,
      utilization: values.reduce((a, b) => a + b, 0) / values.length,
    }));
  }, [metrics]);

  // Color scale for heatmap
  const getColor = (value: number) => {
    if (value < 30) return "#4caf50"; // Green
    if (value < 60) return "#ffc107"; // Yellow
    if (value < 85) return "#ff9800"; // Orange
    return "#f44336"; // Red
  };

  return (
    <div className="chart-container">
      <h3>CPU Utilization Heatmap</h3>
      <ResponsiveContainer width="100%" height={300}>
        <BarChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" />
          <YAxis domain={[0, 100]} />
          <Tooltip />
          <Bar dataKey="utilization">
            {chartData.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={getColor(entry.utilization)} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}

/// Stacked Area Chart for Memory Breakdown
export function MemoryStackedChart({ metrics }: AdvancedChartsProps) {
  const chartData = useMemo(() => {
    const memoryMetrics = metrics.filter((m) => m.metric_type === "memory_usage");
    
    // Group by time
    const timeGroups = new Map<string, { time: string; used: number; available: number }>();
    
    memoryMetrics.forEach((metric) => {
      const date = new Date(metric.timestamp);
      const timeKey = date.toLocaleTimeString();
      
      // Assuming metric.value is percentage used
      const used = metric.value;
      const available = 100 - used;
      
      timeGroups.set(timeKey, {
        time: timeKey,
        used,
        available,
      });
    });
    
    return Array.from(timeGroups.values()).slice(-60); // Last 60 samples
  }, [metrics]);

  return (
    <div className="chart-container">
      <h3>Memory Usage Breakdown</h3>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" />
          <YAxis domain={[0, 100]} />
          <Tooltip />
          <Legend />
          <Area
            type="monotone"
            dataKey="used"
            stackId="1"
            stroke="#f44336"
            fill="#f44336"
            name="Used (%)"
          />
          <Area
            type="monotone"
            dataKey="available"
            stackId="1"
            stroke="#4caf50"
            fill="#4caf50"
            name="Available (%)"
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

/// Scatter Plot for Correlation Analysis
export function CorrelationScatterPlot({ metrics }: AdvancedChartsProps) {
  const chartData = useMemo(() => {
    const cpuMetrics = metrics.filter((m) => m.metric_type === "cpu_utilization");
    const gpuMetrics = metrics.filter((m) => m.metric_type === "gpu_utilization");
    const memoryMetrics = metrics.filter((m) => m.metric_type === "memory_usage");
    
    // Group by timestamp to find correlations
    const timeMap = new Map<string, { cpu?: number; gpu?: number; memory?: number }>();
    
    cpuMetrics.forEach((m) => {
      const key = m.timestamp;
      if (!timeMap.has(key)) {
        timeMap.set(key, {});
      }
      timeMap.get(key)!.cpu = m.value;
    });
    
    gpuMetrics.forEach((m) => {
      const key = m.timestamp;
      if (!timeMap.has(key)) {
        timeMap.set(key, {});
      }
      timeMap.get(key)!.gpu = m.value;
    });
    
    memoryMetrics.forEach((m) => {
      const key = m.timestamp;
      if (!timeMap.has(key)) {
        timeMap.set(key, {});
      }
      timeMap.get(key)!.memory = m.value;
    });
    
    // Create scatter data: CPU vs GPU
    return Array.from(timeMap.values())
      .filter((d) => d.cpu !== undefined && d.gpu !== undefined)
      .map((d) => ({
        cpu: d.cpu!,
        gpu: d.gpu!,
        memory: d.memory || 0,
      }));
  }, [metrics]);

  return (
    <div className="chart-container">
      <h3>CPU vs GPU Correlation</h3>
      <ResponsiveContainer width="100%" height={400}>
        <ScatterChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis
            type="number"
            dataKey="cpu"
            name="CPU Utilization"
            domain={[0, 100]}
            label={{ value: "CPU Utilization (%)", position: "insideBottom", offset: -5 }}
          />
          <YAxis
            type="number"
            dataKey="gpu"
            name="GPU Utilization"
            domain={[0, 100]}
            label={{ value: "GPU Utilization (%)", angle: -90, position: "insideLeft" }}
          />
          <ZAxis
            type="number"
            dataKey="memory"
            range={[50, 500]}
            name="Memory Usage"
          />
          <Tooltip cursor={{ strokeDasharray: "3 3" }} />
          <Scatter name="Samples" data={chartData} fill="#8884d8">
            {chartData.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={getScatterColor(entry.cpu, entry.gpu)} />
            ))}
          </Scatter>
        </ScatterChart>
      </ResponsiveContainer>
      <p className="chart-description">
        Each point represents a sample. Points in the top-left indicate CPU-bound workloads,
        while points in the bottom-right indicate GPU-bound workloads.
      </p>
    </div>
  );
}

/// Get color for scatter plot based on CPU/GPU relationship
function getScatterColor(cpu: number, gpu: number): string {
  const diff = cpu - gpu;
  if (diff > 20) return "#f44336"; // CPU-bound (red)
  if (diff < -20) return "#2196f3"; // GPU-bound (blue)
  return "#4caf50"; // Balanced (green)
}

/// Multi-metric Time Series Chart
export function MultiMetricChart({ metrics }: AdvancedChartsProps) {
  const chartData = useMemo(() => {
    const cpuMetrics = metrics.filter((m) => m.metric_type === "cpu_utilization");
    const gpuMetrics = metrics.filter((m) => m.metric_type === "gpu_utilization");
    const memoryMetrics = metrics.filter((m) => m.metric_type === "memory_usage");
    const tempMetrics = metrics.filter((m) => m.metric_type === "temperature");
    
    // Group by time
    const timeMap = new Map<string, {
      time: string;
      cpu?: number;
      gpu?: number;
      memory?: number;
      temperature?: number;
    }>();
    
    const addMetric = (m: MetricSample, key: string) => {
      const timeKey = new Date(m.timestamp).toLocaleTimeString();
      if (!timeMap.has(timeKey)) {
        timeMap.set(timeKey, { time: timeKey });
      }
      (timeMap.get(timeKey) as any)[key] = m.value;
    };
    
    cpuMetrics.forEach((m) => addMetric(m, "cpu"));
    gpuMetrics.forEach((m) => addMetric(m, "gpu"));
    memoryMetrics.forEach((m) => addMetric(m, "memory"));
    tempMetrics.forEach((m) => addMetric(m, "temperature"));
    
    return Array.from(timeMap.values()).slice(-60);
  }, [metrics]);

  return (
    <div className="chart-container">
      <h3>Multi-Metric Overview</h3>
      <ResponsiveContainer width="100%" height={400}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="time" />
          <YAxis yAxisId="left" domain={[0, 100]} label={{ value: "Utilization (%)", angle: -90, position: "insideLeft" }} />
          <YAxis yAxisId="right" orientation="right" label={{ value: "Temperature (°C)", angle: 90, position: "insideRight" }} />
          <Tooltip />
          <Legend />
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="cpu"
            stroke="#8884d8"
            name="CPU %"
            dot={false}
            strokeWidth={2}
          />
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="gpu"
            stroke="#82ca9d"
            name="GPU %"
            dot={false}
            strokeWidth={2}
          />
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="memory"
            stroke="#ffc658"
            name="Memory %"
            dot={false}
            strokeWidth={2}
          />
          {chartData.some((d) => d.temperature !== undefined) && (
            <Line
              yAxisId="right"
              type="monotone"
              dataKey="temperature"
              stroke="#f44336"
              name="Temperature (°C)"
              dot={false}
              strokeWidth={2}
            />
          )}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}

/// Historical Trend Chart
export function HistoricalTrendChart({ metricType }: { sessions?: any[]; metricType: string }) {
  // This would show trends across multiple sessions
  // For now, it's a placeholder structure
  return (
    <div className="chart-container">
      <h3>Historical Trend: {metricType}</h3>
      <p>Historical trend analysis across sessions would be displayed here.</p>
    </div>
  );
}

