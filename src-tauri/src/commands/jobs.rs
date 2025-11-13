use crate::types::*;
use crate::types::commands::ValidateJobConfigParams;
use crate::types::response_data;
use crate::validation::input;
use crate::validation::job_validation::ValidationResult;
use crate::database::with_database;
use crate::commands::helpers;
use crate::automations;
use crate::{info_log, debug_log, error_log};
use tauri::Emitter;
use serde::Serialize;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_job(app_handle: tauri::AppHandle, params: CreateJobParams) -> ApiResult<JobInfo> {
    // Early validation at command boundary - sanitize job name immediately
    let clean_job_name = match input::sanitize_job_id(&params.job_name) {
        Ok(name) => name,
        Err(e) => {
            return ApiResult::error(format!("Invalid job name: {}", e));
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

async fn create_job_real(app_handle: tauri::AppHandle, params: CreateJobParams) -> ApiResult<JobInfo> {
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
        Ok((_job_id, job_info)) => ApiResult::success(job_info),
        Err(e) => ApiResult::error(e.to_string()),
    }
}


#[tauri::command(rename_all = "snake_case")]
pub async fn submit_job(job_id: String, app_handle: tauri::AppHandle) -> ApiResult<response_data::JobSubmissionData> {
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(error) => {
            return ApiResult::error(error.to_string());
        }
    };

    submit_job_real(app_handle, clean_job_id).await
}

async fn submit_job_real(app_handle: tauri::AppHandle, job_id: String) -> ApiResult<response_data::JobSubmissionData> {
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
        Ok(data) => ApiResult::success(data),
        Err(e) => ApiResult::error(e.to_string()),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_job_status(job_id: String) -> ApiResult<JobInfo> {
    // Retrieve job from database
    let job_id_for_db = job_id.clone();
    match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(job)) => ApiResult::success(job),
        Ok(None) => ApiResult::error(format!("Job {} not found", job_id)),
        Err(e) => ApiResult::error(format!("Failed to load job {}: {}", job_id, e)),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_jobs() -> ApiResult<Vec<JobInfo>> {
    info_log!("[Get All Jobs] Loading jobs from database");

    match with_database(|db| db.load_all_jobs()) {
        Ok(jobs) => ApiResult::success(jobs),
        Err(e) => {
            error_log!("[Get All Jobs] Database error: {}", e);
            ApiResult::error(format!("Failed to load jobs: {}", e))
        }
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
pub async fn delete_job(job_id: String, delete_remote: bool) -> ApiResult<()> {
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(error) => {
            return ApiResult::error(error.to_string());
        }
    };

    delete_job_real(clean_job_id, delete_remote).await
}

async fn delete_job_real(job_id: String, delete_remote: bool) -> ApiResult<()> {
    // Real implementation with safe directory cleanup

    // Get job information from database
    let job_info = match helpers::load_job_or_fail(&job_id, "Delete Job") {
        Ok(info) => info,
        Err(e) => return ApiResult::error(e.to_string()),
    };

    // Cancel SLURM job if it's still actively running or pending
    if matches!(job_info.status, crate::types::JobStatus::Pending | crate::types::JobStatus::Running) {
        if let Some(slurm_job_id) = &job_info.slurm_job_id {
            // Check if connected to cluster and get username
            if let Err(e) = helpers::require_connection("Delete Job").await {
                return ApiResult::error(e.to_string());
            }

            let username = match helpers::get_cluster_username("Delete Job").await {
                Ok(user) => user,
                Err(e) => return ApiResult::error(e.to_string()),
            };

            // Cancel the SLURM job to prevent orphaned cluster jobs
            let slurm_sync = crate::slurm::status::SlurmStatusSync::new(&username);
            if let Err(e) = slurm_sync.cancel_job(slurm_job_id).await {
                return ApiResult::error(format!("Failed to cancel SLURM job {}: {}", slurm_job_id, e));
            }

            info_log!("[Delete Job] Successfully cancelled SLURM job: {}", slurm_job_id);
        }
    }

    // If delete_remote is requested, clean up directories
    if delete_remote {
        // Check if connected to cluster
        let connection_manager = crate::ssh::get_connection_manager();
        if !connection_manager.is_connected().await {
            return ApiResult::error("Cannot delete remote files: Not connected to cluster".to_string());
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
                return ApiResult::error(format!("Refusing to delete directory '{}' - not a NAMDRunner job directory", dir_path));
            }

            // Additional safety check: path should not contain dangerous patterns
            if dir_path.contains("..") || dir_path == "/" || dir_path.starts_with("/etc") || dir_path.starts_with("/usr") {
                return ApiResult::error(format!("Refusing to delete dangerous directory path: {}", dir_path));
            }

            // Use ConnectionManager to safely delete directory with retry logic
            if let Err(e) = connection_manager.delete_directory(&dir_path).await {
                return ApiResult::error(format!("Failed to delete {} directory '{}': {}", dir_type, dir_path, e));
            }
        }
    }

    // Remove job from database

    match with_database(move |db| db.delete_job(&job_info.job_id)) {
        Ok(true) => ApiResult::success(()),
        Ok(false) => ApiResult::error(format!("Job {} not found during removal", job_id)),
        Err(e) => ApiResult::error(format!("Failed to delete job from database: {}", e)),
    }
}

/// Refetch SLURM logs from server, overwriting cached logs
/// Used when user explicitly clicks "Refetch Logs" button
#[tauri::command(rename_all = "snake_case")]
pub async fn refetch_slurm_logs(job_id: String) -> ApiResult<JobInfo> {
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => {
            return ApiResult::error(format!("Invalid job ID: {}", e));
        }
    };

    // Get job from database
    let mut job_info = match helpers::load_job_or_fail(&clean_job_id, "Refetch Logs") {
        Ok(job) => job,
        Err(e) => return ApiResult::error(e.to_string()),
    };

    // Refetch logs from server
    if let Err(e) = automations::refetch_slurm_logs(&mut job_info).await {
        return ApiResult::error(format!("Failed to refetch logs: {}", e));
    }

    // Save updated job to database
    let job_clone = job_info.clone();
    if let Err(e) = with_database(move |db| db.save_job(&job_clone)) {
        return ApiResult::error(format!("Failed to save updated logs: {}", e));
    }

    ApiResult::success(job_info)
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
) -> ApiResult<String> {
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
        scratch_dir: None,
        input_files: None,
        output_files: None,
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
            ApiResult::success(script)
        }
        Err(e) => {
            error_log!("[Jobs] SLURM script preview failed: {}", e);
            ApiResult::error(format!("Script generation error: {}", e))
        }
    }
}

/// Validate complete job configuration
/// Checks job name, template selection, template values, and resource configuration
#[tauri::command(rename_all = "snake_case")]
pub async fn validate_job_config(params: ValidateJobConfigParams) -> ValidationResult {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Validate job name
    if params.job_name.trim().is_empty() {
        issues.push("Job name is required".to_string());
    } else if let Err(e) = input::sanitize_job_id(&params.job_name) {
        issues.push(format!("Job name invalid: {}", e));
    }

    // Validate template selection
    if params.template_id.is_empty() {
        issues.push("Template selection is required".to_string());
    }

    // Validate template values (if template selected)
    if !params.template_id.is_empty() {
        let template_validation = crate::commands::templates::validate_template_values(
            params.template_id.clone(),
            params.template_values.clone()
        ).await;

        // Merge template validation results
        issues.extend(template_validation.issues);
        warnings.extend(template_validation.warnings);
        suggestions.extend(template_validation.suggestions);
    }

    // Validate resource configuration using centralized validator
    let partition_id = params.partition.as_deref().unwrap_or("amilan");
    let qos_id = params.qos.as_deref().unwrap_or("normal");

    // Build SlurmConfig for validation
    let slurm_config = crate::types::SlurmConfig {
        cores: params.cores,
        memory: params.memory,
        walltime: params.walltime,
        partition: Some(partition_id.to_string()),
        qos: Some(qos_id.to_string()),
    };

    // Use centralized resource validation
    let resource_validation = crate::validation::job_validation::validate_resource_allocation(
        &slurm_config,
        partition_id,
        qos_id
    );

    // Merge resource validation results
    issues.extend(resource_validation.issues);
    warnings.extend(resource_validation.warnings);
    suggestions.extend(resource_validation.suggestions);

    ValidationResult {
        is_valid: issues.is_empty(),
        issues,
        warnings,
        suggestions,
    }
}
