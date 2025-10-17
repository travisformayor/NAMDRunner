use crate::types::*;
use crate::demo::{with_demo_state, get_demo_state, advance_demo_progression, execute_with_mode, is_demo_mode};
use crate::validation::input;
use crate::database::with_database;
use crate::automations;
use crate::{info_log, debug_log, error_log};
use chrono::Utc;
use tauri::Emitter;
use serde::Serialize;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_job(app_handle: tauri::AppHandle, params: CreateJobParams) -> CreateJobResult {
    // Early validation at command boundary - sanitize job name immediately
    let clean_job_name = match input::sanitize_job_id(&params.job_name) {
        Ok(name) => name,
        Err(e) => {
            return CreateJobResult {
                success: false,
                job_id: None,
                job: None,
                error: Some(format!("Invalid job name: {}", e)),
            };
        }
    };

    // Create params with validated job name
    let validated_params = CreateJobParams {
        job_name: clean_job_name,
        namd_config: params.namd_config,
        slurm_config: params.slurm_config,
        input_files: params.input_files,
    };

    execute_with_mode(
        create_job_demo(validated_params.clone()),
        create_job_real(app_handle, validated_params)
    ).await
}

async fn create_job_demo(params: CreateJobParams) -> CreateJobResult {
    // Enhanced mock implementation - create job using mock state manager

    // Get realistic delay from mock state
    let delay = get_demo_state(|state| state.get_delay("slurm") / 2).unwrap_or(200);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check for simulated errors
    let should_fail = get_demo_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return CreateJobResult {
            success: false,
            job_id: None,
            job: None,
            error: Some("Failed to create job: Disk quota exceeded".to_string()),
        };
    }

    with_demo_state(|state| {
        state.job_counter += 1;
        let job_id = format!("job_{:03}", state.job_counter);
        let _now = Utc::now().to_rfc3339();

        let mut job_info = JobInfo::new(
            job_id.clone(),
            params.job_name.clone(),
            params.namd_config.clone(),
            params.slurm_config.clone(),
            params.input_files.clone(),
            format!("/projects/mockuser/namdrunner_jobs/{}", job_id),
        );

        // Set the directory paths after creation
        job_info.project_dir = Some(format!("/projects/mockuser/namdrunner_jobs/{}", job_id));
        job_info.scratch_dir = Some(format!("/scratch/alpine/mockuser/namdrunner_jobs/{}", job_id));

        state.jobs.insert(job_id.clone(), job_info.clone());

        // Note: Database not updated in demo mode (would require async context)
        // Demo state is sufficient for testing

        CreateJobResult {
            success: true,
            job_id: Some(job_id.clone()),
            job: Some(job_info),
            error: None,
        }
    }).unwrap_or_else(|| CreateJobResult {
        success: false,
        job_id: None,
        job: None,
        error: Some("Failed to access mock state".to_string()),
    })
}

async fn create_job_real(app_handle: tauri::AppHandle, params: CreateJobParams) -> CreateJobResult {
    // Use automation system with direct Tauri event emission for progress tracking
    let handle_clone = app_handle.clone();

    match automations::execute_job_creation_with_progress(
        app_handle,
        params,
        move |msg| {
            // Direct Tauri event emission - no abstraction layer
            let _ = handle_clone.emit("job-creation-progress", msg);
        }
    ).await {
        Ok((job_id, job_info)) => CreateJobResult {
            success: true,
            job_id: Some(job_id),
            job: Some(job_info),
            error: None,
        },
        Err(e) => CreateJobResult {
            success: false,
            job_id: None,
            job: None,
            error: Some(e.to_string()),
        },
    }
}


#[tauri::command(rename_all = "snake_case")]
pub async fn submit_job(job_id: String, app_handle: tauri::AppHandle) -> SubmitJobResult {
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(error) => {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(error.to_string()),
            };
        }
    };

    execute_with_mode(
        submit_job_demo(clean_job_id.clone()),
        submit_job_real(app_handle, clean_job_id)
    ).await
}

async fn submit_job_demo(job_id: String) -> SubmitJobResult {
    // Enhanced mock implementation - simulate realistic job submission

    // Get realistic delay from mock state
    let delay = get_demo_state(|state| state.get_delay("slurm")).unwrap_or(500);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check if connected to cluster
    let is_connected = get_demo_state(|state| {
        matches!(state.connection_state, ConnectionState::Connected)
    }).unwrap_or(false);

    if !is_connected {
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some("Not connected to cluster".to_string()),
        };
    }

    // Check for simulated errors
    let should_fail = get_demo_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some("SLURM submission failed: Insufficient resources".to_string()),
        };
    }

    with_demo_state(|state| {
        // Generate SLURM job ID first to avoid borrow issues
        let slurm_job_id = state.generate_slurm_job_id();
        let now = Utc::now().to_rfc3339();

        if let Some(job) = state.jobs.get_mut(&job_id) {
            job.status = JobStatus::Pending;
            job.slurm_job_id = Some(slurm_job_id.clone());
            job.submitted_at = Some(now.clone());
            job.updated_at = Some(now.clone());

            // Note: Database not updated in demo mode (would require async context)
            // Demo state is sufficient for testing

            SubmitJobResult {
                success: true,
                slurm_job_id: Some(slurm_job_id),
                submitted_at: Some(now),
                error: None,
            }
        } else {
            SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Job {} not found", job_id)),
            }
        }
    }).unwrap_or_else(|| SubmitJobResult {
        success: false,
        slurm_job_id: None,
        submitted_at: None,
        error: Some("Failed to access mock state".to_string()),
    })
}

async fn submit_job_real(app_handle: tauri::AppHandle, job_id: String) -> SubmitJobResult {
    // Use automation system with direct Tauri event emission for progress tracking
    let handle_clone = app_handle.clone();

    match automations::execute_job_submission_with_progress(
        app_handle,
        job_id,
        move |msg| {
            // Direct Tauri event emission - no abstraction layer
            let _ = handle_clone.emit("job-submission-progress", msg);
        }
    ).await {
        Ok(result) => result,
        Err(e) => SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some(e.to_string()),
        },
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_job_status(job_id: String) -> JobStatusResult {
    // NOTE: Using mock state for delay simulation in development
    // This provides realistic UI behavior during development
    // In production, this would be replaced with actual SLURM query latency
    let delay = get_demo_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Retrieve job from database
    let job_id_for_db = job_id.clone();
    match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(job)) => JobStatusResult {
            success: true,
            job_info: Some(job),
            error: None,
        },
        Ok(None) => JobStatusResult {
            success: false,
            job_info: None,
            error: Some(format!("Job {} not found", job_id)),
        },
        Err(e) => JobStatusResult {
            success: false,
            job_info: None,
            error: Some(format!("Failed to load job {}: {}", job_id, e)),
        },
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_jobs() -> GetAllJobsResult {
    // NOTE: Using mock state for delay simulation in development
    // This provides realistic UI behavior during development
    // In production, this would be replaced with actual database query latency
    let delay = get_demo_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Retrieve all jobs from database

    match with_database(move |db| db.load_all_jobs()) {
        Ok(jobs) => GetAllJobsResult {
            success: true,
            jobs: Some(jobs),
            error: None,
        },
        Err(e) => GetAllJobsResult {
            success: false,
            jobs: None,
            error: Some(format!("Failed to load jobs from database: {}", e)),
        },
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_jobs() -> SyncJobsResult {
    execute_with_mode(
        sync_jobs_demo(),
        sync_jobs_real()
    ).await
}

async fn sync_jobs_demo() -> SyncJobsResult {
    // Mock implementation - simulate realistic job progression

    let delay = get_demo_state(|state| state.get_delay("slurm") * 4).unwrap_or(800);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check if connected to cluster
    let is_connected = get_demo_state(|state| {
        matches!(state.connection_state, ConnectionState::Connected)
    }).unwrap_or(false);

    if !is_connected {
        return SyncJobsResult {
            success: false,
            jobs_updated: 0,
            errors: vec!["Not connected to cluster".to_string()],
        };
    }

    // Check for simulated errors
    let should_fail = get_demo_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return SyncJobsResult {
            success: false,
            jobs_updated: 0,
            errors: vec!["Network error: Unable to contact SLURM controller".to_string()],
        };
    }

    // In demo mode, keep job states static - don't advance progression
    let jobs_updated = if is_demo_mode() {
        // Demo mode: return 0 updates to maintain static demo experience
        0
    } else {
        // Real mode: advance job states using the enhanced mock state manager
        advance_demo_progression();

        get_demo_state(|state| {
            // Count how many jobs were updated in the last sync
            state.jobs.values().filter(|job| {
                matches!(job.status, JobStatus::Running | JobStatus::Completed | JobStatus::Failed)
            }).count() as u32
        }).unwrap_or(0)
    };

    SyncJobsResult {
        success: true,
        jobs_updated,
        errors: vec![],
    }
}

async fn sync_jobs_real() -> SyncJobsResult {
    // Real implementation using job_sync automation
    info_log!("[Sync Jobs] Starting real job sync");

    match automations::sync_all_jobs().await {
        Ok(results) => {
            let jobs_updated = results.iter().filter(|r| r.updated).count() as u32;
            info_log!("[Sync Jobs] Successfully synced {} jobs", jobs_updated);

            SyncJobsResult {
                success: true,
                jobs_updated,
                errors: vec![],
            }
        }
        Err(e) => {
            error_log!("[Sync Jobs] Failed to sync jobs: {}", e);
            SyncJobsResult {
                success: false,
                jobs_updated: 0,
                errors: vec![e.to_string()],
            }
        }
    }
}


#[tauri::command(rename_all = "snake_case")]
pub async fn delete_job(job_id: String, delete_remote: bool) -> DeleteJobResult {
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(error) => {
            return DeleteJobResult {
                success: false,
                error: Some(error.to_string()),
            };
        }
    };

    execute_with_mode(
        delete_job_demo(clean_job_id.clone(), delete_remote),
        delete_job_real(clean_job_id, delete_remote)
    ).await
}

async fn delete_job_demo(job_id: String, delete_remote: bool) -> DeleteJobResult {
    // Enhanced mock implementation - delete job with realistic behavior

    let base_delay = get_demo_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    let delay = if delete_remote { base_delay * 5 } else { base_delay };
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check for simulated errors
    let should_fail = get_demo_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail && delete_remote {
        return DeleteJobResult {
            success: false,
            error: Some("Failed to delete remote files: Permission denied".to_string()),
        };
    }

    with_demo_state(|state| {
        if let Some(_job_info) = state.jobs.remove(&job_id) {
            // Note: Database not updated in demo mode (would require async context)
            // Demo state is sufficient for testing

            DeleteJobResult {
                success: true,
                error: None,
            }
        } else {
            DeleteJobResult {
                success: false,
                error: Some(format!("Job {} not found", job_id)),
            }
        }
    }).unwrap_or_else(|| DeleteJobResult {
        success: false,
        error: Some("Failed to access mock state".to_string()),
    })
}

async fn delete_job_real(job_id: String, delete_remote: bool) -> DeleteJobResult {
    // Real implementation with safe directory cleanup

    // Get job information from database
    let job_id_for_db = job_id.clone();
    let job_info = match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(info)) => info,
        Ok(None) => {
            return DeleteJobResult {
                success: false,
                error: Some(format!("Job {} not found", job_id)),
            };
        }
        Err(e) => {
            return DeleteJobResult {
                success: false,
                error: Some(format!("Failed to load job {}: {}", job_id, e)),
            };
        }
    };

    // Cancel SLURM job if it's still actively running or pending
    if matches!(job_info.status, crate::types::JobStatus::Pending | crate::types::JobStatus::Running) {
        if let Some(slurm_job_id) = &job_info.slurm_job_id {
            // Check if connected to cluster
            let connection_manager = crate::ssh::get_connection_manager();
            if !connection_manager.is_connected().await {
                return DeleteJobResult {
                    success: false,
                    error: Some("Cannot cancel SLURM job: Not connected to cluster".to_string()),
                };
            }

            // Get username for SLURM operations
            let username = match connection_manager.get_username().await {
                Ok(user) => user,
                Err(e) => {
                    return DeleteJobResult {
                        success: false,
                        error: Some(format!("Failed to get cluster username: {}", e)),
                    };
                }
            };

            // Cancel the SLURM job to prevent orphaned cluster jobs
            let slurm_sync = crate::slurm::status::SlurmStatusSync::new(&username);
            if let Err(e) = slurm_sync.cancel_job(slurm_job_id).await {
                return DeleteJobResult {
                    success: false,
                    error: Some(format!("Failed to cancel SLURM job {}: {}", slurm_job_id, e)),
                };
            }

            info_log!("[Delete Job] Successfully cancelled SLURM job: {}", slurm_job_id);
        }
    }

    // If delete_remote is requested, clean up directories
    if delete_remote {
        // Check if connected to cluster
        let connection_manager = crate::ssh::get_connection_manager();
        if !connection_manager.is_connected().await {
            return DeleteJobResult {
                success: false,
                error: Some("Cannot delete remote files: Not connected to cluster".to_string()),
            };
        }

        // Collect directories to delete (both project and scratch)
        let mut directories_to_delete = Vec::new();

        if let Some(project_dir) = &job_info.project_dir {
            directories_to_delete.push(("project", project_dir.clone()));
        }

        if let Some(scratch_dir) = &job_info.scratch_dir {
            directories_to_delete.push(("scratch", scratch_dir.clone()));
        }

        // Safely delete directories with validation
        for (dir_type, dir_path) in directories_to_delete {
            // Safety check: ensure the path is within expected NAMDRunner directories
            if !dir_path.contains("namdrunner_jobs") {
                return DeleteJobResult {
                    success: false,
                    error: Some(format!("Refusing to delete directory '{}' - not a NAMDRunner job directory", dir_path)),
                };
            }

            // Additional safety check: path should not contain dangerous patterns
            if dir_path.contains("..") || dir_path == "/" || dir_path.starts_with("/etc") || dir_path.starts_with("/usr") {
                return DeleteJobResult {
                    success: false,
                    error: Some(format!("Refusing to delete dangerous directory path: {}", dir_path)),
                };
            }

            // Use ConnectionManager to safely delete directory with retry logic
            if let Err(e) = connection_manager.delete_directory(&dir_path).await {
                return DeleteJobResult {
                    success: false,
                    error: Some(format!("Failed to delete {} directory '{}': {}", dir_type, dir_path, e)),
                };
            }
        }
    }

    // Remove job from database

    match with_database(move |db| db.delete_job(&job_info.job_id)) {
        Ok(true) => DeleteJobResult {
            success: true,
            error: None,
        },
        Ok(false) => DeleteJobResult {
            success: false,
            error: Some(format!("Job {} not found during removal", job_id)),
        },
        Err(e) => DeleteJobResult {
            success: false,
            error: Some(format!("Failed to delete job from database: {}", e)),
        },
    }
}

/// Result type for refetch logs operation
#[derive(Debug, Serialize)]
pub struct RefetchLogsResult {
    pub success: bool,
    pub job_info: Option<JobInfo>,
    pub error: Option<String>,
}

/// Refetch SLURM logs from server, overwriting cached logs
/// Used when user explicitly clicks "Refetch Logs" button
#[tauri::command(rename_all = "snake_case")]
pub async fn refetch_slurm_logs(job_id: String) -> RefetchLogsResult {
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => {
            return RefetchLogsResult {
                success: false,
                job_info: None,
                error: Some(format!("Invalid job ID: {}", e)),
            };
        }
    };

    // Get job from database
    let job_id_clone = clean_job_id.clone();
    let mut job_info = match with_database(move |db| db.load_job(&job_id_clone)) {
        Ok(Some(job)) => job,
        Ok(None) => {
            return RefetchLogsResult {
                success: false,
                job_info: None,
                error: Some(format!("Job {} not found", clean_job_id)),
            };
        }
        Err(e) => {
            return RefetchLogsResult {
                success: false,
                job_info: None,
                error: Some(format!("Database error: {}", e)),
            };
        }
    };

    // Refetch logs from server
    if let Err(e) = automations::refetch_slurm_logs(&mut job_info).await {
        return RefetchLogsResult {
            success: false,
            job_info: None,
            error: Some(format!("Failed to refetch logs: {}", e)),
        };
    }

    // Save updated job to database
    let job_clone = job_info.clone();
    if let Err(e) = with_database(move |db| db.save_job(&job_clone)) {
        return RefetchLogsResult {
            success: false,
            job_info: None,
            error: Some(format!("Failed to save updated logs: {}", e)),
        };
    }

    RefetchLogsResult {
        success: true,
        job_info: Some(job_info),
        error: None,
    }
}

// Job completion automation commands

/// Result type for job discovery operation
#[derive(Debug, Serialize)]
pub struct DiscoverJobsResult {
    pub success: bool,
    #[serde(rename = "jobsFound")]
    pub jobs_found: u32,
    #[serde(rename = "jobsImported")]
    pub jobs_imported: u32,
    pub error: Option<String>,
}

/// Discover jobs from server by scanning the remote directory
/// Only runs when local database is empty (0 jobs)
#[tauri::command(rename_all = "snake_case")]
pub async fn discover_jobs_from_server(app_handle: tauri::AppHandle) -> DiscoverJobsResult {
    execute_with_mode(
        discover_jobs_demo(),
        discover_jobs_real(app_handle)
    ).await
}

async fn discover_jobs_demo() -> DiscoverJobsResult {
    // In mock mode, jobs are already in the database
    // Simulate discovering a few jobs
    let delay = get_demo_state(|state| state.get_delay("slurm") * 2).unwrap_or(400);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check if connected
    let is_connected = get_demo_state(|state| {
        matches!(state.connection_state, ConnectionState::Connected)
    }).unwrap_or(false);

    if !is_connected {
        return DiscoverJobsResult {
            success: false,
            jobs_found: 0,
            jobs_imported: 0,
            error: Some("Not connected to cluster".to_string()),
        };
    }

    // Simulate finding some jobs that are already in the DB
    DiscoverJobsResult {
        success: true,
        jobs_found: 3,
        jobs_imported: 0,
        error: None,
    }
}

async fn discover_jobs_real(_app_handle: tauri::AppHandle) -> DiscoverJobsResult {
    // Get SSH connection
    let manager = crate::ssh::get_connection_manager();
    
    // Check if connected
    if !manager.is_connected().await {
        return DiscoverJobsResult {
            success: false,
            jobs_found: 0,
            jobs_imported: 0,
            error: Some("Not connected to cluster".to_string()),
        };
    }

    // Get username for path construction
    let username = match manager.get_username().await {
        Ok(user) => user,
        Err(e) => {
            return DiscoverJobsResult {
                success: false,
                jobs_found: 0,
                jobs_imported: 0,
                error: Some(format!("Failed to get username: {}", e)),
            };
        }
    };

    // Construct remote jobs directory path
    let remote_jobs_dir = format!("/projects/{}/namdrunner_jobs", username);

    info_log!("[JOB DISCOVERY] Scanning remote directory: {}", remote_jobs_dir);

    // List directories in the jobs folder
    let job_dirs = match manager.list_files(&remote_jobs_dir).await {
        Ok(files) => files.into_iter()
            .filter(|f| f.is_directory)
            .map(|f| f.name)
            .collect::<Vec<_>>(),
        Err(e) => {
            error_log!("[JOB DISCOVERY] ERROR: Failed to list directories: {}", e);
            return DiscoverJobsResult {
                success: false,
                jobs_found: 0,
                jobs_imported: 0,
                error: Some(format!("Failed to list job directories: {}", e)),
            };
        }
    };

    info_log!("[JOB DISCOVERY] Found {} directories", job_dirs.len());

    let mut jobs_found = 0;
    let mut jobs_imported = 0;

    // Read job_info.json from each directory
    for job_dir in job_dirs {
        let job_info_path = format!("{}/{}/job_info.json", remote_jobs_dir, job_dir);
        
        debug_log!("[JOB DISCOVERY] Reading: {}", job_info_path);

        // Try to read the job info file
        let job_json = match manager.read_remote_file(&job_info_path).await {
            Ok(content) => content,
            Err(e) => {
                debug_log!("[JOB DISCOVERY] WARNING: Could not read {}: {}", job_info_path, e);
                continue;
            }
        };

        // Parse the JSON
        let job_info: JobInfo = match serde_json::from_str(&job_json) {
            Ok(info) => info,
            Err(e) => {
                debug_log!("[JOB DISCOVERY] WARNING: Invalid JSON in {}: {}", job_info_path, e);
                continue;
            }
        };

        jobs_found += 1;

        // Check if job already exists in database and import if not
        let job_id = job_info.job_id.clone();
        let job_id_for_logs = job_id.clone();
        let job_status = job_info.status.clone();
        let job_info_clone = job_info.clone();

        // First, save the job to database
        let result = with_database(move |db| {
            match db.load_job(&job_id) {
                Ok(Some(_)) => {
                    // Job already exists, skip
                    debug_log!("[JOB DISCOVERY] Job {} already in database", job_id);
                    Ok(false) // false means not imported
                }
                Ok(None) => {
                    // Job doesn't exist, import it
                    match db.save_job(&job_info_clone) {
                        Ok(_) => {
                            info_log!("[JOB DISCOVERY] Imported job: {}", job_id);
                            Ok(true) // true means imported
                        }
                        Err(e) => {
                            error_log!("[JOB DISCOVERY] ERROR: Failed to save job {}: {}", job_id, e);
                            Ok(false)
                        }
                    }
                }
                Err(e) => {
                    error_log!("[JOB DISCOVERY] ERROR: Database error checking job {}: {}", job_id, e);
                    Ok(false)
                }
            }
        });

        if matches!(result, Ok(true)) {
            jobs_imported += 1;

            // If the job was imported AND is already finished, fetch its logs
            if matches!(job_status, crate::types::JobStatus::Completed | crate::types::JobStatus::Failed | crate::types::JobStatus::Cancelled) {
                debug_log!("[JOB DISCOVERY] Job {} is finished, attempting to fetch logs", job_id_for_logs);

                let mut job_info_for_logs = job_info.clone();
                if let Err(e) = crate::automations::fetch_slurm_logs_if_needed(&mut job_info_for_logs).await {
                    debug_log!("[JOB DISCOVERY] Could not fetch logs for {}: {}", job_id_for_logs, e);
                } else {
                    // Save updated job with logs
                    if let Err(e) = with_database(move |db| db.save_job(&job_info_for_logs)) {
                        error_log!("[JOB DISCOVERY] Failed to save logs for {}: {}", job_id_for_logs, e);
                    } else {
                        debug_log!("[JOB DISCOVERY] Updated job {} with logs", job_id_for_logs);
                    }
                }
            }
        }
    }

    info_log!("[JOB DISCOVERY] Discovery complete: {} jobs found, {} imported", jobs_found, jobs_imported);

    DiscoverJobsResult {
        success: true,
        jobs_found,
        jobs_imported,
        error: None,
    }
}
