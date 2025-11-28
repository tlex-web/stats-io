//! Unit tests for hardware detection across all platforms
//!
//! Tests hardware detection functionality following TESTING_PLAN.md.

#[cfg(test)]
mod tests {
    use stats_io_lib::hardware::adapters::windows::WindowsHardwareDetector;
    use stats_io_lib::core::interfaces::HardwareDetector;

    #[tokio::test]
    async fn test_windows_hardware_detection() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        assert!(config.is_ok());
        let config = config.unwrap();
        
        // Verify CPU detection
        assert!(!config.cpu.model.is_empty());
        assert!(config.cpu.cores > 0);
        assert!(config.cpu.threads > 0);
        
        // Verify memory detection
        assert!(config.memory.total_mb > 0);
        
        // Verify metadata
        assert_eq!(config.metadata.platform, "windows");
        assert_eq!(config.metadata.schema_version, 1);
    }

    #[tokio::test]
    async fn test_windows_hardware_refresh() {
        let detector = WindowsHardwareDetector::new();
        let config1 = detector.get_hardware_config().await.unwrap();
        
        // Refresh and verify we get a valid config
        let config2 = detector.refresh().await.unwrap();
        
        // Both should be valid
        assert!(!config1.cpu.model.is_empty());
        assert!(!config2.cpu.model.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_linux_hardware_detection() {
        use stats_io_lib::hardware::adapters::linux::LinuxHardwareDetector;
        
        let detector = LinuxHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        assert!(config.is_ok());
        let config = config.unwrap();
        
        // Verify CPU detection
        assert!(!config.cpu.model.is_empty());
        assert!(config.cpu.cores > 0);
        
        // Verify memory detection
        assert!(config.memory.total_mb > 0);
        
        // Verify metadata
        assert_eq!(config.metadata.platform, "Linux");
    }

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn test_macos_hardware_detection() {
        use stats_io_lib::hardware::adapters::macos::MacOSHardwareDetector;
        
        let detector = MacOSHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        assert!(config.is_ok());
        let config = config.unwrap();
        
        // Verify CPU detection
        assert!(!config.cpu.model.is_empty());
        assert!(config.cpu.cores > 0);
        
        // Verify memory detection
        assert!(config.memory.total_mb > 0);
        
        // Verify metadata
        assert_eq!(config.metadata.platform, "macOS");
    }
}

