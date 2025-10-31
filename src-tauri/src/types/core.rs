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
    #[serde(rename = "exb")]
    Exb,
    #[serde(rename = "other")]
    Other,
}

impl NAMDFileType {
    /// Detect file type from filename (source of truth for type detection)
    pub fn from_filename(filename: &str) -> Self {
        let lower = filename.to_lowercase();
        let ext = lower.split('.').last().unwrap_or("");

        match ext {
            "pdb" => Self::Pdb,
            "psf" => Self::Psf,
            "prm" => Self::Prm,
            "exb" => Self::Exb,
            _ if lower.ends_with(".enm.extra") => Self::Exb,
            _ if lower.contains("extrabonds") => Self::Exb,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFile {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub job_id: String,
    pub job_name: String,
    pub status: JobStatus,
    pub slurm_job_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub submitted_at: Option<String>,
    pub completed_at: Option<String>,
    pub project_dir: Option<String>,
    pub scratch_dir: Option<String>,
    pub error_info: Option<String>,
    pub slurm_stdout: Option<String>,
    pub slurm_stderr: Option<String>,
    pub namd_config: NAMDConfig,
    pub slurm_config: SlurmConfig,
    pub input_files: Vec<InputFile>,
    pub output_files: Option<Vec<OutputFile>>,
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
            slurm_stdout: None,
            slurm_stderr: None,
            namd_config,
            slurm_config,
            input_files,
            output_files: None,
            remote_directory,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NAMDConfig {
    // Basic simulation parameters
    pub outputname: String,
    pub temperature: f64,
    pub timestep: f64,

    // Execution mode and steps
    pub execution_mode: ExecutionMode,  // minimize or run
    pub steps: u32,  // minimize steps or run steps depending on mode

    // Periodic boundary conditions (required for PME)
    pub cell_basis_vector1: Option<CellBasisVector>,
    pub cell_basis_vector2: Option<CellBasisVector>,
    pub cell_basis_vector3: Option<CellBasisVector>,

    // Electrostatics and ensemble
    pub pme_enabled: bool,
    pub npt_enabled: bool,

    // Langevin dynamics parameters
    pub langevin_damping: f64,

    // Output frequencies (all required, no Option)
    pub xst_freq: u32,
    pub output_energies_freq: u32,
    pub dcd_freq: u32,
    pub restart_freq: u32,
    pub output_pressure_freq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    #[serde(rename = "minimize")]
    Minimize,
    #[serde(rename = "run")]
    Run,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellBasisVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
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
            outputname: "output".to_string(),
            temperature: 300.0,
            timestep: 2.0,
            execution_mode: ExecutionMode::Run,
            steps: 10000,
            cell_basis_vector1: None,
            cell_basis_vector2: None,
            cell_basis_vector3: None,
            pme_enabled: false,
            npt_enabled: false,
            langevin_damping: 5.0,
            xst_freq: 1200,
            output_energies_freq: 1200,
            dcd_freq: 1200,
            restart_freq: 1200,
            output_pressure_freq: 1200,
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
    pub local_path: String,
    pub remote_name: Option<String>,
    pub file_type: Option<NAMDFileType>,
    pub size: Option<u64>,
    pub uploaded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub host: String,
    pub username: String,
    pub connected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUpload {
    pub local_path: String,
    pub remote_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFile {
    pub name: String,           // Display name (just filename)
    pub path: String,           // Full relative path from job root (e.g., "outputs/sim.dcd")
    pub size: u64,
    pub modified_at: String,
    pub file_type: FileType,
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Connection status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatusResponse {
    pub state: ConnectionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_info: Option<SessionInfo>,
}

/// File information for SFTP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified_at: String,
    pub is_directory: bool,
}

/// File upload progress information for real-time progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadProgress {
    pub file_name: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub transfer_rate_mbps: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namd_file_type_detection() {
        // Standard extensions
        assert_eq!(NAMDFileType::from_filename("structure.pdb"), NAMDFileType::Pdb);
        assert_eq!(NAMDFileType::from_filename("structure.psf"), NAMDFileType::Psf);
        assert_eq!(NAMDFileType::from_filename("parameters.prm"), NAMDFileType::Prm);
        assert_eq!(NAMDFileType::from_filename("restraints.exb"), NAMDFileType::Exb);

        // Case insensitive
        assert_eq!(NAMDFileType::from_filename("STRUCTURE.PDB"), NAMDFileType::Pdb);
        assert_eq!(NAMDFileType::from_filename("Structure.Psf"), NAMDFileType::Psf);

        // Special extrabonds patterns
        assert_eq!(
            NAMDFileType::from_filename("hextube_MGHH_WI_k0.5.enm.extra"),
            NAMDFileType::Exb
        );
        assert_eq!(
            NAMDFileType::from_filename("mghh_extrabonds"),
            NAMDFileType::Exb
        );
        assert_eq!(
            NAMDFileType::from_filename("my_extrabonds_file.txt"),
            NAMDFileType::Exb
        );

        // Unknown extensions
        assert_eq!(NAMDFileType::from_filename("data.txt"), NAMDFileType::Other);
        assert_eq!(NAMDFileType::from_filename("config.conf"), NAMDFileType::Other);
        assert_eq!(NAMDFileType::from_filename("noextension"), NAMDFileType::Other);
    }
}
