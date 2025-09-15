use serde::{Deserialize, Serialize};

/// Consistent API result type for all Tauri commands
/// Provides type safety and predictable error handling across the IPC boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResult<T> {
    /// Create a successful result with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error result with message
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }

    /// Create an error result from any error type
    pub fn from_error(error: impl std::fmt::Display) -> Self {
        Self::error(error.to_string())
    }

    /// Create an error result from SSH error with structured information
    pub fn from_ssh_error(error: crate::ssh::SSHError) -> Self {
        let conn_error = crate::ssh::map_ssh_error(&error);
        // Create a detailed error message with recovery suggestions
        let detailed_message = format!(
            "{}: {} [Code: {}] - Suggestions: {}",
            conn_error.message,
            conn_error.details.unwrap_or_default(),
            conn_error.code,
            conn_error.suggestions.join("; ")
        );
        Self::error(detailed_message)
    }

    /// Create an error result from anyhow::Error, using SSH error handling if possible
    pub fn from_anyhow_error(error: anyhow::Error) -> Self {
        // Check if this is an SSH error for enhanced error handling
        if let Some(ssh_err) = error.downcast_ref::<crate::ssh::SSHError>() {
            Self::from_ssh_error(ssh_err.clone())
        } else {
            Self::from_error(error)
        }
    }

    /// Map to Result type for easier interop
    pub fn into_result(self) -> Result<T, String> {
        if self.success {
            if let Some(data) = self.data {
                Ok(data)
            } else {
                Err("Success result missing data".to_string())
            }
        } else {
            Err(self.error.unwrap_or("Unknown error".to_string()))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    #[serde(rename = "Disconnected")]
    Disconnected,
    #[serde(rename = "Connecting")]
    Connecting,
    #[serde(rename = "Connected")]
    Connected,
    #[serde(rename = "Expired")]
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "RUNNING")]
    Running,
    #[serde(rename = "COMPLETED")]
    Completed,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "CANCELLED")]
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "output")]
    Output,
    #[serde(rename = "config")]
    Config,
    #[serde(rename = "log")]
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    #[serde(rename = "jobId")]
    pub job_id: String,
    #[serde(rename = "jobName")]
    pub job_name: String,
    pub status: JobStatus,
    #[serde(rename = "slurmJobId")]
    pub slurm_job_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "submittedAt")]
    pub submitted_at: Option<String>,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
    #[serde(rename = "projectDir")]
    pub project_dir: Option<String>,
    #[serde(rename = "scratchDir")]
    pub scratch_dir: Option<String>,
    #[serde(rename = "errorInfo")]
    pub error_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NAMDConfig {
    pub steps: u32,
    pub temperature: f64,
    pub timestep: f64,
    pub outputname: String,
    #[serde(rename = "dcdFreq")]
    pub dcd_freq: Option<u32>,
    #[serde(rename = "restartFreq")]
    pub restart_freq: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmConfig {
    pub cores: u32,
    pub memory: String,
    pub walltime: String,
    pub partition: Option<String>,
    pub qos: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFile {
    pub name: String,
    #[serde(rename = "localPath")]
    pub local_path: String,
    #[serde(rename = "remoteName")]
    pub remote_name: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>, // 'pdb' | 'psf' | 'prm' | 'other'
    #[serde(rename = "fileType")]
    pub file_type: Option<String>, // 'pdb' | 'psf' | 'prm' | 'other'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub host: String,
    pub username: String,
    #[serde(rename = "connectedAt")]
    pub connected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUpload {
    #[serde(rename = "localPath")]
    pub local_path: String,
    #[serde(rename = "remoteName")]
    pub remote_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFile {
    pub name: String,
    pub size: u64,
    #[serde(rename = "modifiedAt")]
    pub modified_at: String,
    #[serde(rename = "fileType")]
    pub file_type: FileType,
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    #[serde(rename = "exitCode")]
    pub exit_code: i32,
}

/// Connection status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatusResponse {
    pub state: ConnectionState,
    #[serde(rename = "sessionInfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_info: Option<SessionInfo>,
}

/// File information for SFTP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    #[serde(rename = "modifiedAt")]
    pub modified_at: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
}