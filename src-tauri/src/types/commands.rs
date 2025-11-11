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

// Job management command parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobParams {
    pub job_name: String,
    pub template_id: String,
    pub template_values: std::collections::HashMap<String, serde_json::Value>,
    pub slurm_config: SlurmConfig,
}

// Complex batch operation results (NOT migrated to ApiResult<T> - domain-specific)
#[derive(Debug, Serialize)]
pub struct SyncJobsResult {
    pub success: bool,
    pub jobs: Vec<JobInfo>,        // Complete job list after sync
    pub jobs_updated: u32,          // Number of jobs updated during sync
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

// DELETED: DownloadResult, ListFilesResult - now use ApiResult<T>
// DELETED: AutoCompleteJobsResult - unused result type

// Job discovery result types
// DELETED: DiscoverJobsResult - defined locally in jobs.rs where it's used
// DELETED: Cluster result types - commands use ApiResult and ValidationResult instead

// DELETED: Template result types - now use ApiResult<T>
// ListTemplatesResult, GetTemplateResult, CreateTemplateResult, UpdateTemplateResult, DeleteTemplateResult


// DELETED: PreviewResult - now use ApiResult<String>
