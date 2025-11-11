use crate::types::*;
use crate::types::response_data::DownloadInfo;
use crate::ssh::get_connection_manager;
use crate::validation::input::sanitize_job_id;
use crate::database::with_database;
use chrono::Utc;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

/// Detect NAMD file type from filename
/// Returns the detected file type (pdb, psf, prm, exb, or other)
#[tauri::command(rename_all = "snake_case")]
pub fn detect_file_type(filename: String) -> String {
    let file_type = NAMDFileType::from_filename(&filename);

    // Serialize to lowercase string matching frontend expectations
    match file_type {
        NAMDFileType::Pdb => "pdb".to_string(),
        NAMDFileType::Psf => "psf".to_string(),
        NAMDFileType::Prm => "prm".to_string(),
        NAMDFileType::Exb => "exb".to_string(),
        NAMDFileType::Other => "other".to_string(),
    }
}

/// Open a file dialog to select a single NAMD input file
/// Returns selected file path with metadata, or None if cancelled
#[tauri::command(rename_all = "snake_case")]
pub async fn select_input_file(_app: AppHandle) -> Result<Option<SelectedFile>, String> {
    use rfd::FileDialog;

    let file = FileDialog::new()
        .add_filter("NAMD Files", &["pdb", "psf", "prm", "exb"])
        .set_title("Select NAMD Input File")
        .pick_file();

    match file {
        Some(path) => {
            let path_str = path.to_string_lossy().to_string();

            // Get file metadata
            if let Ok(metadata) = fs::metadata(&path) {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy().to_string();
                    let extension = path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|s| format!(".{}", s))
                        .unwrap_or_else(|| String::from(""));

                    return Ok(Some(SelectedFile {
                        name: filename_str,
                        path: path_str,
                        size: metadata.len(),
                        file_type: extension,
                    }));
                }
            }

            Err("Failed to read file metadata".to_string())
        }
        None => Ok(None), // User cancelled
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn upload_job_files(app_handle: AppHandle, job_id: String, files: Vec<FileUpload>) -> UploadResult {
    upload_job_files_real(app_handle, job_id, files).await
}

// DELETED: upload_job_files_demo() - demo mode removed

async fn upload_job_files_real(app_handle: AppHandle, job_id: String, files: Vec<FileUpload>) -> UploadResult {
    // Validate and sanitize job ID
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return UploadResult {
            success: false,
            uploaded_files: None,
            failed_uploads: Some(vec![FailedUpload {
                file_name: "validation".to_string(),
                error: format!("Invalid job ID: {}", e),
            }]),
        },
    };

    // Get job info from database to find the project directory
    let job_id_for_db = clean_job_id.clone();
    let job_info = match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(job)) => job,
        Ok(None) => return UploadResult {
            success: false,
            uploaded_files: None,
            failed_uploads: Some(vec![FailedUpload {
                file_name: "job_lookup".to_string(),
                error: format!("Job {} not found", clean_job_id),
            }]),
        },
        Err(e) => return UploadResult {
            success: false,
            uploaded_files: None,
            failed_uploads: Some(vec![FailedUpload {
                file_name: "database".to_string(),
                error: format!("Database error: {}", e),
            }]),
        },
    };

    let project_dir = match &job_info.project_dir {
        Some(dir) => dir,
        None => return UploadResult {
            success: false,
            uploaded_files: None,
            failed_uploads: Some(vec![FailedUpload {
                file_name: "configuration".to_string(),
                error: "Job project directory not configured".to_string(),
            }]),
        },
    };

    let connection_manager = get_connection_manager();
    let mut uploaded_files = Vec::new();
    let mut failed_uploads = Vec::new();

    // Ensure input_files directory exists
    let input_files_dir = format!("{}/{}", project_dir, crate::ssh::JobDirectoryStructure::INPUT_FILES);
    if let Err(e) = connection_manager.create_directory(&input_files_dir).await {
        return UploadResult {
            success: false,
            uploaded_files: None,
            failed_uploads: Some(vec![FailedUpload {
                file_name: "directory_creation".to_string(),
                error: format!("Failed to create input_files directory: {}", e),
            }]),
        };
    }

    // Upload each file
    for file in files {
        // Validate local file exists and is readable
        match validate_upload_file(&file) {
            Ok(_) => {},
            Err(e) => {
                failed_uploads.push(FailedUpload {
                    file_name: file.remote_name,
                    error: format!("File validation failed: {}", e),
                });
                continue;
            }
        }

        // Construct remote path using centralized structure
        let remote_path = crate::ssh::JobDirectoryStructure::full_input_path(project_dir, &file.remote_name);

        // Upload file using ConnectionManager with progress events
        match connection_manager.upload_file_with_progress(&file.local_path, &remote_path, Some(app_handle.clone())).await {
            Ok(_progress) => {
                uploaded_files.push(file.remote_name);
            }
            Err(e) => {
                failed_uploads.push(FailedUpload {
                    file_name: file.remote_name,
                    error: format!("Upload failed: {}", e),
                });
            }
        }
    }

    UploadResult {
        success: failed_uploads.is_empty(),
        uploaded_files: Some(uploaded_files),
        failed_uploads: Some(failed_uploads),
    }
}

/// Validate a file upload request
fn validate_upload_file(file: &FileUpload) -> Result<()> {
    // Check local file exists
    let local_path = Path::new(&file.local_path);
    if !local_path.exists() {
        return Err(anyhow!("Local file does not exist: {}", file.local_path));
    }

    // Check file is readable
    if let Err(e) = fs::File::open(local_path) {
        return Err(anyhow!("Cannot read local file: {}", e));
    }

    // Basic file size check (limit to 1GB for now)
    let metadata = fs::metadata(local_path)?;
    if metadata.len() > 1_073_741_824 {  // 1GB
        return Err(anyhow!("File too large: {} bytes (max 1GB)", metadata.len()));
    }

    // Validate remote filename (no path separators, no dangerous characters)
    if file.remote_name.contains('/') || file.remote_name.contains('\\') {
        return Err(anyhow!("Remote filename cannot contain path separators"));
    }

    if file.remote_name.contains('\0') || file.remote_name.is_empty() {
        return Err(anyhow!("Invalid remote filename"));
    }

    // Basic file type validation for NAMD files
    let extension = local_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "pdb" | "psf" | "prm" | "rtf" | "namd" | "conf" | "dat" | "coor" | "vel" | "xsc" => {
            // Valid NAMD file types
            Ok(())
        }
        _ => {
            // Allow other file types but warn
            log::warn!("Uploading file with non-standard NAMD extension: {}", extension);
            Ok(())
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_job_output(job_id: String, file_path: String) -> ApiResult<DownloadInfo> {
    download_job_output_real(job_id, file_path).await
}

// DELETED: download_job_output_demo - demo mode removed

async fn download_job_output_real(job_id: String, file_path: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize inputs
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ApiResult::error(format!("Invalid job ID: {}", e)),
    };

    // Validate file path using centralized validation
    if let Err(e) = crate::validation::input::validate_relative_file_path(&file_path) {
        return ApiResult::error(format!("Invalid file path: {}", e));
    }

    // Get job info from database to find project directory
    let job_id_for_db = clean_job_id.clone();
    let job_info = match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(job)) => job,
        Ok(None) => return ApiResult::error(format!("Job {} not found", clean_job_id)),
        Err(e) => return ApiResult::error(format!("Database error: {}", e)),
    };

    let project_dir = match &job_info.project_dir {
        Some(dir) => dir,
        None => return ApiResult::error("Job has no project directory".to_string()),
    };

    // Build full remote path (project_dir + relative path)
    let remote_path = format!("{}/{}", project_dir, file_path);

    // Extract filename for save dialog
    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output.dat");

    let connection_manager = get_connection_manager();

    // Check if file exists
    match connection_manager.file_exists(&remote_path).await {
        Ok(true) => {
            // Show save dialog
            let save_path = FileDialog::new()
                .set_file_name(file_name)
                .set_title("Save Output File")
                .save_file();

            let save_path = match save_path {
                Some(path) => path,
                None => return ApiResult::error("Download cancelled".to_string()),
            };

            // Download file from server and write to chosen location
            match connection_manager.download_file(&remote_path, &save_path.to_string_lossy()).await {
                Ok(progress) => {
                    ApiResult::success(DownloadInfo {
                        saved_to: save_path.to_string_lossy().to_string(),
                        file_size: progress.total_bytes,
                    })
                }
                Err(e) => {
                    ApiResult::error(format!("Download failed: {}", e))
                }
            }
        }
        Ok(false) => {
            ApiResult::error(format!("File '{}' not found", file_path))
        }
        Err(e) => {
            ApiResult::error(format!("Error checking file: {}", e))
        }
    }
}

/// Download all output files as a zip archive
#[tauri::command(rename_all = "snake_case")]
pub async fn download_all_outputs(job_id: String) -> ApiResult<DownloadInfo> {
    download_all_outputs_real(job_id).await
}

async fn download_all_outputs_real(job_id: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize job_id
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ApiResult::error(format!("Invalid job ID: {}", e)),
    };

    // Get job info from database
    let job_info = match with_database(|db| db.load_job(&clean_job_id)) {
        Ok(Some(job)) => job,
        Ok(None) => return ApiResult::error(format!("Job '{}' not found", clean_job_id)),
        Err(e) => return ApiResult::error(format!("Database error: {}", e)),
    };

    let project_dir: &str = match &job_info.project_dir {
        Some(dir) => dir.as_str(),
        None => return ApiResult::error("Job project directory not configured".to_string()),
    };

    // Generate zip file path and command
    let (zip_command, temp_zip_path) = match crate::ssh::commands::zip_outputs_command(project_dir, &clean_job_id) {
        Ok(result) => result,
        Err(e) => return ApiResult::error(format!("Failed to generate zip command: {}", e)),
    };

    let connection_manager = get_connection_manager();

    // Execute zip command on server
    match connection_manager.execute_command(&zip_command, None).await {
        Ok(result) => {
            if result.exit_code != 0 {
                return ApiResult::error(format!("Failed to create zip file: {}", result.stderr));
            }
        }
        Err(e) => {
            return ApiResult::error(format!("Failed to execute zip command: {}", e));
        }
    }

    // Show save dialog
    let save_path = FileDialog::new()
        .set_file_name(format!("{}_outputs.zip", clean_job_id))
        .set_title("Save Output Files")
        .add_filter("ZIP Archive", &["zip"])
        .save_file();

    let save_path = match save_path {
        Some(path) => path,
        None => {
            // User cancelled - clean up temp file
            cleanup_temp_file(connection_manager, &temp_zip_path).await;

            return ApiResult::error("Download cancelled".to_string());
        }
    };

    // Download the zip file
    let download_result = match connection_manager.download_file(&temp_zip_path, &save_path.to_string_lossy()).await {
        Ok(progress) => ApiResult::success(DownloadInfo {
            saved_to: save_path.to_string_lossy().to_string(),
            file_size: progress.total_bytes,
        }),
        Err(e) => ApiResult::error(format!("Download failed: {}", e)),
    };

    // Clean up temporary zip file on server
    cleanup_temp_file(connection_manager, &temp_zip_path).await;

    download_result
}

/// Helper to clean up temporary files on the server
/// Logs errors but doesn't fail - cleanup is best-effort
async fn cleanup_temp_file(connection_manager: &crate::ssh::ConnectionManager, file_path: &str) {
    match crate::ssh::commands::remove_temp_file_command(file_path) {
        Ok(cleanup_cmd) => {
            if let Err(e) = connection_manager.execute_command(&cleanup_cmd, None).await {
                log::warn!("Failed to clean up temporary file {}: {}", file_path, e);
            }
        }
        Err(e) => {
            log::warn!("Failed to generate cleanup command for {}: {}", file_path, e);
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_job_files(job_id: String) -> ApiResult<Vec<RemoteFile>> {
    list_job_files_real(job_id).await
}

// DELETED: list_job_files_demo - demo mode removed

async fn list_job_files_real(job_id: String) -> ApiResult<Vec<RemoteFile>> {
    // Validate and sanitize job ID
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ApiResult::error(format!("Invalid job ID: {}", e)),
    };

    // Get job info from database to find directories
    let job_id_for_db = clean_job_id.clone();
    let job_info = match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(job)) => job,
        Ok(None) => return ApiResult::error(format!("Job {} not found", clean_job_id)),
        Err(e) => return ApiResult::error(format!("Database error: {}", e)),
    };

    let connection_manager = get_connection_manager();
    let mut all_files = Vec::new();

    // List files from all subdirectories
    let directories_to_check = get_directories_to_list(&job_info);

    for (dir_path, relative_prefix) in directories_to_check {
        match connection_manager.list_files(&dir_path).await {
            Ok(remote_files) => {
                for remote_file in remote_files {
                    // Skip directories in file listing
                    if remote_file.is_directory {
                        continue;
                    }

                    // Build full relative path
                    let relative_path = if relative_prefix.is_empty() {
                        remote_file.name.clone()
                    } else {
                        format!("{}/{}", relative_prefix, remote_file.name)
                    };

                    // Classify file type based on name and location
                    let file_type = classify_file_type(&remote_file.name, &relative_prefix);
                    let modified_at = remote_file.modified_time
                        .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| Utc::now().to_rfc3339());

                    all_files.push(RemoteFile {
                        name: remote_file.name,
                        path: relative_path,
                        size: remote_file.size,
                        modified_at,
                        file_type,
                    });
                }
            }
            Err(e) => {
                // Log warning but continue with other directories
                log::warn!("Failed to list files in directory {}: {}", dir_path, e);
            }
        }
    }

    ApiResult::success(all_files)
}

/// Get directories to list for a job
/// Get directories to list with their relative path prefixes
/// App ONLY lists from project directory - never scratch (which is temporary)
fn get_directories_to_list(job_info: &JobInfo) -> Vec<(String, String)> {
    let mut directories = Vec::new();

    // Only list from project directory - scratch is temporary and could be wiped
    if let Some(project_dir) = &job_info.project_dir {
        // List from each subdirectory with its relative path prefix
        directories.push((
            format!("{}/{}", project_dir, crate::ssh::JobDirectoryStructure::INPUT_FILES),
            crate::ssh::JobDirectoryStructure::INPUT_FILES.to_string()
        ));
        directories.push((
            format!("{}/{}", project_dir, crate::ssh::JobDirectoryStructure::OUTPUTS),
            crate::ssh::JobDirectoryStructure::OUTPUTS.to_string()
        ));
        // Also list root directory files (like job_info.json, SLURM logs)
        directories.push((project_dir.clone(), String::new()));
    }

    directories
}

/// Classify file type based on name and directory location
fn classify_file_type(filename: &str, relative_prefix: &str) -> FileType {
    use crate::ssh::JobDirectoryStructure;

    let extension = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Classify based on directory location first (most reliable)
    if relative_prefix == JobDirectoryStructure::INPUT_FILES {
        return FileType::Input;
    }
    if relative_prefix == JobDirectoryStructure::OUTPUTS {
        return FileType::Output;
    }

    // For root directory files, classify by extension and name
    match extension.as_str() {
        // NAMD configuration
        "namd" | "conf" => FileType::Config,
        // SLURM script
        "sbatch" | "sh" => FileType::Config,
        // SLURM logs
        "out" | "err" if filename.contains("_") => FileType::Log,  // SLURM logs have job_id format
        // Other logs
        "log" => FileType::Log,
        // Default for unknown root files
        _ => {
            if filename.contains("sbatch") || filename.contains("config") || filename == "job_info.json" {
                FileType::Config
            } else {
                FileType::Log
            }
        }
    }
}

// DELETED: Tests using NAMDConfig - will be rewritten for template system
// DELETED: Test module using NAMDConfig - needs rewrite for template system