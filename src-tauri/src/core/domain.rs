//! Core domain models
//!
//! This module defines all core domain models as specified in AGENT.md Section 7.1.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Hardware configuration containing all detected hardware components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub cpu: CPUInfo,
    pub gpus: Vec<GPUInfo>,
    pub memory: MemoryInfo,
    pub storage_devices: Vec<StorageInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub psu: Option<PSUInfo>,
    pub cooling: Option<CoolingInfo>,
    pub displays: Vec<DisplayInfo>,
    pub metadata: DetectionMetadata,
}

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUInfo {
    pub model: String,
    pub vendor: String,
    pub cores: u32,
    pub threads: u32,
    pub base_clock_mhz: Option<f64>,
    pub boost_clock_mhz: Option<f64>,
    pub architecture: Option<String>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUInfo {
    pub model: String,
    pub vendor: String,
    pub vram_total_mb: Option<u64>,
    pub driver_version: Option<String>,
    pub pci_id: Option<String>,
}

/// Memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub channels: Option<u32>,
    pub speed_mhz: Option<u64>,
    pub modules: Vec<MemoryModule>,
}

/// Memory module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryModule {
    pub size_mb: u64,
    pub speed_mhz: Option<u64>,
    pub manufacturer: Option<String>,
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub model: String,
    pub capacity_mb: u64,
    pub storage_type: StorageType,
    pub interface: Option<String>,
}

/// Storage device type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    SSD,
    HDD,
    NVMe,
    Unknown,
}

/// Motherboard information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub model: String,
    pub manufacturer: String,
    pub chipset: Option<String>,
    pub bios_version: Option<String>,
}

/// PSU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSUInfo {
    pub wattage: u32,
    pub efficiency_rating: Option<String>,
}

/// Cooling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoolingInfo {
    pub cpu_cooler_type: Option<String>,
    pub case_fans: Option<u32>,
}

/// Display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub name: String,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub refresh_rate_hz: Option<u32>,
    pub gpu_attachment: Option<String>,
}

/// Detection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionMetadata {
    pub detection_time: DateTime<Utc>,
    pub platform: String,
    pub warnings: Vec<String>,
    pub schema_version: u32,
}

/// Workload profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProfile {
    pub id: String,
    pub name: String,
    pub workload_type: WorkloadType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub threshold_overrides: Option<ThresholdOverrides>,
}

/// Workload type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkloadType {
    Gaming,
    Rendering,
    AI,
    Productivity,
    General,
}

/// Threshold overrides for workload profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdOverrides {
    pub cpu_high: Option<f64>,
    pub gpu_high: Option<f64>,
    pub ram_high: Option<f64>,
    pub vram_high: Option<f64>,
}

/// Session containing multiple runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub hardware_config_snapshot: HardwareConfig,
    pub profile: WorkloadProfile,
    pub runs: Vec<Run>,
}

/// A single measurement run within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub name: String,
    pub metrics_streams: HashMap<String, Vec<MetricSample>>,
    pub analysis_result: Option<BottleneckAnalysisResult>,
    pub notes: Option<String>,
}

/// Metric sample with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub timestamp: DateTime<Utc>,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub source_component: String,
}

/// Metric type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    CpuUtilization,
    CpuUtilizationPerCore,
    GpuUtilization,
    GpuVramUsage,
    GpuTemperature,
    GpuClock,
    MemoryUsage,
    MemorySwapUsage,
    StorageReadThroughput,
    StorageWriteThroughput,
    StorageQueueDepth,
    MemoryReadThroughput,
    MemoryWriteThroughput,
    GpuMemoryTransfer,
    Temperature,
    FanSpeed,
    Fps,
    FrameTime,
    RenderTime,
}

/// Bottleneck analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckAnalysisResult {
    pub bottlenecks: Vec<Bottleneck>,
    pub timestamp: DateTime<Utc>,
}

/// Detected bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub bottleneck_type: BottleneckType,
    pub severity: u8, // 0-100
    pub evidence: Vec<EvidenceItem>,
    pub summary: String,
    pub details: String,
}

/// Bottleneck type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BottleneckType {
    Cpu,
    Gpu,
    Ram,
    Vram,
    Storage,
    Thermal,
    Bandwidth,
}

/// Evidence item for bottleneck diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub metric_type: MetricType,
    pub threshold: f64,
    pub actual_value: f64,
    pub time_range_start: DateTime<Utc>,
    pub time_range_end: DateTime<Utc>,
}

