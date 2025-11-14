use anyhow::{Result, anyhow};

use crate::types::{JobInfo, JobStatus};
use crate::ssh::get_connection_manager;
use crate::database::with_database;
use crate::slurm::status::SlurmStatusSync;
use crate::{log_info, log_debug, log_error, toast_log};
use crate::automations::common;

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
    log_info!(category: "Job Sync", message: "Starting job status sync");

    // Verify SSH connection and get username
    let (_connection_manager, username) = common::require_connection_with_username("Job Sync").await?;
    log_debug!(category: "Job Sync", message: "Syncing jobs for user", details: "{}", username);

    // Load all jobs from database
    let all_jobs = with_database(move |db| db.load_all_jobs())
        .map_err(|e| {
            log_error!(category: "Job Sync", message: "Failed to load jobs from database", details: "{}", e);
            anyhow!("Failed to load jobs: {}", e)
        })?;

    // Check if database is empty (first connection after DB reset)
    // If empty, automatically discover jobs from cluster
    if all_jobs.is_empty() {
        log_info!(category: "Job Sync", message: "Database empty - triggering automatic job discovery");

        // Attempt discovery (don't fail sync if discovery fails)
        match discover_jobs_from_server_internal(&username).await {
            Ok(report) => {
                if !report.imported_jobs.is_empty() {
                    toast_log!(
                        category: "Job Discovery",
                        message: "Jobs imported from cluster",
                        details: "{} jobs imported",
                        report.imported_jobs.len()
                    );
                } else {
                    log_info!(category: "Job Discovery", message: "No new jobs found on cluster");
                }

                // Log detailed results
                for job in &report.imported_jobs {
                    log_info!(category: "Job Sync", message: "Imported", details: "{} ({})", job.job_id, job.job_name);
                }
                for failure in &report.failed_imports {
                    log_error!(category: "Job Sync", message: "Failed to import", details: "{}: {}", failure.directory, failure.reason);
                }

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
                log_error!(category: "Job Sync", message: "Discovery failed - continuing with sync", details: "{}", e);
            }
        }
    }

    // Filter to only jobs that need syncing (Pending or Running)
    let active_jobs: Vec<JobInfo> = all_jobs.iter()
        .filter(|job| matches!(job.status, JobStatus::Pending | JobStatus::Running))
        .cloned()
        .collect();

    if active_jobs.is_empty() {
        log_info!(category: "Job Sync", message: "No active jobs to sync");
        // Still return complete job list (even if no active jobs)
        return Ok(crate::types::SyncJobsResult {
            success: true,
            jobs: all_jobs,
            jobs_updated: 0,
            errors: vec![],
        });
    }

    log_info!(category: "Job Sync", message: "Found active jobs to sync", details: "{} jobs", active_jobs.len());

    // Create SLURM status sync helper
    let slurm_sync = SlurmStatusSync::new(&username);

    // Extract SLURM job IDs for batch query
    let job_ids: Vec<String> = active_jobs.iter()
        .filter_map(|job| job.slurm_job_id.as_ref())
        .cloned()
        .collect();

    if job_ids.is_empty() {
        log_info!(category: "Job Sync", message: "No jobs have SLURM job IDs, skipping batch query");
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

    log_debug!(category: "Job Sync", message: "Batch querying SLURM jobs", details: "{} jobs", job_ids.len());

    // Use centralized batch query method (1 SSH command instead of N)
    let batch_results = slurm_sync.sync_all_jobs(&job_ids).await
        .map_err(|e| {
            log_error!(category: "Job Sync", message: "Batch SLURM query failed", details: "{}", e);
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
                                log_info!(
                                    category: "Job Sync",
                                    message: "Job status changed",
                                    details: "{}: {:?} -> {:?}",
                                    result.job_id, result.old_status, result.new_status
                                );
                            } else {
                                log_debug!(category: "Job Sync", message: "Job status unchanged", details: "{}: {:?}", result.job_id, result.old_status);
                            }
                            results.push(result);
                        }
                        Err(e) => {
                            log_error!(category: "Job Sync", message: "Failed to process job", details: "{}: {}", job.job_id, e);
                        }
                    }
                }
                Err(e) => {
                    log_error!(category: "Job Sync", message: "Failed to query SLURM status", details: "{}: {}", slurm_job_id, e);
                }
            }
        }
    }

    let jobs_updated = results.iter().filter(|r| r.updated).count() as u32;

    log_info!(category: "Job Sync", message: "Sync completed", details: "{} jobs checked, {} updated",
        results.len(),
        jobs_updated
    );

    // Load complete job list to return (backend owns complete state)
    let all_jobs = with_database(|db| db.load_all_jobs())
        .map_err(|e| {
            log_error!(category: "Job Sync", message: "Failed to load complete job list", details: "{}", e);
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
    log_debug!(category: "Job Sync", message: "Status changed for job", details: "{}: {:?} -> {:?}", job_id, old_status, new_status);

    common::update_job_status(&mut job, new_status.clone());

    // Trigger automatic completion for terminal states
    if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        log_info!(category: "Job Sync", message: "Job reached terminal state", details: "{}: {:?}", job_id, new_status);

        // Trigger automatic job completion (rsync scratch→project, fetch logs, update metadata)
        if let Err(e) = crate::automations::execute_job_completion_internal(&mut job).await {
            log_error!(category: "Job Sync", message: "Automatic completion failed", details: "{}: {}", job_id, e);
            // Don't fail sync - completion will retry on next sync
        } else {
            log_info!(category: "Job Sync", message: "Automatic completion successful", details: "{}", job_id);
        }
    }

    // Update database (Metadata-at-Boundaries: only update DB during execution, not server metadata)
    // Server metadata is updated at lifecycle boundaries (creation, submission, completion)
    common::save_job_to_database(&job, "Job Sync")?;
    log_debug!(category: "Job Sync", message: "Database updated for job", details: "{}", job_id);

    // Log if job finished
    if matches!(new_status, JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled) {
        log_info!(
            category: "Job Sync",
            message: "Job finished",
            details: "{} - status: {:?}, outputs in: {:?}",
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
    log_debug!(category: "Log Fetch", message: "ENTRY", details: "job_id={}, status={:?}, project_dir={:?}, slurm_job_id={:?}",
        job.job_id, job.status, job.project_dir, job.slurm_job_id);

    // Use project_dir instead of scratch_dir (rsync happens first in job completion)
    let project_dir = match &job.project_dir {
        Some(dir) => dir,
        None => {
            log_debug!(category: "Log Fetch", message: "No project directory, skipping", details: "{}", job.job_id);
            return Ok(());
        }
    };

    let slurm_job_id = match &job.slurm_job_id {
        Some(id) => id,
        None => {
            log_debug!(category: "Log Fetch", message: "No SLURM job ID, skipping", details: "{}", job.job_id);
            return Ok(());
        }
    };

    let connection_manager = get_connection_manager();

    // Fetch stdout if not cached (from project directory after rsync)
    if job.slurm_stdout.is_none() {
        let stdout_path = format!("{}/{}_{}.out", project_dir, job.job_name, slurm_job_id);
        log_debug!(category: "Log Fetch", message: "Stdout not cached, constructing path", details: "{}", stdout_path);
        log_debug!(category: "Log Fetch", message: "Attempting to read stdout file from remote");

        match connection_manager.read_remote_file(&stdout_path).await {
            Ok(content) => {
                let content_len = content.len();
                job.slurm_stdout = Some(content);
                log_info!(category: "Log Fetch", message: "Cached stdout for job", details: "{} ({} bytes)", job.job_id, content_len);
            }
            Err(e) => {
                log_debug!(category: "Log Fetch", message: "Could not read stdout (file may not exist yet)", details: "{}: {}", job.job_id, e);
                // Gracefully handle - log file might not exist yet or job produced no output
            }
        }
    } else {
        log_debug!(category: "Log Fetch", message: "Stdout already cached, skipping", details: "{}", job.job_id);
    }

    // Fetch stderr if not cached (from project directory after rsync)
    if job.slurm_stderr.is_none() {
        let stderr_path = format!("{}/{}_{}.err", project_dir, job.job_name, slurm_job_id);
        log_debug!(category: "Log Fetch", message: "Stderr not cached, constructing path", details: "{}", stderr_path);
        log_debug!(category: "Log Fetch", message: "Attempting to read stderr file from remote");

        match connection_manager.read_remote_file(&stderr_path).await {
            Ok(content) => {
                let content_len = content.len();
                job.slurm_stderr = Some(content);
                log_info!(category: "Log Fetch", message: "Cached stderr for job", details: "{} ({} bytes)", job.job_id, content_len);
            }
            Err(e) => {
                log_debug!(category: "Log Fetch", message: "Could not read stderr (file may not exist yet)", details: "{}: {}", job.job_id, e);
            }
        }
    } else {
        log_debug!(category: "Log Fetch", message: "Stderr already cached, skipping", details: "{}", job.job_id);
    }

    log_debug!(category: "Log Fetch", message: "EXIT: Successfully completed", details: "{}", job.job_id);
    Ok(())
}

/// Force refetch SLURM logs from server, overwriting any cached logs
/// Used when user explicitly requests fresh logs via "Refetch Logs" button
///
/// NOTE: Logs are fetched from project_dir (after rsync in job completion)
pub async fn refetch_slurm_logs(job: &mut JobInfo) -> Result<()> {
    log_debug!(category: "Log Refetch", message: "ENTRY", details: "job_id={}, status={:?}", job.job_id, job.status);

    // Use project_dir instead of scratch_dir (logs have been rsynced to project)
    let project_dir = job.project_dir.as_ref()
        .ok_or_else(|| anyhow!("No project directory for job {}", job.job_id))?;

    let slurm_job_id = job.slurm_job_id.as_ref()
        .ok_or_else(|| anyhow!("No SLURM job ID for job {}", job.job_id))?;

    let connection_manager = get_connection_manager();

    // Force fetch stdout (overwrite cache) from project directory
    let stdout_path = format!("{}/{}_{}.out", project_dir, job.job_name, slurm_job_id);
    log_debug!(category: "Log Refetch", message: "Fetching stdout", details: "{}", stdout_path);

    match connection_manager.read_remote_file(&stdout_path).await {
        Ok(content) => {
            let content_len = content.len();
            job.slurm_stdout = Some(content);
            log_info!(category: "Log Refetch", message: "Refetched stdout for job", details: "{} ({} bytes)", job.job_id, content_len);
        }
        Err(e) => {
            log_debug!(category: "Log Refetch", message: "Could not read stdout", details: "{}: {}", job.job_id, e);
            // Set to empty if file doesn't exist
            job.slurm_stdout = Some(String::new());
        }
    }

    // Force fetch stderr (overwrite cache) from project directory
    let stderr_path = format!("{}/{}_{}.err", project_dir, job.job_name, slurm_job_id);
    log_debug!(category: "Log Refetch", message: "Fetching stderr", details: "{}", stderr_path);

    match connection_manager.read_remote_file(&stderr_path).await {
        Ok(content) => {
            let content_len = content.len();
            job.slurm_stderr = Some(content);
            log_info!(category: "Log Refetch", message: "Refetched stderr for job", details: "{} ({} bytes)", job.job_id, content_len);
        }
        Err(e) => {
            log_debug!(category: "Log Refetch", message: "Could not read stderr", details: "{}: {}", job.job_id, e);
            job.slurm_stderr = Some(String::new());
        }
    }

    log_debug!(category: "Log Refetch", message: "EXIT: Successfully completed", details: "{}", job.job_id);
    Ok(())
}

/// Internal helper to discover jobs from server
/// Returns detailed report of imported jobs and failures
async fn discover_jobs_from_server_internal(username: &str) -> Result<crate::types::response_data::DiscoveryReport> {
    use crate::types::response_data::{DiscoveryReport, JobSummary, FailedImport};

    log_info!(category: "Job Discovery", message: "Starting automatic discovery", details: "user: {}", username);

    let connection_manager = get_connection_manager();

    // Construct remote jobs directory path
    use crate::ssh::directory_structure::JobDirectoryStructure;
    let remote_jobs_dir = JobDirectoryStructure::project_base(username);
    log_debug!(category: "Job Discovery", message: "Scanning directory", details: "{}", remote_jobs_dir);

    // List directories in the jobs folder
    let job_dirs = connection_manager.list_files(&remote_jobs_dir).await
        .map_err(|e| {
            log_error!(category: "Job Discovery", message: "Failed to list directories", details: "{}", e);
            anyhow!("Failed to list job directories: {}", e)
        })?
        .into_iter()
        .filter(|f| f.is_directory)
        .map(|f| f.name)
        .collect::<Vec<_>>();

    log_info!(category: "Job Discovery", message: "Found directories", details: "{} directories", job_dirs.len());

    let mut imported_jobs = Vec::new();
    let mut failed_imports = Vec::new();

    // Read job_info.json from each directory
    for job_dir in job_dirs {
        let job_info_path = format!("{}/{}/job_info.json", remote_jobs_dir, job_dir);

        // Try to read the job info file
        let job_json = match connection_manager.read_remote_file(&job_info_path).await {
            Ok(content) => content,
            Err(e) => {
                let error_msg = format!("Could not read job_info.json: {}", e);
                log_debug!(category: "Job Discovery", message: "Failed to read job info", details: "{}: {}", job_dir, error_msg);
                failed_imports.push(FailedImport {
                    directory: job_dir,
                    reason: error_msg,
                });
                continue;
            }
        };

        // Parse the JSON
        let job_info: JobInfo = match serde_json::from_str(&job_json) {
            Ok(info) => info,
            Err(e) => {
                let error_msg = format!("Invalid JSON: {}", e);
                log_debug!(category: "Job Discovery", message: "Invalid JSON", details: "{}: {}", job_dir, error_msg);
                failed_imports.push(FailedImport {
                    directory: job_dir,
                    reason: error_msg,
                });
                continue;
            }
        };

        // Check if job already exists in database
        let job_id = job_info.job_id.clone();
        let job_name = job_info.job_name.clone();
        let job_status = job_info.status.clone();
        let job_info_clone = job_info.clone();

        // Clone for closure to avoid move issues
        let job_id_for_log = job_id.clone();
        let job_name_for_log = job_name.clone();

        let imported = with_database(move |db| {
            match db.load_job(&job_id_for_log) {
                Ok(Some(_)) => {
                    log_debug!(category: "Job Discovery", message: "Job already exists, skipping", details: "{}", job_id_for_log);
                    Ok(false)
                }
                Ok(None) => {
                    db.save_job(&job_info_clone)?;
                    log_info!(category: "Job Discovery", message: "Imported", details: "{} ({})", job_id_for_log, job_name_for_log);
                    Ok(true)
                }
                Err(e) => Err(e),
            }
        })?;

        if imported {
            imported_jobs.push(JobSummary {
                job_id,
                job_name,
                status: job_status,
            });
        }
    }

    log_info!(
        category: "Job Discovery",
        message: "Complete",
        details: "imported {} jobs, {} failures",
        imported_jobs.len(),
        failed_imports.len()
    );

    // Log imported jobs
    for job in &imported_jobs {
        log_debug!(category: "Job Discovery", message: "Imported successfully", details: "{} ({}) - status: {:?}", job.job_id, job.job_name, job.status);
    }

    // Log failures
    for failure in &failed_imports {
        log_debug!(category: "Job Discovery", message: "Failed import", details: "{} - {}", failure.directory, failure.reason);
    }

    Ok(DiscoveryReport {
        imported_jobs,
        failed_imports,
    })
}

