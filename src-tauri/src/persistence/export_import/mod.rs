//! Data export and import functionality
//!
//! This module provides functionality for exporting sessions for sharing
//! and importing external benchmark data, following IMPLEMENTATION_PLAN.md Phase 4.3.

use crate::core::domain::{Run, Session};
use crate::core::error::PersistenceError;
use serde_json;
use std::path::Path;
use tokio::fs;

/// Export a session to a JSON file for sharing
pub async fn export_session(
    session: &Session,
    export_path: &Path,
) -> Result<(), PersistenceError> {
    // Create export format with metadata
    #[derive(serde::Serialize)]
    struct ExportedSession {
        export_version: u32,
        export_timestamp: String,
        session: Session,
    }
    
    let exported = ExportedSession {
        export_version: 1,
        export_timestamp: chrono::Utc::now().to_rfc3339(),
        session: session.clone(),
    };
    
    let json = serde_json::to_string_pretty(&exported)
        .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
    
    fs::write(export_path, json)
        .await
        .map_err(|e| PersistenceError::Io(e))?;
    
    Ok(())
}

/// Import a session from an exported JSON file
pub async fn import_session(import_path: &Path) -> Result<Session, PersistenceError> {
    let content = fs::read_to_string(import_path)
        .await
        .map_err(|e| PersistenceError::Io(e))?;
    
    // Try to parse as exported format first
    #[derive(serde::Deserialize)]
    struct ExportedSession {
        #[serde(default)]
        export_version: u32,
        #[serde(default)]
        export_timestamp: Option<String>,
        session: Session,
    }
    
    // Try exported format
    if let Ok(exported) = serde_json::from_str::<ExportedSession>(&content) {
        return Ok(exported.session);
    }
    
    // Fallback to direct session format
    let session: Session = serde_json::from_str(&content)
        .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
    
    Ok(session)
}

/// Export multiple sessions to a single archive file
pub async fn export_sessions_batch(
    sessions: &[Session],
    export_path: &Path,
) -> Result<(), PersistenceError> {
    #[derive(serde::Serialize)]
    struct BatchExport {
        export_version: u32,
        export_timestamp: String,
        session_count: usize,
        sessions: Vec<Session>,
    }
    
    let batch = BatchExport {
        export_version: 1,
        export_timestamp: chrono::Utc::now().to_rfc3339(),
        session_count: sessions.len(),
        sessions: sessions.to_vec(),
    };
    
    let json = serde_json::to_string_pretty(&batch)
        .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
    
    fs::write(export_path, json)
        .await
        .map_err(|e| PersistenceError::Io(e))?;
    
    Ok(())
}

/// Import multiple sessions from a batch export file
pub async fn import_sessions_batch(
    import_path: &Path,
) -> Result<Vec<Session>, PersistenceError> {
    let content = fs::read_to_string(import_path)
        .await
        .map_err(|e| PersistenceError::Io(e))?;
    
    #[derive(serde::Deserialize)]
    struct BatchExport {
        #[serde(default)]
        export_version: u32,
        #[serde(default)]
        export_timestamp: Option<String>,
        #[serde(default)]
        session_count: Option<usize>,
        sessions: Vec<Session>,
    }
    
    let batch: BatchExport = serde_json::from_str(&content)
        .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
    
    Ok(batch.sessions)
}

/// Export a run for sharing
pub async fn export_run(
    run: &Run,
    export_path: &Path,
) -> Result<(), PersistenceError> {
    #[derive(serde::Serialize)]
    struct ExportedRun {
        export_version: u32,
        export_timestamp: String,
        run: Run,
    }
    
    let exported = ExportedRun {
        export_version: 1,
        export_timestamp: chrono::Utc::now().to_rfc3339(),
        run: run.clone(),
    };
    
    let json = serde_json::to_string_pretty(&exported)
        .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
    
    fs::write(export_path, json)
        .await
        .map_err(|e| PersistenceError::Io(e))?;
    
    Ok(())
}

/// Import a run from an exported file
pub async fn import_run(import_path: &Path) -> Result<Run, PersistenceError> {
    let content = fs::read_to_string(import_path)
        .await
        .map_err(|e| PersistenceError::Io(e))?;
    
    #[derive(serde::Deserialize)]
    struct ExportedRun {
        #[serde(default)]
        export_version: u32,
        #[serde(default)]
        export_timestamp: Option<String>,
        run: Run,
    }
    
    // Try exported format
    if let Ok(exported) = serde_json::from_str::<ExportedRun>(&content) {
        return Ok(exported.run);
    }
    
    // Fallback to direct run format
    let run: Run = serde_json::from_str(&content)
        .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
    
    Ok(run)
}

