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


    /// Create an error result from anyhow::Error
    pub fn from_anyhow_error(error: anyhow::Error) -> Self {
        Self::error(error.to_string())
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NAMDFileType {
    #[serde(rename = "pdb")]
    Pdb,
    #[serde(rename = "psf")]
    Psf,
    #[serde(rename = "prm")]
    Prm,
    #[serde(rename = "other")]
    Other,
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
    #[serde(rename = "namdConfig")]
    pub namd_config: NAMDConfig,
    #[serde(rename = "slurmConfig")]
    pub slurm_config: SlurmConfig,
    #[serde(rename = "inputFiles")]
    pub input_files: Vec<InputFile>,
    #[serde(rename = "remoteDirectory")]
    pub remote_directory: String,
}

impl JobInfo {
    /// Create a new JobInfo with default timestamps
    pub fn new(
        job_id: String,
        job_name: String,
        namd_config: NAMDConfig,
        slurm_config: SlurmConfig,
        input_files: Vec<InputFile>,
        remote_directory: String,
    ) -> Self {
        Self {
            job_id,
            job_name,
            status: JobStatus::Created,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
            submitted_at: None,
            completed_at: None,
            slurm_job_id: None,
            project_dir: None,
            scratch_dir: None,
            error_info: None,
            namd_config,
            slurm_config,
            input_files,
            remote_directory,
        }
    }
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

// Default configuration constants for database persistence
impl Default for NAMDConfig {
    fn default() -> Self {
        Self {
            steps: 10000,
            temperature: 300.0,
            timestep: 2.0,
            outputname: "output".to_string(),
            dcd_freq: Some(1000),
            restart_freq: Some(5000),
        }
    }
}

impl Default for SlurmConfig {
    fn default() -> Self {
        Self {
            cores: 4,
            memory: "4GB".to_string(),
            walltime: "01:00:00".to_string(),
            partition: Some("compute".to_string()),
            qos: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFile {
    pub name: String,
    #[serde(rename = "localPath")]
    pub local_path: String,
    #[serde(rename = "remoteName")]
    pub remote_name: Option<String>,
    #[serde(rename = "fileType")]
    pub file_type: Option<NAMDFileType>,
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