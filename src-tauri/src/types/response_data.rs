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
