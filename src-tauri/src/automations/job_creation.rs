use anyhow::{Result, anyhow};
use tauri::{AppHandle, Emitter};
use std::collections::HashMap;
use serde_json::Value;

use crate::types::{CreateJobParams, JobInfo, JobStatus, SlurmConfig};
use crate::validation::{input, paths};
use crate::{info_log, debug_log, error_log};
use crate::automations::common;

/// Factory function to create a new JobInfo with business logic (status, timestamps)
///
/// This is the correct way to create new jobs with proper initial state.
pub fn create_job_info(
    job_id: String,
    job_name: String,
    template_id: String,
    template_values: HashMap<String, Value>,
    slurm_config: SlurmConfig,
    remote_directory: String,
) -> JobInfo {
    JobInfo {
        job_id,
        job_name,
        status: JobStatus::Created,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: None,
        submitted_at: None,
        completed_at: None,
        slurm_job_id: None,
        project_dir: None,
        scratch_dir: None,
        error_info: None,
        slurm_stdout: None,
        slurm_stderr: None,
        template_id,
        template_values,
        slurm_config,
        output_files: None,
        remote_directory,
    }
}

/// Simplified job creation automation that follows NAMDRunner's direct function patterns
/// This replaces the complex AutomationStep trait system with a simple async function
/// that provides progress reporting through callbacks.
///
/// Key improvement: Job creation ONLY creates project directories, NOT scratch directories.
/// Scratch directories are created during job submission, maintaining proper workflow separation.
pub async fn execute_job_creation_with_progress(
    app_handle: AppHandle,
    params: CreateJobParams,
    progress_callback: impl Fn(&str),
) -> Result<(String, JobInfo)> {
    progress_callback("Starting job creation...");
    info_log!("[Job Creation] Starting job creation for: {}", params.job_name);

    // Validate and sanitize job name
    let clean_job_name = input::sanitize_job_id(&params.job_name)
        .map_err(|e| anyhow!("Invalid job name: {}", e))?;
    debug_log!("[Job Creation] Sanitized job name: {}", clean_job_name);

    progress_callback("Validating connection...");

    // Validate SSH connection and get username
    let (connection_manager, username) = common::require_connection_with_username("Job Creation").await?;
    info_log!("[Job Creation] Creating job for user: {}", username);

    progress_callback("Generating job paths...");

    // Generate unique job ID using timestamp
    let job_id = format!("{}_{}", clean_job_name, chrono::Utc::now().timestamp_micros());
    let project_dir = paths::project_directory(&username, &job_id)?;
    info_log!("[Job Creation] Generated job ID: {} at path: {}", job_id, project_dir);

    progress_callback("Creating project directories...");

    // Create project directory structure
    info_log!("[Job Creation] Creating project directory: {}", project_dir);
    connection_manager.create_directory(&project_dir).await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to create directory {}: {}", project_dir, e);
            anyhow!("Could not create job directory on cluster: {}", e)
        })?;

    // Create standard job subdirectories
    for subdir in crate::ssh::JobDirectoryStructure::subdirectories() {
        let subdir_path = format!("{}/{}", project_dir, subdir);
        debug_log!("[Job Creation] Creating subdirectory: {}", subdir_path);
        connection_manager.create_directory(&subdir_path).await
            .map_err(|e| {
                error_log!("[Job Creation] Failed to create subdirectory {}: {}", subdir_path, e);
                anyhow!("Failed to create subdirectory '{}': {}", subdir, e)
            })?;
    }

    progress_callback("Loading template...");

    // Load template from database (before moving params)
    let template_id_for_db = params.template_id.clone();
    let template = crate::database::with_database(|db| {
        db.load_template(&template_id_for_db)
    })?
        .ok_or_else(|| anyhow!("Template not found: {}", params.template_id))?;

    info_log!("[Job Creation] Loaded template: {}", template.name);

    progress_callback("Uploading input files...");

    // Upload files from template_values
    // Extract FileUpload variables from template and upload their files
    let mut template_values_for_rendering = params.template_values.clone();

    // First pass: collect all filenames to upload and emit the list to frontend
    let mut files_to_upload: Vec<(String, String, String)> = Vec::new(); // (var_key, local_path, filename)
    for (var_key, var_def) in &template.variables {
        if matches!(var_def.var_type, crate::templates::VariableType::FileUpload { .. }) {
            if let Some(file_path_value) = params.template_values.get(var_key) {
                if let Some(local_file_path) = file_path_value.as_str() {
                    if !local_file_path.is_empty() {
                        let filename = std::path::Path::new(local_file_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .ok_or_else(|| anyhow!("Invalid filename in {}: {}", var_key, local_file_path))?
                            .to_string();

                        files_to_upload.push((var_key.clone(), local_file_path.to_string(), filename));
                    }
                }
            }
        }
    }

    // Emit the file list to frontend for progress tracking
    let file_names: Vec<String> = files_to_upload.iter().map(|(_, _, name)| name.clone()).collect();
    let _ = app_handle.emit("file-upload-list", file_names);
    info_log!("[Job Creation] Emitted file upload list: {} files", files_to_upload.len());

    // Second pass: upload each file
    for (var_key, local_file_path, filename) in files_to_upload {
        progress_callback(&format!("Uploading file: {}", filename));
        info_log!("[Job Creation] Uploading file for {}: {} -> {}", var_key, local_file_path, filename);

        // Construct remote path
        let remote_path = crate::ssh::JobDirectoryStructure::full_input_path(&project_dir, &filename);

        // Upload file
        connection_manager.upload_file_with_progress(&local_file_path, &remote_path, Some(app_handle.clone())).await
            .map_err(|e| {
                error_log!("[Job Creation] Failed to upload file {}: {}", filename, e);
                anyhow!("Could not upload file '{}': {}", filename, e)
            })?;

        info_log!("[Job Creation] Successfully uploaded: {} -> {}", local_file_path, remote_path);

        // Update template_values with just the filename (not full path)
        // The renderer will prepend "input_files/" when rendering the template
        template_values_for_rendering.insert(var_key, Value::String(filename));
    }

    progress_callback("Rendering template...");

    // Render NAMD config from template with uploaded filenames
    let namd_config_content = crate::templates::render_template(&template, &template_values_for_rendering)?;
    info_log!("[Job Creation] Rendered NAMD config ({} bytes)", namd_config_content.len());

    progress_callback("Creating job metadata...");

    // Create JobInfo using factory function (sets Created status and timestamp)
    // scratch_dir remains None until job submission
    // Use template_values_for_rendering (filenames only) instead of original params (full paths)
    let mut job_info = create_job_info(
        job_id.clone(),
        clean_job_name,
        params.template_id,
        template_values_for_rendering.clone(),
        params.slurm_config,
        project_dir.clone(),
    );

    // Set only project directory (this fixes the workflow separation issue)
    job_info.project_dir = Some(project_dir.clone());
    // job_info.scratch_dir remains None - set during submission only
    info_log!("[Job Creation] Rendered NAMD config ({} bytes)", namd_config_content.len());

    progress_callback("Generating SLURM batch script...");

    // Generate SLURM script using script generator
    // Pass scratch directory directly (job_info.scratch_dir remains None until submission)
    let scratch_dir = paths::scratch_directory(&username, &job_id)?;
    let slurm_script = crate::slurm::script_generator::SlurmScriptGenerator::generate_namd_script(&job_info, &scratch_dir)?;
    info_log!("[Job Creation] Generated SLURM script ({} bytes)", slurm_script.len());

    // Upload script to job root directory
    let script_path = format!("{}/job.sbatch", project_dir);
    crate::ssh::metadata::upload_content(connection_manager, &slurm_script, &script_path).await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to upload SLURM script: {}", e);
            anyhow!("Failed to upload SLURM script: {}", e)
        })?;
    debug_log!("[Job Creation] SLURM script uploaded to: {}", script_path);

    progress_callback("Uploading NAMD configuration...");

    // Upload rendered config to job root directory
    let config_path = format!("{}/config.namd", project_dir);
    crate::ssh::metadata::upload_content(connection_manager, &namd_config_content, &config_path).await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to upload NAMD config: {}", e);
            anyhow!("Failed to upload NAMD config: {}", e)
        })?;
    debug_log!("[Job Creation] NAMD config uploaded to: {}", config_path);

    progress_callback("Saving job to database...");
    debug_log!("[Job Creation] Saving job {} to database", job_id);

    // Save to database using common helper
    common::save_job_to_database(&job_info, "Job Creation")?;

    progress_callback("Creating job metadata...");

    info_log!("[Job Creation] Creating job metadata at: {}/job_info.json", project_dir);
    crate::ssh::metadata::upload_job_metadata(connection_manager, &job_info, &project_dir, "Job Creation").await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to upload job metadata: {}", e);
            anyhow!("Failed to create job metadata: {}", e)
        })?;
    debug_log!("[Job Creation] Job metadata created: {}/job_info.json", project_dir);

    progress_callback("Job creation completed successfully");
    info_log!("[Job Creation] Job creation completed successfully: {}", job_id);

    Ok((job_id, job_info))
}

// File validation will be handled by template validation
// TODO: Implement template-based file validation in Step 7
