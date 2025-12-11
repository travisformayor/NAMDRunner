use crate::types::*;
use crate::types::response_data::AppInitializationData;
use crate::types::core::AppLogMessage;
use crate::{log_info, log_error};

/// Get recent logs from the buffer (for event sourcing)
#[tauri::command]
pub fn get_recent_logs() -> Vec<AppLogMessage> {
    crate::logging::get_recent_logs()
}

/// Initialize application - loads all startup data in one call
#[tauri::command]
pub async fn initialize_app() -> ApiResult<AppInitializationData> {
    log_info!(category: "App", message: "Initializing application");

    // Ensure default cluster config is loaded
    if let Err(e) = crate::database::ensure_default_cluster_config_loaded() {
        log_error!(category: "Initialization", message: "Failed to ensure default cluster config", details: "{}", e);
    }

    // Load cluster config from DB and cache it - fail fast if missing
    let capabilities = match crate::database::with_database(|db| db.load_cluster_config()) {
        Ok(Some(config)) => {
            // Cache the loaded config for validation and other operations
            crate::cluster::set_cluster_config_cache(config.clone());
            config
        }
        Ok(None) => {
            log_error!(category: "Initialization", message: "Cluster config not found in database");
            return ApiResult::error("Cluster configuration not found in database. Try resetting the database.".to_string());
        }
        Err(e) => {
            log_error!(category: "Initialization", message: "Failed to load cluster config", details: "{}", e);
            return ApiResult::error(format!("Failed to load cluster configuration: {}", e));
        }
    };

    // Ensure default templates are loaded
    if let Err(e) = crate::database::ensure_default_templates_loaded() {
        log_error!(category: "Initialization", message: "Failed to ensure default templates", details: "{}", e);
    }

    // Load templates with error handling
    let templates = match crate::database::with_database(|db| db.list_templates()) {
        Ok(t) => t,
        Err(e) => {
            log_error!(category: "Initialization", message: "Templates unavailable", details: "Database error: {}", e, show_toast: true);
            vec![]
        }
    };

    // Load jobs with error handling
    let jobs = match crate::database::with_database(|db| db.load_all_jobs()) {
        Ok(j) => j,
        Err(e) => {
            log_error!(category: "Initialization", message: "Jobs unavailable", details: "Database error: {}", e, show_toast: true);
            vec![]
        }
    };

    log_info!(category: "App", message: "Initialization complete", details: "{} templates, {} jobs", templates.len(), jobs.len());

    ApiResult::success(AppInitializationData {
        capabilities,
        templates,
        jobs,
    })
}

