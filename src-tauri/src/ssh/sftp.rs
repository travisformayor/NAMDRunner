use ssh2::{Session, Sftp};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
use anyhow::Result;
use super::errors::SSHError;

/// Progress callback for file transfers
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send>;

/// File transfer progress information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileTransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub transfer_rate: f64, // bytes per second
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

/// File information from SFTP
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SftpFileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub permissions: u32,
    pub modified_time: Option<u64>,
}

/// SFTP operations handler
pub struct SFTPOperations<'a> {
    session: &'a Session,
    buffer_size: usize,
}

impl<'a> SFTPOperations<'a> {
    /// Create new SFTP operations handler
    pub fn new(session: &'a Session) -> Self {
        Self {
            session,
            buffer_size: 32768, // 32KB buffer
        }
    }

    /// Create new SFTP operations handler with custom buffer size
    pub fn with_buffer_size(session: &'a Session, buffer_size: usize) -> Self {
        Self {
            session,
            buffer_size,
        }
    }

    /// Get SFTP channel
    fn get_sftp(&self) -> Result<Sftp> {
        self.session.sftp()
            .map_err(|e| SSHError::SessionError(format!("Failed to create SFTP session: {}", e)).into())
    }

    /// Upload bytes directly to remote file
    pub fn upload_bytes(
        &self,
        remote_path: &str,
        content: &[u8],
    ) -> Result<FileTransferProgress> {
        let sftp = self.get_sftp()?;

        // Create remote file
        let mut remote_file = sftp.create(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to create remote file: {}", e)))?;

        let start_time = std::time::Instant::now();
        let file_size = content.len() as u64;

        // Write all content to remote file
        remote_file.write_all(content)
            .map_err(|e| SSHError::FileTransferError(format!("Failed to write to remote file: {}", e)))?;

        // Sync to ensure data is written
        remote_file.fsync()
            .map_err(|e| SSHError::FileTransferError(format!("Failed to sync remote file: {}", e)))?;

        let duration = start_time.elapsed().as_secs_f64();
        let transfer_rate = if duration > 0.0 {
            file_size as f64 / duration
        } else {
            0.0
        };

        Ok(FileTransferProgress {
            bytes_transferred: file_size,
            total_bytes: file_size,
            percentage: 100.0,
            transfer_rate,
            file_name: None,
        })
    }

    /// Upload a file to remote server with chunked writes and progress tracking
    ///
    /// Uses 256KB chunks with per-chunk flush to avoid timeout accumulation.
    /// Each chunk gets a fresh timeout window from the session timeout setting.
    pub fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        progress_callback: Option<ProgressCallback>
    ) -> Result<FileTransferProgress> {
        let sftp = self.get_sftp()?;

        // Open local file
        let local_file = File::open(local_path)
            .map_err(|e| SSHError::FileTransferError(format!("Failed to open local file: {}", e)))?;

        let file_size = local_file.metadata()?.len();
        let file_name = local_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| SSHError::FileTransferError(format!("Failed to extract filename from path: {:?}", local_path)))?
            .to_string();

        let mut reader = BufReader::with_capacity(CHUNK_SIZE, local_file);

        // Create remote file
        let mut remote_file = sftp.create(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to create remote file: {}", e)))?;

        // Transfer file with chunked writes and progress tracking
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut bytes_transferred = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            // Write chunk with error context
            let chunk_start = std::time::Instant::now();

            match remote_file.write_all(&buffer[..bytes_read]) {
                Ok(_) => {
                    // Flush chunk to ensure it's written before next chunk
                    // This prevents timeout accumulation across multiple chunks
                    if let Err(e) = remote_file.fsync() {
                        let chunk_duration = chunk_start.elapsed();
                        return Err(SSHError::FileTransferError(
                            format!("Failed to flush chunk for file '{}' ({} bytes, {:.1}% complete) after {:?}: {}",
                                   file_name,
                                   bytes_transferred,
                                   (bytes_transferred as f32 / file_size as f32) * 100.0,
                                   chunk_duration,
                                   e)
                        ).into());
                    }
                },
                Err(e) => {
                    let chunk_duration = chunk_start.elapsed();
                    return Err(SSHError::FileTransferError(
                        format!("Failed to write chunk for file '{}' ({} bytes, {:.1}% complete) after {:?}: {}",
                               file_name,
                               bytes_transferred,
                               (bytes_transferred as f32 / file_size as f32) * 100.0,
                               chunk_duration,
                               e)
                    ).into());
                }
            }

            bytes_transferred += bytes_read as u64;

            // Call progress callback if provided
            if let Some(ref callback) = progress_callback {
                callback(bytes_transferred, file_size);
            }
        }

        let duration = start_time.elapsed().as_secs_f64();
        let transfer_rate = if duration > 0.0 {
            bytes_transferred as f64 / duration
        } else {
            0.0
        };

        Ok(FileTransferProgress {
            bytes_transferred,
            total_bytes: file_size,
            percentage: (bytes_transferred as f32 / file_size as f32) * 100.0,
            transfer_rate,
            file_name: None,
        })
    }

    /// Download a file from remote server
    pub fn download_file(
        &self,
        remote_path: &str,
        local_path: &Path,
        progress_callback: Option<ProgressCallback>
    ) -> Result<FileTransferProgress> {
        let sftp = self.get_sftp()?;

        // Get remote file info
        let stat = sftp.stat(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to stat remote file: {}", e)))?;

        let file_size = stat.size.ok_or_else(|| {
            crate::log_error!(category: "SFTP", message: "File size unavailable", details: "File: {}", remote_path);
            SSHError::FileTransferError(format!("File size not available for: {}", remote_path))
        })?;

        // Open remote file
        let mut remote_file = sftp.open(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to open remote file: {}", e)))?;

        // Create local file
        let local_file = File::create(local_path)
            .map_err(|e| SSHError::FileTransferError(format!("Failed to create local file: {}", e)))?;

        let mut writer = BufWriter::with_capacity(self.buffer_size, local_file);

        // Transfer file with progress tracking
        let mut buffer = vec![0u8; self.buffer_size];
        let mut bytes_transferred = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let bytes_read = remote_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            writer.write_all(&buffer[..bytes_read])
                .map_err(|e| SSHError::FileTransferError(format!("Failed to write to local file: {}", e)))?;

            bytes_transferred += bytes_read as u64;

            // Call progress callback if provided
            if let Some(ref callback) = progress_callback {
                callback(bytes_transferred, file_size);
            }
        }

        writer.flush()?;

        let duration = start_time.elapsed().as_secs_f64();
        let transfer_rate = if duration > 0.0 {
            bytes_transferred as f64 / duration
        } else {
            0.0
        };

        Ok(FileTransferProgress {
            bytes_transferred,
            total_bytes: file_size,
            percentage: (bytes_transferred as f32 / file_size as f32) * 100.0,
            transfer_rate,
            file_name: None,
        })
    }

    /// List files in a directory
    pub fn list_directory(&self, remote_path: &str) -> Result<Vec<SftpFileEntry>> {
        let sftp = self.get_sftp()?;

        let mut files = Vec::new();
        let entries = sftp.readdir(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to list directory: {}", e)))?;

        for (path, stat) in entries {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| SSHError::FileTransferError(format!("Failed to extract filename from path: {:?}", path)))?
                .to_string();

            let size = if stat.is_dir() {
                0
            } else {
                stat.size.ok_or_else(|| {
                    crate::log_error!(category: "SFTP", message: "File size unavailable", details: "File: {:?}", path);
                    SSHError::FileTransferError(format!("File size not available for: {:?}", path))
                })?
            };

            let permissions = stat.perm.ok_or_else(|| {
                crate::log_error!(category: "SFTP", message: "File permissions unavailable", details: "File: {:?}", path);
                SSHError::FileTransferError(format!("File permissions not available for: {:?}", path))
            })?;

            files.push(SftpFileEntry {
                name: name.clone(),
                path: path.to_string_lossy().to_string(),
                size,
                is_directory: stat.is_dir(),
                permissions,
                modified_time: stat.mtime,
            });
        }

        Ok(files)
    }

    /// Create a directory (single level only - use SSH mkdir -p for recursive)
    /// Note: For recursive directory creation, use SSH commands in manager.rs for better performance
    pub fn create_directory(&self, remote_path: &str, mode: i32) -> Result<()> {
        let sftp = self.get_sftp()?;

        sftp.mkdir(Path::new(remote_path), mode)
            .map_err(|e| SSHError::FileTransferError(format!("Failed to create directory: {}", e)))?;

        Ok(())
    }

    /// Delete a file
    pub fn delete_file(&self, remote_path: &str) -> Result<()> {
        let sftp = self.get_sftp()?;

        sftp.unlink(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to delete file: {}", e)))?;

        Ok(())
    }

    /// Delete a directory
    pub fn delete_directory(&self, remote_path: &str) -> Result<()> {
        let sftp = self.get_sftp()?;

        sftp.rmdir(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to delete directory: {}", e)))?;

        Ok(())
    }

    /// Check if a file or directory exists
    pub fn exists(&self, remote_path: &str) -> Result<bool> {
        let sftp = self.get_sftp()?;
        Ok(sftp.stat(Path::new(remote_path)).is_ok())
    }

    /// Get file or directory information
    pub fn stat(&self, remote_path: &str) -> Result<SftpFileEntry> {
        let sftp = self.get_sftp()?;

        let stat = sftp.stat(Path::new(remote_path))
            .map_err(|e| SSHError::FileTransferError(format!("Failed to stat path: {}", e)))?;

        let name = Path::new(remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| SSHError::FileTransferError(format!("Failed to extract filename from path: {}", remote_path)))?
            .to_string();

        let size = if stat.is_dir() {
            0
        } else {
            stat.size.ok_or_else(|| {
                crate::log_error!(category: "SFTP", message: "File size unavailable", details: "File: {}", remote_path);
                SSHError::FileTransferError(format!("File size not available for: {}", remote_path))
            })?
        };

        let permissions = stat.perm.ok_or_else(|| {
            crate::log_error!(category: "SFTP", message: "File permissions unavailable", details: "File: {}", remote_path);
            SSHError::FileTransferError(format!("File permissions not available for: {}", remote_path))
        })?;

        Ok(SftpFileEntry {
            name,
            path: remote_path.to_string(),
            size,
            is_directory: stat.is_dir(),
            permissions,
            modified_time: stat.mtime,
        })
    }

    /// Rename/move a file or directory
    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<()> {
        let sftp = self.get_sftp()?;

        sftp.rename(Path::new(old_path), Path::new(new_path), None)
            .map_err(|e| SSHError::FileTransferError(format!("Failed to rename: {}", e)))?;

        Ok(())
    }
}

/// Chunk size for file uploads (256KB)
/// Matches SFTP best practices and OpenSSH behavior for large file transfers
const CHUNK_SIZE: usize = 256 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::test_utils::*;

    #[test]
    fn test_file_transfer_progress() {
        let progress = FileTransferProgress {
            bytes_transferred: 5000,
            total_bytes: 10000,
            percentage: 50.0,
            transfer_rate: 1000.0,
            file_name: None,
        };

        assert_eq!(progress.bytes_transferred, 5000);
        assert_eq!(progress.total_bytes, 10000);
        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.transfer_rate, 1000.0);
    }

    #[test]
    fn test_remote_file_info() {
        let info = SftpFileEntry {
            name: "test.txt".to_string(),
            path: "/home/user/test.txt".to_string(),
            size: 1024,
            is_directory: false,
            permissions: 0o644,
            modified_time: Some(1234567890),
        };

        assert_eq!(info.name, "test.txt");
        assert_eq!(info.size, 1024);
        assert!(!info.is_directory);
        assert_eq!(info.permissions, 0o644);
    }

    #[test]
    fn test_progress_calculation_logic() {
        // Test our progress calculation business logic
        let progress = MockProgressBuilder::at_percentage(2048, 25.0);
        assert_eq!(progress.bytes_transferred, 512);
        assert_eq!(progress.percentage, 25.0);
        assert_eq!(progress.total_bytes, 2048);

        // Test edge cases
        let empty_progress = MockProgressBuilder::start(0);
        assert_eq!(empty_progress.percentage, 0.0);

        let complete_progress = MockProgressBuilder::completed(1000);
        assert_eq!(complete_progress.percentage, 100.0);
        assert_eq!(complete_progress.bytes_transferred, complete_progress.total_bytes);
    }



    #[test]
    fn test_transfer_progress_rate_calculation() {
        // Test business logic for calculating transfer rates
        let slow_transfer = FileTransferProgress {
            bytes_transferred: 1000,
            total_bytes: 10000,
            percentage: 10.0,
            transfer_rate: 500.0, // 500 bytes/sec
            file_name: None,
        };

        let fast_transfer = FileTransferProgress {
            bytes_transferred: 5000,
            total_bytes: 10000,
            percentage: 50.0,
            transfer_rate: 2048.0, // 2KB/sec
            file_name: None,
        };

        // Verify our rate calculations make sense
        assert!(fast_transfer.transfer_rate > slow_transfer.transfer_rate);
        assert_eq!(slow_transfer.percentage, 10.0);
        assert_eq!(fast_transfer.percentage, 50.0);

        // Test edge case - zero transfer rate
        let stalled_transfer = FileTransferProgress {
            bytes_transferred: 100,
            total_bytes: 1000,
            percentage: 10.0,
            transfer_rate: 0.0,
            file_name: None,
        };
        assert_eq!(stalled_transfer.transfer_rate, 0.0);
    }

    #[test]
    fn test_path_validation_business_logic() {
        // Test our application's path validation logic
        assert!(MockPathValidator::is_valid_remote_path("/home/user"));
        assert!(MockPathValidator::is_valid_remote_path("/tmp/file.txt"));
        assert!(MockPathValidator::is_valid_remote_path("/"));

        // Invalid paths according to our business rules
        assert!(!MockPathValidator::is_valid_remote_path("../etc/passwd")); // Path traversal
        assert!(!MockPathValidator::is_valid_remote_path("")); // Empty path
        assert!(!MockPathValidator::is_valid_remote_path("relative/path")); // Relative path
        assert!(!MockPathValidator::is_valid_remote_path("/home\0/user")); // Null byte
    }

    #[test]
    fn test_path_normalization_business_logic() {
        // Test our path normalization logic
        assert_eq!(MockPathValidator::normalize_path("home/user"), "/home/user");
        assert_eq!(MockPathValidator::normalize_path("/home/user"), "/home/user");
        assert_eq!(MockPathValidator::normalize_path(""), "/");
        assert_eq!(MockPathValidator::normalize_path("documents/file.txt"), "/documents/file.txt");
    }

    #[test]
    fn test_filename_extraction_logic() {
        // Test our filename extraction business logic
        assert_eq!(MockPathValidator::extract_filename("/home/user/file.txt"), Some("file.txt".to_string()));
        assert_eq!(MockPathValidator::extract_filename("/root/script.sh"), Some("script.sh".to_string()));
        assert_eq!(MockPathValidator::extract_filename("/"), None);
        assert_eq!(MockPathValidator::extract_filename("single"), Some("single".to_string()));
    }

    #[test]
    fn test_mock_filesystem_business_logic() {
        // Test our mock filesystem for consistent behavior
        let mut fs = MockFileSystem::new();

        // Add directories first, then files (proper order for filesystem)
        fs.add_directory("/home/user", 0o755)
          .add_directory("/home/user/projects", 0o755)
          .add_file("/home/user/document.txt", 2048, 0o644)
          .add_file("/home/user/script.sh", 512, 0o755);

        // Test file retrieval
        let doc_info = fs.get_file_info("/home/user/document.txt").unwrap();
        assert_eq!(doc_info.size, 2048);
        assert!(!doc_info.is_directory);
        assert_eq!(doc_info.permissions, 0o644);
        assert_eq!(doc_info.name, "document.txt");

        // Test directory retrieval
        let dir_info = fs.get_file_info("/home/user").unwrap();
        assert!(dir_info.is_directory);
        assert_eq!(dir_info.permissions, 0o755);

        // Test directory listing
        let listing = fs.list_directory("/home/user").unwrap();
        assert_eq!(listing.len(), 3); // Two files + one subdirectory

        let names: Vec<&str> = listing.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"document.txt"));
        assert!(names.contains(&"script.sh"));
        assert!(names.contains(&"projects"));
    }

    #[test]
    fn test_error_handling_in_mock_filesystem() {
        let fs = MockFileSystem::new();

        // Test file not found
        let result = fs.get_file_info("/nonexistent/file.txt");
        assert!(result.is_err());

        if let Err(SSHError::FileTransferError(msg)) = result {
            assert!(msg.contains("File not found"));
        } else {
            panic!("Expected FileTransferError");
        }

        // Test directory not found
        let result = fs.list_directory("/nonexistent/dir");
        assert!(result.is_err());

        if let Err(SSHError::FileTransferError(msg)) = result {
            assert!(msg.contains("Directory not found"));
        } else {
            panic!("Expected FileTransferError");
        }
    }

    #[test]
    fn test_permission_handling_logic() {
        // Test our permission handling business logic
        let mut fs = MockFileSystem::new();

        // Different permission combinations
        fs.add_file("/readonly.txt", 100, 0o444)
          .add_file("/executable.sh", 200, 0o755)
          .add_file("/writable.txt", 300, 0o644)
          .add_directory("/restricted", 0o700);

        let readonly = fs.get_file_info("/readonly.txt").unwrap();
        assert_eq!(readonly.permissions, 0o444);

        let executable = fs.get_file_info("/executable.sh").unwrap();
        assert_eq!(executable.permissions, 0o755);

        let restricted_dir = fs.get_file_info("/restricted").unwrap();
        assert_eq!(restricted_dir.permissions, 0o700);
        assert!(restricted_dir.is_directory);
    }
}