/// Typed error system for automation workflows
/// Provides structured error handling with user-friendly messages and recovery suggestions
use anyhow::Result;
use std::fmt;

/// Automation error types with structured information
#[derive(Debug)]
pub enum AutomationError {
    /// File operation failed (upload, download, copy)
    FileOperation {
        operation: String,
        path: String,
        source: anyhow::Error,
        recovery_suggestion: String,
    },
    /// SSH/SFTP connection or command failed
    Connection {
        operation: String,
        details: String,
        recovery_suggestion: String,
    },
    /// Database operation failed
    Database {
        operation: String,
        source: anyhow::Error,
        recovery_suggestion: String,
    },
    /// Job validation failed (configuration, files, etc.)
    Validation {
        field: String,
        value: String,
        reason: String,
        recovery_suggestion: String,
    },
    /// SLURM operation failed (submission, status check, etc.)
    Slurm {
        operation: String,
        job_id: Option<String>,
        details: String,
        recovery_suggestion: String,
    },
    /// Progress reporting or callback failed
    Progress {
        step: String,
        source: anyhow::Error,
    },
}

impl AutomationError {
    /// Create a file operation error with context
    pub fn file_operation(operation: &str, path: &str, source: anyhow::Error) -> Self {
        let recovery_suggestion = match operation {
            "upload" => "Check file permissions and disk space on remote server".to_string(),
            "download" => "Verify file exists and you have read permissions".to_string(),
            "copy" => "Ensure source file exists and destination is writable".to_string(),
            "delete" => "Check file permissions and that file is not in use".to_string(),
            _ => "Check file permissions and try again".to_string(),
        };

        AutomationError::FileOperation {
            operation: operation.to_string(),
            path: path.to_string(),
            source,
            recovery_suggestion,
        }
    }

    /// Create a connection error with context
    pub fn connection(operation: &str, details: &str) -> Self {
        let recovery_suggestion = "Check network connection and SSH credentials, then retry".to_string();

        AutomationError::Connection {
            operation: operation.to_string(),
            details: details.to_string(),
            recovery_suggestion,
        }
    }

    /// Create a database error with context
    pub fn database(operation: &str, source: anyhow::Error) -> Self {
        let recovery_suggestion = match operation {
            "save" => "Database may be locked. Close other NAMDRunner instances and retry".to_string(),
            "load" => "Database may be corrupted. Check application data directory".to_string(),
            "delete" => "Job may be in use. Stop any running operations and retry".to_string(),
            _ => "Check database file permissions and disk space".to_string(),
        };

        AutomationError::Database {
            operation: operation.to_string(),
            source,
            recovery_suggestion,
        }
    }

    /// Create a validation error with specific field information
    pub fn validation(field: &str, value: &str, reason: &str) -> Self {
        let recovery_suggestion = match field {
            "job_id" => "Use only letters, numbers, underscores, and hyphens".to_string(),
            "filename" => "Remove special characters and path separators from filename".to_string(),
            "file_size" => "Use a smaller file (maximum 1GB)".to_string(),
            "cores" => "Choose a value between 1 and available cluster cores".to_string(),
            "memory" => "Specify memory in format like '4GB' or '2048MB'".to_string(),
            "walltime" => "Use format HH:MM:SS (e.g., '02:30:00')".to_string(),
            _ => "Check the input format and try again".to_string(),
        };

        AutomationError::Validation {
            field: field.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
            recovery_suggestion,
        }
    }

    /// Create a SLURM operation error
    pub fn slurm(operation: &str, job_id: Option<&str>, details: &str) -> Self {
        let recovery_suggestion = match operation {
            "submit" => "Check job script syntax and cluster resource availability".to_string(),
            "status" => "Job may have completed or been cancelled. Check SLURM logs".to_string(),
            "cancel" => "Job may have already finished. Check job status first".to_string(),
            _ => "Check SLURM cluster status and retry".to_string(),
        };

        AutomationError::Slurm {
            operation: operation.to_string(),
            job_id: job_id.map(|s| s.to_string()),
            details: details.to_string(),
            recovery_suggestion,
        }
    }

    /// Create a progress reporting error
    pub fn progress(step: &str, source: anyhow::Error) -> Self {
        AutomationError::Progress {
            step: step.to_string(),
            source,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AutomationError::FileOperation { operation, path, recovery_suggestion, .. } => {
                format!("File {} failed for '{}'. {}", operation, path, recovery_suggestion)
            }
            AutomationError::Connection { operation, details, recovery_suggestion } => {
                format!("Connection failed during {}: {}. {}", operation, details, recovery_suggestion)
            }
            AutomationError::Database { operation, recovery_suggestion, .. } => {
                format!("Database {} failed. {}", operation, recovery_suggestion)
            }
            AutomationError::Validation { field, value, reason, recovery_suggestion } => {
                format!("Invalid {}: '{}' - {}. {}", field, value, reason, recovery_suggestion)
            }
            AutomationError::Slurm { operation, job_id, details, recovery_suggestion } => {
                let job_part = job_id.as_ref().map(|id| format!(" for job {}", id)).unwrap_or_default();
                format!("SLURM {} failed{}: {}. {}", operation, job_part, details, recovery_suggestion)
            }
            AutomationError::Progress { step, .. } => {
                format!("Progress reporting failed at step: {}", step)
            }
        }
    }

    /// Get technical details for logging
    pub fn technical_details(&self) -> String {
        match self {
            AutomationError::FileOperation { source, .. } => format!("File error: {}", source),
            AutomationError::Connection { details, .. } => format!("Connection error: {}", details),
            AutomationError::Database { source, .. } => format!("Database error: {}", source),
            AutomationError::Validation { reason, .. } => format!("Validation error: {}", reason),
            AutomationError::Slurm { details, .. } => format!("SLURM error: {}", details),
            AutomationError::Progress { source, .. } => format!("Progress error: {}", source),
        }
    }
}

impl fmt::Display for AutomationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for AutomationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AutomationError::FileOperation { source, .. } => Some(source.as_ref()),
            AutomationError::Database { source, .. } => Some(source.as_ref()),
            AutomationError::Progress { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}


/// Result type for automation operations
pub type AutomationResult<T> = Result<T, AutomationError>;