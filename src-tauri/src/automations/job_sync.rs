use anyhow::{Result, anyhow};
use chrono::Utc;

use crate::types::{JobInfo, JobStatus};
use crate::ssh::get_connection_manager;
use crate::database::with_database;
use crate::slurm::status::SlurmStatusSync;
use crate::{info_log, debug_log, error_log};

/// Job sync result for a single job
#[derive(Debug, Clone)]
pub struct JobSyncResult {
    pub job_id: String,
    pub old_status: JobStatus,
    pub new_status: JobStatus,
    pub updated: bool,
}

/// Sync all active jobs with SLURM cluster
///
/// This function queries SLURM for current job status and updates:
/// - Local database
/// - job_info.json on server
/// - Triggers job_completion automation when jobs finish
pub async fn sync_all_jobs() -> Result<Vec<JobSyncResult>> {
    info_log!("[Job Sync] Starting job status sync");

    // Verify SSH connection is active
    let connection_manager = get_connection_manager();
    if !connection_manager.is_connected().await {
        error_log!("[Job Sync] SSH connection not active");
        return Err(anyhow!("Not connected to cluster"));
    }

    // Get username
    let username = connection_manager.get_username().await
        .map_err(|e| {
            error_log!("[Job Sync] Failed to get username: {}", e);
            anyhow!("Failed to get cluster username: {}", e)
        })?;
    debug_log!("[Job Sync] Syncing jobs for user: {}", username);

    // Load all jobs from database
    let all_jobs = with_database(|db| db.load_all_jobs())
        .map_err(|e| {
            error_log!("[Job Sync] Failed to load jobs from database: {}", e);
            anyhow!("Failed to load jobs: {}", e)
        })?;

    // Filter to only jobs that need syncing (Pending or Running)
    let active_jobs: Vec<JobInfo> = all_jobs.into_iter()
        .filter(|job| matches!(job.status, JobStatus::Pending | JobStatus::Running))
        .collect();

    if active_jobs.is_empty() {
        info_log!("[Job Sync] No active jobs to sync");
        return Ok(Vec::new());
    }

    info_log!("[Job Sync] Found {} active jobs to sync", active_jobs.len());

    // Create SLURM status sync helper
    let slurm_sync = SlurmStatusSync::new(&username);

    // Extract SLURM job IDs for batch query
    let job_ids: Vec<String> = active_jobs.iter()
        .filter_map(|job| job.slurm_job_id.as_ref())
        .cloned()
        .collect();

    if job_ids.is_empty() {
        info_log!("[Job Sync] No jobs have SLURM job IDs, skipping batch query");
        return Ok(Vec::new());
    }

    debug_log!("[Job Sync] Batch querying {} SLURM jobs", job_ids.len());

    // Use centralized batch query method (1 SSH command instead of N)
    let batch_results = slurm_sync.sync_all_jobs(&job_ids).await
        .map_err(|e| {
            error_log!("[Job Sync] Batch SLURM query failed: {}", e);
            anyhow!("Failed to query SLURM job status: {}", e)
        })?;

    // Create lookup map of SLURM job ID -> JobInfo
    let job_map: std::collections::HashMap<String, JobInfo> = active_jobs.into_iter()
        .filter_map(|job| {
            if let Some(slurm_id) = &job.slurm_job_id {
                Some((slurm_id.clone(), job))
            } else {
                None
            }
        })
        .collect();

    let mut results = Vec::new();

    // Process batch results
    for (slurm_job_id, status_result) in batch_results {
        if let Some(job) = job_map.get(&slurm_job_id) {
            match status_result {
                Ok(new_status) => {
                    match sync_single_job_with_status(&slurm_sync, job.clone(), new_status).await {
                        Ok(result) => {
                            if result.updated {
                                info_log!(
                                    "[Job Sync] Job {} status changed: {:?} -> {:?}",
                                    result.job_id, result.old_status, result.new_status
                                );
                            } else {
                                debug_log!("[Job Sync] Job {} status unchanged: {:?}", result.job_id, result.old_status);
                            }
                            results.push(result);
                        }
                        Err(e) => {
                            error_log!("[Job Sync] Failed to process job {}: {}", job.job_id, e);
                        }
                    }
                }
                Err(e) => {
                    error_log!("[Job Sync] Failed to query SLURM status for {}: {}", slurm_job_id, e);
                }
            }
        }
    }

    info_log!("[Job Sync] Sync completed - {} jobs checked, {} updated",
        results.len(),
        results.iter().filter(|r| r.updated).count()
    );

    Ok(results)
}

/// Sync a single job with already-fetched SLURM status (from batch query)
async fn sync_single_job_with_status(_slurm_sync: &SlurmStatusSync, mut job: JobInfo, new_status: JobStatus) -> Result<JobSyncResult> {
    let job_id = job.job_id.clone();
    let old_status = job.status.clone();

    // Check if status changed
    let status_changed = new_status != old_status;

    if !status_changed {
        return Ok(JobSyncResult {
            job_id,
            old_status,
            new_status,
            updated: false,
        });
    }

    // Status changed - update job
    debug_log!("[Job Sync] Status changed for job {}: {:?} -> {:?}", job_id, old_status, new_status);

    job.status = new_status.clone();
    job.updated_at = Some(Utc::now().to_rfc3339());

    // Set completion timestamp if job finished
    if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        job.completed_at = Some(Utc::now().to_rfc3339());
        info_log!("[Job Sync] Job {} finished with status: {:?}", job_id, new_status);
    }

    // Update database
    with_database(|db| db.save_job(&job))
        .map_err(|e| {
            error_log!("[Job Sync] Failed to save job {} to database: {}", job_id, e);
            anyhow!("Failed to update database: {}", e)
        })?;
    debug_log!("[Job Sync] Database updated for job {}", job_id);

    // Update job_info.json on server
    if let Some(project_dir) = &job.project_dir {
        match update_server_metadata(&job, project_dir).await {
            Ok(_) => {
                debug_log!("[Job Sync] Server metadata updated for job {}", job_id);
            }
            Err(e) => {
                error_log!("[Job Sync] Failed to update server metadata for job {}: {}", job_id, e);
                // Don't fail the sync if metadata update fails
            }
        }
    }

    // Log if job finished
    if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        info_log!(
            "[Job Sync] Job {} finished with status {:?} - outputs available in scratch: {:?}",
            job_id,
            new_status,
            job.scratch_dir
        );
    }

    Ok(JobSyncResult {
        job_id,
        old_status,
        new_status,
        updated: true,
    })
}

/// Update job_info.json on the server
async fn update_server_metadata(job: &JobInfo, project_dir: &str) -> Result<()> {
    let connection_manager = get_connection_manager();
    crate::ssh::metadata::upload_job_metadata(&connection_manager, job, project_dir, "Job Sync").await
}
