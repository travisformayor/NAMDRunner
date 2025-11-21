use serde::{Deserialize, Serialize};
use super::core::*;

/// Response DTOs for commands that return multiple fields
/// These are used with ApiResult<T> for type-safe IPC responses
///
/// Job submission response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSubmissionData {
    pub slurm_job_id: String,
    pub submitted_at: String,
    pub job_id: String,
}

/// Download operation response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub saved_to: String,
    pub file_size: u64,
}

/// Database information response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    pub job_count: usize,
}

/// Database operation response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOperationData {
    pub path: String,
    pub message: String,
}

/// Connection status response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub state: ConnectionState,
    pub session_info: Option<SessionInfo>,
}

/// Job discovery response data
/// Used when scanning cluster for existing jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    pub imported_jobs: Vec<JobSummary>,
    pub failed_imports: Vec<FailedImport>,
}

/// Summary of an imported job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub job_id: String,
    pub job_name: String,
    pub status: JobStatus,
}

/// Information about a failed import during discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedImport {
    pub directory: String,
    pub reason: String,
}

/// App initialization response data
/// Contains all data needed to initialize the frontend on startup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInitializationData {
    pub capabilities: crate::cluster::ClusterCapabilities,
    pub templates: Vec<crate::templates::TemplateSummary>,
    pub jobs: Vec<JobInfo>,
}
