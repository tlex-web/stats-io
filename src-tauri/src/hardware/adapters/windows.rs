//! Windows-specific hardware adapter
//!
//! This module implements HardwareDetector for Windows using sysinfo and Windows APIs.

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

/// Windows hardware detector implementation
pub struct WindowsHardwareDetector {
    system: Arc<Mutex<System>>,
}

impl WindowsHardwareDetector {
    /// Create a new Windows hardware detector
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
        } else if model.contains("AMD") {
            "AMD".to_string()
        } else {
            "Unknown".to_string()
        };
        
        // Try to get frequency (may not be available on all systems)
        let frequency = first_cpu.frequency();
        let base_clock = if frequency > 0 {
            Some(frequency as f64)
        } else {
            None
        };
        
        // Threads = logical cores, cores = physical cores (approximation)
        let threads = cpu_count as u32;
        // On Windows, we can't easily get physical core count from sysinfo alone
        // Use threads as approximation, or assume 1:1 if hyperthreading not detected
        let cores = threads; // Conservative estimate
        
        Ok(CPUInfo {
            model,
            vendor,
            cores,
            threads,
            base_clock_mhz: base_clock,
            boost_clock_mhz: None, // sysinfo doesn't provide boost clock
            architecture: Some("x86_64".to_string()), // Windows typically x86_64
        })
    }
    
    /// Detect GPU information using WMI
    async fn detect_gpus(&self) -> Result<Vec<GPUInfo>, HardwareError> {
        let mut gpus = Vec::new();
        
        // Use wmic command to query Win32_VideoController
        // This is simpler than COM WMI for MVP and works reliably
        let output = tokio::process::Command::new("wmic")
            .args(&[
                "path",
                "win32_VideoController",
                "get",
                "Name,AdapterRAM,DriverVersion,PNPDeviceID",
                "/format:csv",
            ])
            .output()
            .await
            .map_err(|e| HardwareError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to execute wmic: {}", e)
            )))?;
        
        if !output.status.success() {
            // If wmic fails, return empty vector (graceful degradation)
            return Ok(gpus);
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse CSV output from wmic
        // Format: Node,Name,AdapterRAM,DriverVersion,PNPDeviceID
        let lines: Vec<&str> = output_str.lines().collect();
        
        for line in lines.iter().skip(1) { // Skip header
            if line.trim().is_empty() || line.starts_with("Node,") {
                continue;
            }
            
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 5 {
                continue;
            }
            
            let name = fields.get(1).unwrap_or(&"").trim();
            if name.is_empty() || name == "Name" {
                continue;
            }
            
            // Parse AdapterRAM (in bytes, convert to MB)
            let vram_mb = fields.get(2)
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(|bytes| bytes / (1024 * 1024));
            
            let driver_version = fields.get(3)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            
            let pci_id = fields.get(4)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            
            // Determine vendor from name
            let vendor = if name.to_uppercase().contains("NVIDIA") {
                "NVIDIA".to_string()
            } else if name.to_uppercase().contains("AMD") || name.to_uppercase().contains("RADEON") {
                "AMD".to_string()
            } else if name.to_uppercase().contains("INTEL") {
                "Intel".to_string()
            } else {
                "Unknown".to_string()
            };
            
            gpus.push(GPUInfo {
                model: name.to_string(),
                vendor,
                vram_total_mb: vram_mb,
                driver_version,
                pci_id,
            });
        }
        
        Ok(gpus)
    }
    
    /// Detect memory information
    async fn detect_memory(&self) -> Result<MemoryInfo, HardwareError> {
        let system = self.system.lock().await;
        
        let total_mb = system.total_memory() / (1024 * 1024); // sysinfo returns bytes, convert to MB
        
        // sysinfo doesn't provide detailed memory module information
        // For MVP, we'll use basic info
        let modules = vec![MemoryModule {
            size_mb: total_mb,
            speed_mhz: None,
            manufacturer: None,
        }];
        
        Ok(MemoryInfo {
            total_mb,
            channels: None, // Not available from sysinfo
            speed_mhz: None, // Not available from sysinfo
            modules,
        })
    }
    
    /// Detect storage devices using WMI
    async fn detect_storage(&self) -> Result<Vec<StorageInfo>, HardwareError> {
        let mut storage_devices = Vec::new();
        
        // Use wmic to query Win32_DiskDrive for physical disks
        let output = tokio::process::Command::new("wmic")
            .args(&[
                "path",
                "win32_DiskDrive",
                "get",
                "Model,Size,InterfaceType,MediaType",
                "/format:csv",
            ])
            .output()
            .await
            .map_err(|e| HardwareError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to execute wmic: {}", e)
            )))?;
        
        if !output.status.success() {
            // If wmic fails, try to use sysinfo as fallback
            return Self::detect_storage_sysinfo().await;
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        for line in lines.iter().skip(1) { // Skip header
            if line.trim().is_empty() || line.starts_with("Node,") {
                continue;
            }
            
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 5 {
                continue;
            }
            
            let model = fields.get(1).unwrap_or(&"").trim();
            if model.is_empty() || model == "Model" {
                continue;
            }
            
            // Parse Size (in bytes, convert to MB)
            let capacity_mb = fields.get(2)
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(|bytes| bytes / (1024 * 1024))
                .unwrap_or(0);
            
            let interface = fields.get(3).map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            
            let media_type = fields.get(4).map(|s| s.trim().to_lowercase())
                .unwrap_or_default();
            
            // Determine storage type from interface and media type
            let storage_type = if model.to_uppercase().contains("NVME") || 
                                 interface.as_ref().map(|s| s.to_uppercase().contains("NVME")).unwrap_or(false) {
                StorageType::NVMe
            } else if media_type.contains("ssd") || 
                      model.to_uppercase().contains("SSD") ||
                      interface.as_ref().map(|s| s.to_uppercase().contains("SATA")).unwrap_or(false) {
                StorageType::SSD
            } else if media_type.contains("hdd") || 
                      model.to_uppercase().contains("HDD") ||
                      media_type.contains("fixed") {
                StorageType::HDD
            } else {
                StorageType::Unknown
            };
            
            storage_devices.push(StorageInfo {
                model: model.to_string(),
                capacity_mb,
                storage_type,
                interface,
            });
        }
        
        // If no devices found, try sysinfo fallback
        if storage_devices.is_empty() {
            return Self::detect_storage_sysinfo().await;
        }
        
        Ok(storage_devices)
    }
    
    /// Fallback storage detection using sysinfo
    async fn detect_storage_sysinfo() -> Result<Vec<StorageInfo>, HardwareError> {
        // sysinfo 0.30 on Windows doesn't provide disks() method directly
        // Use Disks API instead if needed, but for now return empty
        // WMI should work in most cases
        Ok(Vec::new())
    }
    
    /// Detect motherboard information
    async fn detect_motherboard(&self) -> Result<Option<MotherboardInfo>, HardwareError> {
        // sysinfo doesn't provide motherboard information
        // For MVP, return None - can be enhanced with WMI in later phases
        Ok(None)
    }
    
    /// Detect PSU information
    async fn detect_psu(&self) -> Result<Option<PSUInfo>, HardwareError> {
        // PSU information is typically not detectable via standard APIs
        // Return None for MVP - user can enter manually if needed
        Ok(None)
    }
    
    /// Detect cooling information
    async fn detect_cooling(&self) -> Result<Option<CoolingInfo>, HardwareError> {
        // Cooling information requires specialized sensors
        // Return None for MVP - can be enhanced in later phases
        Ok(None)
    }
    
    /// Detect display information
    async fn detect_displays(&self) -> Result<Vec<DisplayInfo>, HardwareError> {
        // sysinfo doesn't provide display information
        // For MVP, return empty vector - can be enhanced with Windows APIs
        Ok(Vec::new())
    }
}

#[async_trait]
impl HardwareDetector for WindowsHardwareDetector {
    async fn get_hardware_config(&self) -> Result<HardwareConfig, HardwareError> {
        // Refresh system information
        {
            let mut system = self.system.lock().await;
            system.refresh_all();
        }
        
        // Detect all components
        let cpu = self.detect_cpu().await?;
        let gpus = self.detect_gpus().await?;
        let memory = self.detect_memory().await?;
        let storage_devices = self.detect_storage().await?;
        let motherboard = self.detect_motherboard().await?;
        let psu = self.detect_psu().await?;
        let cooling = self.detect_cooling().await?;
        let displays = self.detect_displays().await?;
        
        let metadata = DetectionMetadata {
            detection_time: chrono::Utc::now(),
            platform: "windows".to_string(),
            warnings: Vec::new(), // Can be populated with detection warnings
            schema_version: 1,
        };
        
        Ok(HardwareConfig {
            cpu,
            gpus,
            memory,
            storage_devices,
            motherboard,
            psu,
            cooling,
            displays,
            metadata,
        })
    }
    
    async fn refresh(&self) -> Result<HardwareConfig, HardwareError> {
        // Force refresh by creating new system instance
        {
            let mut system = self.system.lock().await;
            *system = System::new_all();
            system.refresh_all();
        }
        
        self.get_hardware_config().await
    }
}

impl Default for WindowsHardwareDetector {
    fn default() -> Self {
        Self::new()
    }
}
