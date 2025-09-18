mod types;
pub mod commands;
mod mock_state;
pub mod ssh;
mod security;
mod validation;
mod retry;
mod database;
mod slurm;
mod mode_switching;
#[cfg(test)]
mod security_tests;
#[cfg(test)]
mod integration_tests;

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
    // Initialize database on startup
    if let Err(e) = initialize_database() {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Connection management
            commands::connection::connect_to_cluster,
            commands::connection::disconnect,
            commands::connection::get_connection_status,
            // SSH/SFTP operations for Phase 1 integration
            commands::connection::ssh_execute_command,
            commands::connection::sftp_upload_file,
            commands::connection::sftp_download_file,
            commands::connection::sftp_list_files,
            commands::connection::sftp_exists,
            commands::connection::sftp_create_directory,
            commands::connection::sftp_get_file_info,
            // Job management
            commands::jobs::create_job,
            commands::jobs::submit_job,
            commands::jobs::get_job_status,
            commands::jobs::get_all_jobs,
            commands::jobs::sync_jobs,
            commands::jobs::delete_job,
            commands::jobs::sync_job_status,
            commands::jobs::sync_all_jobs,
            // File management
            commands::files::upload_job_files,
            commands::files::download_job_output,
            commands::files::list_job_files,
            commands::files::get_job_logs,
            commands::files::cleanup_job_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
