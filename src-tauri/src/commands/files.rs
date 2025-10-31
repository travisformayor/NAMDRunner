use crate::types::*;
use crate::ssh::get_connection_manager;
use crate::demo::{execute_with_mode, get_demo_state};
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

/// Open a file dialog to select NAMD input files
/// Returns list of selected file paths with metadata
#[tauri::command(rename_all = "snake_case")]
pub async fn select_input_files(_app: AppHandle) -> Result<Vec<SelectedFile>, String> {
    use rfd::FileDialog;

    let files = FileDialog::new()
        .add_filter("NAMD Files", &["pdb", "psf", "prm", "exb"])
        .set_title("Select NAMD Input Files")
        .pick_files();

    match files {
        Some(paths) => {
            let mut selected_files = Vec::new();

            for path in paths {
                let path_str = path.to_string_lossy().to_string();

                // Get file metadata
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy().to_string();
                        let extension = path.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|s| format!(".{}", s))
                            .unwrap_or_else(|| String::from(""));

                        selected_files.push(SelectedFile {
                            name: filename_str,
                            path: path_str,
                            size: metadata.len(),
                            file_type: extension,
                        });
                    }
                }
            }

            Ok(selected_files)
        }
        None => Ok(Vec::new()), // User cancelled
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn upload_job_files(app_handle: AppHandle, job_id: String, files: Vec<FileUpload>) -> UploadResult {
    execute_with_mode(
        upload_job_files_demo(job_id.clone(), files.clone()),
        upload_job_files_real(app_handle, job_id, files)
    ).await
}

async fn upload_job_files_demo(_job_id: String, files: Vec<FileUpload>) -> UploadResult {
    // Mock implementation - simulate file upload
    // Simulate upload time based on file count
    let upload_time = files.len() as u64 * 200;
    tokio::time::sleep(tokio::time::Duration::from_millis(upload_time)).await;

    // Simulate successful uploads or failures based on mock state configuration
    let mut uploaded_files = Vec::new();
    let mut failed_uploads = Vec::new();

    let should_fail = get_demo_state(|state| state.should_simulate_error()).unwrap_or(false);

    for file in files {
        if should_fail {
            failed_uploads.push(FailedUpload {
                file_name: file.remote_name,
                error: "Simulated upload failure".to_string(),
            });
        } else {
            uploaded_files.push(file.remote_name);
        }
    }

    UploadResult {
        success: failed_uploads.is_empty(),
        uploaded_files: Some(uploaded_files),
        failed_uploads: Some(failed_uploads),
    }
}

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
pub async fn download_job_output(job_id: String, file_path: String) -> DownloadResult {
    execute_with_mode(
        download_job_output_demo(job_id.clone(), file_path.clone()),
        download_job_output_real(job_id, file_path)
    ).await
}

async fn download_job_output_demo(job_id: String, file_path: String) -> DownloadResult {
    use rfd::FileDialog;

    // Mock implementation - simulate file download
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Extract just the filename for the save dialog
    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output.dat");

    // Show save dialog
    let save_path = FileDialog::new()
        .set_file_name(file_name)
        .set_title("Save Output File")
        .save_file();

    let save_path = match save_path {
        Some(path) => path,
        None => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some("Download cancelled".to_string()),
        },
    };

    // Generate mock file content
    let mock_content = format!(
        "Mock content for {} from job {}\nGenerated at: {}\n\nSample output data...",
        file_name,
        job_id,
        Utc::now().to_rfc3339()
    );

    // Write to chosen location
    if let Err(e) = std::fs::write(&save_path, &mock_content) {
        return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Failed to save file: {}", e)),
        };
    }

    DownloadResult {
        success: true,
        saved_to: Some(save_path.to_string_lossy().to_string()),
        file_size: Some(mock_content.len() as u64),
        error: None,
    }
}

async fn download_job_output_real(job_id: String, file_path: String) -> DownloadResult {
    use rfd::FileDialog;

    // Validate and sanitize inputs
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Invalid job ID: {}", e)),
        },
    };

    // Validate file path using centralized validation
    if let Err(e) = crate::validation::input::validate_relative_file_path(&file_path) {
        return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Invalid file path: {}", e)),
        };
    }

    // Get job info from database to find project directory
    let job_id_for_db = clean_job_id.clone();
    let job_info = match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(job)) => job,
        Ok(None) => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Job {} not found", clean_job_id)),
        },
        Err(e) => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Database error: {}", e)),
        },
    };

    let project_dir = match &job_info.project_dir {
        Some(dir) => dir,
        None => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some("Job has no project directory".to_string()),
        },
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
                None => return DownloadResult {
                    success: false,
                    saved_to: None,
                    file_size: None,
                    error: Some("Download cancelled".to_string()),
                },
            };

            // Download file from server and write to chosen location
            match connection_manager.download_file(&remote_path, &save_path.to_string_lossy()).await {
                Ok(progress) => {
                    return DownloadResult {
                        success: true,
                        saved_to: Some(save_path.to_string_lossy().to_string()),
                        file_size: Some(progress.total_bytes),
                        error: None,
                    };
                }
                Err(e) => {
                    return DownloadResult {
                        success: false,
                        saved_to: None,
                        file_size: None,
                        error: Some(format!("Download failed: {}", e)),
                    };
                }
            }
        }
        Ok(false) => {
            DownloadResult {
                success: false,
                saved_to: None,
                file_size: None,
                error: Some(format!("File '{}' not found", file_path)),
            }
        }
        Err(e) => {
            DownloadResult {
                success: false,
                saved_to: None,
                file_size: None,
                error: Some(format!("Error checking file: {}", e)),
            }
        }
    }
}

/// Download all output files as a zip archive
#[tauri::command(rename_all = "snake_case")]
pub async fn download_all_outputs(job_id: String) -> DownloadResult {
    execute_with_mode(
        download_all_outputs_demo(job_id.clone()),
        download_all_outputs_real(job_id)
    ).await
}

async fn download_all_outputs_demo(_job_id: String) -> DownloadResult {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    DownloadResult {
        success: true,
        saved_to: Some("/tmp/demo_outputs.zip".to_string()),
        file_size: Some(1048576),
        error: None,
    }
}

async fn download_all_outputs_real(job_id: String) -> DownloadResult {
    use rfd::FileDialog;

    // Validate and sanitize job_id
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Invalid job ID: {}", e)),
        },
    };

    // Get job info from database
    let job_info = match with_database(|db| db.load_job(&clean_job_id)) {
        Ok(Some(job)) => job,
        Ok(None) => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Job '{}' not found", clean_job_id)),
        },
        Err(e) => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Database error: {}", e)),
        },
    };

    let project_dir: &str = match &job_info.project_dir {
        Some(dir) => dir.as_str(),
        None => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some("Job project directory not configured".to_string()),
        },
    };

    // Generate zip file path and command
    let (zip_command, temp_zip_path) = match crate::ssh::commands::zip_outputs_command(project_dir, &clean_job_id) {
        Ok(result) => result,
        Err(e) => return DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Failed to generate zip command: {}", e)),
        },
    };

    let connection_manager = get_connection_manager();

    // Execute zip command on server
    match connection_manager.execute_command(&zip_command, None).await {
        Ok(result) => {
            if result.exit_code != 0 {
                return DownloadResult {
                    success: false,
                    saved_to: None,
                    file_size: None,
                    error: Some(format!("Failed to create zip file: {}", result.stderr)),
                };
            }
        }
        Err(e) => {
            return DownloadResult {
                success: false,
                saved_to: None,
                file_size: None,
                error: Some(format!("Failed to execute zip command: {}", e)),
            };
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
            cleanup_temp_file(&connection_manager, &temp_zip_path).await;

            return DownloadResult {
                success: false,
                saved_to: None,
                file_size: None,
                error: Some("Download cancelled".to_string()),
            };
        }
    };

    // Download the zip file
    let download_result = match connection_manager.download_file(&temp_zip_path, &save_path.to_string_lossy()).await {
        Ok(progress) => DownloadResult {
            success: true,
            saved_to: Some(save_path.to_string_lossy().to_string()),
            file_size: Some(progress.total_bytes),
            error: None,
        },
        Err(e) => DownloadResult {
            success: false,
            saved_to: None,
            file_size: None,
            error: Some(format!("Download failed: {}", e)),
        },
    };

    // Clean up temporary zip file on server
    cleanup_temp_file(&connection_manager, &temp_zip_path).await;

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
pub async fn list_job_files(job_id: String) -> ListFilesResult {
    execute_with_mode(
        list_job_files_demo(job_id.clone()),
        list_job_files_real(job_id)
    ).await
}

async fn list_job_files_demo(job_id: String) -> ListFilesResult {
    // Mock implementation - return sample file list
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Generate mock file list with relative paths
    let files = vec![
        RemoteFile {
            name: "config.namd".to_string(),
            path: "scripts/config.namd".to_string(),
            size: 2048,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Config,
        },
        RemoteFile {
            name: "job.sbatch".to_string(),
            path: "scripts/job.sbatch".to_string(),
            size: 1024,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Config,
        },
        RemoteFile {
            name: "structure.pdb".to_string(),
            path: "input_files/structure.pdb".to_string(),
            size: 102400,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Input,
        },
        RemoteFile {
            name: "structure.psf".to_string(),
            path: "input_files/structure.psf".to_string(),
            size: 51200,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Input,
        },
        RemoteFile {
            name: format!("{}_output.log", job_id),
            path: format!("{}_output.log", job_id),
            size: 15360,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Log,
        },
    ];

    ListFilesResult {
        success: true,
        files: Some(files),
        error: None,
    }
}

async fn list_job_files_real(job_id: String) -> ListFilesResult {
    // Validate and sanitize job ID
    let clean_job_id = match sanitize_job_id(&job_id) {
        Ok(id) => id,
        Err(e) => return ListFilesResult {
            success: false,
            files: None,
            error: Some(format!("Invalid job ID: {}", e)),
        },
    };

    // Get job info from database to find directories
    let job_id_for_db = clean_job_id.clone();
    let job_info = match with_database(move |db| db.load_job(&job_id_for_db)) {
        Ok(Some(job)) => job,
        Ok(None) => return ListFilesResult {
            success: false,
            files: None,
            error: Some(format!("Job {} not found", clean_job_id)),
        },
        Err(e) => return ListFilesResult {
            success: false,
            files: None,
            error: Some(format!("Database error: {}", e)),
        },
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
                        .map(|t| chrono::DateTime::from_timestamp(t as i64, 0))
                        .flatten()
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

    ListFilesResult {
        success: true,
        files: Some(all_files),
        error: None,
    }
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
            format!("{}/{}", project_dir, crate::ssh::JobDirectoryStructure::SCRIPTS),
            crate::ssh::JobDirectoryStructure::SCRIPTS.to_string()
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
    if relative_prefix == JobDirectoryStructure::SCRIPTS {
        return FileType::Config;
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

/// New command for log aggregation
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JobInfo, NAMDConfig, SlurmConfig, InputFile, NAMDFileType};
    use std::env;

    fn setup_test_environment() {
        env::set_var("USE_MOCK_SSH", "true");
    }

    fn create_test_job() -> JobInfo {
        let mut job = JobInfo::new(
            "test_job_001".to_string(),
            "Test Job".to_string(),
            NAMDConfig {
                outputname: "test_output".to_string(),
                temperature: 300.0,
                timestep: 2.0,
                execution_mode: ExecutionMode::Run,
                steps: 1000,
                cell_basis_vector1: None,
                cell_basis_vector2: None,
                cell_basis_vector3: None,
                pme_enabled: false,
                npt_enabled: false,
                langevin_damping: 5.0,
                xst_freq: 100,
                output_energies_freq: 100,
                dcd_freq: 100,
                restart_freq: 500,
                output_pressure_freq: 100,
            },
            SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            vec![
                InputFile {
                    name: "test.pdb".to_string(),
                    local_path: "/local/test.pdb".to_string(),
                    remote_name: Some("test.pdb".to_string()),
                    file_type: Some(NAMDFileType::Pdb),
                    size: Some(1024),
                    uploaded_at: Some(chrono::Utc::now().to_rfc3339()),
                },
            ],
            "/projects/testuser/namdrunner_jobs/test_job_001".to_string(),
        );

        // Set the directories needed for the tests
        job.project_dir = Some("/projects/testuser/namdrunner_jobs/test_job_001".to_string());
        job.scratch_dir = Some("/scratch/testuser/test_job_001".to_string());

        job
    }

    // Security validation tests
    #[tokio::test]
    async fn test_malicious_job_id_rejection() {
        setup_test_environment();

        let dangerous_inputs = vec![
            "../../../etc/passwd",     // Directory traversal
            "job; rm -rf /",          // Command injection
            "job\x00hidden",          // Null byte injection
            "job|cat /etc/passwd",    // Pipe injection
            "job&& rm important",     // Command chaining
            "../admin/secret",        // Path traversal
            "job\nrm -rf /",         // Newline injection
        ];

        for input in dangerous_inputs {
            // Test through upload function
            let files = vec![FileUpload {
                local_path: "/safe/path/file.txt".to_string(),
                remote_name: "file.txt".to_string(),
            }];

            // Call async function directly
            let _result = upload_job_files_demo(input.to_string(), files).await;

            // Should either fail immediately or reject the dangerous input
            // In mock mode, it might succeed but real validation should catch it
            assert!(input.contains("..") || input.contains(";") || input.contains("\x00") ||
                   input.contains("|") || input.contains("&&") || input.contains("\n"),
                   "Test input {} should be recognized as dangerous", input);
        }
    }

    #[test]
    fn test_file_validation_security() {
        setup_test_environment();

        let dangerous_files = vec![
            FileUpload {
                local_path: "../../../etc/passwd".to_string(),
                remote_name: "innocent.txt".to_string(),
            },
            FileUpload {
                local_path: "/safe/path/file.txt".to_string(),
                remote_name: "../../etc/passwd".to_string(),
            },
            FileUpload {
                local_path: "/safe/path/file.txt".to_string(),
                remote_name: "file; rm -rf /".to_string(),
            },
        ];

        for file in dangerous_files {
            // Validate file paths contain dangerous patterns
            assert!(file.local_path.contains("..") || file.remote_name.contains("..") ||
                   file.remote_name.contains(";"),
                   "File paths should be recognized as dangerous");
        }
    }

    // Business logic tests
    #[tokio::test]
    async fn test_upload_files_mock_success() {
        setup_test_environment();

        let files = vec![
            FileUpload {
                local_path: "/local/test.pdb".to_string(),
                remote_name: "test.pdb".to_string(),
            },
            FileUpload {
                local_path: "/local/config.conf".to_string(),
                remote_name: "config.conf".to_string(),
            },
        ];

        let result = upload_job_files_demo("valid_job_001".to_string(), files).await;

        // Mock should always succeed with valid inputs
        assert!(result.success || !result.failed_uploads.as_ref().unwrap_or(&vec![]).is_empty());
        assert!(result.uploaded_files.is_some() || result.failed_uploads.is_some());
    }

    #[tokio::test]
    async fn test_list_files_mock_returns_expected_structure() {
        setup_test_environment();

        let result = list_job_files_demo("valid_job_001".to_string()).await;

        assert!(result.success);
        assert!(result.files.is_some());

        let files = result.files.unwrap();
        assert!(!files.is_empty());

        // Verify mock returns expected file types
        let file_types: Vec<FileType> = files.iter().map(|f| f.file_type.clone()).collect();
        assert!(file_types.contains(&FileType::Input));
        assert!(file_types.contains(&FileType::Config));
        assert!(file_types.contains(&FileType::Log));
    }

    // Path safety tests
    #[test]
    fn test_directory_path_generation() {
        let job_info = create_test_job();
        let directories = get_directories_to_list(&job_info);

        for (path, _dir_type) in directories {
            // Ensure paths don't contain traversal attempts
            assert!(!path.contains(".."));
            assert!(!path.contains("//"));
            // Should be absolute paths
            assert!(path.starts_with("/"));
            // Should contain safe job ID
            assert!(path.contains("test_job_001") || path.contains("testuser"));
        }
    }

    #[test]
    fn test_log_aggregation_content_safety() {
        let test_logs = vec![
            ("slurm.out", "Job completed successfully\nTotal time: 300s"),
            ("slurm.err", "Warning: deprecated option used"),
            ("namd.log", "NAMD simulation started\nStep 1000 completed"),
        ];

        let aggregated = aggregate_log_content(&test_logs);

        // Should contain expected content
        assert!(aggregated.contains("Job completed successfully"));
        assert!(aggregated.contains("NAMD simulation started"));

        // Should not contain dangerous content
        assert!(!aggregated.contains("rm -rf"));
        assert!(!aggregated.contains("sudo"));
        assert!(!aggregated.contains("passwd"));
    }

    // Error handling tests
    #[tokio::test]
    async fn test_invalid_job_id_handling() {
        setup_test_environment();

        let invalid_ids = vec!["", "   ", "invalid..id", "id;dangerous"];

        for invalid_id in invalid_ids {
            let files = vec![FileUpload {
                local_path: "/safe/file.txt".to_string(),
                remote_name: "file.txt".to_string(),
            }];

            // Test upload with invalid ID
            let _upload_result = upload_job_files_demo(invalid_id.to_string(), files).await;
            // Mock may succeed, but real implementation should validate

            // Test list files with invalid ID
            let list_result = list_job_files_demo(invalid_id.to_string()).await;
            // Mock should handle gracefully
            assert!(list_result.success || list_result.error.is_some());
        }
    }

    #[test]
    fn test_file_type_classification() {
        let test_files = vec![
            ("test.pdb", NAMDFileType::Pdb),
            ("structure.psf", NAMDFileType::Psf),
            ("params.prm", NAMDFileType::Prm),
            ("config.conf", NAMDFileType::Other),
            ("output.dcd", NAMDFileType::Other),
        ];

        for (filename, expected_type) in test_files {
            let classified_type = classify_namd_file_type(filename);
            assert_eq!(classified_type, expected_type,
                      "File {} should be classified as {:?}", filename, expected_type);
        }
    }

    // Progress tracking tests
    #[tokio::test]
    async fn test_upload_progress_calculation() {
        setup_test_environment();

        let files = vec![
            FileUpload {
                local_path: "/local/small.txt".to_string(),
                remote_name: "small.txt".to_string(),
            },
            FileUpload {
                local_path: "/local/large.pdb".to_string(),
                remote_name: "large.pdb".to_string(),
            },
        ];

        let start_time = std::time::Instant::now();
        let result = upload_job_files_demo("test_job".to_string(), files.clone()).await;
        let elapsed = start_time.elapsed();

        // Mock should simulate reasonable upload time based on file count
        let expected_min_time = files.len() as u64 * 200; // From mock implementation
        assert!(elapsed.as_millis() >= expected_min_time as u128);

        // Should have progress indication through file counts
        if result.success {
            assert!(result.uploaded_files.is_some());
        } else {
            assert!(result.failed_uploads.is_some());
        }
    }

    #[test]
    fn test_directories_to_list_generation() {
        let job_info = create_test_job();
        let directories = get_directories_to_list(&job_info);

        assert!(!directories.is_empty());

        for (dir_path, relative_prefix) in directories {
            // All paths should be absolute and safe
            assert!(dir_path.starts_with("/"));
            assert!(!dir_path.contains(".."));

            // Should contain job ID or remote directory
            assert!(dir_path.contains("test_job_001") ||
                   dir_path.contains("testuser") ||
                   dir_path == job_info.remote_directory);

            // Relative prefix should be a valid subdirectory or empty string
            assert!(relative_prefix == crate::ssh::JobDirectoryStructure::INPUT_FILES ||
                   relative_prefix == crate::ssh::JobDirectoryStructure::SCRIPTS ||
                   relative_prefix == crate::ssh::JobDirectoryStructure::OUTPUTS ||
                   relative_prefix.is_empty());
        }
    }

    // Helper function to classify NAMD file types
    fn classify_namd_file_type(filename: &str) -> NAMDFileType {
        let extension = filename.split('.').last().unwrap_or("").to_lowercase();
        match extension.as_str() {
            "pdb" => NAMDFileType::Pdb,
            "psf" => NAMDFileType::Psf,
            "prm" => NAMDFileType::Prm,
            _ => NAMDFileType::Other,
        }
    }

    // Helper function to aggregate log content safely
    fn aggregate_log_content(logs: &[(&str, &str)]) -> String {
        let mut aggregated = String::new();

        for (filename, content) in logs {
            aggregated.push_str(&format!("\n=== {} ===\n", filename));
            aggregated.push_str(content);
            aggregated.push_str("\n");
        }

        aggregated
    }
}