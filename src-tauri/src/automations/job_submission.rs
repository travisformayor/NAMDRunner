use anyhow::{Result, anyhow};
use tauri::AppHandle;
use chrono::Utc;

use crate::types::{JobStatus, response_data::JobSubmissionData};
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
) -> Result<JobSubmissionData> {
    progress_callback("Loading job information...");
    info_log!("[Job Submission] Starting job submission for: {}", job_id);

    // Load job from database
    let job_id_clone = job_id.clone();
    let mut job_info = with_database(move |db| db.load_job(&job_id_clone))
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
    let script_relative = "job.sbatch";
    let script_path = format!("{}/{}", scratch_dir, script_relative);
    info_log!("[Job Submission] Executing sbatch with script: {}", script_path);
    let submit_cmd = crate::slurm::commands::submit_job_command(&scratch_dir, script_relative)?;
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
    let job_info_clone = job_info.clone();
    with_database(move |db| db.save_job(&job_info_clone))
        .map_err(|e| {
            error_log!("[Job Submission] Failed to save job to database: {}", e);
            anyhow!("Failed to update job in database: {}", e)
        })?;
    debug_log!("[Job Submission] Saved job to database");

    // Update job_info.json in project directory
    if let Some(project_dir) = &job_info.project_dir {
        info_log!("[Job Submission] Uploading job metadata to: {}/job_info.json", project_dir);
        crate::ssh::metadata::upload_job_metadata(connection_manager, &job_info, project_dir, "Job Submission").await
            .map_err(|e| {
                error_log!("[Job Submission] Failed to upload job metadata: {}", e);
                anyhow!("Failed to update job metadata: {}", e)
            })?;
    }

    progress_callback("Job submission completed successfully");
    info_log!("[Job Submission] Job submission completed successfully for: {}", job_id);

    Ok(JobSubmissionData {
        job_id,
        slurm_job_id,
        submitted_at,
    })
}

// DELETED: Test module using NAMDConfig - needs rewrite for template system