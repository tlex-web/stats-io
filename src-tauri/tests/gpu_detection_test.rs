//! Tests for GPU detection functionality
//!
//! Tests Windows GPU detection using WMI

#[cfg(test)]
mod tests {
    use stats_io_lib::hardware::adapters::windows::WindowsHardwareDetector;
    use stats_io_lib::core::interfaces::HardwareDetector;

    #[tokio::test]
    async fn test_windows_gpu_detection_basic() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        // Should not panic
        assert!(config.is_ok());
        
        let config = config.unwrap();
        let gpus = &config.gpus;
        
        // On systems with GPUs, should detect at least one
        // On systems without GPUs, should return empty vector gracefully
        // Either case is valid - we just verify it doesn't crash
        
        // If GPUs are detected, verify they have valid data
        for gpu in gpus {
            assert!(!gpu.model.is_empty(), "GPU model should not be empty");
            assert!(!gpu.vendor.is_empty(), "GPU vendor should not be empty");
            
            // VRAM, driver version, and PCI ID are optional
            // But if present, should be valid
            if let Some(vram) = gpu.vram_total_mb {
                assert!(vram > 0, "VRAM should be positive if present");
                assert!(vram < 100_000_000, "VRAM should be reasonable (< 100TB)"); // Sanity check
            }
        }
    }

    #[tokio::test]
    async fn test_windows_gpu_detection_vendor_detection() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await.unwrap();
        
        for gpu in &config.gpus {
            let vendor_upper = gpu.vendor.to_uppercase();
            let model_upper = gpu.model.to_uppercase();
            
            // Verify vendor detection logic
            if model_upper.contains("NVIDIA") {
                assert_eq!(gpu.vendor, "NVIDIA", "NVIDIA GPU should have NVIDIA vendor");
            } else if model_upper.contains("AMD") || model_upper.contains("RADEON") {
                assert_eq!(gpu.vendor, "AMD", "AMD GPU should have AMD vendor");
            } else if model_upper.contains("INTEL") {
                assert_eq!(gpu.vendor, "Intel", "Intel GPU should have Intel vendor");
            }
            // Unknown vendors are also acceptable
        }
    }

    #[tokio::test]
    async fn test_windows_gpu_detection_in_hardware_config() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await;
        
        assert!(config.is_ok(), "Hardware config should be retrievable");
        
        let config = config.unwrap();
        
        // GPU detection should not block other hardware detection
        assert!(config.cpu.cores > 0, "CPU should be detected");
        assert!(config.memory.total_mb > 0, "Memory should be detected");
        
        // GPUs may or may not be present, but detection should not fail
        // If GPUs are present, they should be in the config
        for gpu in &config.gpus {
            assert!(!gpu.model.is_empty());
        }
    }

    #[tokio::test]
    async fn test_windows_gpu_detection_serialization() {
        let detector = WindowsHardwareDetector::new();
        let config = detector.get_hardware_config().await.unwrap();
        
        // Verify GPU info can be serialized (required for Tauri)
        let json = serde_json::to_string(&config.gpus);
        assert!(json.is_ok(), "GPU info should be serializable");
        
        let json_str = json.unwrap();
        assert!(!json_str.is_empty());
        
        // Verify it can be deserialized
        let deserialized: Vec<stats_io_lib::core::domain::GPUInfo> = 
            serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.len(), config.gpus.len());
    }
}

