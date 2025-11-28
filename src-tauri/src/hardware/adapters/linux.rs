//! Linux-specific hardware adapter
//!
//! This module implements HardwareDetector for Linux using sysinfo, /proc, /sys,
//! and other Linux-specific APIs.

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

/// Linux hardware detector implementation
pub struct LinuxHardwareDetector {
    system: Arc<Mutex<System>>,
}

impl LinuxHardwareDetector {
    /// Create a new Linux hardware detector
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
        } else if model.contains("ARM") || model.contains("Apple") {
            "ARM".to_string()
        } else {
            "Unknown".to_string()
        };
        
        // Try to get frequency (may not be available on all systems)
        let frequency = first_cpu.frequency();
        let base_clock_mhz = if frequency > 0 {
            Some(frequency as f64)
        } else {
            None
        };
        
        // Try to read CPU info from /proc/cpuinfo for more details
        let architecture = Self::detect_architecture().await;
        
        // Count physical cores (approximation: divide by 2 if hyperthreading)
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
        
        // Try to detect memory channels and speed from /proc/meminfo or dmidecode
        // For now, use defaults
        let channels = None; // Would require dmidecode or parsing /sys
        let speed_mhz = None; // Would require dmidecode
        
        // Try to detect memory modules (would require dmidecode)
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
            
            // Determine storage type from mount point or device name
            let storage_type = if name.contains("nvme") || name.contains("ssd") {
                StorageType::Nvme
            } else if name.contains("sd") {
                StorageType::Sata
            } else {
                StorageType::Unknown
            };
            
            storage_devices.push(StorageInfo {
                model: name.clone(),
                capacity_mb: total_gb * 1024, // Convert GB to MB
                storage_type,
                interface: None, // Would require additional parsing
            });
        }
        
        Ok(storage_devices)
    }
    
    /// Detect GPU information
    async fn detect_gpus(&self) -> Result<Vec<GPUInfo>, HardwareError> {
        let mut gpus = Vec::new();
        
        // Try to detect GPUs using lspci or /sys/class/drm
        // For now, use a placeholder approach
        // In a full implementation, we would:
        // 1. Parse lspci output for VGA/3D controllers
        // 2. Read /sys/class/drm/card*/device/vendor and device
        // 3. Detect hybrid graphics (integrated + discrete)
        
        // Placeholder: Try to detect via sysinfo (limited support on Linux)
        // Real implementation would use lspci or /sys/class/drm
        
        // Check for NVIDIA GPUs
        if Self::check_nvidia_gpu().await {
            gpus.push(GPUInfo {
                model: "NVIDIA GPU (detected)".to_string(),
                vendor: "NVIDIA".to_string(),
                vram_total_mb: None, // Would require nvidia-smi
                driver_version: None,
                pci_id: None,
            });
        }
        
        // Check for AMD GPUs
        if Self::check_amd_gpu().await {
            gpus.push(GPUInfo {
                model: "AMD GPU (detected)".to_string(),
                vendor: "AMD".to_string(),
                vram_total_mb: None, // Would require rocm-smi or similar
                driver_version: None,
                pci_id: None,
            });
        }
        
        // Check for Intel integrated graphics
        if Self::check_intel_gpu().await {
            gpus.push(GPUInfo {
                model: "Intel Integrated Graphics".to_string(),
                vendor: "Intel".to_string(),
                vram_total_mb: None,
                driver_version: None,
                pci_id: None,
            });
        }
        
        // If no GPUs detected, add a placeholder
        if gpus.is_empty() {
            gpus.push(GPUInfo {
                model: "Unknown GPU".to_string(),
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
        // Linux display detection would use:
        // - xrandr (X11)
        // - wayland-info (Wayland)
        // - /sys/class/drm/card*-*/modes
        
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
        // Try to read from /proc/cpuinfo or uname
        std::env::consts::ARCH.to_string()
    }
    
    /// Check for NVIDIA GPU
    async fn check_nvidia_gpu() -> bool {
        // Check if nvidia-smi is available or /sys/class/drm contains NVIDIA
        // Placeholder: check for common NVIDIA indicators
        std::path::Path::new("/sys/class/drm")
            .read_dir()
            .map(|mut entries| {
                entries.any(|entry| {
                    if let Ok(entry) = entry {
                        entry.path().to_string_lossy().contains("nvidia")
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false)
    }
    
    /// Check for AMD GPU
    async fn check_amd_gpu() -> bool {
        // Check /sys/class/drm for AMD devices
        std::path::Path::new("/sys/class/drm")
            .read_dir()
            .map(|mut entries| {
                entries.any(|entry| {
                    if let Ok(entry) = entry {
                        let path_str = entry.path().to_string_lossy();
                        path_str.contains("amdgpu") || path_str.contains("radeon")
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false)
    }
    
    /// Check for Intel GPU
    async fn check_intel_gpu() -> bool {
        // Check /sys/class/drm for Intel devices
        std::path::Path::new("/sys/class/drm")
            .read_dir()
            .map(|mut entries| {
                entries.any(|entry| {
                    if let Ok(entry) = entry {
                        entry.path().to_string_lossy().contains("intel")
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false)
    }
}

#[async_trait]
impl HardwareDetector for LinuxHardwareDetector {
    async fn get_hardware_config(&self) -> Result<HardwareConfig, HardwareError> {
        let cpu_info = self.detect_cpu().await?;
        let memory_info = self.detect_memory().await?;
        let storage_devices = self.detect_storage().await?;
        let gpus = self.detect_gpus().await?;
        let displays = self.detect_displays().await?;
        
        // Collect warnings for unavailable features
        let mut warnings = Vec::new();
        if gpus.iter().any(|g| g.vram_total_mb.is_none()) {
            warnings.push("GPU VRAM detection requires additional tools (nvidia-smi, rocm-smi)".to_string());
        }
        if memory_info.channels.is_none() || memory_info.speed_mhz.is_none() {
            warnings.push("Memory channel and speed detection requires dmidecode".to_string());
        }
        
        Ok(HardwareConfig {
            cpu: cpu_info,
            gpus,
            memory: memory_info,
            storage_devices,
            motherboard: None, // Would require dmidecode
            psu: None,          // Not easily detectable on Linux
            cooling: None,      // Would require sensors or lm-sensors
            displays,
            metadata: DetectionMetadata {
                detection_time: Utc::now(),
                platform: "Linux".to_string(),
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
