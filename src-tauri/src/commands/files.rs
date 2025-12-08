// File operation commands - thin wrappers over automation layer
// UI concerns (file dialogs) handled here, business logic in automations/file_operations

use crate::types::*;
use crate::types::response_data::DownloadInfo;
use crate::automations;
use crate::commands::helpers;
use tauri::AppHandle;

/// Open a file dialog to select a single NAMD input file
/// Returns selected file path with metadata, or None if cancelled
#[tauri::command(rename_all = "snake_case")]
pub async fn select_input_file(_app: AppHandle) -> Result<Option<SelectedFile>, String> {
    use rfd::FileDialog;

    let file = FileDialog::new()
        .set_title("Select NAMD Input File")
        .pick_file();

    match file {
        Some(path) => {
            let path_str = path.to_string_lossy().to_string();

            // Get file metadata
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy().to_string();

                    return Ok(Some(SelectedFile {
                        name: filename_str,
                        path: path_str,
                        size: metadata.len(),
                    }));
                }
            }

            Err("Failed to read file metadata".to_string())
        }
        None => Ok(None), // User cancelled
    }
}

/// Download a single file from a job (input or output)
#[tauri::command(rename_all = "snake_case")]
pub async fn download_file(job_id: String, file_type: String, file_path: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize job ID
    let sanitize_result = helpers::sanitize_command_job_id(&job_id);
    if !sanitize_result.success {
        return ApiResult::error(sanitize_result.error.unwrap_or_else(|| "Invalid job ID".to_string()));
    }
    let clean_job_id = sanitize_result.data.unwrap();

    // Validate file type
    if file_type != "input" && file_type != "output" {
        return ApiResult::error(format!("Invalid file type: {}", file_type));
    }

    // Extract filename for save dialog
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.dat");

    // Show save dialog (UI concern - handled in command layer)
    let title = if file_type == "input" { "Save Input File" } else { "Save Output File" };
    let save_path = FileDialog::new()
        .set_file_name(file_name)
        .set_title(title)
        .save_file();

    let save_path = match save_path {
        Some(path) => path,
        None => return ApiResult::error("Download cancelled".to_string()),
    };

    // Delegate to automation layer (business logic)
    match automations::download_job_file(&clean_job_id, &file_path, &save_path.to_string_lossy()).await {
        Ok(info) => ApiResult::success(info),
        Err(e) => ApiResult::error(e.to_string()),
    }
}

/// Download all files as a zip archive (inputs or outputs)
#[tauri::command(rename_all = "snake_case")]
pub async fn download_all_files(job_id: String, file_type: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize job ID
    let sanitize_result = helpers::sanitize_command_job_id(&job_id);
    if !sanitize_result.success {
        return ApiResult::error(sanitize_result.error.unwrap_or_else(|| "Invalid job ID".to_string()));
    }
    let clean_job_id = sanitize_result.data.unwrap();

    // Validate file type
    if file_type != "input" && file_type != "output" {
        return ApiResult::error(format!("Invalid file type: {}", file_type));
    }

    // Determine folder and dialog settings
    let (folder, title) = if file_type == "input" {
        ("inputs", "Save Input Files")
    } else {
        ("outputs", "Save Output Files")
    };

    // Show save dialog (UI concern)
    let save_path = FileDialog::new()
        .set_file_name(format!("{}_{}.zip", clean_job_id, folder))
        .set_title(title)
        .add_filter("ZIP Archive", &["zip"])
        .save_file();

    let save_path = match save_path {
        Some(path) => path,
        None => return ApiResult::error("Download cancelled".to_string()),
    };

    // Delegate to automation layer (business logic)
    match automations::download_files_zip(&clean_job_id, folder, &save_path.to_string_lossy()).await {
        Ok(info) => ApiResult::success(info),
        Err(e) => ApiResult::error(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_file_invalid_job_id() {
        let result = download_file(
            "../invalid".to_string(),
            "output".to_string(),
            "test.dat".to_string()
        ).await;

        assert!(!result.success);
        // Error is "Job ID contains invalid path sequences" from sanitize_job_id
        assert!(result.error.as_ref().unwrap().contains("Job ID"));
    }

    #[tokio::test]
    async fn test_download_file_invalid_file_type() {
        let result = download_file(
            "test_job_123".to_string(),
            "invalid_type".to_string(),
            "test.dat".to_string()
        ).await;

        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Invalid file type"));
    }

    #[tokio::test]
    async fn test_download_all_files_invalid_job_id() {
        let result = download_all_files(
            "../../../etc/passwd".to_string(),
            "output".to_string()
        ).await;

        assert!(!result.success);
        // Error is "Job ID contains invalid path sequences" from sanitize_job_id
        assert!(result.error.as_ref().unwrap().contains("Job ID"));
    }

    #[tokio::test]
    async fn test_download_all_files_invalid_file_type() {
        let result = download_all_files(
            "test_job_123".to_string(),
            "bad_type".to_string()
        ).await;

        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Invalid file type"));
    }

    #[tokio::test]
    async fn test_download_file_validates_both_input_and_output_types() {
        // Test that both "input" and "output" are valid file types
        // These will fail with "Download cancelled" because file dialog returns None
        // but should NOT fail with "Invalid file type"

        let result = download_file(
            "valid_job_id".to_string(),
            "input".to_string(),
            "test.dat".to_string()
        ).await;
        // Should fail with "Download cancelled" not "Invalid file type"
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Download cancelled"));

        let result = download_file(
            "valid_job_id".to_string(),
            "output".to_string(),
            "test.dat".to_string()
        ).await;
        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("Download cancelled"));
    }
}
