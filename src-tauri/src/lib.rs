mod types;
pub mod commands;
mod demo;
pub mod ssh;
mod security;
mod validation;
mod retry;
mod database;
mod slurm;
mod logging;
pub mod automations;
pub mod cluster;
#[cfg(test)]
mod security_tests;

pub use types::*;

fn initialize_database() -> anyhow::Result<()> {
    // Determine database path based on environment
    let db_path = if cfg!(debug_assertions) {
        // In development, use a local database file
        "./namdrunner_dev.db"
    } else {
        // In production, use a path in the user's data directory
        // For now, use a simple path - this could be improved with proper OS-specific paths
        "./namdrunner.db"
    };

    database::initialize_database(db_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging system first
    logging::init_logging();

    // Initialize database on startup
    if let Err(e) = initialize_database() {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Set up logging bridge to frontend
            logging::set_app_handle(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connection lifecycle
            commands::connection::connect_to_cluster,
            commands::connection::disconnect,
            commands::connection::get_connection_status,
            // System configuration
            commands::system::set_app_mode,
            // Cluster configuration
            commands::cluster::get_cluster_capabilities,
            commands::cluster::suggest_qos_for_partition,
            commands::cluster::estimate_queue_time_for_job,
            commands::cluster::calculate_job_cost,
            commands::cluster::validate_resource_allocation,
            // Job management
            commands::jobs::create_job,
            commands::jobs::submit_job,
            commands::jobs::get_job_status,
            commands::jobs::get_all_jobs,
            commands::jobs::sync_jobs,
            commands::jobs::delete_job,
            commands::jobs::complete_job,
            commands::jobs::discover_jobs_from_server,
            // File management
            commands::files::select_input_files,
            commands::files::upload_job_files,
            commands::files::download_job_output,
            commands::files::list_job_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
