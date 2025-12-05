use anyhow::{Result, anyhow};
use crate::types::JobInfo;
use crate::templates::Template;
use crate::database::with_database;
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
