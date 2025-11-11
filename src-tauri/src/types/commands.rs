use serde::{Deserialize, Serialize};
use super::core::*;
use crate::security::SecurePassword;
use std::collections::HashMap;
use serde_json::Value;

// Job validation request parameters
#[derive(Debug, Deserialize)]
pub struct ValidateJobConfigParams {
    pub job_name: String,
    pub template_id: String,
    pub template_values: HashMap<String, Value>,
    pub cores: u32,
    pub memory: String,
    pub walltime: String,
    pub partition: Option<String>,
    pub qos: Option<String>,
}

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
    pub template_id: String,
    pub template_values: std::collections::HashMap<String, serde_json::Value>,
    pub slurm_config: SlurmConfig,
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
    pub jobs: Vec<JobInfo>,        // Complete job list after sync
    pub jobs_updated: u32,          // Number of jobs updated during sync
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteJobResult {
    pub success: bool,
    pub error: Option<String>,
}

// DELETED: SyncJobStatusResult - unused result type
// DELETED: SyncAllJobsResult - unused result type

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
    pub saved_to: Option<String>,  // Local path where file was saved
    pub file_size: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListFilesResult {
    pub success: bool,
    pub files: Option<Vec<RemoteFile>>,
    pub error: Option<String>,
}

// DELETED: AutoCompleteJobsResult - unused result type

// Job discovery result types
// DELETED: DiscoverJobsResult - defined locally in jobs.rs where it's used
// DELETED: Cluster result types - commands use ApiResult and ValidationResult instead

// Template management result types
#[derive(Debug, Serialize)]
pub struct ListTemplatesResult {
    pub success: bool,
    pub templates: Option<Vec<crate::templates::TemplateSummary>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetTemplateResult {
    pub success: bool,
    pub template: Option<crate::templates::Template>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateTemplateResult {
    pub success: bool,
    pub template_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTemplateResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeleteTemplateResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidateTemplateValuesResult {
    pub valid: bool,
    pub errors: Vec<String>,
}


// Preview result types
#[derive(Debug, Serialize)]
pub struct PreviewResult {
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
}
