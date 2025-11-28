//! Error types for the application
//!
//! This module defines all error types used throughout the application,
//! following the error handling strategy outlined in AGENT.md Section 10.4.

use thiserror::Error;

/// Hardware detection errors
#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
    
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    
    #[error("Sensor not available: {0}")]
    SensorNotAvailable(String),
    
    #[error("Detection failed: {0}")]
    DetectionFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Metrics collection errors
#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),
    
    #[error("Sampling failed: {0}")]
    SamplingFailed(String),
    
    #[error("Collection failed: {0}")]
    CollectionFailed(String),
    
    #[error("Invalid metric value: {0}")]
    InvalidValue(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Analysis errors
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Persistence errors
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Schema version mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: u32, found: u32 },
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// General application error
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Hardware error: {0}")]
    Hardware(#[from] HardwareError),
    
    #[error("Metrics error: {0}")]
    Metrics(#[from] MetricsError),
    
    #[error("Analysis error: {0}")]
    Analysis(#[from] AnalysisError),
    
    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

