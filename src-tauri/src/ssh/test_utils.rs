//! Test utilities for SSH/SFTP testing with basic mocking
//!
//! This module provides mock helpers to test our business logic without
//! requiring actual SSH connections or file operations.

use std::collections::HashMap;
use super::errors::SSHError;
use super::sftp::{RemoteFileInfo, FileTransferProgress};
use super::commands::CommandResult;

/// Mock file system state for SFTP operations testing
#[derive(Debug, Clone, Default)]
pub struct MockFileSystem {
    pub files: HashMap<String, MockFile>,
    pub directories: HashMap<String, Vec<String>>,
}

/// Mock file representation for testing
#[derive(Debug, Clone)]
pub struct MockFile {
    pub size: u64,
    pub permissions: u32,
    pub is_directory: bool,
    pub modified_time: u64,
    pub content: Vec<u8>,
}

impl MockFileSystem {
    /// Create a new empty mock file system
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mock file to the file system
    pub fn add_file(&mut self, path: &str, size: u64, permissions: u32) -> &mut Self {
        let file = MockFile {
            size,
            permissions,
            is_directory: false,
            modified_time: 1234567890,
            content: vec![0u8; size as usize],
        };
        self.files.insert(path.to_string(), file);

        // Add to parent directory listing
        if let Some(parent) = std::path::Path::new(path).parent() {
            let parent_str = parent.to_str().unwrap_or("/");
            let filename = std::path::Path::new(path).file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);

            self.directories.entry(parent_str.to_string())
                .or_default()
                .push(filename.to_string());
        }

        self
    }

    /// Add a mock directory to the file system
    pub fn add_directory(&mut self, path: &str, permissions: u32) -> &mut Self {
        let dir = MockFile {
            size: 4096,
            permissions,
            is_directory: true,
            modified_time: 1234567890,
            content: vec![],
        };
        self.files.insert(path.to_string(), dir);
        self.directories.insert(path.to_string(), Vec::new());

        // Add to parent directory listing
        if let Some(parent) = std::path::Path::new(path).parent() {
            let parent_str = parent.to_str().unwrap_or("/");
            let dirname = std::path::Path::new(path).file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);

            self.directories.entry(parent_str.to_string())
                .or_default()
                .push(dirname.to_string());
        }

        self
    }

    /// Get file info for a path (simulates SFTP stat operation)
    pub fn get_file_info(&self, path: &str) -> Result<RemoteFileInfo, SSHError> {
        match self.files.get(path) {
            Some(file) => {
                let name = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path)
                    .to_string();

                Ok(RemoteFileInfo {
                    name,
                    path: path.to_string(),
                    size: file.size,
                    is_directory: file.is_directory,
                    permissions: file.permissions,
                    modified_time: Some(file.modified_time),
                })
            }
            None => Err(SSHError::FileTransferError(format!("File not found: {}", path)))
        }
    }

    /// List directory contents (simulates SFTP readdir operation)
    pub fn list_directory(&self, path: &str) -> Result<Vec<RemoteFileInfo>, SSHError> {
        match self.directories.get(path) {
            Some(entries) => {
                let mut results = Vec::new();
                for entry in entries {
                    let full_path = if path.ends_with('/') {
                        format!("{}{}", path, entry)
                    } else {
                        format!("{}/{}", path, entry)
                    };

                    if let Ok(info) = self.get_file_info(&full_path) {
                        results.push(info);
                    }
                }
                Ok(results)
            }
            None => Err(SSHError::FileTransferError(format!("Directory not found: {}", path)))
        }
    }
}

/// Mock command execution results for testing
pub struct MockCommandExecutor {
    pub predefined_responses: HashMap<String, CommandResult>,
    pub default_response: CommandResult,
}

impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl MockCommandExecutor {
    /// Create a new mock command executor
    pub fn new() -> Self {
        Self {
            predefined_responses: HashMap::new(),
            default_response: CommandResult {
                exit_code: 0,
                stdout: "".to_string(),
                stderr: "".to_string(),
                duration_ms: 10,
            timed_out: false,
            },
        }
    }

    /// Add a predefined response for a specific command
    pub fn add_response(&mut self, command: &str, result: CommandResult) -> &mut Self {
        self.predefined_responses.insert(command.to_string(), result);
        self
    }

    /// Get response for a command (simulates command execution)
    pub fn execute(&self, command: &str) -> CommandResult {
        self.predefined_responses.get(command)
            .cloned()
            .unwrap_or_else(|| self.default_response.clone())
    }
}

/// Helper to create common error scenarios for testing
pub struct MockErrorBuilder;

impl MockErrorBuilder {
    /// Create a network error
    pub fn network_error(message: &str) -> SSHError {
        SSHError::NetworkError(message.to_string())
    }

    /// Create an authentication error
    pub fn auth_error(message: &str) -> SSHError {
        SSHError::AuthenticationError(message.to_string())
    }

    /// Create a timeout error
    pub fn timeout_error(message: &str) -> SSHError {
        SSHError::TimeoutError(message.to_string())
    }

    /// Create a permission error
    pub fn permission_error(message: &str) -> SSHError {
        SSHError::PermissionError(message.to_string())
    }

    /// Create a file transfer error
    pub fn file_transfer_error(message: &str) -> SSHError {
        SSHError::FileTransferError(message.to_string())
    }
}

/// Helper to create progress objects for testing
pub struct MockProgressBuilder;

impl MockProgressBuilder {
    /// Create progress at the start of transfer
    pub fn start(total_bytes: u64) -> FileTransferProgress {
        FileTransferProgress {
            bytes_transferred: 0,
            total_bytes,
            percentage: 0.0,
            transfer_rate: 0.0,
        }
    }

    /// Create progress at a specific percentage
    pub fn at_percentage(total_bytes: u64, percentage: f32) -> FileTransferProgress {
        let bytes_transferred = (total_bytes as f32 * percentage / 100.0) as u64;
        FileTransferProgress {
            bytes_transferred,
            total_bytes,
            percentage,
            transfer_rate: 1024.0, // Mock transfer rate
        }
    }

    /// Create completed progress
    pub fn completed(total_bytes: u64) -> FileTransferProgress {
        FileTransferProgress {
            bytes_transferred: total_bytes,
            total_bytes,
            percentage: 100.0,
            transfer_rate: 1024.0,
        }
    }
}

/// Path utilities for testing various path formats
pub struct MockPathValidator;

impl MockPathValidator {
    /// Test if a path is valid for our application
    pub fn is_valid_remote_path(path: &str) -> bool {
        // Our business logic for path validation
        !path.is_empty()
            && !path.contains("..")
            && !path.contains('\0')
            && path.starts_with('/')
    }

    /// Normalize a path according to our rules
    pub fn normalize_path(path: &str) -> String {
        // Our path normalization logic
        if path.is_empty() {
            "/".to_string()
        } else if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        }
    }

    /// Extract filename from path
    pub fn extract_filename(path: &str) -> Option<String> {
        std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_filesystem_creation() {
        let mut fs = MockFileSystem::new();
        fs.add_file("/home/user/test.txt", 1024, 0o644)
          .add_directory("/home/user", 0o755);

        let file_info = fs.get_file_info("/home/user/test.txt").unwrap();
        assert_eq!(file_info.size, 1024);
        assert!(!file_info.is_directory);
        assert_eq!(file_info.permissions, 0o644);

        let dir_info = fs.get_file_info("/home/user").unwrap();
        assert!(dir_info.is_directory);
        assert_eq!(dir_info.permissions, 0o755);
    }

    #[test]
    fn test_mock_filesystem_directory_listing() {
        let mut fs = MockFileSystem::new();
        fs.add_directory("/home", 0o755)
          .add_file("/home/file1.txt", 100, 0o644)
          .add_file("/home/file2.txt", 200, 0o644);

        let listing = fs.list_directory("/home").unwrap();
        assert_eq!(listing.len(), 2);

        let filenames: Vec<&str> = listing.iter().map(|f| f.name.as_str()).collect();
        assert!(filenames.contains(&"file1.txt"));
        assert!(filenames.contains(&"file2.txt"));
    }

    #[test]
    fn test_mock_command_executor() {
        let mut executor = MockCommandExecutor::new();
        executor.add_response("ls -la", CommandResult {
            exit_code: 0,
            stdout: "total 8\ndrwxr-xr-x 2 user user 4096 Jan 1 12:00 .\n".to_string(),
            stderr: "".to_string(),
            duration_ms: 50,
            timed_out: false,
        });

        let result = executor.execute("ls -la");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("total 8"));
    }

    #[test]
    fn test_path_validation() {
        assert!(MockPathValidator::is_valid_remote_path("/home/user"));
        assert!(!MockPathValidator::is_valid_remote_path("../etc/passwd"));
        assert!(!MockPathValidator::is_valid_remote_path(""));
        assert!(!MockPathValidator::is_valid_remote_path("relative/path"));
    }

    #[test]
    fn test_path_normalization() {
        assert_eq!(MockPathValidator::normalize_path("home/user"), "/home/user");
        assert_eq!(MockPathValidator::normalize_path("/home/user"), "/home/user");
        assert_eq!(MockPathValidator::normalize_path(""), "/");
    }

    #[test]
    fn test_progress_builder() {
        let start = MockProgressBuilder::start(1000);
        assert_eq!(start.bytes_transferred, 0);
        assert_eq!(start.percentage, 0.0);

        let half = MockProgressBuilder::at_percentage(1000, 50.0);
        assert_eq!(half.bytes_transferred, 500);
        assert_eq!(half.percentage, 50.0);

        let complete = MockProgressBuilder::completed(1000);
        assert_eq!(complete.bytes_transferred, 1000);
        assert_eq!(complete.percentage, 100.0);
    }
}