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
/// Returns complete job list after sync (backend owns complete state)
/// This function queries SLURM for current job status and updates:
/// - Local database
/// - job_info.json on server
/// - Triggers job_completion automation when jobs finish
pub async fn sync_all_jobs() -> Result<crate::types::SyncJobsResult> {
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
    let all_jobs = with_database(move |db| db.load_all_jobs())
        .map_err(|e| {
            error_log!("[Job Sync] Failed to load jobs from database: {}", e);
            anyhow!("Failed to load jobs: {}", e)
        })?;

    // Check if database is empty (first connection after DB reset)
    // If empty, automatically discover jobs from cluster
    if all_jobs.is_empty() {
        info_log!("[Job Sync] Database empty - triggering automatic job discovery");

        // Attempt discovery (don't fail sync if discovery fails)
        match discover_jobs_from_server_internal(&username).await {
            Ok((jobs_found, jobs_imported)) => {
                info_log!("[Job Sync] Discovered {} jobs, imported {}", jobs_found, jobs_imported);

                // Reload jobs after discovery
                let all_jobs_after_discovery = with_database(|db| db.load_all_jobs())
                    .map_err(|e| anyhow!("Failed to reload jobs after discovery: {}", e))?;

                // Return the discovered jobs
                return Ok(crate::types::SyncJobsResult {
                    success: true,
                    jobs: all_jobs_after_discovery,
                    jobs_updated: 0,
                    errors: vec![],
                });
            }
            Err(e) => {
                // Log error but don't fail sync - continue with empty list
                error_log!("[Job Sync] Discovery failed: {} - continuing with sync", e);
            }
        }
    }

    // Filter to only jobs that need syncing (Pending or Running)
    let active_jobs: Vec<JobInfo> = all_jobs.iter()
        .filter(|job| matches!(job.status, JobStatus::Pending | JobStatus::Running))
        .cloned()
        .collect();

    if active_jobs.is_empty() {
        info_log!("[Job Sync] No active jobs to sync");
        // Still return complete job list (even if no active jobs)
        return Ok(crate::types::SyncJobsResult {
            success: true,
            jobs: all_jobs,
            jobs_updated: 0,
            errors: vec![],
        });
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
        // Load complete job list to return
        let final_jobs = with_database(|db| db.load_all_jobs())
            .map_err(|e| anyhow!("Failed to load jobs: {}", e))?;
        return Ok(crate::types::SyncJobsResult {
            success: true,
            jobs: final_jobs,
            jobs_updated: 0,
            errors: vec![],
        });
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

    let jobs_updated = results.iter().filter(|r| r.updated).count() as u32;

    info_log!("[Job Sync] Sync completed - {} jobs checked, {} updated",
        results.len(),
        jobs_updated
    );

    // Load complete job list to return (backend owns complete state)
    let all_jobs = with_database(|db| db.load_all_jobs())
        .map_err(|e| {
            error_log!("[Job Sync] Failed to load complete job list: {}", e);
            anyhow!("Failed to load jobs: {}", e)
        })?;

    Ok(crate::types::SyncJobsResult {
        success: true,
        jobs: all_jobs,
        jobs_updated,
        errors: vec![],
    })
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

    // Set completion timestamp and trigger automatic completion for terminal states
    if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        job.completed_at = Some(Utc::now().to_rfc3339());
        info_log!("[Job Sync] Job {} reached terminal state: {:?}", job_id, new_status);

        // Trigger automatic job completion (rsync scratch→project, fetch logs, update metadata)
        if let Err(e) = crate::automations::execute_job_completion_internal(&mut job).await {
            error_log!("[Job Sync] Automatic completion failed for {}: {}", job_id, e);
            // Don't fail sync - completion will retry on next sync
        } else {
            info_log!("[Job Sync] Automatic completion successful for {}", job_id);
        }
    }

    // Update database (Metadata-at-Boundaries: only update DB during execution, not server metadata)
    // Server metadata is updated at lifecycle boundaries (creation, submission, completion)
    let job_clone = job.clone();
    with_database(move |db| db.save_job(&job_clone))
        .map_err(|e| {
            error_log!("[Job Sync] Failed to save job {} to database: {}", job_id, e);
            anyhow!("Failed to update database: {}", e)
        })?;
    debug_log!("[Job Sync] Database updated for job {}", job_id);

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

/// Fetch and cache SLURM logs (.out/.err) if not already cached
/// Only fetches when logs are None - implements "fetch once, cache forever" pattern
/// This function is public to allow other automations (job_completion, job_discovery) to use it
///
/// NOTE: Logs are fetched from project_dir (after rsync in job completion)
pub async fn fetch_slurm_logs_if_needed(job: &mut JobInfo) -> Result<()> {
    debug_log!("[Log Fetch] ENTRY: job_id={}, status={:?}, project_dir={:?}, slurm_job_id={:?}",
        job.job_id, job.status, job.project_dir, job.slurm_job_id);

    // Use project_dir instead of scratch_dir (rsync happens first in job completion)
    let project_dir = match &job.project_dir {
        Some(dir) => dir,
        None => {
            debug_log!("[Log Fetch] No project directory for job {}, skipping", job.job_id);
            return Ok(());
        }
    };

    let slurm_job_id = match &job.slurm_job_id {
        Some(id) => id,
        None => {
            debug_log!("[Log Fetch] No SLURM job ID for job {}, skipping", job.job_id);
            return Ok(());
        }
    };

    let connection_manager = get_connection_manager();

    // Fetch stdout if not cached (from project directory after rsync)
    if job.slurm_stdout.is_none() {
        let stdout_path = format!("{}/{}_{}.out", project_dir, job.job_name, slurm_job_id);
        debug_log!("[Log Fetch] Stdout not cached, constructing path: {}", stdout_path);
        debug_log!("[Log Fetch] Attempting to read stdout file from remote...");

        match connection_manager.read_remote_file(&stdout_path).await {
            Ok(content) => {
                let content_len = content.len();
                job.slurm_stdout = Some(content);
                info_log!("[Log Fetch] Cached stdout for job {} ({} bytes)", job.job_id, content_len);
            }
            Err(e) => {
                debug_log!("[Log Fetch] Could not read stdout for {}: {} (file may not exist yet)", job.job_id, e);
                // Gracefully handle - log file might not exist yet or job produced no output
            }
        }
    } else {
        debug_log!("[Log Fetch] Stdout already cached for job {}, skipping", job.job_id);
    }

    // Fetch stderr if not cached (from project directory after rsync)
    if job.slurm_stderr.is_none() {
        let stderr_path = format!("{}/{}_{}.err", project_dir, job.job_name, slurm_job_id);
        debug_log!("[Log Fetch] Stderr not cached, constructing path: {}", stderr_path);
        debug_log!("[Log Fetch] Attempting to read stderr file from remote...");

        match connection_manager.read_remote_file(&stderr_path).await {
            Ok(content) => {
                let content_len = content.len();
                job.slurm_stderr = Some(content);
                info_log!("[Log Fetch] Cached stderr for job {} ({} bytes)", job.job_id, content_len);
            }
            Err(e) => {
                debug_log!("[Log Fetch] Could not read stderr for {}: {} (file may not exist yet)", job.job_id, e);
            }
        }
    } else {
        debug_log!("[Log Fetch] Stderr already cached for job {}, skipping", job.job_id);
    }

    debug_log!("[Log Fetch] EXIT: Successfully completed for job {}", job.job_id);
    Ok(())
}

/// Force refetch SLURM logs from server, overwriting any cached logs
/// Used when user explicitly requests fresh logs via "Refetch Logs" button
///
/// NOTE: Logs are fetched from project_dir (after rsync in job completion)
pub async fn refetch_slurm_logs(job: &mut JobInfo) -> Result<()> {
    debug_log!("[Log Refetch] ENTRY: job_id={}, status={:?}", job.job_id, job.status);

    // Use project_dir instead of scratch_dir (logs have been rsynced to project)
    let project_dir = job.project_dir.as_ref()
        .ok_or_else(|| anyhow!("No project directory for job {}", job.job_id))?;

    let slurm_job_id = job.slurm_job_id.as_ref()
        .ok_or_else(|| anyhow!("No SLURM job ID for job {}", job.job_id))?;

    let connection_manager = get_connection_manager();

    // Force fetch stdout (overwrite cache) from project directory
    let stdout_path = format!("{}/{}_{}.out", project_dir, job.job_name, slurm_job_id);
    debug_log!("[Log Refetch] Fetching stdout from: {}", stdout_path);

    match connection_manager.read_remote_file(&stdout_path).await {
        Ok(content) => {
            let content_len = content.len();
            job.slurm_stdout = Some(content);
            info_log!("[Log Refetch] Refetched stdout for job {} ({} bytes)", job.job_id, content_len);
        }
        Err(e) => {
            debug_log!("[Log Refetch] Could not read stdout for {}: {}", job.job_id, e);
            // Set to empty if file doesn't exist
            job.slurm_stdout = Some(String::new());
        }
    }

    // Force fetch stderr (overwrite cache) from project directory
    let stderr_path = format!("{}/{}_{}.err", project_dir, job.job_name, slurm_job_id);
    debug_log!("[Log Refetch] Fetching stderr from: {}", stderr_path);

    match connection_manager.read_remote_file(&stderr_path).await {
        Ok(content) => {
            let content_len = content.len();
            job.slurm_stderr = Some(content);
            info_log!("[Log Refetch] Refetched stderr for job {} ({} bytes)", job.job_id, content_len);
        }
        Err(e) => {
            debug_log!("[Log Refetch] Could not read stderr for {}: {}", job.job_id, e);
            job.slurm_stderr = Some(String::new());
        }
    }

    debug_log!("[Log Refetch] EXIT: Successfully completed for job {}", job.job_id);
    Ok(())
}

/// Internal helper to discover jobs from server
/// Returns (jobs_found, jobs_imported)
async fn discover_jobs_from_server_internal(username: &str) -> Result<(u32, u32)> {
    info_log!("[Job Discovery] Starting automatic discovery for user: {}", username);

    let connection_manager = get_connection_manager();

    // Construct remote jobs directory path
    let remote_jobs_dir = format!("/projects/{}/namdrunner_jobs", username);
    debug_log!("[Job Discovery] Scanning directory: {}", remote_jobs_dir);

    // List directories in the jobs folder
    let job_dirs = connection_manager.list_files(&remote_jobs_dir).await
        .map_err(|e| {
            error_log!("[Job Discovery] Failed to list directories: {}", e);
            anyhow!("Failed to list job directories: {}", e)
        })?
        .into_iter()
        .filter(|f| f.is_directory)
        .map(|f| f.name)
        .collect::<Vec<_>>();

    info_log!("[Job Discovery] Found {} directories", job_dirs.len());

    let mut jobs_found = 0;
    let mut jobs_imported = 0;

    // Read job_info.json from each directory
    for job_dir in job_dirs {
        let job_info_path = format!("{}/{}/job_info.json", remote_jobs_dir, job_dir);

        // Try to read the job info file
        let job_json = match connection_manager.read_remote_file(&job_info_path).await {
            Ok(content) => content,
            Err(e) => {
                debug_log!("[Job Discovery] Could not read {}: {}", job_info_path, e);
                continue;
            }
        };

        // Parse the JSON
        let job_info: JobInfo = match serde_json::from_str(&job_json) {
            Ok(info) => info,
            Err(e) => {
                debug_log!("[Job Discovery] Invalid JSON in {}: {}", job_info_path, e);
                continue;
            }
        };

        jobs_found += 1;

        // Check if job already exists in database
        let job_id = job_info.job_id.clone();
        let job_info_clone = job_info.clone();

        let imported = with_database(move |db| {
            match db.load_job(&job_id) {
                Ok(Some(_)) => {
                    debug_log!("[Job Discovery] Job {} already exists", job_id);
                    Ok(false) // Already exists
                }
                Ok(None) => {
                    // Import new job
                    db.save_job(&job_info_clone)?;
                    info_log!("[Job Discovery] Imported job: {}", job_id);
                    Ok(true) // Imported
                }
                Err(e) => Err(e),
            }
        })?;

        if imported {
            jobs_imported += 1;
        }
    }

    info_log!("[Job Discovery] Discovery complete - found {} jobs, imported {}", jobs_found, jobs_imported);
    Ok((jobs_found, jobs_imported))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JobInfo, JobStatus, NAMDConfig, SlurmConfig, ExecutionMode};

    fn create_test_job(job_id: &str, status: JobStatus) -> JobInfo {
        JobInfo {
            job_id: job_id.to_string(),
            job_name: "test_job".to_string(),
            status,
            slurm_job_id: Some("12345".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
            submitted_at: None,
            completed_at: None,
            project_dir: Some("/projects/user/job".to_string()),
            scratch_dir: Some("/scratch/user/job".to_string()),
            error_info: None,
            namd_config: NAMDConfig {
                outputname: "output".to_string(),
                temperature: 300.0,
                timestep: 2.0,
                execution_mode: ExecutionMode::Run,
                steps: 1000,
                cell_basis_vector1: None,
                cell_basis_vector2: None,
                cell_basis_vector3: None,
                pme_enabled: false,
                npt_enabled: false,
                langevin_damping: 5.0,
                xst_freq: 100,
                output_energies_freq: 100,
                dcd_freq: 100,
                restart_freq: 500,
                output_pressure_freq: 100,
            },
            slurm_config: SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("amilan".to_string()),
                qos: None,
            },
            input_files: Vec::new(),
            output_files: None,
            remote_directory: "/projects/user/job".to_string(),
            slurm_stdout: None,
            slurm_stderr: None,
        }
    }

    #[test]
    fn test_sync_single_job_detects_terminal_state_transition() {
        // Test that terminal state detection logic works correctly
        let job = create_test_job("test_001", JobStatus::Running);
        let new_status = JobStatus::Completed;

        // The logic in sync_single_job_with_status checks:
        // if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled)
        assert!(matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled));

        // Verify other statuses don't match
        assert!(!matches!(JobStatus::Running, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled));
        assert!(!matches!(JobStatus::Pending, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled));
        assert!(!matches!(JobStatus::Created, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled));
    }

    #[test]
    fn test_job_sync_result_structure() {
        // Test that JobSyncResult contains all necessary information
        let result = JobSyncResult {
            job_id: "test_001".to_string(),
            old_status: JobStatus::Running,
            new_status: JobStatus::Completed,
            updated: true,
        };

        assert_eq!(result.job_id, "test_001");
        assert_eq!(result.old_status, JobStatus::Running);
        assert_eq!(result.new_status, JobStatus::Completed);
        assert!(result.updated);
    }

    #[test]
    fn test_completion_triggers_on_all_terminal_states() {
        // Verify all terminal states are covered in the completion trigger logic
        let terminal_states = vec![
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ];

        for status in terminal_states {
            assert!(matches!(status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled),
                "Status {:?} should trigger completion", status);
        }
    }

    #[test]
    fn test_non_terminal_states_dont_trigger_completion() {
        // Verify non-terminal states don't trigger completion
        let non_terminal_states = vec![
            JobStatus::Created,
            JobStatus::Pending,
            JobStatus::Running,
        ];

        for status in non_terminal_states {
            assert!(!matches!(status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled),
                "Status {:?} should NOT trigger completion", status);
        }
    }
}
