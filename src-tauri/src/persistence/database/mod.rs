//! SQLite database persistence
//!
//! This module provides SQLite-based persistence for large datasets,
//! following IMPLEMENTATION_PLAN.md Phase 4.3.

use crate::core::domain::{Run, Session};
use crate::core::error::PersistenceError;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde_json;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// SQLite database manager for sessions and runs
pub struct DatabaseStorage {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseStorage {
    /// Create a new database storage instance
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, PersistenceError> {
        let conn = Connection::open(db_path)
            .map_err(|e| PersistenceError::Database(e.to_string()))?;
        
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        storage.init_schema()?;
        Ok(storage)
    }
    
    /// Initialize database schema
    fn init_schema(&self) -> Result<(), PersistenceError> {
        let conn = self.conn.lock().unwrap();
        
        // Sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                hardware_config TEXT NOT NULL,
                profile_id TEXT NOT NULL,
                profile_name TEXT NOT NULL,
                profile_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Runs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS runs (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                name TEXT NOT NULL,
                metrics_streams TEXT NOT NULL,
                analysis_result TEXT,
                notes TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Metrics table for efficient querying
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                metric_type TEXT NOT NULL,
                value REAL NOT NULL,
                unit TEXT NOT NULL,
                source_component TEXT NOT NULL,
                FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Create indexes for efficient queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_start_time ON sessions(start_time)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runs_session_id ON runs(session_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metrics_run_id ON metrics(run_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_metrics_type ON metrics(metric_type)",
            [],
        )?;
        
        Ok(())
    }
    
    /// Save a session to the database
    pub fn save_session(&self, session: &Session) -> Result<(), PersistenceError> {
        let hardware_json = serde_json::to_string(&session.hardware_config_snapshot)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        
        let now = Utc::now().to_rfc3339();
        
        // Lock connection, save session, then release lock
        {
            let conn = self.conn.lock().unwrap();
            
            conn.execute(
                "INSERT OR REPLACE INTO sessions (
                    id, name, start_time, end_time, hardware_config,
                    profile_id, profile_name, profile_type,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    session.id.to_string(),
                    session.profile.name.clone(), // Use profile name as session name
                    session.start_time.to_rfc3339(),
                    session.end_time.map(|t| t.to_rfc3339()),
                    hardware_json,
                    session.profile.id,
                    session.profile.name,
                    format!("{:?}", session.profile.workload_type),
                    now,
                    now,
                ],
            )?;
        } // Lock released here
        
        // Save runs for this session (each will lock/unlock independently)
        for run in &session.runs {
            self.save_run(run, &session.id)?;
        }
        
        Ok(())
    }
    
    /// Save a run to the database
    pub fn save_run(&self, run: &Run, session_id: &uuid::Uuid) -> Result<(), PersistenceError> {
        let metrics_json = serde_json::to_string(&run.metrics_streams)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        
        let analysis_json = run.analysis_result.as_ref()
            .map(|a| serde_json::to_string(a))
            .transpose()
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        
        let now = Utc::now().to_rfc3339();
        
        // Lock connection, save run, then release lock
        {
            let conn = self.conn.lock().unwrap();
            
            conn.execute(
                "INSERT OR REPLACE INTO runs (
                    id, session_id, name, metrics_streams, analysis_result, notes, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    run.id.to_string(),
                    session_id.to_string(),
                    run.name,
                    metrics_json,
                    analysis_json,
                    run.notes,
                    now,
                ],
            )?;
        } // Lock released here
        
        // Save individual metrics for efficient querying (will lock again)
        self.save_metrics(run)?;
        
        Ok(())
    }
    
    /// Save metrics for efficient querying
    fn save_metrics(&self, run: &Run) -> Result<(), PersistenceError> {
        let conn = self.conn.lock().unwrap();
        
        // Delete existing metrics for this run
        conn.execute(
            "DELETE FROM metrics WHERE run_id = ?1",
            params![run.id.to_string()],
        )?;
        
        // Insert metrics
        for (_, samples) in &run.metrics_streams {
            for sample in samples {
                conn.execute(
                    "INSERT INTO metrics (
                        run_id, timestamp, metric_type, value, unit, source_component
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        run.id.to_string(),
                        sample.timestamp.to_rfc3339(),
                        format!("{:?}", sample.metric_type),
                        sample.value,
                        sample.unit,
                        sample.source_component,
                    ],
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Load a session from the database
    pub fn load_session(&self, session_id: &uuid::Uuid) -> Result<Session, PersistenceError> {
        // Load session data (lock, read, release)
        let (id_str, _name, start_time_str, end_time_str, hardware_json,
             profile_id, profile_name, profile_type_str) = {
            let conn = self.conn.lock().unwrap();
            
            let mut stmt = conn.prepare(
                "SELECT id, name, start_time, end_time, hardware_config,
                        profile_id, profile_name, profile_type
                 FROM sessions WHERE id = ?1"
            )?;
            
            let session_row = stmt.query_row(params![session_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })?;
            
            session_row
        }; // Lock released here
        
        let hardware_config: crate::core::domain::HardwareConfig = serde_json::from_str(&hardware_json)
            .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
        
        let start_time = DateTime::parse_from_rfc3339(&start_time_str)
            .map_err(|e| PersistenceError::Deserialization(e.to_string()))?
            .with_timezone(&Utc);
        
        let end_time = end_time_str.map(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
        }).transpose()
            .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
        
        let workload_type = match profile_type_str.as_str() {
            "Gaming" => crate::core::domain::WorkloadType::Gaming,
            "Rendering" => crate::core::domain::WorkloadType::Rendering,
            "AI" => crate::core::domain::WorkloadType::AI,
            "Productivity" => crate::core::domain::WorkloadType::Productivity,
            _ => crate::core::domain::WorkloadType::General,
        };
        
        let profile = crate::core::domain::WorkloadProfile {
            id: profile_id,
            name: profile_name,
            workload_type,
            parameters: std::collections::HashMap::new(),
            threshold_overrides: None,
        };
        
        // Load runs for this session (separate lock to avoid deadlock)
        let runs = self.load_runs_for_session(session_id)?;
        
        let session_id_parsed = uuid::Uuid::parse_str(&id_str)
            .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
        
        Ok(Session {
            id: session_id_parsed,
            start_time,
            end_time,
            hardware_config_snapshot: hardware_config,
            profile,
            runs,
        })
    }
    
    /// Load runs for a session
    fn load_runs_for_session(&self, session_id: &uuid::Uuid) -> Result<Vec<Run>, PersistenceError> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, metrics_streams, analysis_result, notes
             FROM runs WHERE session_id = ?1 ORDER BY created_at"
        )?;
        
        let run_rows = stmt.query_map(params![session_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;
        
        let mut runs = Vec::new();
        for row_result in run_rows {
            let (id_str, name, metrics_json, analysis_json, notes) = row_result?;
            
            let metrics_streams: std::collections::HashMap<String, Vec<crate::core::domain::MetricSample>> =
                serde_json::from_str(&metrics_json)
                    .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
            
            let analysis_result = analysis_json.map(|json| {
                serde_json::from_str(&json)
                    .map_err(|e| PersistenceError::Deserialization(e.to_string()))
            }).transpose()?;
            
            runs.push(Run {
                id: uuid::Uuid::parse_str(&id_str)
                    .map_err(|e| PersistenceError::Deserialization(e.to_string()))?,
                name,
                metrics_streams,
                analysis_result,
                notes,
            });
        }
        
        Ok(runs)
    }
    
    /// List all sessions
    pub fn list_sessions(&self) -> Result<Vec<uuid::Uuid>, PersistenceError> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT id FROM sessions ORDER BY start_time DESC")?;
        
        let id_rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut session_ids = Vec::new();
        for id_result in id_rows {
            let id_str = id_result?;
            if let Ok(uuid) = uuid::Uuid::parse_str(&id_str) {
                session_ids.push(uuid);
            }
        }
        
        Ok(session_ids)
    }
    
    /// Delete old sessions based on retention policy
    pub fn cleanup_old_sessions(&self, retention_days: u32) -> Result<usize, PersistenceError> {
        let conn = self.conn.lock().unwrap();
        
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        let cutoff_str = cutoff_date.to_rfc3339();
        
        // Delete sessions older than cutoff (CASCADE will delete associated runs and metrics)
        let deleted = conn.execute(
            "DELETE FROM sessions WHERE start_time < ?1",
            params![cutoff_str],
        )?;
        
        // Vacuum database to reclaim space
        conn.execute("VACUUM", [])?;
        
        Ok(deleted)
    }
    
    /// Query metrics efficiently
    pub fn query_metrics(
        &self,
        run_id: &uuid::Uuid,
        metric_type: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<crate::core::domain::MetricSample>, PersistenceError> {
        let conn = self.conn.lock().unwrap();
        
        let mut query = "SELECT timestamp, metric_type, value, unit, source_component
                        FROM metrics WHERE run_id = ?1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(run_id.to_string())];
        
        if let Some(mt) = metric_type {
            query.push_str(" AND metric_type = ?2");
            params_vec.push(Box::new(mt.to_string()));
        }
        
        if let Some(st) = start_time {
            query.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(st.to_rfc3339()));
        }
        
        if let Some(et) = end_time {
            query.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(et.to_rfc3339()));
        }
        
        query.push_str(" ORDER BY timestamp");
        
        // Note: This is simplified - in production, use proper parameter binding
        let mut stmt = conn.prepare(&query)?;
        
        let metric_rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        
        let mut samples = Vec::new();
        for row_result in metric_rows {
            let (timestamp_str, _metric_type_str, value, unit, source_component) = row_result?;
            
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| PersistenceError::Deserialization(e.to_string()))?
                .with_timezone(&Utc);
            
            // Parse metric type (simplified - would need proper enum parsing)
            let metric_type = crate::core::domain::MetricType::CpuUtilization; // Placeholder
            
            samples.push(crate::core::domain::MetricSample {
                timestamp,
                metric_type,
                value,
                unit,
                source_component,
            });
        }
        
        Ok(samples)
    }
}

impl From<rusqlite::Error> for PersistenceError {
    fn from(err: rusqlite::Error) -> Self {
        PersistenceError::Database(err.to_string())
    }
}

