//! Storage implementations
//!
//! This module provides file-based storage using JSON serialization.

use crate::core::domain::Session;
use crate::core::error::PersistenceError;
use serde_json;
use std::path::PathBuf;
use tokio::fs;

/// Session storage manager
pub struct SessionStorage {
    base_path: PathBuf,
}

impl SessionStorage {
    /// Create a new session storage manager
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
    
    /// Ensure the storage directory exists
    async fn ensure_directory(&self) -> Result<(), PersistenceError> {
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| PersistenceError::Io(e))?;
        Ok(())
    }
    
    /// Get the path for a session file
    fn session_path(&self, session_id: &uuid::Uuid) -> PathBuf {
        self.base_path.join(format!("{}.json", session_id))
    }
    
    /// Save a session to disk
    pub async fn save_session(&self, session: &Session) -> Result<(), PersistenceError> {
        self.ensure_directory().await?;
        
        let path = self.session_path(&session.id);
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        
        fs::write(&path, json)
            .await
            .map_err(|e| PersistenceError::Io(e))?;
        
        Ok(())
    }
    
    /// Load a session from disk
    pub async fn load_session(&self, session_id: &uuid::Uuid) -> Result<Session, PersistenceError> {
        let path = self.session_path(session_id);
        
        if !path.exists() {
            return Err(PersistenceError::FileNotFound(format!("Session {} not found", session_id)));
        }
        
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| PersistenceError::Io(e))?;
        
        let session: Session = serde_json::from_str(&content)
            .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
        
        Ok(session)
    }
    
    /// List all saved sessions
    pub async fn list_sessions(&self) -> Result<Vec<uuid::Uuid>, PersistenceError> {
        self.ensure_directory().await?;
        
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| PersistenceError::Io(e))?;
        
        let mut session_ids = Vec::new();
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| PersistenceError::Io(e))? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(uuid) = uuid::Uuid::parse_str(file_stem) {
                        session_ids.push(uuid);
                    }
                }
            }
        }
        
        Ok(session_ids)
    }
    
    /// Delete a session
    pub async fn delete_session(&self, session_id: &uuid::Uuid) -> Result<(), PersistenceError> {
        let path = self.session_path(session_id);
        
        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|e| PersistenceError::Io(e))?;
        }
        
        Ok(())
    }
}
