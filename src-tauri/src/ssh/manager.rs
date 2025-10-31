use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use tauri::Emitter;
use super::{SSHConnection, ConnectionConfig, ConnectionInfo};
use super::commands::CommandResult;
use super::sftp::{FileTransferProgress, RemoteFileInfo};
use crate::security::SecurePassword;
use crate::retry::patterns;
use crate::{debug_log, info_log, error_log};

/// Connection lifecycle management with proper cleanup and error handling
#[derive(Debug)]
pub struct ConnectionManager {
    connection: Arc<Mutex<Option<SSHConnection>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
        }
    }

    /// Establish a new SSH connection, cleaning up any existing connection first
    pub async fn connect(&self, host: String, port: u16, username: String, password: &SecurePassword) -> Result<ConnectionInfo> {
        // Ensure any existing connection is properly cleaned up
        self.disconnect().await?;

        // Create new connection with default config
        let config = ConnectionConfig::default();
        let mut connection = SSHConnection::new(host, port, username, config);

        // Attempt to connect using secure password
        // We need to extract the password before the async call since closures can't be async
        let pwd_string = password.with_password(|pwd| pwd.to_string());
        connection.connect(&pwd_string).await?;

        // Get connection info before storing
        let info = connection.get_info();

        // Store the connection
        {
            let mut conn = self.connection.lock().await;
            *conn = Some(connection);
        }

        Ok(info)
    }

    /// Disconnect and clean up the current connection
    pub async fn disconnect(&self) -> Result<()> {
        let mut conn = self.connection.lock().await;
        if let Some(mut connection) = conn.take() {
            connection.disconnect().await?;
        }
        Ok(())
    }

    /// Check if there's an active connection
    pub async fn is_connected(&self) -> bool {
        let conn = self.connection.lock().await;
        conn.as_ref().map_or(false, |c| c.is_connected())
    }

    /// Get current connection information
    pub async fn get_connection_info(&self) -> Option<ConnectionInfo> {
        let conn = self.connection.lock().await;
        conn.as_ref().map(|c| c.get_info())
    }

    /// Execute a command using the current connection
    pub async fn execute_command(&self, command: &str, timeout: Option<u64>) -> Result<CommandResult> {
        // Use retry logic for command execution
        patterns::retry_quick_operation(|| self.execute_command_once(command, timeout)).await
    }

    async fn execute_command_once(&self, command: &str, timeout: Option<u64>) -> Result<CommandResult> {
        let conn = self.connection.lock().await;
        match conn.as_ref() {
            Some(connection) => {
                if !connection.is_connected() {
                    error_log!("[SSH] ERROR: SSH connection is no longer active");
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }
                info_log!("[SSH] Executing command: {}", command);
                let session = connection.get_session()?;
                let executor = super::commands::CommandExecutor::new(session, timeout.unwrap_or(crate::cluster::timeouts::DEFAULT_COMMAND));
                let result = executor.execute(command).await?;
                debug_log!("[SSH] Command output: {} bytes stdout, {} bytes stderr",
                    result.stdout.len(), result.stderr.len());

                // Show stderr content if present (useful for debugging unexpected output)
                if !result.stderr.is_empty() {
                    debug_log!("[SSH] Command stderr: {}", result.stderr);
                }

                Ok(result)
            }
            None => {
                error_log!("[SSH] ERROR: Not connected to cluster");
                Err(anyhow::anyhow!("Please connect to the cluster first"))
            }
        }
    }

    /// Upload bytes directly to remote server with retry logic
    pub async fn upload_bytes(&self, remote_path: &str, content: &[u8]) -> Result<FileTransferProgress> {
        // Use retry logic for file uploads
        patterns::retry_file_operation(|| self.upload_bytes_once(remote_path, content)).await
    }

    async fn upload_bytes_once(&self, remote_path: &str, content: &[u8]) -> Result<FileTransferProgress> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.upload_bytes(remote_path, content);

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                result
            }
            None => Err(super::SSHError::SessionError("No active connection".to_string()).into())
        }
    }

    /// Upload a file using the current connection (without progress events)
    pub async fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<FileTransferProgress> {
        self.upload_file_with_progress(local_path, remote_path, None).await
    }

    /// Upload a file with optional progress event emission
    pub async fn upload_file_with_progress(
        &self,
        local_path: &str,
        remote_path: &str,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<FileTransferProgress> {
        // Use retry logic for file uploads
        patterns::retry_file_operation(|| self.upload_file_once(local_path, remote_path, app_handle.clone())).await
    }

    async fn upload_file_once(
        &self,
        local_path: &str,
        remote_path: &str,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<FileTransferProgress> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    error_log!("[SFTP] ERROR: SSH connection is no longer active");
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }
                info_log!("[SFTP] Uploading file: {} -> {}", local_path, remote_path);

                // Get file name for progress reporting
                let file_name = std::path::Path::new(local_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Get file size for progress calculation (used in closure below)
                let _file_size = std::fs::metadata(local_path)
                    .map(|m| m.len())
                    .unwrap_or(0);

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);

                // Create progress callback if app_handle is provided
                let progress_callback: Option<super::sftp::ProgressCallback> = app_handle.map(|handle| {
                    let file_name = file_name.clone();
                    let start_time = std::time::Instant::now();

                    Box::new(move |bytes_transferred: u64, total_bytes: u64| {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let transfer_rate = if elapsed > 0.0 {
                            (bytes_transferred as f64 / elapsed) / (1024.0 * 1024.0) // Convert to MB/s
                        } else {
                            0.0
                        };

                        let percentage = if total_bytes > 0 {
                            (bytes_transferred as f32 / total_bytes as f32) * 100.0
                        } else {
                            0.0
                        };

                        let progress = crate::types::FileUploadProgress {
                            file_name: file_name.clone(),
                            bytes_transferred,
                            total_bytes,
                            percentage,
                            transfer_rate_mbps: transfer_rate,
                        };

                        // Emit progress event to frontend
                        let _ = handle.emit("file-upload-progress", progress);
                    }) as super::sftp::ProgressCallback
                });

                let result = sftp.upload_file(
                    std::path::Path::new(local_path),
                    remote_path,
                    progress_callback
                );

                // Reset to command timeout after operation (regardless of success/failure)
                connection.reset_command_timeout()?;

                let final_result = result?;
                info_log!("[SFTP] Upload complete: {} bytes transferred", final_result.bytes_transferred);
                Ok(final_result)
            }
            None => {
                error_log!("[SFTP] ERROR: Not connected to cluster");
                Err(anyhow::anyhow!("Please connect to the cluster first"))
            }
        }
    }

    /// Download a file using the current connection
    pub async fn download_file(&self, remote_path: &str, local_path: &str) -> Result<FileTransferProgress> {
        // Use retry logic for file downloads
        patterns::retry_file_operation(|| self.download_file_once(remote_path, local_path)).await
    }

    async fn download_file_once(&self, remote_path: &str, local_path: &str) -> Result<FileTransferProgress> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    error_log!("[SFTP] ERROR: SSH connection is no longer active");
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }
                info_log!("[SFTP] Downloading file: {} -> {}", remote_path, local_path);

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.download_file(remote_path, std::path::Path::new(local_path), None);

                // Reset to command timeout after operation (regardless of success/failure)
                connection.reset_command_timeout()?;

                let final_result = result?;
                info_log!("[SFTP] Download complete: {} bytes transferred", final_result.bytes_transferred);
                Ok(final_result)
            }
            None => {
                error_log!("[SFTP] ERROR: Not connected to cluster");
                Err(anyhow::anyhow!("Please connect to the cluster first"))
            }
        }
    }

    /// List files in a directory using the current connection
    pub async fn list_files(&self, remote_path: &str) -> Result<Vec<RemoteFileInfo>> {
        // Use retry logic for directory listing
        patterns::retry_quick_operation(|| self.list_files_once(remote_path)).await
    }

    async fn list_files_once(&self, remote_path: &str) -> Result<Vec<RemoteFileInfo>> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.list_directory(remote_path);

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                result
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// List files in a directory with metadata for OutputFile
    /// Only returns regular files (not directories)
    pub async fn list_files_with_metadata(&self, remote_path: &str) -> Result<Vec<crate::types::OutputFile>> {
        patterns::retry_quick_operation(|| self.list_files_with_metadata_once(remote_path)).await
    }

    async fn list_files_with_metadata_once(&self, remote_path: &str) -> Result<Vec<crate::types::OutputFile>> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.list_files_with_metadata(remote_path);

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                result
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// Get file metadata (stat) for a remote file
    pub async fn stat_file(&self, remote_path: &str) -> Result<RemoteFileInfo> {
        patterns::retry_quick_operation(|| self.stat_file_once(remote_path)).await
    }

    async fn stat_file_once(&self, remote_path: &str) -> Result<RemoteFileInfo> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.stat(remote_path);

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                result
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// Create a directory using SSH mkdir -p command
    pub async fn create_directory(&self, remote_path: &str) -> Result<()> {
        // Use retry logic for directory creation
        patterns::retry_quick_operation(|| self.create_directory_once(remote_path)).await
    }

    async fn create_directory_once(&self, remote_path: &str) -> Result<()> {
        // Use mkdir -p command for directory creation (matches delete_directory pattern)
        info_log!("[SSH] Creating directory: {}", remote_path);
        let mkdir_command = format!("mkdir -p -m 0755 {}", crate::validation::shell::escape_parameter(remote_path));
        self.execute_command(&mkdir_command, Some(crate::cluster::timeouts::QUICK_OPERATION)).await?;
        info_log!("[SSH] Directory created successfully: {}", remote_path);
        Ok(())
    }

    /// Delete a directory and all its contents using SSH command
    pub async fn delete_directory(&self, remote_path: &str) -> Result<CommandResult> {
        // Use rm -rf command for directory deletion with retry logic
        info_log!("[SSH] Deleting directory: {}", remote_path);
        let rm_command = format!("rm -rf {}", crate::validation::shell::escape_parameter(remote_path));
        let result = self.execute_command(&rm_command, Some(crate::cluster::timeouts::QUICK_OPERATION)).await?;
        info_log!("[SSH] Directory deleted successfully: {}", remote_path);
        Ok(result)
    }

    /// Sync directory from source to destination using rsync (cluster-side operation)
    ///
    /// Uses rsync to efficiently mirror directory contents. Only copies changed files on repeat syncs.
    /// This is a cluster-side operation (both source and dest are on the cluster).
    ///
    /// # Arguments
    /// * `source` - Source directory path (must end with / to sync contents)
    /// * `destination` - Destination directory path
    ///
    /// # Example
    /// ```rust
    /// // Mirror project to scratch
    /// sync_directory_rsync(
    ///     "/projects/user/namdrunner_jobs/job_123/",
    ///     "/scratch/alpine/user/namdrunner_jobs/job_123/"
    /// ).await?;
    /// ```
    pub async fn sync_directory_rsync(&self, source: &str, destination: &str) -> Result<CommandResult> {
        info_log!("[SSH] Syncing directory: {} -> {}", source, destination);

        // Use rsync with archive mode and compression
        // -a: archive mode (preserves permissions, timestamps, etc.)
        // -z: compress during transfer
        let rsync_command = format!(
            "rsync -az {} {}",
            crate::validation::shell::escape_parameter(source),
            crate::validation::shell::escape_parameter(destination)
        );

        // Use default command timeout (rsync is efficient for cluster-side operations)
        let result = self.execute_command(&rsync_command, Some(crate::cluster::timeouts::DEFAULT_COMMAND)).await?;

        info_log!("[SSH] Directory sync completed: {} -> {}", source, destination);
        Ok(result)
    }

    /// Get file information using native SFTP
    pub async fn get_file_info(&self, remote_path: &str) -> Result<RemoteFileInfo> {
        // Use retry logic for file info retrieval
        patterns::retry_quick_operation(|| self.get_file_info_once(remote_path)).await
    }

    async fn get_file_info_once(&self, remote_path: &str) -> Result<RemoteFileInfo> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.stat(remote_path);

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                result
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// Check if a file or directory exists
    pub async fn file_exists(&self, remote_path: &str) -> Result<bool> {
        // Use retry logic for existence checking
        patterns::retry_quick_operation(|| self.file_exists_once(remote_path)).await
    }

    async fn file_exists_once(&self, remote_path: &str) -> Result<bool> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);

                // Try to stat the file - if it succeeds, the file exists
                let stat_result = match sftp.stat(remote_path) {
                    Ok(_) => Ok(true),
                    Err(e) => {
                        // Check if the error indicates the file doesn't exist
                        let error_msg = e.to_string().to_lowercase();
                        if error_msg.contains("no such file") ||
                           error_msg.contains("not found") ||
                           error_msg.contains("does not exist") {
                            Ok(false)
                        } else {
                            // Some other error occurred
                            Err(e)
                        }
                    }
                };

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                stat_result
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// Upload a file with existence checking
    pub async fn upload_file_with_check(&self, local_path: &str, remote_path: &str, overwrite: bool) -> Result<FileTransferProgress> {
        // Check if the file already exists
        let exists = self.file_exists(remote_path).await?;

        if exists && !overwrite {
            return Err(anyhow::anyhow!("File '{}' already exists and overwrite is not enabled", remote_path));
        }

        // Proceed with upload
        self.upload_file(local_path, remote_path).await
    }

    /// Get the username of the current connection
    pub async fn get_username(&self) -> Result<String> {
        let conn = self.connection.lock().await;
        match conn.as_ref() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }
                Ok(connection.get_username().to_string())
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// Check if a directory exists
    pub async fn directory_exists(&self, remote_path: &str) -> Result<bool> {
        // Directories can be checked the same way as files
        self.file_exists(remote_path).await
    }

    /// Get the size of a remote file
    pub async fn get_file_size(&self, remote_path: &str) -> Result<u64> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.stat(remote_path);

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                let stat = result?;
                Ok(stat.size)
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// Read content from a remote file
    pub async fn read_remote_file(&self, remote_path: &str) -> Result<String> {
        let command = format!("cat '{}'", remote_path.replace('\'', "'\\''"));
        self.execute_command(&command, None).await.map(|result| result.stdout)
    }

    /// Send keepalive to maintain the connection
    pub async fn keepalive(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        match conn.as_ref() {
            Some(connection) => {
                connection.keepalive().await
            }
            None => Ok(()) // No connection to keep alive
        }
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        // Best-effort cleanup when manager is dropped
        // Note: This is synchronous cleanup since Drop can't be async
        // The connection's own Drop implementation will handle basic cleanup
        if let Ok(mut conn) = self.connection.try_lock() {
            let _ = conn.take(); // This will trigger SSHConnection's Drop
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurePassword;

    #[tokio::test]
    async fn test_connection_manager_creation() {
        let manager = ConnectionManager::new();
        assert!(!manager.is_connected().await);
        assert!(manager.get_connection_info().await.is_none());
    }

    #[tokio::test]
    async fn test_disconnect_without_connection() {
        let manager = ConnectionManager::new();
        // Should not fail even if no connection exists
        assert!(manager.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn test_execute_command_without_connection() {
        let manager = ConnectionManager::new();
        let result = manager.execute_command("echo test", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Please connect to the cluster"));
    }

    #[tokio::test]
    async fn test_keepalive_without_connection() {
        let manager = ConnectionManager::new();
        // Should succeed (no-op) even without connection
        assert!(manager.keepalive().await.is_ok());
    }

    #[tokio::test]
    async fn test_file_operations_without_connection() {
        let manager = ConnectionManager::new();

        // Test upload without connection
        let upload_result = manager.upload_file("/local/file.txt", "/remote/file.txt").await;
        assert!(upload_result.is_err());
        assert!(upload_result.unwrap_err().to_string().contains("Please connect to the cluster"));

        // Test download without connection
        let download_result = manager.download_file("/remote/file.txt", "/local/file.txt").await;
        assert!(download_result.is_err());
        assert!(download_result.unwrap_err().to_string().contains("Please connect to the cluster"));

        // Test list files without connection
        let list_result = manager.list_files("/home/user").await;
        assert!(list_result.is_err());
        assert!(list_result.unwrap_err().to_string().contains("Please connect to the cluster"));

        // Test create directory without connection
        let mkdir_result = manager.create_directory("/remote/newdir").await;
        assert!(mkdir_result.is_err());
        assert!(mkdir_result.unwrap_err().to_string().contains("Please connect to the cluster"));

        // Test get file info without connection
        let stat_result = manager.get_file_info("/remote/file.txt").await;
        assert!(stat_result.is_err());
        assert!(stat_result.unwrap_err().to_string().contains("Please connect to the cluster"));
    }

    #[tokio::test]
    async fn test_connection_manager_state_consistency() {
        let manager = ConnectionManager::new();

        // Initial state should be consistent
        assert!(!manager.is_connected().await);
        assert!(manager.get_connection_info().await.is_none());

        // Multiple disconnects should be safe
        assert!(manager.disconnect().await.is_ok());
        assert!(manager.disconnect().await.is_ok());
        assert!(manager.disconnect().await.is_ok());

        // State should remain consistent
        assert!(!manager.is_connected().await);
        assert!(manager.get_connection_info().await.is_none());
    }



    #[tokio::test]
    async fn test_secure_password_handling_in_connect() {
        // Test our SecurePassword integration without actual connection
        let manager = ConnectionManager::new();

        let password = SecurePassword::from_str("testpassword");

        // This will fail because we can't actually connect, but tests password handling
        let result = manager.connect(
            "nonexistent.host.test".to_string(),
            22,
            "testuser".to_string(),
            &password
        ).await;

        // Should fail due to network error, not password handling
        assert!(result.is_err());
        // The error should be about connection, not password format
        let error_msg = result.unwrap_err().to_string();
        assert!(!error_msg.contains("password"));
    }

    #[tokio::test]
    async fn test_connection_info_consistency() {
        let manager = ConnectionManager::new();

        // Before connection
        assert!(manager.get_connection_info().await.is_none());
        assert!(!manager.is_connected().await);

        // After failed connection attempt (state should remain consistent)
        let password = SecurePassword::from_str("testpass");
        let _ = manager.connect(
            "invalid.host".to_string(),
            22,
            "user".to_string(),
            &password
        ).await;

        // Should still be disconnected
        assert!(!manager.is_connected().await);
        assert!(manager.get_connection_info().await.is_none());
    }

    #[tokio::test]
    async fn test_drop_cleanup_behavior() {
        // Test that ConnectionManager can be safely dropped
        {
            let manager = ConnectionManager::new();
            let password = SecurePassword::from_str("testpass");

            // Attempt connection (will fail)
            let _ = manager.connect(
                "test.host".to_string(),
                22,
                "user".to_string(),
                &password
            ).await;

            // Manager goes out of scope here - should not panic
        }

        // Test multiple managers don't interfere
        let manager1 = ConnectionManager::new();
        let manager2 = ConnectionManager::new();

        assert!(!manager1.is_connected().await);
        assert!(!manager2.is_connected().await);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager = Arc::new(ConnectionManager::new());

        // Test concurrent access to manager state
        let manager1 = manager.clone();
        let manager2 = manager.clone();

        let task1 = tokio::spawn(async move {
            manager1.is_connected().await
        });

        let task2 = tokio::spawn(async move {
            manager2.get_connection_info().await
        });

        let (connected, info) = tokio::join!(task1, task2);

        assert!(!connected.unwrap());
        assert!(info.unwrap().is_none());
    }

}