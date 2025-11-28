//! PC Rig Hardware & Bottleneck Analyzer
//!
//! A desktop application for analyzing PC hardware and identifying performance bottlenecks.

pub mod core;
pub mod hardware;
pub mod metrics;
pub mod analysis;
pub mod persistence;
mod commands;

use commands::hardware::*;
use commands::metrics::*;
use commands::analysis::*;
use commands::sessions::*;
use commands::profiles::*;
use commands::comparison::*;
use commands::reports::*;
use commands::settings::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(
      tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .build(),
    )
    .invoke_handler(tauri::generate_handler![
        // Hardware commands
        get_hardware_config,
        refresh_hardware_config,
        // Metrics commands
        start_metrics_collection,
        stop_metrics_collection,
        get_metrics_buffer,
        get_metrics_in_range,
        get_aggregated_metrics,
        // Analysis commands
        analyze_bottlenecks,
        generate_insights,
        // Session commands
        create_session,
        save_session,
        load_session,
        list_sessions,
        delete_session,
        add_run_to_session,
        end_session,
        // Profile commands
        get_preset_profiles,
        get_profile_by_id,
        // Comparison commands
        compare_runs_command,
        // Report commands
        generate_report,
        generate_comparison_report_command,
        // Settings commands
        get_settings,
        update_settings,
        update_thresholds,
        update_sampling,
        update_units,
        update_theme,
        reset_settings,
    ])
    .setup(|app| {
        // Initialize settings manager
        let app_handle = app.handle();
        if let Err(e) = commands::settings::init_settings_manager(app_handle.clone()) {
            eprintln!("Failed to initialize settings manager: {}", e);
        }
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
