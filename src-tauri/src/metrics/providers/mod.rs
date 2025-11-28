//! Metrics providers
//!
//! This module contains implementations of metrics providers for different components.

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod storage;

pub use cpu::SysInfoCpuMetricsProvider;
pub use gpu::{GpuMetricsProviderImpl, PlaceholderGpuMetricsProvider};
pub use memory::SysInfoMemoryMetricsProvider;
pub use storage::SysInfoStorageMetricsProvider;
