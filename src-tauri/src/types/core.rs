use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobStatus {
    Created,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Input,
    Output,
    Config,
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
    #[serde(rename = "fileType")]
    pub file_type: String,
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