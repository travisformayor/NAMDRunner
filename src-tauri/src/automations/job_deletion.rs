use anyhow::{Result, anyhow};
use crate::{log_info, log_debug, toast_log};
use crate::commands::helpers;
use crate::database::with_database;
use crate::ssh::get_connection_manager;

/// Execute job deletion with optional remote file cleanup
/// Provides progress reporting through callbacks
pub async fn execute_job_deletion(
    job_id: String,
    delete_remote: bool,
    progress_callback: impl Fn(&str),
) -> Result<()> {
    progress_callback("Loading job information...");
    log_debug!(category: "Job Deletion", message: "Starting deletion for job", details: "{}", job_id);

    // Load job from database
    let job_info = helpers::load_job_or_fail(&job_id, "Job Deletion")?;
    log_info!(category: "Job Deletion", message: "Loaded job", details: "{} ({})", job_info.job_id, job_info.job_name);

    // Cancel SLURM job if still active
    if matches!(job_info.status, crate::types::JobStatus::Pending | crate::types::JobStatus::Running) {
        if let Some(slurm_job_id) = &job_info.slurm_job_id {
            progress_callback("Cancelling SLURM job...");

            helpers::require_connection("Job Deletion").await?;
            let username = helpers::get_cluster_username("Job Deletion").await?;

            log_debug!(category: "Job Deletion", message: "Cancelling SLURM job", details: "{}", slurm_job_id);
            let slurm_sync = crate::slurm::status::SlurmStatusSync::new(&username);
            slurm_sync.cancel_job(slurm_job_id).await
                .map_err(|e| anyhow!("Failed to cancel SLURM job {}: {}", slurm_job_id, e))?;

            log_info!(category: "Job Deletion", message: "Successfully cancelled SLURM job", details: "{}", slurm_job_id);
        }
    }

    // Delete remote directories if requested
    if delete_remote {
        progress_callback("Deleting remote directories...");

        let connection_manager = get_connection_manager();
        if !connection_manager.is_connected().await {
            return Err(anyhow!("Cannot delete remote files: Not connected to cluster"));
        }

        // Collect directories to delete
        let mut directories_to_delete = Vec::new();

        if let Some(project_dir) = &job_info.project_dir {
            directories_to_delete.push(("project", project_dir.clone()));
        }

        if let Some(scratch_dir) = &job_info.scratch_dir {
            directories_to_delete.push(("scratch", scratch_dir.clone()));
        }

        // Validate and delete each directory
        for (dir_type, dir_path) in directories_to_delete {
            // Safety validation: ensure path is a NAMDRunner directory
            if !dir_path.contains(crate::ssh::directory_structure::JOB_BASE_DIRECTORY) {
                return Err(anyhow!(
                    "Refusing to delete '{}' - not a NAMDRunner job directory",
                    dir_path
                ));
            }

            // Safety validation: no dangerous path patterns
            if dir_path.contains("..") || dir_path == "/" || dir_path.starts_with("/etc") || dir_path.starts_with("/usr") {
                return Err(anyhow!("Refusing to delete dangerous directory: {}", dir_path));
            }

            log_debug!(category: "Job Deletion", message: "Deleting directory", details: "{}: {}", dir_type, dir_path);
            connection_manager.delete_directory(&dir_path).await
                .map_err(|e| anyhow!("Failed to delete {} directory '{}': {}", dir_type, dir_path, e))?;

            log_info!(category: "Job Deletion", message: "Deleted directory", details: "{}: {}", dir_type, dir_path);
        }
    }

    // Remove from database
    progress_callback("Removing from database...");

    let job_id_for_db = job_info.job_id.clone();
    with_database(move |db| {
        match db.delete_job(&job_id_for_db) {
            Ok(true) => Ok(()),
            Ok(false) => Err(anyhow!("Job {} not found in database", job_id_for_db)),
            Err(e) => Err(e),
        }
    })?;

    toast_log!(category: "Job Deletion", message: "Job deleted successfully", details: "{}", job_id);
    Ok(())
}
