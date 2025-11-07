use crate::types::*;
use crate::validation::input;
use crate::database::with_database;
use crate::automations;
use crate::{info_log, debug_log, error_log};
use tauri::Emitter;
use serde::Serialize;
use std::collections::HashMap;
use serde_json::Value;

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
        template_id: params.template_id,
        template_values: params.template_values,
        slurm_config: params.slurm_config,
    };

    create_job_real(app_handle, validated_params).await
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

    submit_job_real(app_handle, clean_job_id).await
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
    // Removed demo delay - real SLURM queries have their own latency

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
    // Removed demo delay - database queries have their own latency
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
    sync_jobs_real().await
}

async fn sync_jobs_real() -> SyncJobsResult {
    // Real implementation using job_sync automation
    info_log!("[Sync Jobs] Starting real job sync");

    match automations::sync_all_jobs().await {
        Ok(result) => {
            info_log!("[Sync Jobs] Successfully synced {} jobs", result.jobs_updated);
            result
        }
        Err(e) => {
            error_log!("[Sync Jobs] Failed to sync jobs: {}", e);
            SyncJobsResult {
                success: false,
                jobs: vec![],
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

    delete_job_real(clean_job_id, delete_remote).await
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
            if !dir_path.contains(crate::ssh::directory_structure::JOB_BASE_DIRECTORY) {
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
    pub jobs_found: u32,
    pub jobs_imported: u32,
    pub error: Option<String>,
}

/// Discover jobs from server by scanning the remote directory
/// Only runs when local database is empty (0 jobs)
#[tauri::command(rename_all = "snake_case")]
pub async fn discover_jobs_from_server(app_handle: tauri::AppHandle) -> DiscoverJobsResult {
    discover_jobs_real(app_handle).await
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

    // Construct remote jobs directory path using centralized function
    use crate::ssh::directory_structure::JobDirectoryStructure;
    let remote_jobs_dir = JobDirectoryStructure::project_base(&username);

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

/// Preview SLURM script with given resource configuration
/// Returns what the job.sbatch file will look like
#[tauri::command(rename_all = "snake_case")]
pub async fn preview_slurm_script(
    job_name: String,
    cores: u32,
    memory: String,
    walltime: String,
    partition: Option<String>,
    qos: Option<String>
) -> PreviewResult {
    info_log!("[Jobs] Generating SLURM script preview");

    // Create minimal JobInfo for script generation
    let job_info = crate::types::JobInfo {
        job_id: "preview".to_string(),
        job_name: job_name.clone(),
        template_id: "preview_template".to_string(),
        template_values: std::collections::HashMap::new(),
        slurm_config: crate::types::SlurmConfig {
            cores,
            memory,
            walltime,
            partition,
            qos,
        },
        slurm_job_id: None,
        status: crate::types::JobStatus::Created,
        project_dir: Some("/projects/user/namdrunner_jobs/preview_job".to_string()),
        scratch_dir: None, // Not needed for preview (passed as parameter)
        output_files: Some(vec![]),
        slurm_stdout: None,
        slurm_stderr: None,
        error_info: None,
        remote_directory: "/projects/user/namdrunner_jobs/preview_job".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        submitted_at: None,
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        completed_at: None,
    };

    // Generate script (pass scratch_dir directly for preview)
    let preview_scratch_dir = "/scratch/alpine/user/job_preview";
    match crate::slurm::script_generator::SlurmScriptGenerator::generate_namd_script(&job_info, preview_scratch_dir) {
        Ok(script) => {
            info_log!("[Jobs] SLURM script preview generated");
            PreviewResult {
                success: true,
                content: Some(script),
                error: None,
            }
        }
        Err(e) => {
            error_log!("[Jobs] SLURM script preview failed: {}", e);
            PreviewResult {
                success: false,
                content: None,
                error: Some(format!("Script generation error: {}", e)),
            }
        }
    }
}

/// Validate complete job configuration
/// Checks job name, template selection, template values, and resource configuration
#[tauri::command(rename_all = "snake_case")]
pub async fn validate_job_config(
    job_name: String,
    template_id: String,
    template_values: HashMap<String, Value>,
    cores: u32,
    memory: String,
    walltime: String,
    partition: Option<String>,
    qos: Option<String>
) -> JobValidationResult {
    let mut errors = Vec::new();

    // Validate job name
    if job_name.trim().is_empty() {
        errors.push("Job name is required".to_string());
    } else if let Err(e) = input::sanitize_job_id(&job_name) {
        errors.push(format!("Job name invalid: {}", e));
    }

    // Validate template selection
    if template_id.is_empty() {
        errors.push("Template selection is required".to_string());
    }

    // Validate template values (if template selected)
    if !template_id.is_empty() {
        let template_validation = crate::commands::templates::validate_template_values(
            template_id.clone(),
            template_values.clone()
        ).await;

        errors.extend(template_validation.errors);
    }

    // Validate resource configuration
    if cores == 0 {
        errors.push("Cores must be greater than 0".to_string());
    }

    if memory.trim().is_empty() {
        errors.push("Memory is required".to_string());
    }

    if walltime.trim().is_empty() || !walltime.contains(':') {
        errors.push("Wall time must be in HH:MM:SS format".to_string());
    }

    // Validate partition and QoS if provided
    if let Some(p) = &partition {
        if p.trim().is_empty() {
            errors.push("Partition cannot be empty if specified".to_string());
        }
    }

    if let Some(q) = &qos {
        if q.trim().is_empty() {
            errors.push("QoS cannot be empty if specified".to_string());
        }
    }

    JobValidationResult {
        is_valid: errors.is_empty(),
        errors,
    }
}
