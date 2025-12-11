use anyhow::{Result, anyhow};
use crate::types::JobInfo;
use super::ConnectionManager;
use std::io::Write;
use tempfile::NamedTempFile;

/// Upload job metadata to remote server as job_info.json
///
/// This is the canonical way to persist JobInfo to the cluster.
/// Used by job creation, submission, and sync automations.
///
/// # Arguments
/// * `connection` - Active SSH connection manager
/// * `job` - JobInfo to serialize and upload
/// * `project_dir` - Project directory path (e.g., `/projects/user/{JOB_BASE_DIRECTORY}/job_001`)
/// * `log_context` - Context string for logging (e.g., "Job Creation", "Job Sync")
///
/// # Returns
/// * `Ok(())` on success
/// * `Err` if serialization or upload fails
pub async fn upload_job_metadata(
    connection: &ConnectionManager,
    job: &JobInfo,
    project_dir: &str,
    log_context: &str,
) -> Result<()> {
    use crate::{log_info, log_error, log_debug};

    // Serialize job info to pretty JSON
    let metadata = serde_json::to_string_pretty(job)
        .map_err(|e| {
            log_error!(category: log_context, message: "Failed to serialize job metadata", details: "{}", e);
            anyhow!("Failed to serialize job metadata: {}", e)
        })?;

    let metadata_path = format!("{}/job_info.json", project_dir);

    log_info!(category: log_context, message: "Uploading job metadata", details: "{}", metadata_path);

    // Upload using temporary file (secure, no command injection)
    upload_content(connection, &metadata, &metadata_path).await
        .map_err(|e| {
            log_error!(category: log_context, message: "Failed to upload job metadata", details: "{}", e);
            anyhow!("Failed to upload job metadata: {}", e)
        })?;

    log_debug!(category: log_context, message: "Job metadata uploaded successfully", details: "{}", metadata_path);

    Ok(())
}

/// Upload string content to remote path via SFTP using temporary file
///
/// This is a secure upload method that avoids shell command injection.
/// Creates a local temporary file, writes content to it, then uploads via SFTP.
///
/// This is a general-purpose helper for uploading any text content (scripts, configs, JSON, etc.)
pub async fn upload_content(
    connection: &ConnectionManager,
    content: &str,
    remote_path: &str,
) -> Result<()> {
    let mut temp_file = NamedTempFile::new()
        .map_err(|e| anyhow!("Failed to create temporary file: {}", e))?;

    temp_file.write_all(content.as_bytes())
        .map_err(|e| anyhow!("Failed to write to temporary file: {}", e))?;

    let temp_path = temp_file.path().to_string_lossy().to_string();

    connection.upload_file(&temp_path, remote_path, None, None).await
        .map_err(|e| anyhow!("Failed to upload to {}: {}", remote_path, e))?;

    Ok(())
}
