use crate::types::*;
use crate::mock_state::{with_mock_state, get_mock_state, advance_job_progression};
use chrono::Utc;

#[tauri::command]
pub async fn create_job(params: CreateJobParams) -> CreateJobResult {
    // Enhanced mock implementation - create job using mock state manager
    // In Phase 2, this will persist to SQLite
    
    // Validate parameters
    if params.job_name.is_empty() {
        return CreateJobResult {
            success: false,
            job_id: None,
            error: Some("Job name is required".to_string()),
        };
    }
    
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
            project_dir: Some(format!("/projects/testuser/namdrunner_jobs/{}", job_id)),
            scratch_dir: Some(format!("/scratch/alpine/testuser/namdrunner_jobs/{}", job_id)),
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

#[tauri::command]
pub async fn submit_job(job_id: String) -> SubmitJobResult {
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

#[tauri::command]
pub async fn get_job_status(job_id: String) -> JobStatusResult {
    // Enhanced mock implementation - return job status with realistic delays
    
    let delay = get_mock_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    
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
    // Enhanced mock implementation - return all jobs with realistic delays
    
    let delay = get_mock_state(|state| state.get_delay("slurm") / 5).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    
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

#[tauri::command]
pub async fn delete_job(job_id: String, delete_remote: bool) -> DeleteJobResult {
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