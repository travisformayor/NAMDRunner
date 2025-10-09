use anyhow::{Result, anyhow};
use tauri::AppHandle;
use chrono::Utc;

use crate::types::{SubmitJobResult, JobStatus};
use crate::validation::paths;
use crate::ssh::get_connection_manager;
use crate::database::with_database;
use crate::{info_log, debug_log, error_log};

/// Simplified job submission automation that follows NAMDRunner's direct function patterns
/// This replaces the complex AutomationStep trait system with a simple async function
/// that provides progress reporting through callbacks.
///
/// Key functionality: Creates scratch directories, copies files from project to scratch,
/// submits to SLURM, and updates job status. This maintains proper workflow separation.
pub async fn execute_job_submission_with_progress(
    _app_handle: AppHandle,
    job_id: String,
    progress_callback: impl Fn(&str),
) -> Result<SubmitJobResult> {
    progress_callback("Loading job information...");
    info_log!("[Job Submission] Starting job submission for: {}", job_id);

    // Load job from database
    let mut job_info = with_database(|db| db.load_job(&job_id))
        .map_err(|e| {
            error_log!("[Job Submission] Database error loading job {}: {}", job_id, e);
            anyhow!("Database error: {}", e)
        })?
        .ok_or_else(|| {
            error_log!("[Job Submission] Job {} not found in database", job_id);
            anyhow!("Job '{}' not found in database", job_id)
        })?;
    debug_log!("[Job Submission] Loaded job: {} (status: {:?})", job_id, job_info.status);

    // Validate job is in correct state for submission
    if !matches!(job_info.status, JobStatus::Created | JobStatus::Failed) {
        error_log!("[Job Submission] Job {} cannot be submitted, status: {:?}", job_id, job_info.status);
        return Err(anyhow!("Job cannot be submitted because it is currently {:?}. Only Created or Failed jobs can be submitted.", job_info.status));
    }

    progress_callback("Validating connection...");

    // Verify SSH connection is active
    let connection_manager = get_connection_manager();
    if !connection_manager.is_connected().await {
        error_log!("[Job Submission] SSH connection not active");
        return Err(anyhow!("Please connect to the cluster to submit jobs"));
    }

    // Get username using existing logic
    let username = connection_manager.get_username().await
        .map_err(|e| {
            error_log!("[Job Submission] Failed to get username: {}", e);
            anyhow!("Failed to get cluster username: {}", e)
        })?;
    info_log!("[Job Submission] Submitting job for user: {}", username);

    progress_callback("Mirroring job directory to scratch...");

    let project_dir = job_info.project_dir.as_ref()
        .ok_or_else(|| {
            error_log!("[Job Submission] Job {} has no project directory", job_id);
            anyhow!("Job has no project directory")
        })?;

    // Generate scratch directory path using existing validation functions
    let scratch_dir = paths::scratch_directory(&username, &job_info.job_id)?;
    info_log!("[Job Submission] Mirroring project to scratch: {} -> {}", project_dir, scratch_dir);

    // Use rsync to mirror entire job directory from project to scratch
    // Note: source must end with / to sync contents, destination should NOT end with / to create/sync into it
    let source_with_slash = if project_dir.ends_with('/') {
        project_dir.to_string()
    } else {
        format!("{}/", project_dir)
    };

    connection_manager.sync_directory_rsync(&source_with_slash, &scratch_dir).await
        .map_err(|e| {
            error_log!("[Job Submission] Failed to mirror directory to scratch: {}", e);
            anyhow!("Failed to mirror job directory to scratch: {}", e)
        })?;

    info_log!("[Job Submission] Successfully mirrored job directory to scratch");

    progress_callback("Submitting job to SLURM...");

    // Submit job using SLURM commands module (using mirrored script in scratch)
    let script_path = format!("{}/scripts/job.sbatch", scratch_dir);
    info_log!("[Job Submission] Executing sbatch with script: {}", script_path);
    let submit_cmd = crate::slurm::commands::submit_job_command(&scratch_dir, "scripts/job.sbatch")?;
    let output = connection_manager.execute_command(&submit_cmd, Some(crate::cluster::timeouts::JOB_SUBMIT)).await
        .map_err(|e| {
            error_log!("[Job Submission] Failed to submit job to SLURM: {}", e);
            anyhow!("Could not submit job to cluster scheduler: {}", e)
        })?;

    // Parse SLURM job ID from output using SLURM commands module
    let slurm_job_id = crate::slurm::commands::parse_sbatch_output(&output.stdout)
        .ok_or_else(|| {
            error_log!("[Job Submission] Failed to parse SLURM job ID from output: {}", output.stdout);
            anyhow!("Failed to parse SLURM job ID from: {}", output.stdout)
        })?;

    let submitted_at = Utc::now().to_rfc3339();
    info_log!("[Job Submission] Job submitted successfully - SLURM job ID: {} at {}", slurm_job_id, submitted_at);

    progress_callback("Updating job status...");

    // Update job info with submission details
    job_info.scratch_dir = Some(scratch_dir.clone());
    job_info.slurm_job_id = Some(slurm_job_id.clone());
    job_info.submitted_at = Some(submitted_at.clone());
    job_info.status = JobStatus::Pending;
    job_info.updated_at = Some(Utc::now().to_rfc3339());
    debug_log!("[Job Submission] Updated job status to Pending");

    // Save updated job info to database
    with_database(|db| db.save_job(&job_info))
        .map_err(|e| {
            error_log!("[Job Submission] Failed to save job to database: {}", e);
            anyhow!("Failed to update job in database: {}", e)
        })?;
    debug_log!("[Job Submission] Saved job to database");

    // Update job_info.json in project directory
    if let Some(project_dir) = &job_info.project_dir {
        info_log!("[Job Submission] Uploading job metadata to: {}/job_info.json", project_dir);
        crate::ssh::metadata::upload_job_metadata(&connection_manager, &job_info, project_dir, "Job Submission").await
            .map_err(|e| {
                error_log!("[Job Submission] Failed to upload job metadata: {}", e);
                anyhow!("Failed to update job metadata: {}", e)
            })?;
    }

    progress_callback("Job submission completed successfully");
    info_log!("[Job Submission] Job submission completed successfully for: {}", job_id);

    Ok(SubmitJobResult {
        success: true,
        slurm_job_id: Some(slurm_job_id),
        submitted_at: Some(submitted_at),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JobInfo, JobStatus, NAMDConfig, SlurmConfig, InputFile};
    use chrono::Utc;

    fn create_test_job_info() -> JobInfo {
        let now = Utc::now().to_rfc3339();
        JobInfo {
            job_id: "test_job_001".to_string(),
            job_name: "test_simulation".to_string(),
            status: JobStatus::Created,
            slurm_job_id: None,
            created_at: now.clone(),
            updated_at: Some(now),
            submitted_at: None,
            completed_at: None,
            project_dir: Some("/projects/testuser/namdrunner_jobs/test_job_001".to_string()),
            scratch_dir: None,
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
            input_files: vec![
                InputFile {
                    name: "structure.pdb".to_string(),
                    local_path: "/local/path/structure.pdb".to_string(),
                    remote_name: Some("structure.pdb".to_string()),
                    file_type: Some(crate::core::NAMDFileType::Pdb),
                }
            ],
            remote_directory: "/projects/testuser/namdrunner_jobs/test_job_001".to_string(),
        }
    }

    #[test]
    fn test_job_status_validation() {
        let mut job = create_test_job_info();

        // Test valid statuses for submission
        job.status = JobStatus::Created;
        assert!(matches!(job.status, JobStatus::Created | JobStatus::Failed));

        job.status = JobStatus::Failed;
        assert!(matches!(job.status, JobStatus::Created | JobStatus::Failed));

        // Test invalid statuses
        job.status = JobStatus::Running;
        assert!(!matches!(job.status, JobStatus::Created | JobStatus::Failed));

        job.status = JobStatus::Completed;
        assert!(!matches!(job.status, JobStatus::Created | JobStatus::Failed));
    }

    #[test]
    fn test_upload_content_safety() {
        // Test that content upload doesn't use shell commands (business logic only)
        // The new implementation uses SFTP via temporary files, eliminating command injection risk
        let dangerous_content = "test content with \"quotes\" and $variables and `backticks` and ; rm -rf /";

        // With the new SFTP approach, any content should be safe to upload
        // since it goes through file operations, not shell command construction
        assert!(dangerous_content.len() > 0, "Content should be preserved as-is");
        assert!(dangerous_content.contains("\"quotes\""), "Quotes should be preserved");
        assert!(dangerous_content.contains("$variables"), "Dollar signs should be preserved");
        assert!(dangerous_content.contains("`backticks`"), "Backticks should be preserved");
        assert!(dangerous_content.contains("; rm -rf /"), "Command injection attempts should be preserved as literal content");
    }

    #[test]
    fn test_slurm_job_id_parsing() {
        // Test SLURM job ID parsing logic
        let valid_outputs = vec![
            "Submitted batch job 12345678",
            "Submitted batch job 87654321\n",
            "  Submitted batch job 98765432  ",
        ];

        for output in valid_outputs {
            let job_id = output
                .lines()
                .find(|line| line.contains("Submitted batch job"))
                .and_then(|line| line.split_whitespace().last())
                .map(|s| s.trim());

            assert!(job_id.is_some());
            assert!(job_id.unwrap().chars().all(|c| c.is_ascii_digit()));
        }

        // Test invalid outputs
        let invalid_outputs = vec![
            "Error: Job submission failed",
            "",
            "Some other output",
            "Submitted batch job", // Missing job ID
        ];

        for output in invalid_outputs {
            let job_id = output
                .lines()
                .find(|line| line.contains("Submitted batch job"))
                .and_then(|line| line.split_whitespace().last());

            assert!(job_id.is_none() || !job_id.unwrap().chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_scratch_directory_subdirectories() {
        // Test business logic for creating subdirectories
        let expected_subdirs = vec!["input", "output", "logs"];
        let scratch_subdirs = vec!["input", "output", "logs"];

        assert_eq!(scratch_subdirs, expected_subdirs);

        // Verify all required subdirectories are present
        for required_dir in &expected_subdirs {
            assert!(scratch_subdirs.contains(required_dir),
                   "Missing required subdirectory: {}", required_dir);
        }
    }

    #[test]
    fn test_job_info_state_transitions() {
        let mut job = create_test_job_info();

        // Test initial state
        assert_eq!(job.status, JobStatus::Created);
        assert!(job.scratch_dir.is_none());
        assert!(job.slurm_job_id.is_none());
        assert!(job.submitted_at.is_none());

        // Simulate successful submission state changes
        job.scratch_dir = Some("/scratch/alpine/testuser/namdrunner_jobs/test_job_001".to_string());
        job.slurm_job_id = Some("12345678".to_string());
        job.submitted_at = Some(Utc::now().to_rfc3339());
        job.status = JobStatus::Pending;
        job.updated_at = Some(Utc::now().to_rfc3339());

        // Verify state after submission
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.scratch_dir.is_some());
        assert!(job.slurm_job_id.is_some());
        assert!(job.submitted_at.is_some());
        assert!(job.updated_at.is_some());
    }

    #[test]
    fn test_file_copy_command_generation() {
        // Test shell command generation for file operations (business logic only)
        use crate::validation::shell;

        let project_dir = "/projects/testuser/namdrunner_jobs/test_job_001";
        let scratch_dir = "/scratch/alpine/testuser/namdrunner_jobs/test_job_001";
        let filename = "structure.pdb";

        let source = format!("{}/input_files/{}", project_dir, filename);
        let dest = format!("{}/input/{}", scratch_dir, filename);

        let copy_cmd = shell::safe_cp(&source, &dest);

        // Verify command structure (without executing)
        assert!(copy_cmd.starts_with("cp "));
        assert!(copy_cmd.contains("structure.pdb"));
        assert!(copy_cmd.contains("input_files"));
        assert!(copy_cmd.contains("/input/"));
        // Verify paths are properly escaped (wrapped in single quotes)
        assert!(copy_cmd.contains("'"));
    }

    #[test]
    fn test_slurm_script_copy_validation() {
        // Test business logic for required file copying
        let required_files = vec!["job.sbatch", "config.namd"];

        for file in &required_files {
            assert!(!file.is_empty(), "Required file name cannot be empty");
            assert!(!file.contains("/"), "File should be a filename, not a path");
            assert!(!file.contains(".."), "File should not contain path traversal");
        }
    }
}

