use serde::{Deserialize, Serialize};
use super::core::*;
use crate::security::SecurePassword;

// Connection management command parameters and results
#[derive(Debug, Deserialize)]
pub struct ConnectParams {
    pub host: String,
    pub username: String,
    pub password: SecurePassword,
}

#[derive(Debug, Serialize)]
pub struct ConnectResult {
    pub success: bool,
    #[serde(rename = "sessionInfo")]
    pub session_info: Option<SessionInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DisconnectResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConnectionStatusResult {
    pub state: ConnectionState,
    #[serde(rename = "sessionInfo")]
    pub session_info: Option<SessionInfo>,
}

// Job management command parameters and results
#[derive(Debug, Deserialize)]
pub struct CreateJobParams {
    #[serde(rename = "jobName")]
    pub job_name: String,
    #[serde(rename = "namdConfig")]
    pub namd_config: NAMDConfig,
    #[serde(rename = "slurmConfig")]
    pub slurm_config: SlurmConfig,
    #[serde(rename = "inputFiles")]
    pub input_files: Vec<InputFile>,
}

#[derive(Debug, Serialize)]
pub struct CreateJobResult {
    pub success: bool,
    #[serde(rename = "jobId")]
    pub job_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitJobResult {
    pub success: bool,
    #[serde(rename = "slurmJobId")]
    pub slurm_job_id: Option<String>,
    #[serde(rename = "submittedAt")]
    pub submitted_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResult {
    pub success: bool,
    #[serde(rename = "jobInfo")]
    pub job_info: Option<JobInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetAllJobsResult {
    pub success: bool,
    pub jobs: Option<Vec<JobInfo>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncJobsResult {
    pub success: bool,
    #[serde(rename = "jobsUpdated")]
    pub jobs_updated: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteJobResult {
    pub success: bool,
    pub error: Option<String>,
}

// File management command parameters and results
#[derive(Debug, Serialize)]
pub struct UploadResult {
    pub success: bool,
    #[serde(rename = "uploadedFiles")]
    pub uploaded_files: Option<Vec<String>>,
    #[serde(rename = "failedUploads")]
    pub failed_uploads: Option<Vec<FailedUpload>>,
}

#[derive(Debug, Serialize)]
pub struct FailedUpload {
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct DownloadResult {
    pub success: bool,
    pub content: Option<String>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    #[serde(rename = "fileSize")]
    pub file_size: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListFilesResult {
    pub success: bool,
    pub files: Option<Vec<RemoteFile>>,
    pub error: Option<String>,
}