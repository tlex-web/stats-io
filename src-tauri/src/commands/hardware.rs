//! Tauri commands for hardware detection
//!
//! This module exposes hardware detection functionality to the frontend.

use crate::core::domain::HardwareConfig;
use crate::hardware;

/// Get the current hardware configuration
#[tauri::command]
pub async fn get_hardware_config() -> Result<HardwareConfig, String> {
    hardware::get_hardware_config()
        .await
        .map_err(|e| e.to_string())
}

/// Refresh hardware configuration (force new detection)
#[tauri::command]
pub async fn refresh_hardware_config() -> Result<HardwareConfig, String> {
    hardware::refresh_hardware_config()
        .await
        .map_err(|e| e.to_string())
}
