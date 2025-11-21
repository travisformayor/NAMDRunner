use anyhow::{Result, anyhow};
use tauri::AppHandle;
use chrono::Utc;

use crate::types::{JobStatus, response_data::JobSubmissionData};
use crate::validation::paths;
use crate::database::with_database;
use crate::{log_info, log_debug, log_error};
use crate::automations::common;

/// Simplified job submission automation that follows NAMDRunner's direct function patterns.
/// Provides progress reporting through callbacks.
///
/// Key functionality: Creates scratch directories, copies files from project to scratch,
/// submits to SLURM, and updates job status. This maintains proper workflow separation.
pub async fn execute_job_submission_with_progress(
    _app_handle: AppHandle,
    job_id: String,
    progress_callback: impl Fn(&str),
) -> Result<JobSubmissionData> {
    progress_callback("Loading job information...");
    log_info!(category: "Job Submission", message: "Starting job submission", details: "{}", job_id);

    // Load job from database
    let job_id_clone = job_id.clone();
    let mut job_info = with_database(move |db| db.load_job(&job_id_clone))
        .map_err(|e| {
            log_error!(category: "Job Submission", message: "Database error loading job", details: "{}: {}", job_id, e);
            anyhow!("Database error: {}", e)
        })?
        .ok_or_else(|| {
            log_error!(category: "Job Submission", message: "Job not found in database", details: "{}", job_id);
            anyhow!("Job '{}' not found in database", job_id)
        })?;
    log_debug!(category: "Job Submission", message: "Loaded job", details: "{} (status: {:?})", job_id, job_info.status);

    // Validate job is in correct state for submission
    if !matches!(job_info.status, JobStatus::Created | JobStatus::Failed) {
        log_error!(category: "Job Submission", message: "Job cannot be submitted", details: "{} - status: {:?}", job_id, job_info.status);
        return Err(anyhow!("Job cannot be submitted because it is currently {:?}. Only Created or Failed jobs can be submitted.", job_info.status));
    }

    progress_callback("Validating connection...");

    // Verify SSH connection and get username
    let (connection_manager, username) = common::require_connection_with_username("Job Submission").await?;
    log_info!(category: "Job Submission", message: "Submitting job for user", details: "{}", username);

    progress_callback("Mirroring job directory to scratch...");

    let project_dir = common::require_project_dir(&job_info, "Job Submission")?;

    // Generate scratch directory path using existing validation functions
    let scratch_dir = paths::scratch_directory(&username, &job_info.job_id)?;
    log_info!(category: "Job Submission", message: "Mirroring project to scratch", details: "{} -> {}", project_dir, scratch_dir);

    // Use rsync to mirror entire job directory from project to scratch
    // Note: source must end with / to sync contents, destination should NOT end with / to create/sync into it
    let source_with_slash = common::ensure_trailing_slash(project_dir);

    connection_manager.sync_directory_rsync(&source_with_slash, &scratch_dir).await
        .map_err(|e| {
            log_error!(category: "Job Submission", message: "Failed to mirror directory to scratch", details: "{}", e);
            anyhow!("Failed to mirror job directory to scratch: {}", e)
        })?;

    log_info!(category: "Job Submission", message: "Successfully mirrored job directory to scratch");

    progress_callback("Submitting job to SLURM...");

    // Submit job using SLURM commands module (using mirrored script in scratch)
    let script_relative = "job.sbatch";
    let script_path = format!("{}/{}", scratch_dir, script_relative);
    log_info!(category: "Job Submission", message: "Executing sbatch with script", details: "{}", script_path);
    let submit_cmd = crate::slurm::commands::submit_job_command(&scratch_dir, script_relative)?;
    let output = connection_manager.execute_command(&submit_cmd, Some(crate::cluster::timeouts::JOB_SUBMIT)).await
        .map_err(|e| {
            log_error!(category: "Job Submission", message: "Failed to submit job to SLURM", details: "{}", e);
            anyhow!("Could not submit job to cluster scheduler: {}", e)
        })?;

    // Parse SLURM job ID from output using SLURM commands module
    let slurm_job_id = crate::slurm::commands::parse_sbatch_output(&output.stdout)
        .ok_or_else(|| {
            log_error!(category: "Job Submission", message: "Failed to parse SLURM job ID from output", details: "{}", output.stdout);
            anyhow!("Failed to parse SLURM job ID from: {}", output.stdout)
        })?;

    let submitted_at = Utc::now().to_rfc3339();
    log_info!(category: "Job Submission", message: "Job submitted successfully", details: "SLURM job ID: {} at {}", slurm_job_id, submitted_at);

    progress_callback("Updating job status...");

    // Update job info with submission details
    job_info.scratch_dir = Some(scratch_dir.clone());
    job_info.slurm_job_id = Some(slurm_job_id.clone());
    job_info.submitted_at = Some(submitted_at.clone());
    common::update_job_status(&mut job_info, JobStatus::Pending);
    log_debug!(category: "Job Submission", message: "Updated job status to Pending");

    // Save updated job info to database
    common::save_job_to_database(&job_info, "Job Submission")?;
    log_debug!(category: "Job Submission", message: "Saved job to database");

    // Update job_info.json in project directory
    if let Some(project_dir) = &job_info.project_dir {
        log_info!(category: "Job Submission", message: "Uploading job metadata", details: "{}/job_info.json", project_dir);
        crate::ssh::metadata::upload_job_metadata(connection_manager, &job_info, project_dir, "Job Submission").await
            .map_err(|e| {
                log_error!(category: "Job Submission", message: "Failed to upload job metadata", details: "{}", e);
                anyhow!("Failed to update job metadata: {}", e)
            })?;
    }

    progress_callback("Job submission completed successfully");
    log_info!(category: "Job Submission", message: "Job submitted successfully", details: "SLURM job ID: {}", slurm_job_id, show_toast: true);

    Ok(JobSubmissionData {
        job_id,
        slurm_job_id,
        submitted_at,
    })
}

