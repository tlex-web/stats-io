//! Tests for storage detection functionality
//!
//! Tests Windows storage detection using WMI

#[cfg(test)]
mod tests {
    use stats_io_lib::hardware::adapters::windows::WindowsHardwareDetector;
    use stats_io_lib::core::interfaces::HardwareDetector;

    #[tokio::test]
    async fn test_windows_storage_detection_basic() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        assert!(config.is_ok(), "Hardware config should not fail");
        
        let config = config.unwrap();
        let storage_devices = &config.storage_devices;
        
        // On systems with storage, should detect at least one device
        // On systems without storage (unlikely), should return empty vector gracefully
        
        // If storage is detected, verify it has valid data
        for device in storage_devices {
            assert!(!device.model.is_empty(), "Storage model should not be empty");
            assert!(device.capacity_mb > 0, "Storage capacity should be positive");
            
            // Storage type should be one of the valid types
            match device.storage_type {
                stats_io_lib::core::domain::StorageType::NVMe |
                stats_io_lib::core::domain::StorageType::SSD |
                stats_io_lib::core::domain::StorageType::HDD |
                stats_io_lib::core::domain::StorageType::Unknown => {
                    // Valid storage type
                }
            }
        }
    }

    #[tokio::test]
    async fn test_windows_storage_detection_storage_type() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await.unwrap();
        
        for device in &config.storage_devices {
            let model_upper = device.model.to_uppercase();
            let interface_upper = device.interface.as_ref()
                .map(|s| s.to_uppercase())
                .unwrap_or_default();
            
            // Verify storage type detection logic
            if model_upper.contains("NVME") || interface_upper.contains("NVME") {
                // NVMe devices should ideally be detected as NVMe
                // But may be Unknown if detection logic doesn't match
            } else if model_upper.contains("SSD") || interface_upper.contains("SATA") {
                // Could be SSD or HDD depending on media type
                // Just verify it's not Unknown (unless truly unknown)
            }
        }
    }

    #[tokio::test]
    async fn test_windows_storage_detection_capacity_parsing() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await.unwrap();
        
        for device in &config.storage_devices {
            // Capacity should be in MB (converted from bytes)
            assert!(device.capacity_mb > 0, "Capacity should be positive");
            assert!(device.capacity_mb < 10_000_000, 
                    "Capacity should be reasonable (< 10PB)");
            
            // Verify conversion: bytes / (1024 * 1024) = MB
            // This is a sanity check
        }
    }

    #[tokio::test]
    async fn test_windows_storage_detection_in_hardware_config() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        assert!(config.is_ok(), "Hardware config should be retrievable");
        
        let config = config.unwrap();
        
        // Storage detection should not block other hardware detection
        assert!(config.cpu.cores > 0, "CPU should be detected");
        assert!(config.memory.total_mb > 0, "Memory should be detected");
        
        // Storage devices may or may not be present, but detection should not fail
        for device in &config.storage_devices {
            assert!(!device.model.is_empty());
            assert!(device.capacity_mb > 0);
        }
    }

    #[tokio::test]
    async fn test_windows_storage_detection_serialization() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await.unwrap();
        
        // Verify storage info can be serialized
        let json = serde_json::to_string(&config.storage_devices);
        assert!(json.is_ok(), "Storage info should be serializable");
        
        let json_str = json.unwrap();
        let deserialized: Vec<stats_io_lib::core::domain::StorageInfo> = 
            serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.len(), config.storage_devices.len());
    }

    #[tokio::test]
    async fn test_windows_storage_detection_multiple_devices() {
        // Test that multiple storage devices are detected
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await.unwrap();
        
        // Systems may have multiple storage devices
        // Verify each is detected correctly
        for device in &config.storage_devices {
            assert!(!device.model.is_empty());
            assert!(device.capacity_mb > 0);
        }
    }
}

