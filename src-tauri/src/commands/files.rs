use crate::types::*;
use chrono::Utc;

#[tauri::command]
pub async fn upload_job_files(_job_id: String, files: Vec<FileUpload>) -> UploadResult {
    // Mock implementation - simulate file upload
    // In Phase 2, this will use SFTP for real file transfers
    
    // Simulate upload time based on file count
    let upload_time = files.len() as u64 * 200;
    tokio::time::sleep(tokio::time::Duration::from_millis(upload_time)).await;
    
    // Simulate some successful and failed uploads
    let mut uploaded_files = Vec::new();
    let mut failed_uploads = Vec::new();
    
    for file in files {
        if rand::random::<f32>() > 0.1 {  // 90% success rate
            uploaded_files.push(file.remote_name);
        } else {
            failed_uploads.push(FailedUpload {
                file_name: file.remote_name,
                error: "Simulated upload failure".to_string(),
            });
        }
    }
    
    UploadResult {
        success: failed_uploads.is_empty(),
        uploaded_files: Some(uploaded_files),
        failed_uploads: Some(failed_uploads),
    }
}

#[tauri::command]
pub async fn download_job_output(job_id: String, file_name: String) -> DownloadResult {
    // Mock implementation - simulate file download
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Generate mock file content
    let mock_content = format!(
        "Mock content for {} from job {}\nGenerated at: {}\n\nSample output data...",
        file_name,
        job_id,
        Utc::now().to_rfc3339()
    );
    
    DownloadResult {
        success: true,
        content: Some(mock_content.clone()),
        file_path: Some(format!("/tmp/{}", file_name)),
        file_size: Some(mock_content.len() as u64),
        error: None,
    }
}

#[tauri::command]
pub async fn list_job_files(job_id: String) -> ListFilesResult {
    // Mock implementation - return sample file list
    
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    
    // Generate mock file list
    let files = vec![
        RemoteFile {
            name: "config.namd".to_string(),
            size: 2048,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Config,
        },
        RemoteFile {
            name: "job.sbatch".to_string(),
            size: 1024,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Config,
        },
        RemoteFile {
            name: "structure.pdb".to_string(),
            size: 102400,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Input,
        },
        RemoteFile {
            name: "structure.psf".to_string(),
            size: 51200,
            modified_at: Utc::now().to_rfc3339(),
            file_type: FileType::Input,
        },
        RemoteFile {
            name: format!("{}_output.log", job_id),
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