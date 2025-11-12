use anyhow::{Result, anyhow};
use chrono::Utc;
use crate::types::{JobInfo, JobStatus};
use crate::database::with_database;
use crate::ssh::ConnectionManager;
use crate::error_log;

/// Save job to database with error handling
/// Clones job internally to satisfy database closure requirements
pub fn save_job_to_database(job: &JobInfo, context: &str) -> Result<()> {
    let job_clone = job.clone();

    with_database(move |db| db.save_job(&job_clone))
        .map_err(|e| {
            error_log!("[{}] Failed to save job to database: {}", context, e);
            anyhow!("Failed to save job to database: {}", e)
        })
}

/// Require connection and get username in one call
/// Common pattern in automation workflows
pub async fn require_connection_with_username(context: &str) -> Result<(&'static ConnectionManager, String)> {
    let connection_manager = crate::ssh::get_connection_manager();

    if !connection_manager.is_connected().await {
        error_log!("[{}] SSH connection not active", context);
        return Err(anyhow!("Not connected to cluster"));
    }

    let username = connection_manager.get_username().await
        .map_err(|e| {
            error_log!("[{}] Failed to get username: {}", context, e);
            anyhow!("Failed to get cluster username: {}", e)
        })?;

    Ok((connection_manager, username))
}

/// Get required project directory from job
/// Returns error if project_dir is None
pub fn require_project_dir<'a>(job: &'a JobInfo, context: &str) -> Result<&'a str> {
    job.project_dir.as_ref()
        .ok_or_else(|| {
            error_log!("[{}] Job {} has no project directory", context, job.job_id);
            anyhow!("Job has no project directory")
        })
        .map(|s| s.as_str())
}

/// Get required scratch directory from job
/// Returns error if scratch_dir is None
pub fn require_scratch_dir<'a>(job: &'a JobInfo, context: &str) -> Result<&'a str> {
    job.scratch_dir.as_ref()
        .ok_or_else(|| {
            error_log!("[{}] Job {} has no scratch directory", context, job.job_id);
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
                partition: Some("amilan".to_string()),
                qos: Some("normal".to_string()),
            },
            output_files: None,
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
}
