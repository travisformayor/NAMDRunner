use serde::{Deserialize, Serialize};
use super::core::*;
use crate::security::SecurePassword;

// Connection management command parameters and results
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectParams {
    pub host: String,
    pub username: String,
    pub password: SecurePassword,
}

#[derive(Debug, Serialize)]
pub struct ConnectResult {
    pub success: bool,
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
    pub session_info: Option<SessionInfo>,
}

// Job management command parameters and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobParams {
    pub job_name: String,
    pub namd_config: NAMDConfig,
    pub slurm_config: SlurmConfig,
    pub input_files: Vec<InputFile>,
}

#[derive(Debug, Serialize)]
pub struct CreateJobResult {
    pub success: bool,
    pub job_id: Option<String>,
    pub job: Option<JobInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitJobResult {
    pub success: bool,
    pub slurm_job_id: Option<String>,
    pub submitted_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResult {
    pub success: bool,
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
    pub jobs_updated: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteJobResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncJobStatusResult {
    pub success: bool,
    pub job_info: Option<JobInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncAllJobsResult {
    pub success: bool,
    pub jobs_updated: u32,
    pub errors: Vec<String>,
}

// File management command parameters and results
#[derive(Debug, Serialize)]
pub struct UploadResult {
    pub success: bool,
    pub uploaded_files: Option<Vec<String>>,
    pub failed_uploads: Option<Vec<FailedUpload>>,
}

#[derive(Debug, Serialize)]
pub struct FailedUpload {
    pub file_name: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct DownloadResult {
    pub success: bool,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListFilesResult {
    pub success: bool,
    pub files: Option<Vec<RemoteFile>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AutoCompleteJobsResult {
    pub success: bool,
    pub processed_jobs: Option<Vec<String>>,
    pub error: Option<String>,
}

// Job discovery result types
#[derive(Debug, Serialize)]
pub struct DiscoverJobsResult {
    pub success: bool,
    pub jobs_found: u32,
    pub jobs_imported: u32,
    pub error: Option<String>,
}

// Cluster configuration types
#[derive(Debug, Serialize)]
pub struct GetClusterCapabilitiesResult {
    pub success: bool,
    pub data: Option<crate::cluster::ClusterCapabilities>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidateResourceAllocationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}
