//! Hardware detection subsystem
//!
//! This module provides hardware detection functionality following AGENT.md Section 6.2.

pub mod hal;
pub mod adapters;

use crate::core::domain::HardwareConfig;
use crate::core::error::HardwareError;
use crate::core::interfaces::HardwareDetector;
use std::sync::Arc;
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use adapters::windows::WindowsHardwareDetector;

/// Global hardware detector instance (lazy-initialized)
static HARDWARE_DETECTOR: OnceLock<Arc<dyn HardwareDetector>> = OnceLock::new();

/// Get or create the hardware detector for the current platform
pub fn get_hardware_detector() -> Arc<dyn HardwareDetector> {
    HARDWARE_DETECTOR.get_or_init(|| {
        #[cfg(target_os = "windows")]
        {
            Arc::new(WindowsHardwareDetector::new()) as Arc<dyn HardwareDetector>
        }
        #[cfg(target_os = "linux")]
        {
            use adapters::linux::LinuxHardwareDetector;
            Arc::new(LinuxHardwareDetector::new()) as Arc<dyn HardwareDetector>
        }
        #[cfg(target_os = "macos")]
        {
            use adapters::macos::MacOSHardwareDetector;
            Arc::new(MacOSHardwareDetector::new()) as Arc<dyn HardwareDetector>
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            panic!("Unsupported platform")
        }
    })
    .clone()
}

/// Get hardware configuration (cached)
pub async fn get_hardware_config() -> Result<HardwareConfig, HardwareError> {
    let detector = get_hardware_detector();
    detector.get_hardware_config().await
}

/// Refresh hardware configuration (force new detection)
pub async fn refresh_hardware_config() -> Result<HardwareConfig, HardwareError> {
    let detector = get_hardware_detector();
    detector.refresh().await
}
