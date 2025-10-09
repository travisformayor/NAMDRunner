use anyhow::{Result, anyhow};
use tauri::AppHandle;
use std::path::Path;

use crate::types::{CreateJobParams, JobInfo, InputFile};
use crate::validation::{input, paths, files};
use crate::ssh::get_connection_manager;
use crate::database::with_database;
use crate::{info_log, debug_log, error_log};

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

    // Validate SSH connection is active
    let connection_manager = get_connection_manager();
    if !connection_manager.is_connected().await {
        error_log!("[Job Creation] SSH connection not active");
        return Err(anyhow!("Please connect to the cluster to create jobs"));
    }

    // Get cluster username using existing logic
    let username = connection_manager.get_username().await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to get username: {}", e);
            anyhow!("Failed to get cluster username: {}", e)
        })?;
    info_log!("[Job Creation] Creating job for user: {}", username);

    progress_callback("Generating job paths...");

    // Generate unique job ID and project directory path using existing validation functions
    let job_id = paths::generate_job_id(&clean_job_name);
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

    // Create subdirectories using existing path utilities
    for subdir in paths::job_subdirectories() {
        let subdir_path = format!("{}/{}", project_dir, subdir);
        debug_log!("[Job Creation] Creating subdirectory: {}", subdir_path);
        connection_manager.create_directory(&subdir_path).await
            .map_err(|e| {
                error_log!("[Job Creation] Failed to create subdirectory {}: {}", subdir_path, e);
                anyhow!("Failed to create subdirectory '{}': {}", subdir, e)
            })?;
    }

    progress_callback("Validating required files...");

    // Validate that required NAMD files are present (.pdb, .psf, .prm)
    let file_names: Vec<String> = params.input_files.iter()
        .map(|f| f.remote_name.clone().unwrap_or_else(|| {
            Path::new(&f.local_path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        }))
        .collect();

    files::validate_required_namd_files(&file_names)
        .map_err(|e| {
            error_log!("[Job Creation] Required file validation failed: {}", e);
            anyhow!("{}", e)
        })?;
    info_log!("[Job Creation] Required NAMD files validated successfully");

    progress_callback("Uploading input files...");

    // Upload input files if any are provided
    if !params.input_files.is_empty() {
        let input_files_dir = format!("{}/input_files", project_dir);
        info_log!("[Job Creation] Uploading {} input files", params.input_files.len());

        for (i, file) in params.input_files.iter().enumerate() {
            // Use the remote_name if provided, otherwise use the file name from local_path
            let remote_name = file.remote_name.as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| {
                    Path::new(&file.local_path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown_file")
                });

            progress_callback(&format!("Uploading file {} of {}: {}", i + 1, params.input_files.len(), remote_name));
            info_log!("[Job Creation] Uploading file {} of {}: {}", i + 1, params.input_files.len(), remote_name);

            // Validate file before upload
            validate_upload_file(file)?;
            debug_log!("[Job Creation] Validation passed for: {}", remote_name);

            // Construct remote path
            let remote_path = format!("{}/{}", input_files_dir, remote_name);

            // Upload file using ConnectionManager with progress events
            connection_manager.upload_file_with_progress(&file.local_path, &remote_path, Some(app_handle.clone())).await
                .map_err(|e| {
                    error_log!("[Job Creation] Failed to upload file {}: {}", remote_name, e);
                    anyhow!("Could not upload file '{}' to cluster: {}", remote_name, e)
                })?;
            info_log!("[Job Creation] Successfully uploaded: {} -> {}", file.local_path, remote_path);
        }
    }

    progress_callback("Creating job metadata...");

    // Create JobInfo with ONLY project directory set
    // scratch_dir remains None until job submission
    let mut job_info = JobInfo::new(
        job_id.clone(),
        clean_job_name,
        params.namd_config,
        params.slurm_config,
        params.input_files,
        project_dir.clone(),
    );

    // Set only project directory (this fixes the workflow separation issue)
    job_info.project_dir = Some(project_dir.clone());
    // job_info.scratch_dir remains None - set during submission only

    progress_callback("Generating SLURM batch script...");

    // Generate SLURM script using script generator
    // Note: Script generator requires scratch_dir to be set temporarily for working directory
    // We'll set a placeholder that will be replaced during submission
    let temp_scratch_dir = paths::scratch_directory(&username, &job_id)?;
    job_info.scratch_dir = Some(temp_scratch_dir.clone());

    let slurm_script = crate::slurm::script_generator::SlurmScriptGenerator::generate_namd_script(&job_info)?;
    info_log!("[Job Creation] Generated SLURM script ({} bytes)", slurm_script.len());

    // Clear scratch_dir again - it will be set properly during submission
    job_info.scratch_dir = None;

    // Upload script to scripts/ subdirectory
    let script_path = format!("{}/scripts/job.sbatch", project_dir);
    crate::ssh::metadata::upload_content(&connection_manager, &slurm_script, &script_path).await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to upload SLURM script: {}", e);
            anyhow!("Failed to upload SLURM script: {}", e)
        })?;
    debug_log!("[Job Creation] SLURM script uploaded to: {}", script_path);

    progress_callback("Generating NAMD configuration...");

    // Generate NAMD config
    let namd_config_content = crate::slurm::script_generator::SlurmScriptGenerator::generate_namd_config(&job_info)?;
    info_log!("[Job Creation] Generated NAMD config ({} bytes)", namd_config_content.len());

    // Upload config to scripts/ subdirectory
    let config_path = format!("{}/scripts/config.namd", project_dir);
    crate::ssh::metadata::upload_content(&connection_manager, &namd_config_content, &config_path).await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to upload NAMD config: {}", e);
            anyhow!("Failed to upload NAMD config: {}", e)
        })?;
    debug_log!("[Job Creation] NAMD config uploaded to: {}", config_path);

    progress_callback("Saving job to database...");
    debug_log!("[Job Creation] Saving job {} to database", job_id);

    // Save to database using existing database utilities
    with_database(|db| db.save_job(&job_info))
        .map_err(|e| {
            error_log!("[Job Creation] Failed to save job to database: {}", e);
            anyhow!("Failed to save job to database: {}", e)
        })?;

    progress_callback("Creating job metadata...");

    info_log!("[Job Creation] Creating job metadata at: {}/job_info.json", project_dir);
    crate::ssh::metadata::upload_job_metadata(&connection_manager, &job_info, &project_dir, "Job Creation").await
        .map_err(|e| {
            error_log!("[Job Creation] Failed to upload job metadata: {}", e);
            anyhow!("Failed to create job metadata: {}", e)
        })?;
    debug_log!("[Job Creation] Job metadata created: {}/job_info.json", project_dir);

    progress_callback("Job creation completed successfully");
    info_log!("[Job Creation] Job creation completed successfully: {}", job_id);

    Ok((job_id, job_info))
}

/// Validate a file upload request using centralized validation
fn validate_upload_file(file: &InputFile) -> Result<()> {
    // Convert to centralized validation format
    let validation_file = files::InputFile {
        local_path: file.local_path.clone(),
        remote_name: file.remote_name.clone(),
        file_type: match file.local_path.to_lowercase() {
            path if path.ends_with(".pdb") => files::FileType::PDB,
            path if path.ends_with(".psf") => files::FileType::PSF,
            path if path.ends_with(".prm") || path.ends_with(".par") || path.ends_with(".str") => files::FileType::Parameter,
            path if path.ends_with(".conf") || path.ends_with(".namd") => files::FileType::Config,
            _ => files::FileType::Other,
        },
    };

    // Use centralized validation
    files::validate_upload_file(&validation_file)
}

