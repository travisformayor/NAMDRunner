use crate::types::*;
use crate::mock_state::{with_mock_state, get_mock_state, advance_job_progression};
use crate::validation::{input, paths};
use chrono::Utc;
use std::env;

/// Check if we should use mock implementation
fn use_mock_mode() -> bool {
    // Check environment variable
    if let Ok(val) = env::var("USE_MOCK_SSH") {
        return val.to_lowercase() == "true" || val == "1";
    }

    // In debug builds, default to mock unless explicitly disabled
    #[cfg(debug_assertions)]
    {
        if let Ok(val) = env::var("USE_REAL_SSH") {
            return !(val.to_lowercase() == "true" || val == "1");
        }
        true // Default to mock in debug
    }

    // In release builds, default to real SSH
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

/// Get the username for cluster operations
/// TODO Phase 2.3: Replace with proper session management
fn get_cluster_username() -> String {
    // Try environment variable first for development/testing
    if let Ok(username) = env::var("NAMDRUNNER_USERNAME") {
        return username;
    }

    // Try system username as fallback
    if let Ok(username) = env::var("USER") {
        return username;
    }

    // Final fallback for development
    "testuser".to_string()
}

/// Validation layer for create job parameters
fn validate_create_job_params(params: &CreateJobParams) -> Result<(), String> {
    if params.job_name.is_empty() {
        return Err("Job name is required".to_string());
    }

    // Validate job name using security validation (same rules as job_id)
    match input::sanitize_job_id(&params.job_name) {
        Ok(_) => {}, // Name is valid
        Err(e) => return Err(format!("Invalid job name: {}", e)),
    }

    // Additional validation can be added here
    // - NAMD config validation
    // - SLURM config validation
    // - File existence checks

    Ok(())
}

/// Business logic layer for job creation
async fn create_job_business_logic(params: CreateJobParams) -> CreateJobResult {
    // Use mock implementation if in development mode or explicitly requested
    if use_mock_mode() {
        return create_job_mock(params).await;
    }

    // Real implementation with directory creation
    create_job_real(params).await
}

#[tauri::command]
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    // Input validation layer
    if let Err(error) = validate_create_job_params(&params) {
        return CreateJobResult {
            success: false,
            job_id: None,
            error: Some(error),
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

        let job_info = JobInfo {
            job_id: job_id.clone(),
            job_name: params.job_name.clone(),
            status: JobStatus::Created,
            slurm_job_id: None,
            created_at: now.clone(),
            updated_at: Some(now),
            submitted_at: None,
            completed_at: None,
            project_dir: Some(format!("/projects/{}/namdrunner_jobs/{}", get_cluster_username(), job_id)),
            scratch_dir: Some(format!("/scratch/alpine/{}/namdrunner_jobs/{}", get_cluster_username(), job_id)),
            error_info: None,
        };

        state.jobs.insert(job_id.clone(), job_info);

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
    let username = get_cluster_username();

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
    use crate::connection_utils::ConnectionUtils;

    // Create main project directory
    if let Err(e) = ConnectionUtils::create_directory_with_retry(&project_dir).await {
        return CreateJobResult {
            success: false,
            job_id: None,
            error: Some(format!("Failed to create project directory: {}", e)),
        };
    }

    // Create subdirectories
    for subdir in paths::job_subdirectories() {
        let subdir_path = format!("{}/{}", project_dir, subdir);
        if let Err(e) = ConnectionUtils::create_directory_with_retry(&subdir_path).await {
            return CreateJobResult {
                success: false,
                job_id: None,
                error: Some(format!("Failed to create subdirectory '{}': {}", subdir, e)),
            };
        }
    }

    // TODO: In a real implementation, this would be saved to SQLite database
    // For now, we'll still use the mock state for job tracking
    let now = Utc::now().to_rfc3339();
    let job_info = JobInfo {
        job_id: job_id.clone(),
        job_name: clean_job_name,
        status: JobStatus::Created,
        slurm_job_id: None,
        created_at: now.clone(),
        updated_at: Some(now),
        submitted_at: None,
        completed_at: None,
        project_dir: Some(project_dir),
        scratch_dir: Some(scratch_dir),
        error_info: None,
    };

    // Store job information in mock state
    // TODO Phase 2.3: Replace with SQLite database storage
    with_mock_state(|state| {
        state.jobs.insert(job_id.clone(), job_info);
        CreateJobResult {
            success: true,
            job_id: Some(job_id),
            error: None,
        }
    }).unwrap_or_else(|| CreateJobResult {
        success: false,
        job_id: None,
        error: Some("Failed to store job information".to_string()),
    })
}

/// Validation layer for submit job parameters
fn validate_submit_job_params(job_id: &str) -> Result<String, String> {
    match input::sanitize_job_id(job_id) {
        Ok(id) => Ok(id),
        Err(e) => Err(format!("Invalid job ID: {}", e)),
    }
}

/// Business logic layer for job submission
async fn submit_job_business_logic(clean_job_id: String) -> SubmitJobResult {
    // Use mock implementation if in development mode or explicitly requested
    if use_mock_mode() {
        return submit_job_mock(clean_job_id).await;
    }

    // Real implementation with scratch directory creation
    submit_job_real(clean_job_id).await
}

#[tauri::command]
pub async fn submit_job(job_id: String) -> SubmitJobResult {
    // Input validation layer
    let clean_job_id = match validate_submit_job_params(&job_id) {
        Ok(id) => id,
        Err(error) => {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(error),
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
    use crate::connection_utils::ConnectionUtils;

    // Get job information from mock state
    // TODO Phase 2.3: Replace with SQLite database query
    let job_info = get_mock_state(|state| {
        state.jobs.get(&job_id).cloned()
    }).flatten();

    let job_info = match job_info {
        Some(info) => info,
        None => {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Job {} not found", job_id)),
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
    if let Err(e) = ConnectionUtils::create_directory_with_retry(&scratch_dir).await {
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
        if let Err(e) = ConnectionUtils::create_directory_with_retry(&subdir_path).await {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(format!("Failed to create scratch subdirectory '{}': {}", subdir, e)),
            };
        }
    }

    // TODO: Here we would normally:
    // 1. Copy files from project directory to scratch directory
    // 2. Generate SLURM submission script
    // 3. Submit job to SLURM queue
    // For now, we'll simulate with a mock SLURM job ID

    let slurm_job_id = format!("slurm_{}", Utc::now().timestamp());
    let now = Utc::now().to_rfc3339();

    // Update job status in mock state
    // TODO Phase 2.3: Replace with SQLite database update
    with_mock_state(|state| {
        if let Some(job) = state.jobs.get_mut(&job_id) {
            job.status = JobStatus::Pending;
            job.slurm_job_id = Some(slurm_job_id.clone());
            job.submitted_at = Some(now.clone());
            job.updated_at = Some(now.clone());

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
                error: Some(format!("Job {} not found during update", job_id)),
            }
        }
    }).unwrap_or_else(|| SubmitJobResult {
        success: false,
        slurm_job_id: None,
        submitted_at: None,
        error: Some("Failed to update job information".to_string()),
    })
}

#[tauri::command]
pub async fn get_job_status(job_id: String) -> JobStatusResult {
    // NOTE: Using mock state for delay simulation in development
    // This provides realistic UI behavior during development
    // In production, this would be replaced with actual SLURM query latency
    let delay = get_mock_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Retrieve job from mock state
    // TODO Phase 2.3: Replace with SQLite database query
    get_mock_state(|state| {
        if let Some(job) = state.jobs.get(&job_id) {
            JobStatusResult {
                success: true,
                job_info: Some(job.clone()),
                error: None,
            }
        } else {
            JobStatusResult {
                success: false,
                job_info: None,
                error: Some(format!("Job {} not found", job_id)),
            }
        }
    }).unwrap_or_else(|| JobStatusResult {
        success: false,
        job_info: None,
        error: Some("Failed to access mock state".to_string()),
    })
}

#[tauri::command]
pub async fn get_all_jobs() -> GetAllJobsResult {
    // NOTE: Using mock state for delay simulation in development
    // This provides realistic UI behavior during development
    // In production, this would be replaced with actual database query latency
    let delay = get_mock_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Retrieve all jobs from mock state
    // TODO Phase 2.3: Replace with SQLite database query
    get_mock_state(|state| {
        let jobs: Vec<JobInfo> = state.jobs.values().cloned().collect();

        GetAllJobsResult {
            success: true,
            jobs: Some(jobs),
            error: None,
        }
    }).unwrap_or_else(|| GetAllJobsResult {
        success: false,
        jobs: None,
        error: Some("Failed to access mock state".to_string()),
    })
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
fn validate_delete_job_params(job_id: &str) -> Result<String, String> {
    match input::sanitize_job_id(job_id) {
        Ok(id) => Ok(id),
        Err(e) => Err(format!("Invalid job ID: {}", e)),
    }
}

/// Business logic layer for job deletion
async fn delete_job_business_logic(clean_job_id: String, delete_remote: bool) -> DeleteJobResult {
    // Use mock implementation if in development mode or explicitly requested
    if use_mock_mode() {
        return delete_job_mock(clean_job_id, delete_remote).await;
    }

    // Real implementation with safe directory cleanup
    delete_job_real(clean_job_id, delete_remote).await
}

#[tauri::command]
pub async fn delete_job(job_id: String, delete_remote: bool) -> DeleteJobResult {
    // Input validation layer
    let clean_job_id = match validate_delete_job_params(&job_id) {
        Ok(id) => id,
        Err(error) => {
            return DeleteJobResult {
                success: false,
                error: Some(error),
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
        if state.jobs.remove(&job_id).is_some() {
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
    use crate::connection_utils::ConnectionUtils;

    // Get job information from mock state
    // TODO Phase 2.3: Replace with SQLite database query
    let job_info = get_mock_state(|state| {
        state.jobs.get(&job_id).cloned()
    }).flatten();

    let job_info = match job_info {
        Some(info) => info,
        None => {
            return DeleteJobResult {
                success: false,
                error: Some(format!("Job {} not found", job_id)),
            };
        }
    };

    // If delete_remote is requested, clean up directories
    if delete_remote {
        // Check if connected using ConnectionUtils
        if let Err(e) = ConnectionUtils::ensure_connected().await {
            return DeleteJobResult {
                success: false,
                error: Some(format!("Cannot delete remote files: {}", e)),
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

            // Use ConnectionUtils to safely delete directory with retry logic
            if let Err(e) = ConnectionUtils::delete_directory_with_retry(&dir_path).await {
                return DeleteJobResult {
                    success: false,
                    error: Some(format!("Failed to delete {} directory '{}': {}", dir_type, dir_path, e)),
                };
            }
        }
    }

    // Remove job from mock state
    // TODO Phase 2.3: Replace with SQLite database deletion
    with_mock_state(|state| {
        if state.jobs.remove(&job_id).is_some() {
            DeleteJobResult {
                success: true,
                error: None,
            }
        } else {
            DeleteJobResult {
                success: false,
                error: Some(format!("Job {} not found during removal", job_id)),
            }
        }
    }).unwrap_or_else(|| DeleteJobResult {
        success: false,
        error: Some("Failed to access job state".to_string()),
    })
}