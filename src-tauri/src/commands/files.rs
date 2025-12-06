// File operation commands - thin wrappers over automation layer
// UI concerns (file dialogs) handled here, business logic in automations/file_operations

use crate::types::*;
use crate::types::response_data::DownloadInfo;
use crate::automations;
use crate::validation::input::sanitize_job_id;
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

/// Download a single output file from a job
#[tauri::command(rename_all = "snake_case")]
pub async fn download_job_output(job_id: String, file_path: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize job ID
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ApiResult::error(format!("Invalid job ID: {}", e)),
    };

    // Extract filename for save dialog
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output.dat");

    // Show save dialog (UI concern - handled in command layer)
    let save_path = FileDialog::new()
        .set_file_name(file_name)
        .set_title("Save Output File")
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

/// Download all output files as a zip archive
#[tauri::command(rename_all = "snake_case")]
pub async fn download_all_outputs(job_id: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize job ID
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ApiResult::error(format!("Invalid job ID: {}", e)),
    };

    // Show save dialog (UI concern)
    let save_path = FileDialog::new()
        .set_file_name(format!("{}_outputs.zip", clean_job_id))
        .set_title("Save Output Files")
        .add_filter("ZIP Archive", &["zip"])
        .save_file();

    let save_path = match save_path {
        Some(path) => path,
        None => return ApiResult::error("Download cancelled".to_string()),
    };

    // Delegate to automation layer (business logic)
    match automations::download_all_files_as_zip(&clean_job_id, "outputs", &save_path.to_string_lossy()).await {
        Ok(info) => ApiResult::success(info),
        Err(e) => ApiResult::error(e.to_string()),
    }
}

/// Download a single input file from a job
#[tauri::command(rename_all = "snake_case")]
pub async fn download_job_input(job_id: String, file_path: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize job ID
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ApiResult::error(format!("Invalid job ID: {}", e)),
    };

    // Extract filename for save dialog
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("input.dat");

    // Show save dialog (UI concern)
    let save_path = FileDialog::new()
        .set_file_name(file_name)
        .set_title("Save Input File")
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

/// Download all input files as a zip archive
#[tauri::command(rename_all = "snake_case")]
pub async fn download_all_inputs(job_id: String) -> ApiResult<DownloadInfo> {
    use rfd::FileDialog;

    // Validate and sanitize job ID
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ApiResult::error(format!("Invalid job ID: {}", e)),
    };

    // Show save dialog (UI concern)
    let save_path = FileDialog::new()
        .set_file_name(format!("{}_inputs.zip", clean_job_id))
        .set_title("Save Input Files")
        .add_filter("ZIP Archive", &["zip"])
        .save_file();

    let save_path = match save_path {
        Some(path) => path,
        None => return ApiResult::error("Download cancelled".to_string()),
    };

    // Delegate to automation layer (business logic)
    match automations::download_all_files_as_zip(&clean_job_id, "inputs", &save_path.to_string_lossy()).await {
        Ok(info) => ApiResult::success(info),
        Err(e) => ApiResult::error(e.to_string()),
    }
}
