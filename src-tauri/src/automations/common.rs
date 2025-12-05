use anyhow::{Result, anyhow};
use chrono::Utc;
use crate::types::{JobInfo, JobStatus};
use crate::database::with_database;
use crate::ssh::ConnectionManager;
use crate::log_error;

/// Save job to database with error handling
/// Clones job internally to satisfy database closure requirements
pub fn save_job_to_database(job: &JobInfo, context: &str) -> Result<()> {
    let job_clone = job.clone();

    with_database(move |db| db.save_job(&job_clone))
        .map_err(|e| {
            log_error!(category: context, message: "Failed to save job to database", details: "{}", e);
            anyhow!("Failed to save job to database: {}", e)
        })
}

/// Require connection and get username in one call
/// Common pattern in automation workflows
pub async fn require_connection_with_username(context: &str) -> Result<(&'static ConnectionManager, String)> {
    let connection_manager = crate::ssh::get_connection_manager();

    if !connection_manager.is_connected().await {
        log_error!(category: context, message: "SSH connection not active");
        return Err(anyhow!("Not connected to cluster"));
    }

    let username = connection_manager.get_username().await
        .map_err(|e| {
            log_error!(category: context, message: "Failed to get username", details: "{}", e);
            anyhow!("Failed to get cluster username: {}", e)
        })?;

    Ok((connection_manager, username))
}

/// Get required project directory from job
/// Returns error if project_dir is None
pub fn require_project_dir<'a>(job: &'a JobInfo, context: &str) -> Result<&'a str> {
    job.project_dir.as_ref()
        .ok_or_else(|| {
            log_error!(category: context, message: "Job has no project directory", details: "{}", job.job_id);
            anyhow!("Job has no project directory")
        })
        .map(|s| s.as_str())
}

/// Get required scratch directory from job
/// Returns error if scratch_dir is None
pub fn require_scratch_dir<'a>(job: &'a JobInfo, context: &str) -> Result<&'a str> {
    job.scratch_dir.as_ref()
        .ok_or_else(|| {
            log_error!(category: context, message: "Job has no scratch directory", details: "{}", job.job_id);
            anyhow!("Job has no scratch directory")
        })
        .map(|s| s.as_str())
}

/// Update job status and timestamps
/// Sets updated_at, and sets completed_at for terminal statuses
pub fn update_job_status(job: &mut JobInfo, new_status: JobStatus) {
    job.status = new_status.clone();
    job.updated_at = Some(Utc::now().to_rfc3339());

    if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        job.completed_at = Some(Utc::now().to_rfc3339());
    }
}

/// Update job timestamp without changing status
/// Used when job state doesn't change but job metadata is updated (e.g., log refresh, file sync)
pub fn touch_job_timestamp(job: &mut JobInfo) {
    job.updated_at = Some(Utc::now().to_rfc3339());
}

/// Ensure path has trailing slash for rsync operations
/// rsync requires trailing slash on source directory
pub fn ensure_trailing_slash(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{}/", path)
    }
}

/// Load job from database or return error
/// Common pattern in automation workflows
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_trailing_slash_adds_when_missing() {
        assert_eq!(ensure_trailing_slash("/path/to/dir"), "/path/to/dir/");
    }

    #[test]
    fn test_ensure_trailing_slash_preserves_when_present() {
        assert_eq!(ensure_trailing_slash("/path/to/dir/"), "/path/to/dir/");
    }

    #[test]
    fn test_update_job_status_sets_timestamps() {
        let mut job = JobInfo {
            job_id: "test_job".to_string(),
            job_name: "Test".to_string(),
            status: JobStatus::Created,
            slurm_job_id: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            submitted_at: None,
            completed_at: None,
            project_dir: None,
            scratch_dir: None,
            error_info: None,
            slurm_stdout: None,
            slurm_stderr: None,
            template_id: "test".to_string(),
            template_values: std::collections::HashMap::new(),
            slurm_config: crate::types::SlurmConfig {
                cores: 1,
                memory: "1GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: "amilan".to_string(),
                qos: "normal".to_string(),
            },
            input_files: vec![],
            output_files: vec![],
            remote_directory: "/test/dir".to_string(),
        };

        update_job_status(&mut job, JobStatus::Running);
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.updated_at.is_some());
        assert!(job.completed_at.is_none());

        update_job_status(&mut job, JobStatus::Completed);
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_touch_job_timestamp_updates_timestamp_only() {
        let mut job = JobInfo {
            job_id: "test_job".to_string(),
            job_name: "Test".to_string(),
            status: JobStatus::Running,
            slurm_job_id: Some("12345".to_string()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            submitted_at: Some(Utc::now().to_rfc3339()),
            completed_at: None,
            project_dir: Some("/test/project".to_string()),
            scratch_dir: Some("/test/scratch".to_string()),
            error_info: None,
            slurm_stdout: None,
            slurm_stderr: None,
            template_id: "test".to_string(),
            template_values: std::collections::HashMap::new(),
            slurm_config: crate::types::SlurmConfig {
                cores: 4,
                memory: "16GB".to_string(),
                walltime: "04:00:00".to_string(),
                partition: "amilan".to_string(),
                qos: "normal".to_string(),
            },
            input_files: vec![],
            output_files: vec![],
            remote_directory: "/test/dir".to_string(),
        };

        // Record original state
        let original_status = job.status.clone();
        let original_completed_at = job.completed_at.clone();

        // Call touch_job_timestamp
        touch_job_timestamp(&mut job);

        // Verify only updated_at changed
        assert!(job.updated_at.is_some(), "updated_at should be set");
        assert_eq!(job.status, original_status, "status should not change");
        assert_eq!(job.completed_at, original_completed_at, "completed_at should not change");
    }

    #[test]
    fn test_touch_job_timestamp_on_completed_job() {
        let mut job = JobInfo {
            job_id: "completed_job".to_string(),
            job_name: "Test".to_string(),
            status: JobStatus::Completed,
            slurm_job_id: Some("12345".to_string()),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Some(Utc::now().to_rfc3339()),
            submitted_at: Some(Utc::now().to_rfc3339()),
            completed_at: Some(Utc::now().to_rfc3339()),
            project_dir: Some("/test/project".to_string()),
            scratch_dir: Some("/test/scratch".to_string()),
            error_info: None,
            slurm_stdout: None,
            slurm_stderr: None,
            template_id: "test".to_string(),
            template_values: std::collections::HashMap::new(),
            slurm_config: crate::types::SlurmConfig {
                cores: 4,
                memory: "16GB".to_string(),
                walltime: "04:00:00".to_string(),
                partition: "amilan".to_string(),
                qos: "normal".to_string(),
            },
            input_files: vec![],
            output_files: vec![],
            remote_directory: "/test/dir".to_string(),
        };

        // Record original state
        let old_completed_at = job.completed_at.clone();

        // Touch timestamp
        touch_job_timestamp(&mut job);

        // Verify completed_at is NOT changed (only updated_at changes)
        assert_eq!(job.completed_at, old_completed_at, "completed_at should not be overwritten");
        assert_eq!(job.status, JobStatus::Completed, "status should remain Completed");
    }
}
