//! Persistence data models
//!
//! This module defines data structures for persistence with schema versioning.

use serde::{Deserialize, Serialize};

/// Schema version constant
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Persisted session with schema version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub schema_version: u32,
    pub session: crate::core::domain::Session,
}

/// Persisted run with schema version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedRun {
    pub schema_version: u32,
    pub run: crate::core::domain::Run,
}
