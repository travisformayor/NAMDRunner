use anyhow::{Result, anyhow};
use crate::types::{JobStatus, JobInfo};
use crate::{log_info, log_error};
use crate::automations::common;

/// Execute job completion automation (called automatically when job reaches terminal state)
///
/// This function:
/// 1. Rsyncs all files from scratch directory to project directory (DATA BOUNDARY CROSSED)
/// 2. Fetches SLURM logs from project directory (after rsync)
/// 3. Updates database with final state
///
/// Called automatically by job_sync when a job reaches terminal state (Completed, Failed, etc.)
pub async fn execute_job_completion_internal(job: &mut JobInfo) -> Result<()> {
    let job_id = job.job_id.clone();
    log_info!(category: "Job Completion", message: "Starting automatic completion", details: "{}", job_id);

    // Validate job is in terminal state
    if !matches!(job.status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        return Err(anyhow!("Job not in terminal state: {:?}", job.status));
    }

    // Verify SSH connection is active
    let (connection_manager, _username) = common::require_connection_with_username("Job Completion").await?;

    // Ensure we have both project and scratch directories
    let project_dir = common::require_project_dir(job, "Job Completion")?.to_string();
    let scratch_dir = common::require_scratch_dir(job, "Job Completion")?.to_string();

    // CRITICAL: Rsync scratch→project FIRST (DATA BOUNDARY CROSSED)
    // This preserves all results including SLURM logs before they're cleaned up
    let source_with_slash = common::ensure_trailing_slash(&scratch_dir);

    log_info!(category: "Job Completion", message: "Rsyncing scratch to project", details: "{} -> {}", scratch_dir, project_dir);
    connection_manager.sync_directory_rsync(&source_with_slash, &project_dir).await
        .map_err(|e| {
            log_error!(category: "Job Completion", message: "Rsync failed", details: "{}", e);
            anyhow!("Failed to rsync: {}", e)
        })?;

    log_info!(category: "Job Completion", message: "Rsync complete - all files now in project directory");

    // Fetch logs from project directory (after rsync)
    if let Err(e) = crate::automations::fetch_slurm_logs_if_needed(job).await {
        log_error!(category: "Job Completion", message: "Failed to fetch logs", details: "{}", e);
        // Don't fail completion if log fetch fails - logs are nice-to-have
    }

    // Fetch output file metadata from project directory (after rsync)
    let output_dir = format!("{}/outputs", project_dir);
    log_info!(category: "Job Completion", message: "Fetching output file metadata", details: "{}", output_dir);

    match connection_manager.list_files_with_metadata(&output_dir).await {
        Ok(output_files) => {
            log_info!(category: "Job Completion", message: "Found output files", details: "{} files", output_files.len());
            job.output_files = Some(output_files);
        }
        Err(e) => {
            log_error!(category: "Job Completion", message: "Failed to fetch output file metadata", details: "{}", e);
            // Don't fail completion if metadata fetch fails - it's nice-to-have
            job.output_files = None;
        }
    }

    // Update database with timestamp
    common::touch_job_timestamp(job);
    common::save_job_to_database(job, "Job Completion")?;

    log_info!(category: "Job Completion", message: "Job completion successful", details: "{}", job_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rsync_source_trailing_slash() {
        use crate::ssh::directory_structure::JobDirectoryStructure;
        // Test business logic for rsync source path formatting using centralized paths
        let scratch_dir_without_slash = JobDirectoryStructure::scratch_dir("testuser", "test_job_001");
        let scratch_dir_with_slash = format!("{}/", scratch_dir_without_slash);

        // Without trailing slash - should add it
        let source1 = if scratch_dir_without_slash.ends_with('/') {
            scratch_dir_without_slash.to_string()
        } else {
            format!("{}/", scratch_dir_without_slash)
        };
        assert!(source1.ends_with('/'));
        assert_eq!(source1, format!("{}/", JobDirectoryStructure::scratch_dir("testuser", "test_job_001")));

        // With trailing slash - should keep it
        let source2 = if scratch_dir_with_slash.ends_with('/') {
            scratch_dir_with_slash.to_string()
        } else {
            format!("{}/", scratch_dir_with_slash)
        };
        assert!(source2.ends_with('/'));
        assert_eq!(source2, format!("{}/", JobDirectoryStructure::scratch_dir("testuser", "test_job_001")));
    }

    #[test]
    fn test_directory_mirroring_paths() {
        use crate::ssh::directory_structure::JobDirectoryStructure;
        // Test business logic for mirrored directory structure using centralized paths
        let project_dir = JobDirectoryStructure::project_dir("testuser", "test_job_001");
        let scratch_dir = JobDirectoryStructure::scratch_dir("testuser", "test_job_001");

        // Both should have matching job_id component
        assert!(project_dir.ends_with("test_job_001"));
        assert!(scratch_dir.ends_with("test_job_001"));

        // Both should be under namdrunner_jobs
        assert!(project_dir.contains(crate::ssh::directory_structure::JOB_BASE_DIRECTORY));
        assert!(scratch_dir.contains(crate::ssh::directory_structure::JOB_BASE_DIRECTORY));

        // Scratch should have /scratch/alpine/ prefix
        assert!(scratch_dir.starts_with("/scratch/alpine/"));

        // Project should have /projects/ prefix
        assert!(project_dir.starts_with("/projects/"));
    }

}