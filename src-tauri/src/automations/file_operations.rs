// File operations automation module
// Handles file validation, classification, upload, download, and listing operations
// All functions are pure business logic with no UI dependencies

use anyhow::{Result, anyhow};
use std::path::Path;

use crate::types::FileUpload;
use crate::types::response_data::DownloadInfo;
use crate::ssh::ConnectionManager;
use crate::security::input;
use crate::{log_info, log_debug};
use crate::automations::common;
use crate::commands::helpers;

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

    Ok(())
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
    let job_info = helpers::load_job_or_fail(job_id, "File Download")?;
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
pub async fn download_files_zip(
    job_id: &str,
    file_type: &str, // "inputs" or "outputs"
    local_destination: &str,
) -> Result<DownloadInfo> {
    log_info!(category: "File Operations", message: "Creating zip archive", details: "{}: {}", job_id, file_type);

    // Load job and validate connection
    let job_info = helpers::load_job_or_fail(job_id, "File Download")?;
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
