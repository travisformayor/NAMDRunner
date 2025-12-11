use anyhow::{Result, anyhow};
use crate::types::{JobInfo, ApiResult};
use crate::templates::Template;
use crate::database::{with_database, get_current_database_path};
use crate::security::input;
use crate::log_error;

/// Load job from database or return error
/// Validates job ID and handles database errors
/// Returns JobInfo or descriptive error
pub fn load_job_or_fail(job_id: &str, context: &str) -> Result<JobInfo> {
    let job_id_owned = job_id.to_string();

    with_database(move |db| db.load_job(&job_id_owned))
        .map_err(|e| {
            log_error!(category: context, message: "Database error", details: "{}", e);
            anyhow!("Database error: {}", e)
        })?
        .ok_or_else(|| {
            log_error!(category: context, message: "Job not found", details: "Job ID: {}", job_id);
            anyhow!("Job '{}' not found", job_id)
        })
}

/// Load template from database or return error
/// Returns Template or descriptive error
pub fn load_template_or_fail(template_id: &str, context: &str) -> Result<Template> {
    let template_id_owned = template_id.to_string();

    with_database(move |db| db.load_template(&template_id_owned))
        .map_err(|e| {
            log_error!(category: context, message: "Database error", details: "{}", e);
            anyhow!("Database error: {}", e)
        })?
        .ok_or_else(|| {
            log_error!(category: context, message: "Template not found", details: "Template ID: {}", template_id);
            anyhow!("Template '{}' not found", template_id)
        })
}

/// Sanitize and validate job_id from command input
/// Returns ApiResult with sanitized job_id on success, error on validation failure
pub fn sanitize_command_job_id(job_id: &str) -> ApiResult<String> {
    match input::sanitize_job_id(job_id) {
        Ok(clean_id) => ApiResult::success(clean_id),
        Err(error) => ApiResult::error(error.to_string()),
    }
}

/// Get database path or return ApiResult error
/// Used by commands that need to return database path
pub fn get_database_path_or_fail() -> ApiResult<String> {
    match get_current_database_path() {
        Some(path) => ApiResult::success(path.to_string_lossy().to_string()),
        None => ApiResult::error("Database not initialized".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_job_or_fail_with_nonexistent_job() {
        let result = load_job_or_fail("nonexistent_job_id", "Test");
        assert!(result.is_err());
        // Error could be "not found" or "database error" depending on state
    }

    #[test]
    fn test_load_template_or_fail_with_nonexistent_template() {
        let result = load_template_or_fail("nonexistent_template", "Test");
        assert!(result.is_err());
    }
}
