use crate::types::*;
use crate::types::commands::ValidateJobConfigParams;
use crate::types::response_data;
use crate::validation::input;
use crate::validation::job_validation::ValidationResult;
use crate::database::with_database;
use crate::commands::helpers;
use crate::automations;
use crate::{info_log, error_log};
use tauri::Emitter;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_job(app_handle: tauri::AppHandle, params: CreateJobParams) -> ApiResult<JobInfo> {
    // Validate job name at command boundary
    let clean_job_name = match input::sanitize_job_id(&params.job_name) {
        Ok(name) => name,
        Err(e) => {
            return ApiResult::error(format!("Invalid job name: {}", e));
        }
    };

    // Create validated params
    let validated_params = CreateJobParams {
        job_name: clean_job_name,
        template_id: params.template_id,
        template_values: params.template_values,
        slurm_config: params.slurm_config,
    };

    // Call automation with progress tracking
    let handle_clone = app_handle.clone();

    match automations::execute_job_creation_with_progress(
        app_handle,
        validated_params,
        move |msg| {
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

    // Call automation with progress tracking
    let handle_clone = app_handle.clone();

    match automations::execute_job_submission_with_progress(
        app_handle,
        clean_job_id,
        move |msg| {
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
    info_log!("[Sync Jobs] Starting job sync");

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
pub async fn delete_job(job_id: String, delete_remote: bool, app_handle: tauri::AppHandle) -> ApiResult<()> {
    let clean_job_id = match input::sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(error) => {
            return ApiResult::error(error.to_string());
        }
    };

    let handle_clone = app_handle.clone();

    match automations::execute_job_deletion(
        clean_job_id,
        delete_remote,
        move |msg| {
            let _ = handle_clone.emit("job-deletion-progress", msg);
        }
    ).await {
        Ok(_) => ApiResult::success(()),
        Err(e) => ApiResult::error(e.to_string()),
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

    let slurm_config = crate::types::SlurmConfig {
        cores,
        memory,
        walltime,
        partition,
        qos,
    };

    match crate::slurm::script_generator::SlurmScriptGenerator::preview_script(job_name, slurm_config) {
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
    crate::validation::job_validation::validate_complete_job_config(params).await
}
