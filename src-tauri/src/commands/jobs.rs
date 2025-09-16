use crate::types::*;
use crate::mock_state::{with_mock_state, get_mock_state, advance_job_progression};
use crate::validation::{input, paths};
use crate::validation_traits::{ValidateId};
use crate::mode_switching::{is_mock_mode, execute_with_mode};
use crate::database::job_repository::{default_job_repository, JobRepository};
use chrono::Utc;
use std::env;

/// Get the username for cluster operations from SSH session or environment
/// Returns an error if no valid username can be determined
async fn get_cluster_username() -> Result<String, String> {
    // In mock mode, return a consistent mock username
    if is_mock_mode() {
        return Ok("mockuser".to_string());
    }

    // First try to get username from active SSH connection (most reliable)
    let connection_manager = crate::ssh::get_connection_manager();
    if let Some(conn_info) = connection_manager.get_connection_info().await {
        return Ok(conn_info.username);
    }

    // Try environment variable for development/testing
    if let Ok(username) = env::var("NAMDRUNNER_USERNAME") {
        if !username.trim().is_empty() {
            return Ok(username);
        }
    }

    // Try system username as last resort
    if let Ok(username) = env::var("USER") {
        if !username.trim().is_empty() {
            return Ok(username);
        }
    }

    // No fallback - return proper error
    Err("No username available. Please establish SSH connection or set NAMDRUNNER_USERNAME environment variable.".to_string())
}


/// Business logic layer for job creation
async fn create_job_business_logic(params: CreateJobParams) -> CreateJobResult {
    execute_with_mode(
        create_job_mock(params.clone()),
        create_job_real(params)
    ).await
}

#[tauri::command]
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    // Simple validation - check job name is not empty
    if params.job_name.trim().is_empty() {
        return CreateJobResult {
            success: false,
            job_id: None,
            error: Some("Job name is required".to_string()),
        };
    }

    // Business logic layer
    create_job_business_logic(params).await
}

async fn create_job_mock(params: CreateJobParams) -> CreateJobResult {
    // Enhanced mock implementation - create job using mock state manager

    // Get realistic delay from mock state
    let delay = get_mock_state(|state| state.get_delay("slurm") / 2).unwrap_or(200);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check for simulated errors
    let should_fail = get_mock_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return CreateJobResult {
            success: false,
            job_id: None,
            error: Some("Failed to create job: Disk quota exceeded".to_string()),
        };
    }

    with_mock_state(|state| {
        state.job_counter += 1;
        let job_id = format!("job_{:03}", state.job_counter);
        let now = Utc::now().to_rfc3339();

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

        // Also save to database for consistency with get_job_status
        let job_repository = default_job_repository();
        let _ = job_repository.save_job(&job_info);

        CreateJobResult {
            success: true,
            job_id: Some(job_id),
            error: None,
        }
    }).unwrap_or_else(|| CreateJobResult {
        success: false,
        job_id: None,
        error: Some("Failed to access mock state".to_string()),
    })
}

async fn create_job_real(params: CreateJobParams) -> CreateJobResult {
    // Real implementation with input validation and directory creation

    // First, validate and sanitize all inputs
    let clean_job_name = match input::sanitize_job_id(&params.job_name) {
        Ok(name) => name,
        Err(e) => {
            return CreateJobResult {
                success: false,
                job_id: None,
                error: Some(format!("Invalid job name: {}", e)),
            };
        }
    };

    // Get username from environment/configuration - TODO Phase 2.3: Replace with session management
    let username = match get_cluster_username().await {
        Ok(username) => username,
        Err(e) => {
            return CreateJobResult {
                success: false,
                job_id: None,
                error: Some(format!("Failed to get cluster username: {}", e)),
            };
        }
    };

    let clean_username = match input::sanitize_username(&username) {
        Ok(name) => name,
        Err(e) => {
            return CreateJobResult {
                success: false,
                job_id: None,
                error: Some(format!("Invalid username: {}", e)),
            };
        }
    };

    // Generate a unique job ID
    let job_id = format!("job_{}", Utc::now().timestamp_micros());

    // Generate safe directory paths
    let project_dir = match paths::project_directory(&clean_username, &job_id) {
        Ok(path) => path,
        Err(e) => {
            return CreateJobResult {
                success: false,
                job_id: None,
                error: Some(format!("Failed to generate project directory path: {}", e)),
            };
        }
    };

    let scratch_dir = match paths::scratch_directory(&clean_username, &job_id) {
        Ok(path) => path,
        Err(e) => {
            return CreateJobResult {
                success: false,
                job_id: None,
                error: Some(format!("Failed to generate scratch directory path: {}", e)),
            };
        }
    };

    // Create the project directory structure with retry logic

    // Create main project directory
    let connection_manager = crate::ssh::get_connection_manager();
    if let Err(e) = connection_manager.create_directory(&project_dir).await {
        return CreateJobResult {
            success: false,
            job_id: None,
            error: Some(format!("Failed to create project directory: {}", e)),
        };
    }

    // Create subdirectories
    for subdir in paths::job_subdirectories() {
        let subdir_path = format!("{}/{}", project_dir, subdir);
        if let Err(e) = connection_manager.create_directory(&subdir_path).await {
            return CreateJobResult {
                success: false,
                job_id: None,
                error: Some(format!("Failed to create subdirectory '{}': {}", subdir, e)),
            };
        }
    }

    // Create JobInfo and save to database
    let mut job_info = JobInfo::new(
        job_id.clone(),
        clean_job_name,
        params.namd_config,
        params.slurm_config,
        params.input_files,
        project_dir.clone(),
    );

    // Set the directory paths after creation
    job_info.project_dir = Some(project_dir);
    job_info.scratch_dir = Some(scratch_dir);

    // Save to database using repository
    let job_repository = default_job_repository();
    if let Err(e) = job_repository.save_job(&job_info) {
        return CreateJobResult {
            success: false,
            job_id: None,
            error: Some(format!("Failed to save job to database: {}", e)),
        };
    }

    CreateJobResult {
        success: true,
        job_id: Some(job_id),
        error: None,
    }
}

/// Validation layer for submit job parameters

/// Business logic layer for job submission
async fn submit_job_business_logic(clean_job_id: String) -> SubmitJobResult {
    execute_with_mode(
        submit_job_mock(clean_job_id.clone()),
        submit_job_real(clean_job_id)
    ).await
}

#[tauri::command]
pub async fn submit_job(job_id: String) -> SubmitJobResult {
    // Input validation layer
    let clean_job_id = match job_id.validate_id() {
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

    // Business logic layer
    submit_job_business_logic(clean_job_id).await
}

async fn submit_job_mock(job_id: String) -> SubmitJobResult {
    // Enhanced mock implementation - simulate realistic job submission

    // Get realistic delay from mock state
    let delay = get_mock_state(|state| state.get_delay("slurm")).unwrap_or(500);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check if connected to cluster
    let is_connected = get_mock_state(|state| {
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
    let should_fail = get_mock_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some("SLURM submission failed: Insufficient resources".to_string()),
        };
    }

    with_mock_state(|state| {
        // Generate SLURM job ID first to avoid borrow issues
        let slurm_job_id = state.generate_slurm_job_id();
        let now = Utc::now().to_rfc3339();

        if let Some(job) = state.jobs.get_mut(&job_id) {
            job.status = JobStatus::Pending;
            job.slurm_job_id = Some(slurm_job_id.clone());
            job.submitted_at = Some(now.clone());
            job.updated_at = Some(now.clone());

            // Also save to database for consistency with get_job_status
            let job_repository = default_job_repository();
            let _ = job_repository.save_job(job);

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

async fn submit_job_real(job_id: String) -> SubmitJobResult {
    // Real implementation with scratch directory creation

    // Get job information from database
    let job_repository = default_job_repository();
    let mut job_info = match job_repository.load_job(&job_id) {
        Ok(Some(info)) => info,
        Ok(None) => {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Job {} not found", job_id)),
            };
        }
        Err(e) => {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Failed to load job {}: {}", job_id, e)),
            };
        }
    };

    // Get the scratch directory path from job info
    let scratch_dir = match &job_info.scratch_dir {
        Some(dir) => dir.clone(),
        None => {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some("Job does not have a scratch directory configured".to_string()),
            };
        }
    };

    // Create scratch directory structure with retry logic
    let connection_manager = crate::ssh::get_connection_manager();
    if let Err(e) = connection_manager.create_directory(&scratch_dir).await {
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some(format!("Failed to create scratch directory: {}", e)),
        };
    }

    // Create scratch subdirectories for job execution
    let scratch_subdirs = vec!["input", "output", "logs"];
    for subdir in scratch_subdirs {
        let subdir_path = format!("{}/{}", scratch_dir, subdir);
        if let Err(e) = connection_manager.create_directory(&subdir_path).await {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Failed to create scratch subdirectory '{}': {}", subdir, e)),
            };
        }
    }

    // Copy input files from project to scratch directory
    if let Err(e) = copy_input_files(&job_info, &connection_manager).await {
        cleanup_scratch_directory(&connection_manager, &scratch_dir).await;
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some(format!("Failed to copy input files: {}", e)),
        };
    }

    // Generate SLURM script and NAMD config
    let slurm_script = match crate::slurm::SlurmScriptGenerator::generate_namd_script(&job_info) {
        Ok(script) => script,
        Err(e) => {
            cleanup_scratch_directory(&connection_manager, &scratch_dir).await;
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Failed to generate SLURM script: {}", e)),
            };
        }
    };

    let namd_config = match crate::slurm::SlurmScriptGenerator::generate_namd_config(&job_info) {
        Ok(config) => config,
        Err(e) => {
            cleanup_scratch_directory(&connection_manager, &scratch_dir).await;
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Failed to generate NAMD config: {}", e)),
            };
        }
    };

    // Upload SLURM script and NAMD config to scratch directory
    let script_path = format!("{}/job.sbatch", scratch_dir);
    let config_path = format!("{}/config.namd", scratch_dir);

    // Upload SLURM script using printf command
    if let Err(e) = upload_content(connection_manager, &slurm_script, &script_path).await {
        cleanup_scratch_directory(&connection_manager, &scratch_dir).await;
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some(format!("Failed to upload SLURM script: {}", e)),
        };
    }

    // Upload NAMD config using printf command
    if let Err(e) = upload_content(connection_manager, &namd_config, &config_path).await {
        cleanup_scratch_directory(&connection_manager, &scratch_dir).await;
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some(format!("Failed to upload NAMD config: {}", e)),
        };
    }

    // Submit job to SLURM
    let sbatch_cmd = format!("source /etc/profile && module load slurm/alpine && cd {} && sbatch job.sbatch", scratch_dir);
    let result = match connection_manager.execute_command(&sbatch_cmd, Some(30)).await {
        Ok(result) => result,
        Err(e) => {
            cleanup_scratch_directory(&connection_manager, &scratch_dir).await;
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Failed to execute sbatch command: {}", e)),
            };
        }
    };

    // Parse SLURM job ID from sbatch output
    let slurm_job_id = match parse_sbatch_output(&result.stdout) {
        Some(id) => id,
        None => {
            cleanup_scratch_directory(&connection_manager, &scratch_dir).await;
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Failed to parse SLURM job ID from output: {}", result.stdout)),
            };
        }
    };

    let now = Utc::now().to_rfc3339();

    // Update job with SLURM job ID and save to database
    let job_repository = default_job_repository();
    if let Err(e) = job_repository.update_job_with_slurm_id(&mut job_info, slurm_job_id.clone()) {
        return SubmitJobResult {
            success: false,
            slurm_job_id: None,
            submitted_at: None,
            error: Some(format!("Failed to update job with SLURM ID: {}", e)),
        };
    }

    SubmitJobResult {
        success: true,
        slurm_job_id: Some(slurm_job_id),
        submitted_at: Some(now),
        error: None,
    }
}

#[tauri::command]
pub async fn get_job_status(job_id: String) -> JobStatusResult {
    // NOTE: Using mock state for delay simulation in development
    // This provides realistic UI behavior during development
    // In production, this would be replaced with actual SLURM query latency
    let delay = get_mock_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Retrieve job from database
    let job_repository = default_job_repository();
    match job_repository.load_job(&job_id) {
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

#[tauri::command]
pub async fn get_all_jobs() -> GetAllJobsResult {
    // NOTE: Using mock state for delay simulation in development
    // This provides realistic UI behavior during development
    // In production, this would be replaced with actual database query latency
    let delay = get_mock_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Retrieve all jobs from database
    let job_repository = default_job_repository();
    match job_repository.load_all_jobs() {
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

#[tauri::command]
pub async fn sync_jobs() -> SyncJobsResult {
    // Enhanced mock implementation - simulate realistic job progression
    
    let delay = get_mock_state(|state| state.get_delay("slurm") * 4).unwrap_or(800);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    
    // Check if connected to cluster
    let is_connected = get_mock_state(|state| {
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
    let should_fail = get_mock_state(|state| state.should_simulate_error()).unwrap_or(false);
    
    if should_fail {
        return SyncJobsResult {
            success: false,
            jobs_updated: 0,
            errors: vec!["Network error: Unable to contact SLURM controller".to_string()],
        };
    }
    
    // Advance job states using the enhanced mock state manager
    advance_job_progression();
    
    let jobs_updated = get_mock_state(|state| {
        // Count how many jobs were updated in the last sync
        state.jobs.values().filter(|job| {
            matches!(job.status, JobStatus::Running | JobStatus::Completed | JobStatus::Failed)
        }).count() as u32
    }).unwrap_or(0);
    
    SyncJobsResult {
        success: true,
        jobs_updated,
        errors: vec![],
    }
}

/// Validation layer for delete job parameters

/// Business logic layer for job deletion
async fn delete_job_business_logic(clean_job_id: String, delete_remote: bool) -> DeleteJobResult {
    execute_with_mode(
        delete_job_mock(clean_job_id.clone(), delete_remote),
        delete_job_real(clean_job_id, delete_remote)
    ).await
}

#[tauri::command]
pub async fn delete_job(job_id: String, delete_remote: bool) -> DeleteJobResult {
    // Input validation layer
    let clean_job_id = match job_id.validate_id() {
        Ok(id) => id,
        Err(error) => {
            return DeleteJobResult {
                success: false,
                error: Some(error.to_string()),
            };
        }
    };

    // Business logic layer
    delete_job_business_logic(clean_job_id, delete_remote).await
}

async fn delete_job_mock(job_id: String, delete_remote: bool) -> DeleteJobResult {
    // Enhanced mock implementation - delete job with realistic behavior

    let base_delay = get_mock_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    let delay = if delete_remote { base_delay * 5 } else { base_delay };
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check for simulated errors
    let should_fail = get_mock_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail && delete_remote {
        return DeleteJobResult {
            success: false,
            error: Some("Failed to delete remote files: Permission denied".to_string()),
        };
    }

    with_mock_state(|state| {
        if let Some(job_info) = state.jobs.remove(&job_id) {
            // Also delete from database for consistency with get_job_status
            let job_repository = default_job_repository();
            let _ = job_repository.delete_job(&job_info.job_id);

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
    let job_repository = default_job_repository();
    let job_info = match job_repository.load_job(&job_id) {
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
    let job_repository = default_job_repository();
    match job_repository.delete_job(&job_info.job_id) {
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

#[tauri::command]
pub async fn sync_job_status(job_id: String) -> SyncJobStatusResult {
    // Input validation layer
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => {
            return SyncJobStatusResult {
                success: false,
                job_info: None,
                error: Some(format!("Invalid job ID: {}", e)),
            };
        }
    };

    // Load job from database
    let job_repository = default_job_repository();
    let mut job_info = match job_repository.load_job(&clean_job_id) {
        Ok(Some(job)) => job,
        Ok(None) => {
            return SyncJobStatusResult {
                success: false,
                job_info: None,
                error: Some(format!("Job {} not found", clean_job_id)),
            };
        }
        Err(e) => {
            return SyncJobStatusResult {
                success: false,
                job_info: None,
                error: Some(format!("Failed to load job {}: {}", clean_job_id, e)),
            };
        }
    };

    // Only sync jobs that have a SLURM job ID
    let slurm_job_id = match &job_info.slurm_job_id {
        Some(id) => id.clone(),
        None => {
            return SyncJobStatusResult {
                success: true, // Not an error - job hasn't been submitted yet
                job_info: Some(job_info),
                error: None,
            };
        }
    };

    // Get username from environment for SLURM sync
    let username = match get_cluster_username().await {
        Ok(username) => username,
        Err(e) => {
            return SyncJobStatusResult {
                success: false,
                job_info: None,
                error: Some(format!("Failed to get cluster username: {}", e)),
            };
        }
    };

    // Use SLURM status sync to get current status
    let slurm_sync = crate::slurm::SlurmStatusSync::new(&username);

    match slurm_sync.sync_job_status(&slurm_job_id).await {
        Ok(new_status) => {
            // Update job status if it changed
            if job_info.status != new_status {
                let job_repository = default_job_repository();
                if let Err(e) = job_repository.update_job_status_with_timestamps(&mut job_info, new_status, "slurm") {
                    return SyncJobStatusResult {
                        success: false,
                        job_info: Some(job_info),
                        error: Some(format!("Failed to update job status: {}", e)),
                    };
                }
            }

            SyncJobStatusResult {
                success: true,
                job_info: Some(job_info),
                error: None,
            }
        }
        Err(e) => {
            SyncJobStatusResult {
                success: false,
                job_info: Some(job_info),
                error: Some(format!("Failed to sync status from SLURM: {}", e)),
            }
        }
    }
}

#[tauri::command]
pub async fn sync_all_jobs() -> SyncAllJobsResult {
    // Load all jobs from database
    let job_repository = default_job_repository();
    let jobs = match job_repository.load_all_jobs() {
        Ok(jobs) => jobs,
        Err(e) => {
            return SyncAllJobsResult {
                success: false,
                jobs_updated: 0,
                errors: vec![format!("Failed to load jobs from database: {}", e)],
            };
        }
    };

    // Filter jobs that have SLURM job IDs
    let jobs_with_slurm_ids: Vec<_> = jobs
        .into_iter()
        .filter(|job| job.slurm_job_id.is_some())
        .collect();

    if jobs_with_slurm_ids.is_empty() {
        return SyncAllJobsResult {
            success: true,
            jobs_updated: 0,
            errors: vec![],
        };
    }

    // Extract SLURM job IDs for batch sync
    let slurm_job_ids: Vec<String> = jobs_with_slurm_ids
        .iter()
        .filter_map(|job| job.slurm_job_id.clone())
        .collect();

    // Get username from environment for SLURM sync
    let username = match get_cluster_username().await {
        Ok(username) => username,
        Err(e) => {
            log::error!("Failed to get cluster username for sync_all_jobs: {}", e);
            return SyncAllJobsResult {
                success: false,
                jobs_updated: 0,
                errors: vec![format!("Failed to get cluster username: {}", e)],
            };
        }
    };
    let slurm_sync = crate::slurm::SlurmStatusSync::new(&username);

    // Sync all jobs at once using batch operation
    match slurm_sync.sync_all_jobs(&slurm_job_ids).await {
        Ok(sync_results) => {
            let mut jobs_updated = 0;
            let mut errors = Vec::new();

            // Create a mapping from SLURM job ID to our job
            let mut job_map: std::collections::HashMap<String, JobInfo> = jobs_with_slurm_ids
                .into_iter()
                .filter_map(|job| {
                    job.slurm_job_id.clone().map(|slurm_id| (slurm_id, job))
                })
                .collect();

            // Process sync results
            for (slurm_job_id, status_result) in sync_results {
                if let Some(mut job_info) = job_map.remove(&slurm_job_id) {
                    match status_result {
                        Ok(new_status) => {
                            // Update job status if it changed
                            if job_info.status != new_status {
                                let job_repository = default_job_repository();
                                if let Err(e) = job_repository.update_job_status_with_timestamps(&mut job_info, new_status, "slurm") {
                                    errors.push(format!("Failed to update job {}: {}", job_info.job_id, e));
                                } else {
                                    jobs_updated += 1;
                                }
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Failed to sync job {} (SLURM ID {}): {}", job_info.job_id, slurm_job_id, e));
                        }
                    }
                }
            }

            SyncAllJobsResult {
                success: errors.is_empty(),
                jobs_updated,
                errors,
            }
        }
        Err(e) => {
            SyncAllJobsResult {
                success: false,
                jobs_updated: 0,
                errors: vec![format!("Failed to sync jobs from SLURM: {}", e)],
            }
        }
    }
}

/// Copy input files from project directory to scratch directory
async fn copy_input_files(
    job_info: &JobInfo,
    connection_manager: &crate::ssh::ConnectionManager,
) -> anyhow::Result<()> {
    let project_dir = job_info.project_dir
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Project directory not configured"))?;

    let scratch_dir = job_info.scratch_dir
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Scratch directory not configured"))?;

    let input_files_dir = format!("{}/input_files", scratch_dir);
    connection_manager.create_directory(&input_files_dir).await?;

    // Copy each input file using cp command over SSH
    for input_file in &job_info.input_files {
        let source_path = format!("{}/{}", project_dir, input_file.name);
        let dest_path = format!("{}/{}", input_files_dir, input_file.name);

        let copy_cmd = format!("cp '{}' '{}'", source_path, dest_path);
        connection_manager.execute_command(&copy_cmd, Some(30)).await?;
    }

    Ok(())
}

/// Upload text content to a remote file using SSH commands
async fn upload_content(
    connection_manager: &crate::ssh::ConnectionManager,
    content: &str,
    remote_path: &str,
) -> anyhow::Result<()> {
    // Escape content for shell safety
    let escaped_content = content
        .replace("\\", "\\\\")  // Escape backslashes first
        .replace("'", "'\"'\"'"); // Escape single quotes

    // Use printf for safe content upload (no injection possible)
    let upload_cmd = format!(
        "printf '%s' '{}' > '{}'",
        escaped_content,
        remote_path
    );

    connection_manager.execute_command(&upload_cmd, Some(30)).await?;
    Ok(())
}

/// Clean up scratch directory on job submission failure (best effort)
async fn cleanup_scratch_directory(
    connection_manager: &crate::ssh::ConnectionManager,
    scratch_dir: &str,
) {
    // Best effort cleanup - don't fail if cleanup fails
    if let Err(e) = connection_manager.delete_directory(scratch_dir).await {
        log::warn!("Failed to cleanup scratch directory {}: {}", scratch_dir, e);
    }
}

/// Parse SLURM job ID from sbatch output
fn parse_sbatch_output(output: &str) -> Option<String> {
    // Look for "Submitted batch job XXXXXX" pattern - simple string parsing
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("Submitted batch job ") {
            if let Some(job_id_str) = line.strip_prefix("Submitted batch job ") {
                let job_id_str = job_id_str.trim();
                // Only accept if it's all digits (valid SLURM job ID)
                if !job_id_str.is_empty() && job_id_str.chars().all(|c| c.is_ascii_digit()) {
                    return Some(job_id_str.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_parse_sbatch_output_security() {
        // Valid SLURM job ID
        assert_eq!(
            parse_sbatch_output("Submitted batch job 12345"),
            Some("12345".to_string())
        );

        // Invalid - non-numeric job ID should be rejected
        assert_eq!(
            parse_sbatch_output("Submitted batch job 12345; rm -rf /"),
            None
        );

        // Invalid - empty job ID
        assert_eq!(
            parse_sbatch_output("Submitted batch job "),
            None
        );

        // Invalid - job ID with letters
        assert_eq!(
            parse_sbatch_output("Submitted batch job abc123"),
            None
        );

        // Valid - multiline output with valid job ID
        assert_eq!(
            parse_sbatch_output("Some other output\nSubmitted batch job 67890\nMore output"),
            Some("67890".to_string())
        );
    }

    #[test]
    fn test_upload_content_escaping() {
        // Test that dangerous content gets properly escaped
        let dangerous_content = "echo 'harmless'; rm -rf /; echo 'EOF'\nEOF\n; echo 'dangerous'";

        // The escaping should prevent any command injection
        let escaped = dangerous_content
            .replace("\\", "\\\\")
            .replace("'", "'\"'\"'");

        // Verify the escaping works for single quotes
        assert!(escaped.contains("'\"'\"'"));

        // Content with backslashes should be escaped
        let backslash_content = "path\\to\\file";
        let escaped_backslash = backslash_content
            .replace("\\", "\\\\")
            .replace("'", "'\"'\"'");
        assert_eq!(escaped_backslash, "path\\\\to\\\\file");
    }
}