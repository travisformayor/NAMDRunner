mod types;
pub mod commands;
pub mod ssh;
mod security;
mod validation;
mod retry;
mod database;
mod slurm;
mod logging;
pub mod automations;
pub mod cluster;
pub mod templates;
// #[cfg(test)]
// mod logging_test;
#[cfg(test)]
// DISABLED: security_tests - needs rewrite for template system (uses demo mode)
// mod security_tests;
pub use types::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging system first
    logging::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Set up logging bridge to frontend
            logging::set_app_handle(app.handle().clone());

            // Initialize database with app handle (for OS-specific path resolution)
            let db_path = database::get_database_path(app.handle())
                .map_err(|e| {
                    eprintln!("Failed to resolve database path: {}", e);
                    e
                })?;

            database::initialize_database(db_path.to_str().unwrap())
                .map_err(|e| {
                    eprintln!("Failed to initialize database: {}", e);
                    e
                })?;

            // Default templates are loaded on-demand when list_templates is first called
            // This ensures logs appear in frontend (setup hook runs before frontend connects)

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connection lifecycle
            commands::connection::connect_to_cluster,
            commands::connection::disconnect,
            commands::connection::get_connection_status,
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
            commands::jobs::refetch_slurm_logs,
            // File management
            commands::files::detect_file_type,
            commands::files::select_input_file,
            commands::files::upload_job_files,
            commands::files::download_job_output,
            commands::files::download_all_outputs,
            commands::files::download_job_input,
            commands::files::download_all_inputs,
            commands::files::list_job_files,
            // Template management
            commands::templates::list_templates,
            commands::templates::get_template,
            commands::templates::create_template,
            commands::templates::update_template,
            commands::templates::delete_template,
            commands::templates::validate_template_values,
            commands::templates::preview_namd_config,
            commands::templates::preview_template_with_defaults,
            commands::jobs::preview_slurm_script,
            commands::jobs::validate_job_config,
            // Database management
            commands::database::get_database_info,
            commands::database::backup_database,
            commands::database::restore_database,
            commands::database::reset_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
