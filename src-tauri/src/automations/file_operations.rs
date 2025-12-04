// File operations automation module
// Handles file validation, classification, upload, download, and listing operations
// All functions are pure business logic with no UI dependencies

use anyhow::{Result, anyhow};
use std::path::Path;
use tauri::AppHandle;

use crate::types::{FileUpload, UploadResult, FailedUpload, RemoteFile, FileType};
use crate::types::response_data::DownloadInfo;
use crate::ssh::{ConnectionManager, JobDirectoryStructure};
use crate::validation::input;
use crate::{log_info, log_debug, log_error};
use crate::automations::common;

/// Validate a file for upload
/// Checks: file exists, readable, size limits, safe filename
/// Returns Ok(()) if valid, Err with descriptive message if invalid
pub fn validate_upload_file(file: &FileUpload) -> Result<()> {
    log_debug!(category: "File Operations", message: "Validating upload file", details: "{}", file.remote_name);

    // Check local file exists
    let local_path = Path::new(&file.local_path);
    if !local_path.exists() {
        return Err(anyhow!("Local file does not exist: {}", file.local_path));
    }

    // Check file is readable
    if let Err(e) = std::fs::File::open(local_path) {
        return Err(anyhow!("Cannot read local file: {}", e));
    }

    // File size check (1GB limit)
    let metadata = std::fs::metadata(local_path)?;
    if metadata.len() > 1_073_741_824 {
        return Err(anyhow!("File too large: {} bytes (max 1GB)", metadata.len()));
    }

    // Validate remote filename (no path separators, no dangerous characters)
    if file.remote_name.contains('/') || file.remote_name.contains('\\') {
        return Err(anyhow!("Remote filename cannot contain path separators"));
    }

    if file.remote_name.contains('\0') || file.remote_name.is_empty() {
        return Err(anyhow!("Invalid remote filename"));
    }

    // File type validation for NAMD files (warning only for non-standard types)
    let extension = local_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "pdb" | "psf" | "prm" | "rtf" | "namd" | "conf" | "dat" | "coor" | "vel" | "xsc" | "exb" => {
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

/// Classify file type based on filename and directory location
/// Pure function - no I/O, just logic
pub fn classify_file_type(filename: &str, relative_prefix: &str) -> FileType {
    log_debug!(category: "File Operations", message: "Classifying file type", details: "{} in {}", filename, relative_prefix);

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
        // SLURM logs (have underscore format: job_123456.out)
        "out" | "err" if filename.contains("_") => FileType::Log,
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

/// Get directories to list for a job with their relative path prefixes
/// App only lists from project directory - scratch is temporary and may be wiped
/// Returns Vec<(full_path, relative_prefix)>
pub fn get_directories_to_list(project_dir: &str) -> Vec<(String, String)> {
    log_debug!(category: "File Operations", message: "Getting directories to list", details: "{}", project_dir);

    vec![
        // Input files subdirectory
        (
            format!("{}/{}", project_dir, JobDirectoryStructure::INPUT_FILES),
            JobDirectoryStructure::INPUT_FILES.to_string()
        ),
        // Outputs subdirectory
        (
            format!("{}/{}", project_dir, JobDirectoryStructure::OUTPUTS),
            JobDirectoryStructure::OUTPUTS.to_string()
        ),
        // Root directory files (job_info.json, SLURM logs, etc.)
        (project_dir.to_string(), String::new()),
    ]
}

/// Upload files to a job's input directory with progress tracking
/// Pure business logic - no file dialogs or UI concerns
pub async fn upload_files_to_job(
    job_id: &str,
    files: Vec<FileUpload>,
    app_handle: Option<AppHandle>,
    progress_callback: impl Fn(&str),
) -> Result<UploadResult> {
    progress_callback("Validating job and connection...");
    log_info!(category: "File Operations", message: "Starting file upload", details: "{}: {} files", job_id, files.len());

    // Load job and validate connection
    let job_info = common::load_job_or_fail(job_id, "File Upload")?;
    let (connection_manager, _username) = common::require_connection_with_username("File Upload").await?;
    let project_dir = common::require_project_dir(&job_info, "File Upload")?;

    // Ensure input_files directory exists
    let input_files_dir = format!("{}/{}", project_dir, JobDirectoryStructure::INPUT_FILES);
    progress_callback("Creating input directory...");
    connection_manager.create_directory(&input_files_dir).await
        .map_err(|e| anyhow!("Failed to create input_files directory: {}", e))?;

    let mut uploaded_files = Vec::new();
    let mut failed_uploads = Vec::new();

    // Upload each file
    for file in files {
        // Validate file
        if let Err(e) = validate_upload_file(&file) {
            failed_uploads.push(FailedUpload {
                file_name: file.remote_name,
                error: format!("Validation failed: {}", e),
            });
            continue;
        }

        // Construct remote path
        let remote_path = JobDirectoryStructure::full_input_path(project_dir, &file.remote_name);

        // Upload with progress
        progress_callback(&format!("Uploading {}...", file.remote_name));
        log_debug!(category: "File Upload", message: "Uploading file", details: "{} -> {}", file.local_path, remote_path);

        match connection_manager.upload_file_with_progress(&file.local_path, &remote_path, app_handle.clone()).await {
            Ok(_) => {
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

    let success = failed_uploads.is_empty();
    if success {
        log_info!(category: "File Operations", message: "All files uploaded successfully", details: "{} files", uploaded_files.len());
    } else {
        log_error!(category: "File Operations", message: "Some uploads failed", details: "{} succeeded, {} failed", uploaded_files.len(), failed_uploads.len());
    }

    Ok(UploadResult {
        success,
        uploaded_files: Some(uploaded_files),
        failed_uploads: Some(failed_uploads),
    })
}

/// Download a single file from a job
/// Returns download info (path and size)
pub async fn download_job_file(
    job_id: &str,
    file_path: &str,
    local_destination: &str,
) -> Result<DownloadInfo> {
    log_info!(category: "File Operations", message: "Downloading file", details: "{}: {}", job_id, file_path);

    // Validate inputs
    input::validate_relative_file_path(file_path)
        .map_err(|e| anyhow!("Invalid file path: {}", e))?;

    // Load job and validate connection
    let job_info = common::load_job_or_fail(job_id, "File Download")?;
    let (connection_manager, _username) = common::require_connection_with_username("File Download").await?;
    let project_dir = common::require_project_dir(&job_info, "File Download")?;

    // Build full remote path
    let remote_path = format!("{}/{}", project_dir, file_path);

    // Check if file exists
    if !connection_manager.file_exists(&remote_path).await? {
        return Err(anyhow!("File '{}' not found", file_path));
    }

    // Download file
    log_debug!(category: "File Download", message: "Downloading from remote", details: "{} -> {}", remote_path, local_destination);
    let progress = connection_manager.download_file(&remote_path, local_destination).await
        .map_err(|e| anyhow!("Download failed: {}", e))?;

    Ok(DownloadInfo {
        saved_to: local_destination.to_string(),
        file_size: progress.total_bytes,
    })
}

/// Download all files of a specific type as a zip archive
/// file_type: "inputs" or "outputs"
pub async fn download_all_files_as_zip(
    job_id: &str,
    file_type: &str, // "inputs" or "outputs"
    local_destination: &str,
) -> Result<DownloadInfo> {
    log_info!(category: "File Operations", message: "Creating zip archive", details: "{}: {}", job_id, file_type);

    // Load job and validate connection
    let job_info = common::load_job_or_fail(job_id, "File Download")?;
    let (connection_manager, _username) = common::require_connection_with_username("File Download").await?;
    let project_dir = common::require_project_dir(&job_info, "File Download")?;

    // Generate zip command based on file type
    let (zip_command, temp_zip_path) = match file_type {
        "inputs" => crate::ssh::commands::zip_inputs_command(project_dir, job_id)?,
        "outputs" => crate::ssh::commands::zip_outputs_command(project_dir, job_id)?,
        _ => return Err(anyhow!("Invalid file type: {}", file_type)),
    };

    // Execute zip command on server
    log_debug!(category: "File Download", message: "Creating zip on server", details: "{}", zip_command);
    let result = connection_manager.execute_command(&zip_command, None).await?;
    if result.exit_code != 0 {
        return Err(anyhow!("Failed to create zip file: {}", result.stderr));
    }

    // Download the zip file
    log_debug!(category: "File Download", message: "Downloading zip file", details: "{} -> {}", temp_zip_path, local_destination);
    let download_result = connection_manager.download_file(&temp_zip_path, local_destination).await;

    // Clean up temporary zip file (best effort, don't fail if cleanup fails)
    cleanup_temp_file(connection_manager, &temp_zip_path).await;

    // Return download result
    let progress = download_result.map_err(|e| anyhow!("Download failed: {}", e))?;

    log_info!(category: "File Operations", message: "Zip download completed", details: "{} bytes", progress.total_bytes);

    Ok(DownloadInfo {
        saved_to: local_destination.to_string(),
        file_size: progress.total_bytes,
    })
}

/// List all files in a job's directories
/// Returns files from input_files/, outputs/, and root directory
pub async fn list_job_files(job_id: &str) -> Result<Vec<RemoteFile>> {
    log_info!(category: "File Operations", message: "Listing job files", details: "{}", job_id);

    // Load job and validate connection
    let job_info = common::load_job_or_fail(job_id, "File Listing")?;
    let (connection_manager, _username) = common::require_connection_with_username("File Listing").await?;
    let project_dir = common::require_project_dir(&job_info, "File Listing")?;

    let mut all_files = Vec::new();
    let directories = get_directories_to_list(project_dir);

    // List files from each directory
    for (dir_path, relative_prefix) in directories {
        match connection_manager.list_files(&dir_path).await {
            Ok(remote_files) => {
                for remote_file in remote_files {
                    // Skip directories
                    if remote_file.is_directory {
                        continue;
                    }

                    // Build full relative path
                    let relative_path = if relative_prefix.is_empty() {
                        remote_file.name.clone()
                    } else {
                        format!("{}/{}", relative_prefix, remote_file.name)
                    };

                    // Classify file type
                    let file_type = classify_file_type(&remote_file.name, &relative_prefix);

                    // Format timestamp
                    let modified_at = remote_file.modified_time
                        .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

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

    log_info!(category: "File Operations", message: "File listing completed", details: "{} files found", all_files.len());

    Ok(all_files)
}

/// Clean up a temporary file on the server
/// Best-effort operation - logs errors but doesn't fail
pub async fn cleanup_temp_file(connection_manager: &ConnectionManager, file_path: &str) {
    log_debug!(category: "File Operations", message: "Cleaning up temporary file", details: "{}", file_path);

    match crate::ssh::commands::remove_temp_file_command(file_path) {
        Ok(cleanup_cmd) => {
            if let Err(e) = connection_manager.execute_command(&cleanup_cmd, None).await {
                log::warn!("Failed to clean up temporary file {}: {}", file_path, e);
            } else {
                log_debug!(category: "File Operations", message: "Temporary file cleaned up", details: "{}", file_path);
            }
        }
        Err(e) => {
            log::warn!("Failed to generate cleanup command for {}: {}", file_path, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_file_type_by_location() {
        // Files in input_files directory
        assert_eq!(
            classify_file_type("structure.pdb", JobDirectoryStructure::INPUT_FILES),
            FileType::Input
        );

        // Files in outputs directory
        assert_eq!(
            classify_file_type("trajectory.dcd", JobDirectoryStructure::OUTPUTS),
            FileType::Output
        );
    }

    #[test]
    fn test_classify_file_type_by_extension() {
        // NAMD config files
        assert_eq!(classify_file_type("simulation.namd", ""), FileType::Config);
        assert_eq!(classify_file_type("config.conf", ""), FileType::Config);

        // SLURM scripts
        assert_eq!(classify_file_type("job.sbatch", ""), FileType::Config);
        assert_eq!(classify_file_type("run.sh", ""), FileType::Config);

        // SLURM logs (with underscore pattern)
        assert_eq!(classify_file_type("job_123456.out", ""), FileType::Log);
        assert_eq!(classify_file_type("job_123456.err", ""), FileType::Log);

        // Regular logs
        assert_eq!(classify_file_type("simulation.log", ""), FileType::Log);
    }

    #[test]
    fn test_classify_file_type_by_name_pattern() {
        // Special filenames
        assert_eq!(classify_file_type("job_info.json", ""), FileType::Config);
        assert_eq!(classify_file_type("job.sbatch", ""), FileType::Config);
    }

    #[test]
    fn test_get_directories_to_list() {
        let project_dir = "/projects/testuser/namdrunner_jobs/job_123";
        let dirs = get_directories_to_list(project_dir);

        // Should return 3 directories
        assert_eq!(dirs.len(), 3);

        // Check input_files directory
        assert_eq!(dirs[0].0, format!("{}/input_files", project_dir));
        assert_eq!(dirs[0].1, "input_files");

        // Check outputs directory
        assert_eq!(dirs[1].0, format!("{}/outputs", project_dir));
        assert_eq!(dirs[1].1, "outputs");

        // Check root directory
        assert_eq!(dirs[2].0, project_dir);
        assert_eq!(dirs[2].1, "");
    }

    #[test]
    fn test_validate_upload_file_path_traversal() {
        // Test path traversal in remote name (tested before filesystem checks)
        // Create a temporary file first
        let temp_file = std::env::temp_dir().join("namdrunner_test_file.pdb");
        std::fs::write(&temp_file, b"test content").expect("Failed to create test file");

        let file = FileUpload {
            local_path: temp_file.to_string_lossy().to_string(),
            remote_name: "../../../etc/passwd".to_string(),
        };

        let result = validate_upload_file(&file);

        // Clean up
        let _ = std::fs::remove_file(&temp_file);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path separators"));
    }

    #[test]
    fn test_validate_upload_file_empty_name() {
        // Create a temporary file first
        let temp_file = std::env::temp_dir().join("namdrunner_test_empty_name.pdb");
        std::fs::write(&temp_file, b"test content").expect("Failed to create test file");

        let file = FileUpload {
            local_path: temp_file.to_string_lossy().to_string(),
            remote_name: "".to_string(),
        };

        let result = validate_upload_file(&file);

        // Clean up
        let _ = std::fs::remove_file(&temp_file);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid remote filename"));
    }

    #[test]
    fn test_validate_upload_file_null_character() {
        // Create a temporary file first
        let temp_file = std::env::temp_dir().join("namdrunner_test_null.pdb");
        std::fs::write(&temp_file, b"test content").expect("Failed to create test file");

        let file = FileUpload {
            local_path: temp_file.to_string_lossy().to_string(),
            remote_name: "test\0file.pdb".to_string(),
        };

        let result = validate_upload_file(&file);

        // Clean up
        let _ = std::fs::remove_file(&temp_file);

        assert!(result.is_err());
    }
}
