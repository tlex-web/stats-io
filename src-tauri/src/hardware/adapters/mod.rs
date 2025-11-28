//! Platform-specific hardware adapters
//!
//! This module contains platform-specific implementations of HardwareDetector
//! for Windows, Linux, and macOS.

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

