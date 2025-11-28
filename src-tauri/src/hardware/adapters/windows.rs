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

#[cfg(target_os = "windows")]
use wmi::WMIConnection;

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
    
    /// Detect GPU information using direct WMI COM interface (no process spawning)
    async fn detect_gpus(&self) -> Result<Vec<GPUInfo>, HardwareError> {
        let mut gpus = Vec::new();
        
        // Use WMI COM interface directly - no process spawning, no windows
        let wmi_con = match WMIConnection::new() {
            Ok(con) => con,
            Err(e) => {
                log::error!("Failed to connect to WMI for GPU detection: {}", e);
                return Ok(gpus); // Return empty list on connection failure
            }
        };
        
        // Query Win32_VideoController using WMI
        // Note: Property names in WMI are case-sensitive
        let query = "SELECT Name, AdapterRAM, DriverVersion, PNPDeviceID FROM Win32_VideoController";
        let results: Result<Vec<serde_json::Value>, _> = wmi_con.raw_query(query);
        
        match results {
            Ok(video_controllers) => {
                log::debug!("Found {} video controllers via WMI", video_controllers.len());
                
                for controller in video_controllers {
                    // WMI returns properties in various formats, try different access methods
                    let name = controller.get("Name")
                        .or_else(|| controller.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    if name.is_empty() {
                        log::debug!("Skipping GPU with empty name");
                        continue;
                    }
                    
                    // Skip basic display adapters and virtual GPUs
                    let name_upper = name.to_uppercase();
                    if name_upper.contains("MICROSOFT") || 
                       name_upper.contains("BASIC DISPLAY") ||
                       name_upper.contains("REMOTE DESKTOP") ||
                       name_upper.contains("VIRTUAL") {
                        log::debug!("Skipping virtual/basic GPU: {}", name);
                        continue;
                    }
                    
                    // Parse AdapterRAM (in bytes, convert to MB)
                    // AdapterRAM can be null for some GPUs, or might be a string
                    let vram_mb = controller.get("AdapterRAM")
                        .or_else(|| controller.get("adapterRAM"))
                        .and_then(|v| {
                            // Try as u64 first
                            v.as_u64()
                                .or_else(|| {
                                    // Try as string and parse
                                    v.as_str()
                                        .and_then(|s| s.parse::<u64>().ok())
                                })
                        })
                        .map(|bytes| {
                            // Convert bytes to MB
                            bytes / (1024 * 1024)
                        });
                    
                    let driver_version = controller.get("DriverVersion")
                        .or_else(|| controller.get("driverVersion"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty());
                    
                    let pci_id = controller.get("PNPDeviceID")
                        .or_else(|| controller.get("pnpDeviceID"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty());
                    
                    // Determine vendor from name
                    let vendor = if name_upper.contains("NVIDIA") {
                        "NVIDIA".to_string()
                    } else if name_upper.contains("AMD") || name_upper.contains("RADEON") {
                        "AMD".to_string()
                    } else if name_upper.contains("INTEL") {
                        "Intel".to_string()
                    } else {
                        "Unknown".to_string()
                    };
                    
                    log::info!("Detected GPU: {} ({}), VRAM: {:?} MB", name, vendor, vram_mb);
                    
                    gpus.push(GPUInfo {
                        model: name,
                        vendor,
                        vram_total_mb: vram_mb,
                        driver_version,
                        pci_id,
                    });
                }
            }
            Err(e) => {
                log::error!("WMI GPU query failed: {}", e);
            }
        }
        
        if gpus.is_empty() {
            log::warn!("No GPUs detected via WMI");
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
    
    /// Detect storage devices using direct WMI COM interface (no process spawning)
    async fn detect_storage(&self) -> Result<Vec<StorageInfo>, HardwareError> {
        let mut storage_devices = Vec::new();
        
        // Use WMI COM interface directly - no process spawning, no windows
        let wmi_con = match WMIConnection::new() {
            Ok(con) => con,
            Err(e) => {
                log::error!("Failed to connect to WMI for storage detection: {}", e);
                return Ok(storage_devices); // Return empty list on connection failure
            }
        };
        
        // Query Win32_DiskDrive using WMI
        // Note: Property names in WMI are case-sensitive
        let query = "SELECT Model, Size, InterfaceType, MediaType FROM Win32_DiskDrive";
        let results: Result<Vec<serde_json::Value>, _> = wmi_con.raw_query(query);
        
        match results {
            Ok(disk_drives) => {
                log::debug!("Found {} disk drives via WMI", disk_drives.len());
                
                for drive in disk_drives {
                    // WMI returns properties in various formats, try different access methods
                    let model = drive.get("Model")
                        .or_else(|| drive.get("model"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    
                    if model.is_empty() {
                        log::debug!("Skipping storage device with empty model");
                        continue;
                    }
                    
                    // Parse Size (in bytes, convert to MB)
                    // Size can be null for some drives, or might be a string
                    let capacity_mb = drive.get("Size")
                        .or_else(|| drive.get("size"))
                        .and_then(|v| {
                            // Try as u64 first
                            v.as_u64()
                                .or_else(|| {
                                    // Try as string and parse
                                    v.as_str()
                                        .and_then(|s| s.parse::<u64>().ok())
                                })
                        })
                        .map(|bytes| bytes / (1024 * 1024))
                        .unwrap_or(0);
                    
                    let interface = drive.get("InterfaceType")
                        .or_else(|| drive.get("interfaceType"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                    
                    let media_type = drive.get("MediaType")
                        .or_else(|| drive.get("mediaType"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim().to_lowercase())
                        .unwrap_or_default();
                    
                    // Determine storage type from interface and media type
                    let model_upper = model.to_uppercase();
                    let interface_upper = interface.as_ref().map(|s| s.to_uppercase());
                    
                    let storage_type = if model_upper.contains("NVME") || 
                                         interface_upper.as_ref().map(|s| s.contains("NVME")).unwrap_or(false) {
                        StorageType::NVMe
                    } else if media_type.contains("ssd") || 
                              model_upper.contains("SSD") ||
                              interface_upper.as_ref().map(|s| s.contains("SATA")).unwrap_or(false) {
                        StorageType::SSD
                    } else if media_type.contains("hdd") || 
                              model_upper.contains("HDD") ||
                              media_type.contains("fixed") ||
                              media_type.contains("disk") {
                        StorageType::HDD
                    } else {
                        StorageType::Unknown
                    };
                    
                    log::info!("Detected storage: {} ({}), Capacity: {} MB, Type: {:?}", 
                        model, interface.as_ref().unwrap_or(&"Unknown".to_string()), capacity_mb, storage_type);
                    
                    storage_devices.push(StorageInfo {
                        model,
                        capacity_mb,
                        storage_type,
                        interface,
                    });
                }
            }
            Err(e) => {
                log::error!("WMI storage query failed: {}", e);
            }
        }
        
        if storage_devices.is_empty() {
            log::warn!("No storage devices detected via WMI");
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
        
        // Detect all components - allow partial failures
        let cpu = self.detect_cpu().await?; // CPU detection must succeed
        let memory = self.detect_memory().await?; // Memory detection must succeed
        
        // GPU and storage detection can fail gracefully
        let gpus = self.detect_gpus().await.unwrap_or_else(|e| {
            log::warn!("GPU detection failed: {}, continuing with empty GPU list", e);
            Vec::new()
        });
        
        let storage_devices = self.detect_storage().await.unwrap_or_else(|e| {
            log::warn!("Storage detection failed: {}, continuing with empty storage list", e);
            Vec::new()
        });
        
        // Optional components
        let motherboard = self.detect_motherboard().await.unwrap_or_else(|e| {
            log::warn!("Motherboard detection failed: {}", e);
            None
        });
        
        let psu = self.detect_psu().await.unwrap_or_else(|e| {
            log::warn!("PSU detection failed: {}", e);
            None
        });
        
        let cooling = self.detect_cooling().await.unwrap_or_else(|e| {
            log::warn!("Cooling detection failed: {}", e);
            None
        });
        
        let displays = self.detect_displays().await.unwrap_or_else(|e| {
            log::warn!("Display detection failed: {}", e);
            Vec::new()
        });
        
        // Collect warnings for missing components
        let mut warnings = Vec::new();
        if gpus.is_empty() {
            warnings.push("No GPUs detected. GPU detection may have failed.".to_string());
        }
        if storage_devices.is_empty() {
            warnings.push("No storage devices detected. Storage detection may have failed.".to_string());
        }
        
        let metadata = DetectionMetadata {
            detection_time: chrono::Utc::now(),
            platform: "windows".to_string(),
            warnings,
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



