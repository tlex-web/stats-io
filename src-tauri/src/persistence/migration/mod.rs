//! Schema migration
//!
//! This module handles migration between different schema versions.

use crate::core::error::PersistenceError;
use crate::persistence::models::CURRENT_SCHEMA_VERSION;

/// Migration trait for version upgrades
pub trait Migration {
    fn migrate(&self, from_version: u32, to_version: u32, data: &str) -> Result<String, PersistenceError>;
}

/// Default migration implementation
pub struct DefaultMigration;

impl Migration for DefaultMigration {
    fn migrate(&self, from_version: u32, to_version: u32, _data: &str) -> Result<String, PersistenceError> {
        if from_version > to_version {
            return Err(PersistenceError::MigrationFailed(
                format!("Cannot downgrade from version {} to {}", from_version, to_version)
            ));
        }
        
        if from_version == to_version {
            return Ok(_data.to_string());
        }
        
        // For MVP, we only support version 1
        // Future versions will implement actual migration logic
        Err(PersistenceError::MigrationFailed(
            format!("Migration from version {} to {} not yet implemented", from_version, to_version)
        ))
    }
}

/// Check and migrate data if needed
pub fn check_and_migrate(
    data: &str,
    current_version: u32,
) -> Result<String, PersistenceError> {
    // Try to parse schema version from JSON
    // For MVP, we assume all data is version 1
    if current_version != CURRENT_SCHEMA_VERSION {
        let migration = DefaultMigration;
        migration.migrate(current_version, CURRENT_SCHEMA_VERSION, data)
    } else {
        Ok(data.to_string())
    }
}
