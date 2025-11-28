//! User settings and preferences
//!
//! This module provides functionality for managing user settings and preferences
//! following AGENT.md Section 3.5 and IMPLEMENTATION_PLAN.md Phase 3.3.

use serde::{Deserialize, Serialize};

/// User settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub thresholds: ThresholdSettings,
    pub sampling: SamplingSettings,
    pub units: UnitPreferences,
    pub theme: ThemePreferences,
    pub advanced: AdvancedSettings,
}

/// Threshold settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSettings {
    pub cpu_high: f64,
    pub gpu_high: f64,
    pub ram_high: f64,
    pub vram_high: f64,
    pub temperature_warning: f64,
    pub temperature_critical: f64,
}

/// Sampling settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingSettings {
    pub interval_ms: u64,
    pub buffer_size: usize,
}

/// Unit preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitPreferences {
    pub temperature: TemperatureUnit,
    pub memory: MemoryUnit,
}

/// Temperature unit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

/// Memory unit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryUnit {
    MB,
    GB,
}

/// Theme preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePreferences {
    pub mode: ThemeMode,
    pub high_contrast: bool,
}

/// Theme mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

/// Advanced settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub enable_debug_logging: bool,
    pub auto_save_sessions: bool,
    pub session_retention_days: Option<u32>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            thresholds: ThresholdSettings::default(),
            sampling: SamplingSettings::default(),
            units: UnitPreferences::default(),
            theme: ThemePreferences::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

impl Default for ThresholdSettings {
    fn default() -> Self {
        Self {
            cpu_high: 85.0,
            gpu_high: 90.0,
            ram_high: 90.0,
            vram_high: 90.0,
            temperature_warning: 80.0,
            temperature_critical: 90.0,
        }
    }
}

impl Default for SamplingSettings {
    fn default() -> Self {
        Self {
            interval_ms: 1000, // 1 second
            buffer_size: 3600,  // 1 hour at 1 sample/second
        }
    }
}

impl Default for UnitPreferences {
    fn default() -> Self {
        Self {
            temperature: TemperatureUnit::Celsius,
            memory: MemoryUnit::GB,
        }
    }
}

impl Default for ThemePreferences {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            high_contrast: false,
        }
    }
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            enable_debug_logging: false,
            auto_save_sessions: true,
            session_retention_days: Some(30),
        }
    }
}

/// Settings manager
pub struct SettingsManager {
    settings: UserSettings,
    settings_path: std::path::PathBuf,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(settings_path: std::path::PathBuf) -> Result<Self, SettingsError> {
        let settings = if settings_path.exists() {
            Self::load_from_file(&settings_path)?
        } else {
            UserSettings::default()
        };

        Ok(Self {
            settings,
            settings_path,
        })
    }

    /// Get current settings
    pub fn get_settings(&self) -> &UserSettings {
        &self.settings
    }

    /// Update settings
    pub fn update_settings(&mut self, new_settings: UserSettings) -> Result<(), SettingsError> {
        self.settings = new_settings;
        self.save()?;
        Ok(())
    }

    /// Update threshold settings
    pub fn update_thresholds(&mut self, thresholds: ThresholdSettings) -> Result<(), SettingsError> {
        self.settings.thresholds = thresholds;
        self.save()?;
        Ok(())
    }

    /// Update sampling settings
    pub fn update_sampling(&mut self, sampling: SamplingSettings) -> Result<(), SettingsError> {
        self.settings.sampling = sampling;
        self.save()?;
        Ok(())
    }

    /// Update unit preferences
    pub fn update_units(&mut self, units: UnitPreferences) -> Result<(), SettingsError> {
        self.settings.units = units;
        self.save()?;
        Ok(())
    }

    /// Update theme preferences
    pub fn update_theme(&mut self, theme: ThemePreferences) -> Result<(), SettingsError> {
        self.settings.theme = theme;
        self.save()?;
        Ok(())
    }

    /// Reset to default settings
    pub fn reset_to_defaults(&mut self) -> Result<(), SettingsError> {
        self.settings = UserSettings::default();
        self.save()?;
        Ok(())
    }

    /// Save settings to file
    fn save(&self) -> Result<(), SettingsError> {
        if let Some(parent) = self.settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SettingsError::IoError(format!("Failed to create settings directory: {}", e)))?;
        }

        let json = serde_json::to_string_pretty(&self.settings)
            .map_err(|e| SettingsError::SerializationError(e.to_string()))?;

        std::fs::write(&self.settings_path, json)
            .map_err(|e| SettingsError::IoError(format!("Failed to write settings file: {}", e)))?;

        Ok(())
    }

    /// Load settings from file
    fn load_from_file(path: &std::path::Path) -> Result<UserSettings, SettingsError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SettingsError::IoError(format!("Failed to read settings file: {}", e)))?;

        let settings: UserSettings = serde_json::from_str(&content)
            .map_err(|e| SettingsError::DeserializationError(e.to_string()))?;

        Ok(settings)
    }
}

/// Settings error
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

