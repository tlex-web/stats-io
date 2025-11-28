//! Data retention policies
//!
//! This module provides automatic cleanup of old sessions based on
//! configurable retention periods, following IMPLEMENTATION_PLAN.md Phase 4.3.

use crate::core::error::PersistenceError;
use chrono::{DateTime, Utc};
use std::path::Path;
use tokio::fs;

/// Retention policy configuration
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Number of days to retain sessions
    pub retention_days: u32,
    /// Whether to enable automatic cleanup
    pub auto_cleanup_enabled: bool,
    /// Minimum number of sessions to keep (even if older than retention period)
    pub min_sessions_to_keep: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 90, // Default: 90 days
            auto_cleanup_enabled: false, // Disabled by default
            min_sessions_to_keep: 10, // Always keep at least 10 sessions
        }
    }
}

/// Cleanup old sessions from file-based storage
pub async fn cleanup_old_sessions_file(
    sessions_dir: &Path,
    policy: &RetentionPolicy,
) -> Result<usize, PersistenceError> {
    if !sessions_dir.exists() {
        return Ok(0);
    }
    
    let cutoff_date = Utc::now() - chrono::Duration::days(policy.retention_days as i64);
    
    let mut entries = fs::read_dir(sessions_dir).await
        .map_err(|e| PersistenceError::Io(e))?;
    
    let mut session_files: Vec<(DateTime<Utc>, std::path::PathBuf)> = Vec::new();
    
    // Collect all session files with their timestamps
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| PersistenceError::Io(e))? {
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            // Try to read session to get start_time
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Ok(session_json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(start_time_str) = session_json.get("start_time")
                        .and_then(|v| v.as_str()) {
                        if let Ok(start_time) = DateTime::parse_from_rfc3339(start_time_str) {
                            session_files.push((start_time.with_timezone(&Utc), path));
                        }
                    }
                }
            }
        }
    }
    
    // Sort by start_time (oldest first)
    session_files.sort_by_key(|(time, _)| *time);
    
    // Calculate how many to delete
    let total_sessions = session_files.len();
    let mut to_delete = session_files.iter()
        .filter(|(time, _)| *time < cutoff_date)
        .count();
    
    // Ensure we keep at least min_sessions_to_keep
    if total_sessions - to_delete < policy.min_sessions_to_keep {
        to_delete = total_sessions.saturating_sub(policy.min_sessions_to_keep);
    }
    
    // Delete old sessions
    let mut deleted = 0;
    for (_, path) in session_files.iter().take(to_delete) {
        if let Err(e) = fs::remove_file(path).await {
            eprintln!("Failed to delete session file {:?}: {}", path, e);
        } else {
            deleted += 1;
        }
    }
    
    Ok(deleted)
}

/// Get statistics about session retention
pub async fn get_retention_stats(
    sessions_dir: &Path,
    policy: &RetentionPolicy,
) -> Result<RetentionStats, PersistenceError> {
    if !sessions_dir.exists() {
        return Ok(RetentionStats {
            total_sessions: 0,
            sessions_to_delete: 0,
            oldest_session_date: None,
            newest_session_date: None,
        });
    }
    
    let cutoff_date = Utc::now() - chrono::Duration::days(policy.retention_days as i64);
    
    let mut entries = fs::read_dir(sessions_dir).await
        .map_err(|e| PersistenceError::Io(e))?;
    
    let mut session_dates: Vec<DateTime<Utc>> = Vec::new();
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| PersistenceError::Io(e))? {
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Ok(session_json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(start_time_str) = session_json.get("start_time")
                        .and_then(|v| v.as_str()) {
                        if let Ok(start_time) = DateTime::parse_from_rfc3339(start_time_str) {
                            session_dates.push(start_time.with_timezone(&Utc));
                        }
                    }
                }
            }
        }
    }
    
    let total_sessions = session_dates.len();
    let sessions_to_delete = session_dates.iter()
        .filter(|&date| *date < cutoff_date)
        .count()
        .saturating_sub(total_sessions.saturating_sub(policy.min_sessions_to_keep));
    
    let oldest_session_date = session_dates.iter().min().copied();
    let newest_session_date = session_dates.iter().max().copied();
    
    Ok(RetentionStats {
        total_sessions,
        sessions_to_delete,
        oldest_session_date,
        newest_session_date,
    })
}

/// Retention statistics
#[derive(Debug, Clone)]
pub struct RetentionStats {
    pub total_sessions: usize,
    pub sessions_to_delete: usize,
    pub oldest_session_date: Option<DateTime<Utc>>,
    pub newest_session_date: Option<DateTime<Utc>>,
}

