//! Tauri commands for settings management
//!
//! This module exposes settings functionality to the frontend.

use crate::core::settings::{
    SamplingSettings, SettingsManager, ThresholdSettings, ThemePreferences,
    UnitPreferences, UserSettings,
};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

// Global settings manager (initialized on app start)
static SETTINGS_MANAGER: Mutex<Option<SettingsManager>> = Mutex::new(None);

/// Initialize settings manager
pub fn init_settings_manager(app_handle: AppHandle) -> Result<(), String> {
    // Use the same approach as PersistenceManager
    use tauri::path::BaseDirectory;
    let app_data_dir = app_handle
        .path()
        .resolve("", BaseDirectory::AppData)
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let settings_path = app_data_dir.join("settings.json");

    let manager = SettingsManager::new(settings_path)
        .map_err(|e| format!("Failed to initialize settings manager: {}", e))?;

    *SETTINGS_MANAGER.lock().unwrap() = Some(manager);
    Ok(())
}

/// Get current settings
#[tauri::command]
pub fn get_settings() -> Result<UserSettings, String> {
    let manager_guard = SETTINGS_MANAGER
        .lock()
        .unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Settings manager not initialized")?;

    Ok(manager.get_settings().clone())
}

/// Update settings
#[tauri::command]
pub fn update_settings(settings: UserSettings) -> Result<(), String> {
    let mut manager_guard = SETTINGS_MANAGER
        .lock()
        .unwrap();
    let manager = manager_guard
        .as_mut()
        .ok_or("Settings manager not initialized")?;

    manager
        .update_settings(settings)
        .map_err(|e| e.to_string())
}

/// Update threshold settings
#[tauri::command]
pub fn update_thresholds(thresholds: ThresholdSettings) -> Result<(), String> {
    let mut manager_guard = SETTINGS_MANAGER
        .lock()
        .unwrap();
    let manager = manager_guard
        .as_mut()
        .ok_or("Settings manager not initialized")?;

    manager
        .update_thresholds(thresholds)
        .map_err(|e| e.to_string())
}

/// Update sampling settings
#[tauri::command]
pub fn update_sampling(sampling: SamplingSettings) -> Result<(), String> {
    let mut manager_guard = SETTINGS_MANAGER
        .lock()
        .unwrap();
    let manager = manager_guard
        .as_mut()
        .ok_or("Settings manager not initialized")?;

    manager
        .update_sampling(sampling)
        .map_err(|e| e.to_string())
}

/// Update unit preferences
#[tauri::command]
pub fn update_units(units: UnitPreferences) -> Result<(), String> {
    let mut manager_guard = SETTINGS_MANAGER
        .lock()
        .unwrap();
    let manager = manager_guard
        .as_mut()
        .ok_or("Settings manager not initialized")?;

    manager
        .update_units(units)
        .map_err(|e| e.to_string())
}

/// Update theme preferences
#[tauri::command]
pub fn update_theme(theme: ThemePreferences) -> Result<(), String> {
    let mut manager_guard = SETTINGS_MANAGER
        .lock()
        .unwrap();
    let manager = manager_guard
        .as_mut()
        .ok_or("Settings manager not initialized")?;

    manager
        .update_theme(theme)
        .map_err(|e| e.to_string())
}

/// Reset to default settings
#[tauri::command]
pub fn reset_settings() -> Result<(), String> {
    let mut manager_guard = SETTINGS_MANAGER
        .lock()
        .unwrap();
    let manager = manager_guard
        .as_mut()
        .ok_or("Settings manager not initialized")?;

    manager
        .reset_to_defaults()
        .map_err(|e| e.to_string())
}

