//! macOS-specific hardware adapter
//!
//! This module implements HardwareDetector for macOS using sysinfo, IOKit,
//! and System Profiler.

use crate::core::domain::{
    CoolingInfo, CPUInfo, DetectionMetadata, DisplayInfo, GPUInfo, HardwareConfig, MemoryInfo,
    MemoryModule, MotherboardInfo, PSUInfo, StorageInfo, StorageType,
};
use crate::core::error::HardwareError;
use crate::core::interfaces::HardwareDetector;
use async_trait::async_trait;
use sysinfo::System;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;

/// macOS hardware detector implementation
pub struct MacOSHardwareDetector {
    system: Arc<Mutex<System>>,
}

impl MacOSHardwareDetector {
    /// Create a new macOS hardware detector
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system: Arc::new(Mutex::new(system)),
        }
    }
    
    /// Detect CPU information
    async fn detect_cpu(&self) -> Result<CPUInfo, HardwareError> {
        let system = self.system.lock().await;
        
        // Get CPU information from sysinfo
        let cpu_count = system.cpus().len();
        let first_cpu = system.cpus().first()
            .ok_or_else(|| HardwareError::DetectionFailed("No CPU detected".to_string()))?;
        
        let model = first_cpu.name().to_string();
        let vendor = if model.contains("Intel") {
            "Intel".to_string()
        } else if model.contains("Apple") || model.contains("M1") || model.contains("M2") || model.contains("M3") {
            "Apple Silicon".to_string()
        } else {
            "Unknown".to_string()
        };
        
        // Try to get frequency
        let frequency = first_cpu.frequency();
        let base_clock_mhz = if frequency > 0 {
            Some(frequency as f64)
        } else {
            None
        };
        
        // Detect architecture
        let architecture = Self::detect_architecture().await;
        
        // Count cores
        let cores = cpu_count as u32;
        let threads = cpu_count as u32;
        
        Ok(CPUInfo {
            model,
            vendor,
            architecture: Some(architecture),
            cores,
            threads,
            base_clock_mhz,
            boost_clock_mhz: None, // Would require additional parsing
        })
    }
    
    /// Detect memory information
    async fn detect_memory(&self) -> Result<MemoryInfo, HardwareError> {
        let system = self.system.lock().await;
        
        let total_mb = (system.total_memory() / 1024 / 1024) as u64;
        
        // macOS memory detection would use:
        // - system_profiler SPHardwareDataType
        // - IOKit for detailed memory info
        
        // Try to get memory info from system_profiler
        let (channels, speed_mhz) = Self::get_memory_details().await;
        
        // Memory modules would require system_profiler SPMemoryDataType
        let modules = Vec::new();
        
        Ok(MemoryInfo {
            total_mb,
            channels,
            speed_mhz,
            modules,
        })
    }
    
    /// Detect storage devices
    async fn detect_storage(&self) -> Result<Vec<StorageInfo>, HardwareError> {
        let system = self.system.lock().await;
        let mut storage_devices = Vec::new();
        
        system.refresh_disks();
        for disk in system.disks() {
            let name = disk.name().to_string_lossy().to_string();
            let total_gb = (disk.total_space() / 1024 / 1024 / 1024) as u64;
            
            // Determine storage type
            let storage_type = if name.contains("SSD") || name.contains("NVMe") {
                StorageType::Nvme
            } else {
                StorageType::Sata
            };
            
            storage_devices.push(StorageInfo {
                model: name.clone(),
                capacity_mb: total_gb * 1024, // Convert GB to MB
                storage_type,
                interface: None,
            });
        }
        
        Ok(storage_devices)
    }
    
    /// Detect GPU information
    async fn detect_gpus(&self) -> Result<Vec<GPUInfo>, HardwareError> {
        let mut gpus = Vec::new();
        
        // macOS GPU detection would use:
        // - system_profiler SPDisplaysDataType
        // - IOKit for GPU details
        
        // Try to detect GPUs
        // On Apple Silicon, GPU is integrated
        // On Intel Macs, could have integrated or discrete
        
        let system_info = Self::get_system_info().await;
        
        // Check for Apple Silicon (unified memory architecture)
        if system_info.contains("Apple") || system_info.contains("M1") || 
           system_info.contains("M2") || system_info.contains("M3") {
            gpus.push(GPUInfo {
                model: "Apple Integrated GPU".to_string(),
                vendor: "Apple".to_string(),
                vram_total_mb: None, // Unified memory on Apple Silicon
                driver_version: None,
                pci_id: None,
            });
        } else {
            // Intel Mac - could have Intel integrated or AMD discrete
            // Placeholder detection
            gpus.push(GPUInfo {
                model: "GPU (detected)".to_string(),
                vendor: "Unknown".to_string(),
                vram_total_mb: None,
                driver_version: None,
                pci_id: None,
            });
        }
        
        Ok(gpus)
    }
    
    /// Detect display information
    async fn detect_displays(&self) -> Result<Vec<DisplayInfo>, HardwareError> {
        // macOS display detection would use:
        // - system_profiler SPDisplaysDataType
        // - CoreGraphics APIs
        
        // Placeholder implementation
        Ok(vec![DisplayInfo {
            name: "Display".to_string(),
            resolution: None,
            refresh_rate_hz: None,
            is_primary: true,
        }])
    }
    
    /// Detect architecture
    async fn detect_architecture() -> String {
        std::env::consts::ARCH.to_string()
    }
    
    /// Get memory details from system_profiler
    async fn get_memory_details() -> (Option<u32>, Option<u32>) {
        // Would execute: system_profiler SPHardwareDataType
        // Parse memory speed and channels
        // For now, return None
        (None, None)
    }
    
    /// Get system info string
    async fn get_system_info() -> String {
        // Would execute: system_profiler SPHardwareDataType | grep "Model"
        // For now, return a placeholder
        std::env::consts::ARCH.to_string()
    }
}

#[async_trait]
impl HardwareDetector for MacOSHardwareDetector {
    async fn get_hardware_config(&self) -> Result<HardwareConfig, HardwareError> {
        let cpu_info = self.detect_cpu().await?;
        let memory_info = self.detect_memory().await?;
        let storage_devices = self.detect_storage().await?;
        let gpus = self.detect_gpus().await?;
        let displays = self.detect_displays().await?;
        
        // Collect warnings for unavailable features
        let mut warnings = Vec::new();
        if gpus.iter().any(|g| g.vram_total_mb.is_none()) {
            warnings.push("GPU VRAM detection requires system_profiler or IOKit".to_string());
        }
        if memory_info.channels.is_none() || memory_info.speed_mhz.is_none() {
            warnings.push("Memory channel and speed detection requires system_profiler".to_string());
        }
        
        Ok(HardwareConfig {
            cpu: cpu_info,
            gpus,
            memory: memory_info,
            storage_devices,
            motherboard: None, // Would require system_profiler
            psu: None,          // Not applicable for Macs
            cooling: None,      // Would require IOKit or sensors
            displays,
            metadata: DetectionMetadata {
                detection_time: Utc::now(),
                platform: "macOS".to_string(),
                warnings,
                schema_version: 1,
            },
        })
    }
    
    async fn refresh(&self) -> Result<HardwareConfig, HardwareError> {
        {
            let mut system = self.system.lock().await;
            system.refresh_all();
        }
        self.get_hardware_config().await
    }
}
