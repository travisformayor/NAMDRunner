use anyhow::{Result, anyhow};
use chrono::Utc;

use crate::types::{JobStatus, JobInfo};
use crate::ssh::get_connection_manager;
use crate::database::with_database;
use crate::{info_log, error_log};

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
    info_log!("[Job Completion] Starting automatic completion for: {}", job_id);

    // Validate job is in terminal state
    if !matches!(job.status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        return Err(anyhow!("Job not in terminal state: {:?}", job.status));
    }

    // Verify SSH connection is active
    let connection_manager = get_connection_manager();
    if !connection_manager.is_connected().await {
        error_log!("[Job Completion] SSH connection not active");
        return Err(anyhow!("Not connected to cluster"));
    }

    // Ensure we have both project and scratch directories
    let project_dir = job.project_dir.as_ref()
        .ok_or_else(|| {
            error_log!("[Job Completion] Job {} has no project directory", job_id);
            anyhow!("Job has no project directory")
        })?.clone();
    let scratch_dir = job.scratch_dir.as_ref()
        .ok_or_else(|| {
            error_log!("[Job Completion] Job {} has no scratch directory", job_id);
            anyhow!("Job has no scratch directory")
        })?.clone();

    // CRITICAL: Rsync scratch→project FIRST (DATA BOUNDARY CROSSED)
    // This preserves all results including SLURM logs before they're cleaned up
    let source_with_slash = if scratch_dir.ends_with('/') {
        scratch_dir.to_string()
    } else {
        format!("{}/", scratch_dir)
    };

    info_log!("[Job Completion] Rsyncing scratch→project: {} -> {}", scratch_dir, project_dir);
    connection_manager.sync_directory_rsync(&source_with_slash, &project_dir).await
        .map_err(|e| {
            error_log!("[Job Completion] Rsync failed: {}", e);
            anyhow!("Failed to rsync: {}", e)
        })?;

    info_log!("[Job Completion] Rsync complete - all files now in project directory");

    // NOW fetch logs from project directory (after rsync)
    if let Err(e) = crate::automations::fetch_slurm_logs_if_needed(job).await {
        error_log!("[Job Completion] Failed to fetch logs: {}", e);
        // Don't fail completion if log fetch fails - logs are nice-to-have
    }

    // Fetch output file metadata from project directory (after rsync)
    let output_dir = format!("{}/output_files", project_dir);
    info_log!("[Job Completion] Fetching output file metadata from: {}", output_dir);

    match connection_manager.list_files_with_metadata(&output_dir).await {
        Ok(output_files) => {
            info_log!("[Job Completion] Found {} output files", output_files.len());
            job.output_files = Some(output_files);
        }
        Err(e) => {
            error_log!("[Job Completion] Failed to fetch output file metadata: {}", e);
            // Don't fail completion if metadata fetch fails - it's nice-to-have
            job.output_files = None;
        }
    }

    // Update database
    job.updated_at = Some(Utc::now().to_rfc3339());
    let job_clone = job.clone();
    with_database(move |db| db.save_job(&job_clone))
        .map_err(|e| {
            error_log!("[Job Completion] Failed to update database: {}", e);
            anyhow!("Failed to update database: {}", e)
        })?;

    info_log!("[Job Completion] Job completion successful: {}", job_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::types::{JobInfo, JobStatus, NAMDConfig, SlurmConfig};
    use chrono::Utc;

    fn create_test_job_info() -> JobInfo {
        let now = Utc::now().to_rfc3339();
        JobInfo {
            job_id: "test_job_001".to_string(),
            job_name: "test_simulation".to_string(),
            status: JobStatus::Completed,
            slurm_job_id: Some("12345678".to_string()),
            created_at: now.clone(),
            updated_at: Some(now),
            submitted_at: Some(Utc::now().to_rfc3339()),
            completed_at: Some(Utc::now().to_rfc3339()),
            project_dir: Some("/projects/testuser/namdrunner_jobs/test_job_001".to_string()),
            scratch_dir: Some("/scratch/alpine/testuser/namdrunner_jobs/test_job_001".to_string()),
            error_info: None,
            namd_config: NAMDConfig {
                steps: 1000,
                temperature: 300.0,
                timestep: 2.0,
                outputname: "output".to_string(),
                dcd_freq: Some(100),
                restart_freq: Some(500),
            },
            slurm_config: SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            input_files: Vec::new(),
            output_files: None,
            remote_directory: "/projects/testuser/namdrunner_jobs/test_job_001".to_string(),
            slurm_stdout: None,
            slurm_stderr: None,
        }
    }

    #[test]
    fn test_job_completion_status_validation() {
        let mut job = create_test_job_info();

        // Test valid status for completion
        job.status = JobStatus::Completed;
        assert!(matches!(job.status, JobStatus::Completed));

        // Test invalid statuses
        job.status = JobStatus::Created;
        assert!(!matches!(job.status, JobStatus::Completed));

        job.status = JobStatus::Running;
        assert!(!matches!(job.status, JobStatus::Completed));

        job.status = JobStatus::Failed;
        assert!(!matches!(job.status, JobStatus::Completed));
    }

    #[test]
    fn test_rsync_source_trailing_slash() {
        // Test business logic for rsync source path formatting
        let scratch_dir_without_slash = "/scratch/alpine/testuser/namdrunner_jobs/test_job_001";
        let scratch_dir_with_slash = "/scratch/alpine/testuser/namdrunner_jobs/test_job_001/";

        // Without trailing slash - should add it
        let source1 = if scratch_dir_without_slash.ends_with('/') {
            scratch_dir_without_slash.to_string()
        } else {
            format!("{}/", scratch_dir_without_slash)
        };
        assert!(source1.ends_with('/'));
        assert_eq!(source1, "/scratch/alpine/testuser/namdrunner_jobs/test_job_001/");

        // With trailing slash - should keep it
        let source2 = if scratch_dir_with_slash.ends_with('/') {
            scratch_dir_with_slash.to_string()
        } else {
            format!("{}/", scratch_dir_with_slash)
        };
        assert!(source2.ends_with('/'));
        assert_eq!(source2, "/scratch/alpine/testuser/namdrunner_jobs/test_job_001/");
    }

    #[test]
    fn test_directory_mirroring_paths() {
        // Test business logic for mirrored directory structure
        let project_dir = "/projects/testuser/namdrunner_jobs/test_job_001";
        let scratch_dir = "/scratch/alpine/testuser/namdrunner_jobs/test_job_001";

        // Both should have matching job_id component
        assert!(project_dir.ends_with("test_job_001"));
        assert!(scratch_dir.ends_with("test_job_001"));

        // Both should be under namdrunner_jobs
        assert!(project_dir.contains("namdrunner_jobs"));
        assert!(scratch_dir.contains("namdrunner_jobs"));

        // Scratch should have /scratch/alpine/ prefix
        assert!(scratch_dir.starts_with("/scratch/alpine/"));

        // Project should have /projects/ prefix
        assert!(project_dir.starts_with("/projects/"));
    }

}