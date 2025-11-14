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

/// Unified log message structure for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLogMessage {
    pub level: String,
    pub category: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub show_toast: bool,
    pub timestamp: String,
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
        let ext = lower.split('.').next_back().unwrap_or("");

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

    // Template-based configuration
    pub template_id: String,
    pub template_values: std::collections::HashMap<String, serde_json::Value>,

    pub slurm_config: SlurmConfig,
    pub input_files: Option<Vec<String>>,
    pub output_files: Option<Vec<OutputFile>>,
    pub remote_directory: String,
}

// JobInfo has no custom constructor - construct directly using struct literal syntax
// or let serde handle deserialization from JSON/database
// For creating new jobs with business logic, use `crate::automations::job_creation::create_job_info()`

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmConfig {
    pub cores: u32,
    pub memory: String,
    pub walltime: String,
    pub partition: Option<String>,
    pub qos: Option<String>,
}

impl SlurmConfig {
    /// Parse memory string to GB (e.g., "16GB", "32", "2048MB")
    pub fn parse_memory_gb(&self) -> anyhow::Result<f64> {
        let clean = self.memory.trim().to_lowercase();

        if clean.is_empty() {
            return Err(anyhow::anyhow!("Memory value is required"));
        }

        // Try to extract number and unit
        let re = regex::Regex::new(r"^(\d+(?:\.\d+)?)\s*(gb|g|mb|m)?$").unwrap();

        if let Some(captures) = re.captures(&clean) {
            let value: f64 = captures.get(1)
                .ok_or_else(|| anyhow::anyhow!("Invalid memory format"))?
                .as_str()
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid memory number"))?;

            let unit = captures.get(2).map(|m| m.as_str()).unwrap_or("gb");

            match unit {
                "gb" | "g" | "" => Ok(value),
                "mb" | "m" => Ok(value / 1024.0),
                _ => Err(anyhow::anyhow!("Unsupported memory unit: {}", unit)),
            }
        } else {
            Err(anyhow::anyhow!("Invalid memory format: {}. Use format like '16GB', '32', or '2048MB'", self.memory))
        }
    }

    /// Parse walltime string to hours (e.g., "24:00:00", "04:30:00")
    pub fn parse_walltime_hours(&self) -> anyhow::Result<f64> {
        if self.walltime.trim().is_empty() {
            return Err(anyhow::anyhow!("Walltime is required"));
        }

        let parts: Vec<&str> = self.walltime.split(':').collect();

        if parts.len() != 3 {
            return Err(anyhow::anyhow!("Walltime must be in HH:MM:SS format (e.g., '24:00:00')"));
        }

        let hours: u32 = parts[0].parse()
            .map_err(|_| anyhow::anyhow!("Invalid hours in walltime"))?;
        let minutes: u32 = parts[1].parse()
            .map_err(|_| anyhow::anyhow!("Invalid minutes in walltime"))?;
        let seconds: u32 = parts[2].parse()
            .map_err(|_| anyhow::anyhow!("Invalid seconds in walltime"))?;

        if minutes >= 60 {
            return Err(anyhow::anyhow!("Minutes must be less than 60"));
        }
        if seconds >= 60 {
            return Err(anyhow::anyhow!("Seconds must be less than 60"));
        }

        Ok(hours as f64 + (minutes as f64 / 60.0) + (seconds as f64 / 3600.0))
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

    #[test]
    fn test_parse_memory_gb_standard_formats() {
        // GB formats
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 16.0);

        let config = SlurmConfig {
            cores: 1,
            memory: "16G".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 16.0);

        // Plain number (assumes GB)
        let config = SlurmConfig {
            cores: 1,
            memory: "32".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 32.0);

        // MB formats
        let config = SlurmConfig {
            cores: 1,
            memory: "2048MB".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 2.0);

        let config = SlurmConfig {
            cores: 1,
            memory: "512M".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 0.5);
    }

    #[test]
    fn test_parse_memory_gb_decimal_values() {
        let config = SlurmConfig {
            cores: 1,
            memory: "1.5GB".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 1.5);

        let config = SlurmConfig {
            cores: 1,
            memory: "0.5G".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 0.5);
    }

    #[test]
    fn test_parse_memory_gb_whitespace() {
        let config = SlurmConfig {
            cores: 1,
            memory: "  16GB  ".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 16.0);

        let config = SlurmConfig {
            cores: 1,
            memory: "16 GB".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 16.0);
    }

    #[test]
    fn test_parse_memory_gb_case_insensitive() {
        let config = SlurmConfig {
            cores: 1,
            memory: "16gb".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 16.0);

        let config = SlurmConfig {
            cores: 1,
            memory: "2048mb".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_memory_gb().unwrap(), 2.0);
    }

    #[test]
    fn test_parse_memory_gb_invalid_formats() {
        // Empty string
        let config = SlurmConfig {
            cores: 1,
            memory: "".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_memory_gb().is_err());

        // Invalid format
        let config = SlurmConfig {
            cores: 1,
            memory: "invalid".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_memory_gb().is_err());

        // Unsupported unit
        let config = SlurmConfig {
            cores: 1,
            memory: "16TB".to_string(),
            walltime: "01:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_memory_gb().is_err());
    }

    #[test]
    fn test_parse_walltime_hours_standard_formats() {
        // Whole hours
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "24:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_walltime_hours().unwrap(), 24.0);

        // Hours with minutes
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "04:30:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_walltime_hours().unwrap(), 4.5);

        // Hours with minutes and seconds
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "01:30:30".to_string(),
            partition: None,
            qos: None,
        };
        // 1 hour + 30 minutes (0.5) + 30 seconds (0.00833...)
        let result = config.parse_walltime_hours().unwrap();
        assert!((result - 1.508333).abs() < 0.0001);
    }

    #[test]
    fn test_parse_walltime_hours_edge_cases() {
        // Zero time
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "00:00:00".to_string(),
            partition: None,
            qos: None,
        };
        assert_eq!(config.parse_walltime_hours().unwrap(), 0.0);

        // Maximum valid values
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "99:59:59".to_string(),
            partition: None,
            qos: None,
        };
        let result = config.parse_walltime_hours().unwrap();
        assert!(result > 99.9 && result < 100.0);
    }

    #[test]
    fn test_parse_walltime_hours_invalid_formats() {
        // Empty string
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_walltime_hours().is_err());

        // Wrong format (no colons)
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "24".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_walltime_hours().is_err());

        // Wrong format (only one colon)
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "24:00".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_walltime_hours().is_err());

        // Invalid minutes (>= 60)
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "01:60:00".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_walltime_hours().is_err());

        // Invalid seconds (>= 60)
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "01:00:60".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_walltime_hours().is_err());

        // Non-numeric values
        let config = SlurmConfig {
            cores: 1,
            memory: "16GB".to_string(),
            walltime: "aa:bb:cc".to_string(),
            partition: None,
            qos: None,
        };
        assert!(config.parse_walltime_hours().is_err());
    }
}
