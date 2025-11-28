//! Integration tests for settings management
//!
//! Tests settings persistence and retrieval following IMPLEMENTATION_PLAN.md Phase 3.6.

#[cfg(test)]
mod tests {
    use stats_io_lib::core::settings::{
        SettingsManager, ThresholdSettings, UserSettings,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_temp_settings_manager() -> (SettingsManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let manager = SettingsManager::new(settings_path).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_settings_defaults() {
        let (manager, _temp_dir) = create_temp_settings_manager();
        let settings = manager.get_settings();

        assert_eq!(settings.thresholds.cpu_high, 85.0);
        assert_eq!(settings.thresholds.gpu_high, 90.0);
        assert_eq!(settings.sampling.interval_ms, 1000);
    }

    #[test]
    fn test_update_thresholds() {
        let (mut manager, _temp_dir) = create_temp_settings_manager();
        
        let new_thresholds = ThresholdSettings {
            cpu_high: 90.0,
            gpu_high: 95.0,
            ram_high: 85.0,
            vram_high: 85.0,
            temperature_warning: 75.0,
            temperature_critical: 85.0,
        };

        manager.update_thresholds(new_thresholds.clone()).unwrap();
        
        let settings = manager.get_settings();
        assert_eq!(settings.thresholds.cpu_high, 90.0);
        assert_eq!(settings.thresholds.gpu_high, 95.0);
    }

    #[test]
    fn test_settings_persistence() {
        let (temp_dir, settings_path) = {
            let temp_dir = TempDir::new().unwrap();
            let settings_path = temp_dir.path().join("settings.json");
            let mut manager = SettingsManager::new(settings_path.clone()).unwrap();
            
            let mut settings = UserSettings::default();
            settings.thresholds.cpu_high = 95.0;
            manager.update_settings(settings).unwrap();
            
            (temp_dir, settings_path)
        };

        // Create a new manager with the same path
        let manager2 = SettingsManager::new(settings_path).unwrap();
        let settings = manager2.get_settings();
        
        assert_eq!(settings.thresholds.cpu_high, 95.0);
    }

    #[test]
    fn test_reset_to_defaults() {
        let (mut manager, _temp_dir) = create_temp_settings_manager();
        
        // Change some settings
        let mut settings = UserSettings::default();
        settings.thresholds.cpu_high = 99.0;
        manager.update_settings(settings).unwrap();
        
        // Reset
        manager.reset_to_defaults().unwrap();
        
        let settings = manager.get_settings();
        assert_eq!(settings.thresholds.cpu_high, 85.0); // Back to default
    }
}

