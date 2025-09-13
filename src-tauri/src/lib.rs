mod types;
pub mod commands;
mod mock_state;

pub use types::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Connection management
            commands::connection::connect_to_cluster,
            commands::connection::disconnect,
            commands::connection::get_connection_status,
            // Job management
            commands::jobs::create_job,
            commands::jobs::submit_job,
            commands::jobs::get_job_status,
            commands::jobs::get_all_jobs,
            commands::jobs::sync_jobs,
            commands::jobs::delete_job,
            // File management
            commands::files::upload_job_files,
            commands::files::download_job_output,
            commands::files::list_job_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
