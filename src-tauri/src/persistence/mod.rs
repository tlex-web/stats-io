//! Persistence subsystem
//!
//! This module provides data persistence functionality following AGENT.md Section 7.2.

pub mod models;
pub mod storage;
pub mod migration;
pub mod reports;
pub mod database;
pub mod export_import;
pub mod retention;

pub use storage::SessionStorage;
pub use database::DatabaseStorage;

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Get the app data directory for persistence
pub fn get_app_data_dir(app: &AppHandle) -> Result<PathBuf, crate::core::error::PersistenceError> {
    // Tauri v2: Use path() method with BaseDirectory::AppData
    use tauri::path::BaseDirectory;
    app.path()
        .resolve("", BaseDirectory::AppData)
        .map_err(|e| crate::core::error::PersistenceError::Unknown(
            format!("Could not get app data directory: {}", e)
        ))
}

/// Initialize session storage
pub fn init_session_storage(app: &AppHandle) -> Result<SessionStorage, crate::core::error::PersistenceError> {
    let base_dir = get_app_data_dir(app)?;
    let sessions_dir = base_dir.join("sessions");
    Ok(SessionStorage::new(sessions_dir))
}
