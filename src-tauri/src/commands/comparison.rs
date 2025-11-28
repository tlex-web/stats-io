//! Tauri commands for comparison functionality
//!
//! This module exposes comparison functionality to the frontend.

use crate::analysis::comparison::compare_runs;
use crate::core::domain::Run;

/// Compare two runs
#[tauri::command]
pub fn compare_runs_command(
    run1: Run,
    run2: Run,
) -> Result<crate::analysis::ComparisonResult, String> {
    Ok(compare_runs(&run1, &run2))
}

