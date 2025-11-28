//! TypeScript type definitions
//!
//! This module defines TypeScript types that mirror the Rust domain models
//! for type-safe communication between frontend and backend.

// Hardware types
export interface HardwareConfig {
  cpu: CPUInfo;
  gpus: GPUInfo[];
  memory: MemoryInfo;
  storage_devices: StorageInfo[];
  motherboard: MotherboardInfo | null;
  psu: PSUInfo | null;
  cooling: CoolingInfo | null;
  displays: DisplayInfo[];
  metadata: DetectionMetadata;
}

export interface CPUInfo {
  model: string;
  vendor: string;
  cores: number;
  threads: number;
  base_clock_mhz: number | null;
  boost_clock_mhz: number | null;
  architecture: string | null;
}

export interface GPUInfo {
  model: string;
  vendor: string;
  vram_total_mb: number | null;
  driver_version: string | null;
  pci_id: string | null;
}

export interface MemoryInfo {
  total_mb: number;
  channels: number | null;
  speed_mhz: number | null;
  modules: MemoryModule[];
}

export interface MemoryModule {
  size_mb: number;
  speed_mhz: number | null;
  manufacturer: string | null;
}

export interface StorageInfo {
  model: string;
  capacity_mb: number;
  storage_type: StorageType;
  interface: string | null;
}

export type StorageType = "ssd" | "hdd" | "nvme" | "unknown";

export interface MotherboardInfo {
  model: string;
  manufacturer: string;
  chipset: string | null;
  bios_version: string | null;
}

export interface PSUInfo {
  wattage: number;
  efficiency_rating: string | null;
}

export interface CoolingInfo {
  cpu_cooler_type: string | null;
  case_fans: number | null;
}

export interface DisplayInfo {
  name: string;
  resolution_width: number;
  resolution_height: number;
  refresh_rate_hz: number | null;
  gpu_attachment: string | null;
}

export interface DetectionMetadata {
  detection_time: string; // ISO 8601 string
  platform: string;
  warnings: string[];
  schema_version: number;
}

// Metrics types
export interface MetricSample {
  timestamp: string; // ISO 8601 string
  metric_type: MetricType;
  value: number;
  unit: string;
  source_component: string;
}

export type MetricType =
  | "cpu_utilization"
  | "cpu_utilization_per_core"
  | "gpu_utilization"
  | "gpu_vram_usage"
  | "gpu_temperature"
  | "gpu_clock"
  | "memory_usage"
  | "memory_swap_usage"
  | "storage_read_throughput"
  | "storage_write_throughput"
  | "storage_queue_depth"
  | "temperature"
  | "fan_speed"
  | "fps"
  | "frame_time"
  | "render_time";

// Analysis types
export interface BottleneckAnalysisResult {
  bottlenecks: Bottleneck[];
  timestamp: string; // ISO 8601 string
}

export interface Bottleneck {
  bottleneck_type: BottleneckType;
  severity: number; // 0-100
  evidence: EvidenceItem[];
  summary: string;
  details: string;
}

export type BottleneckType =
  | "cpu"
  | "gpu"
  | "ram"
  | "vram"
  | "storage"
  | "thermal"
  | "bandwidth";

export interface EvidenceItem {
  metric_type: MetricType;
  threshold: number;
  actual_value: number;
  time_range_start: string; // ISO 8601 string
  time_range_end: string; // ISO 8601 string
}

export interface UserFacingInsights {
  summary: string;
  recommendations: string[];
  severity: number; // 0-100
}

// Session types
export interface Session {
  id: string;
  start_time: string; // ISO 8601 string
  end_time: string | null;
  hardware_config_snapshot: HardwareConfig;
  profile: WorkloadProfile;
  runs: Run[];
}

export interface Run {
  id: string;
  name: string;
  metrics_streams: Record<string, MetricSample[]>;
  analysis_result: BottleneckAnalysisResult | null;
  notes: string | null;
}

export interface WorkloadProfile {
  id: string;
  name: string;
  workload_type: WorkloadType;
  parameters: Record<string, unknown>;
  threshold_overrides: ThresholdOverrides | null;
}

export type WorkloadType = "gaming" | "rendering" | "ai" | "productivity" | "general";

export interface ThresholdOverrides {
  cpu_high: number | null;
  gpu_high: number | null;
  ram_high: number | null;
  vram_high: number | null;
}

// Comparison types
export interface ComparisonResult {
  run1_id: string;
  run2_id: string;
  metric_deltas: Record<string, MetricDelta>;
  bottleneck_changes: BottleneckChange[];
  summary: string;
}

export interface MetricDelta {
  metric_type: string;
  run1_avg: number;
  run2_avg: number;
  delta: number;
  delta_percent: number;
  unit: string;
}

export interface BottleneckChange {
  bottleneck_type: string;
  run1_severity: number | null;
  run2_severity: number | null;
  severity_delta: number;
  status: BottleneckStatus;
}

export type BottleneckStatus = "new" | "resolved" | "improved" | "worsened" | "unchanged";

// Settings types
export interface UserSettings {
  thresholds: ThresholdSettings;
  sampling: SamplingSettings;
  units: UnitPreferences;
  theme: ThemePreferences;
  advanced: AdvancedSettings;
}

export interface ThresholdSettings {
  cpu_high: number;
  gpu_high: number;
  ram_high: number;
  vram_high: number;
  temperature_warning: number;
  temperature_critical: number;
}

export interface SamplingSettings {
  interval_ms: number;
  buffer_size: number;
}

export interface UnitPreferences {
  temperature: "celsius" | "fahrenheit";
  memory: "mb" | "gb";
}

export interface ThemePreferences {
  mode: "light" | "dark" | "system";
  high_contrast: boolean;
}

export interface AdvancedSettings {
  enable_debug_logging: boolean;
  auto_save_sessions: boolean;
  session_retention_days: number | null;
}
