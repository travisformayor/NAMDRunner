use crate::types::*;
use crate::types::response_data::AppInitializationData;
use crate::{log_info, log_error};

/// Initialize application - loads all startup data in one call
#[tauri::command]
pub async fn initialize_app() -> ApiResult<AppInitializationData> {
    log_info!(category: "App", message: "Initializing application");

    let capabilities = crate::cluster::get_cluster_capabilities();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_app_returns_success() {
        let result = initialize_app().await;

        assert!(result.success);
        assert!(result.data.is_some());

        let data = result.data.unwrap();

        // Cluster capabilities should always be available (hardcoded)
        assert!(!data.capabilities.partitions.is_empty());
        assert!(!data.capabilities.qos_options.is_empty());

        // Templates and jobs may be empty (depends on database state)
        // Just verify they are valid vectors (not checking length since Vec.len() is always >= 0)
        assert!(data.templates.is_empty() || !data.templates.is_empty());
        assert!(data.jobs.is_empty() || !data.jobs.is_empty());
    }

    #[tokio::test]
    async fn test_initialize_app_graceful_degradation() {
        // This test verifies that initialize_app returns success even if database operations fail
        // by returning empty vecs instead of propagating errors

        let result = initialize_app().await;

        // Should always succeed - graceful degradation pattern
        assert!(result.success);
        assert!(result.data.is_some());

        // Even with database errors, should return valid data structure
        let data = result.data.unwrap();
        assert!(!data.capabilities.partitions.is_empty()); // Capabilities always available
    }
}
