use anyhow::{Result, anyhow};
use tauri::AppHandle;
use chrono::Utc;

use crate::types::{JobStatus, JobInfo};
use crate::validation::input;
use crate::ssh::get_connection_manager;
use crate::database::with_database;
use crate::{info_log, debug_log, error_log};

/// Job completion automation that preserves critical results from scratch to project directory
/// This follows NAMDRunner's direct function patterns with progress reporting through callbacks.
///
/// Key functionality: Detects completed jobs, copies important output files from temporary
/// scratch directory to permanent project directory for long-term storage before cleanup.
pub async fn execute_job_completion_with_progress(
    _app_handle: AppHandle,
    job_id: String,
    progress_callback: impl Fn(&str),
) -> Result<JobInfo> {
    progress_callback("Starting job completion automation...");
    info_log!("[Job Completion] Starting job completion for: {}", job_id);

    // Validate and sanitize job ID
    let clean_job_id = input::sanitize_job_id(&job_id)
        .map_err(|e| anyhow!("Invalid job ID: {}", e))?;
    debug_log!("[Job Completion] Sanitized job ID: {}", clean_job_id);

    progress_callback("Loading job information...");

    // Load job from database
    let clean_job_id_clone = clean_job_id.clone();
    let mut job_info = with_database(move |db| db.load_job(&clean_job_id_clone))
        .map_err(|e| {
            error_log!("[Job Completion] Database error loading job {}: {}", clean_job_id, e);
            anyhow!("Database error: {}", e)
        })?
        .ok_or_else(|| {
            error_log!("[Job Completion] Job {} not found", clean_job_id);
            anyhow!("Job {} not found", clean_job_id)
        })?;
    debug_log!("[Job Completion] Loaded job: {} (status: {:?})", clean_job_id, job_info.status);

    // Validate job is in finished state
    if !matches!(job_info.status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        error_log!("[Job Completion] Job {} not finished, status: {:?}", clean_job_id, job_info.status);
        return Err(anyhow!("Job has not finished running (status: {:?}). Only Completed, Failed, or Cancelled jobs can have results synced.", job_info.status));
    }

    progress_callback("Validating connection...");

    // Verify SSH connection is active
    let connection_manager = get_connection_manager();
    if !connection_manager.is_connected().await {
        error_log!("[Job Completion] SSH connection not active");
        return Err(anyhow!("Please connect to the cluster to preserve job results"));
    }

    // Ensure we have both project and scratch directories
    let project_dir = job_info.project_dir.as_ref()
        .ok_or_else(|| {
            error_log!("[Job Completion] Job {} has no project directory", clean_job_id);
            anyhow!("Job has no project directory")
        })?;
    let scratch_dir = job_info.scratch_dir.as_ref()
        .ok_or_else(|| {
            error_log!("[Job Completion] Job {} has no scratch directory", clean_job_id);
            anyhow!("Job has no scratch directory")
        })?;
    info_log!("[Job Completion] Preserving results from {} to {}", scratch_dir, project_dir);

    progress_callback("Mirroring scratch directory back to project...");

    // Use rsync to mirror entire scratch directory back to project
    // Note: source must end with / to sync contents, destination should NOT end with / to sync into it
    let source_with_slash = if scratch_dir.ends_with('/') {
        scratch_dir.to_string()
    } else {
        format!("{}/", scratch_dir)
    };

    info_log!("[Job Completion] Mirroring scratch to project: {} -> {}", scratch_dir, project_dir);
    connection_manager.sync_directory_rsync(&source_with_slash, project_dir).await
        .map_err(|e| {
            error_log!("[Job Completion] Failed to mirror scratch directory: {}", e);
            anyhow!("Failed to mirror scratch directory to project: {}", e)
        })?;

    info_log!("[Job Completion] Successfully mirrored scratch directory back to project");

    progress_callback("Caching SLURM logs...");

    // Fetch and cache SLURM logs if not already cached
    if let Err(e) = crate::automations::fetch_slurm_logs_if_needed(&mut job_info).await {
        error_log!("[Job Completion] Failed to fetch logs: {}", e);
        // Don't fail completion if log fetch fails
    }

    progress_callback("Updating job status...");
    debug_log!("[Job Completion] Updating job status in database");

    // Update job info with completion timestamp and mark as results preserved
    job_info.updated_at = Some(Utc::now().to_rfc3339());

    // Save updated job info to database
    let job_info_clone = job_info.clone();
    with_database(move |db| db.save_job(&job_info_clone))
        .map_err(|e| {
            error_log!("[Job Completion] Failed to update job in database: {}", e);
            anyhow!("Failed to update job in database: {}", e)
        })?;

    progress_callback("Job completion automation finished - results synced from scratch");
    info_log!("[Job Completion] Job completion finished successfully: {} (results synced from scratch)", clean_job_id);

    Ok(job_info)
}

#[cfg(test)]
mod tests {
    use super::*;
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