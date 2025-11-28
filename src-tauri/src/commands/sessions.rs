//! Tauri commands for session management
//!
//! This module exposes session management functionality to the frontend.

use crate::core::domain::{Run, Session, WorkloadProfile};
use crate::persistence::init_session_storage;
use chrono::Utc;
use tauri::AppHandle;
use uuid::Uuid;

/// Create a new session
#[tauri::command]
pub async fn create_session(
    app: AppHandle,
    _name: String,
    profile: WorkloadProfile,
    hardware_config: crate::core::domain::HardwareConfig,
) -> Result<Session, String> {
    let session = Session {
        id: Uuid::new_v4(),
        start_time: Utc::now(),
        end_time: None,
        hardware_config_snapshot: hardware_config,
        profile,
        runs: Vec::new(),
    };
    
    // Save session immediately
    let storage = init_session_storage(&app)
        .map_err(|e| e.to_string())?;
    storage.save_session(&session)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(session)
}

/// Save a session
#[tauri::command]
pub async fn save_session(
    app: AppHandle,
    session: Session,
) -> Result<(), String> {
    let storage = init_session_storage(&app)
        .map_err(|e| e.to_string())?;
    storage.save_session(&session)
        .await
        .map_err(|e| e.to_string())
}

/// Load a session
#[tauri::command]
pub async fn load_session(
    app: AppHandle,
    session_id: String,
) -> Result<Session, String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;
    
    let storage = init_session_storage(&app)
        .map_err(|e| e.to_string())?;
    storage.load_session(&uuid)
        .await
        .map_err(|e| e.to_string())
}

/// List all sessions
#[tauri::command]
pub async fn list_sessions(
    app: AppHandle,
) -> Result<Vec<String>, String> {
    let storage = init_session_storage(&app)
        .map_err(|e| e.to_string())?;
    let session_ids = storage.list_sessions()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(session_ids.iter().map(|id| id.to_string()).collect())
}

/// Delete a session
#[tauri::command]
pub async fn delete_session(
    app: AppHandle,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;
    
    let storage = init_session_storage(&app)
        .map_err(|e| e.to_string())?;
    storage.delete_session(&uuid)
        .await
        .map_err(|e| e.to_string())
}

/// Add a run to a session
#[tauri::command]
pub async fn add_run_to_session(
    app: AppHandle,
    session_id: String,
    run: Run,
) -> Result<Session, String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;
    
    let storage = init_session_storage(&app)
        .map_err(|e| e.to_string())?;
    
    let mut session = storage.load_session(&uuid)
        .await
        .map_err(|e| e.to_string())?;
    
    session.runs.push(run);
    
    storage.save_session(&session)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(session)
}

/// End a session
#[tauri::command]
pub async fn end_session(
    app: AppHandle,
    session_id: String,
) -> Result<Session, String> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;
    
    let storage = init_session_storage(&app)
        .map_err(|e| e.to_string())?;
    
    let mut session = storage.load_session(&uuid)
        .await
        .map_err(|e| e.to_string())?;
    
    session.end_time = Some(Utc::now());
    
    storage.save_session(&session)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(session)
}
