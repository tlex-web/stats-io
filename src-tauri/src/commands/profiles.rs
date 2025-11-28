//! Tauri commands for workload profile management
//!
//! This module exposes workload profile functionality to the frontend.

use crate::core::profiles::WorkloadProfiles;
use crate::core::domain::WorkloadProfile;

/// Get all preset workload profiles
#[tauri::command]
pub fn get_preset_profiles() -> Vec<WorkloadProfile> {
    WorkloadProfiles::get_presets()
}

/// Get a workload profile by ID
#[tauri::command]
pub fn get_profile_by_id(id: String) -> Result<WorkloadProfile, String> {
    WorkloadProfiles::get_by_id(&id)
        .ok_or_else(|| format!("Profile with ID '{}' not found", id))
}

