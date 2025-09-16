
/// Trait for validating and extracting ID parameters
pub trait ValidateId {
    /// Validate and sanitize an ID parameter
    fn validate_id(&self) -> anyhow::Result<String>;
}

/// Implementation for job ID strings
impl ValidateId for String {
    fn validate_id(&self) -> anyhow::Result<String> {
        if self.trim().is_empty() {
            anyhow::bail!("Job ID is required");
        }

        crate::validation::input::sanitize_job_id(self)
            .map_err(|e| anyhow::anyhow!("Invalid job ID: {}", e))
    }
}

impl ValidateId for &str {
    fn validate_id(&self) -> anyhow::Result<String> {
        if self.trim().is_empty() {
            anyhow::bail!("Job ID is required");
        }

        crate::validation::input::sanitize_job_id(self)
            .map_err(|e| anyhow::anyhow!("Invalid job ID: {}", e))
    }
}