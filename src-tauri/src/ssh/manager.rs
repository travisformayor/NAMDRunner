use std::sync::Arc;
use std::time::Duration;
use std::future::Future;
use tokio::sync::Mutex;
use anyhow::Result;
use tauri::Emitter;
use super::{SSHConnection, ConnectionConfig, ConnectionInfo};
use super::commands::CommandResult;
use super::sftp::{FileTransferProgress, SftpFileEntry};
use crate::security::SecurePassword;
use crate::{log_debug, log_info, log_error};

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
        conn.as_ref().is_some_and(|c| c.is_connected())
    }

    /// Get current connection information
    pub async fn get_connection_info(&self) -> Option<ConnectionInfo> {
        let conn = self.connection.lock().await;
        conn.as_ref().map(|c| c.get_info())
    }

    /// Execute a command using the current connection
    pub async fn execute_command(&self, command: &str, timeout: Option<u64>) -> Result<CommandResult> {
        // Use retry logic for command execution
        retry_quick(|| self.execute_command_once(command, timeout)).await
    }

    async fn execute_command_once(&self, command: &str, timeout: Option<u64>) -> Result<CommandResult> {
        let conn = self.connection.lock().await;
        match conn.as_ref() {
            Some(connection) => {
                if !connection.is_connected() {
                    log_error!(category: "SSH", message: "SSH connection is no longer active");
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }
                log_info!(category: "SSH", message: "Executing command", details: "{}", command);
                let session = connection.get_session()?;
                let executor = super::commands::CommandExecutor::new(session, timeout.unwrap_or(crate::cluster::timeouts::DEFAULT_COMMAND));
                let result = executor.execute(command).await?;
                log_debug!(category: "SSH", message: "Command output", details: "{} bytes stdout, {} bytes stderr", result.stdout.len(), result.stderr.len());

                // Show stderr content if present (useful for debugging unexpected output)
                if !result.stderr.is_empty() {
                    log_debug!(category: "SSH", message: "Command stderr", details: "{}", result.stderr);
                }

                Ok(result)
            }
            None => {
                log_error!(category: "SSH", message: "Not connected to cluster");
                Err(anyhow::anyhow!("Please connect to the cluster first"))
            }
        }
    }

    /// Upload bytes directly to remote server with retry logic
    pub async fn upload_bytes(&self, remote_path: &str, content: &[u8]) -> Result<FileTransferProgress> {
        // Use retry logic for file uploads
        retry_files(|| self.upload_bytes_once(remote_path, content)).await
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

    /// Upload a file with optional progress event emission
    pub async fn upload_file(
        &self,
        local_path: &str,
        remote_path: &str,
        app_handle: Option<tauri::AppHandle>,
        progress_key: Option<String>,
    ) -> Result<FileTransferProgress> {
        // Use retry logic for file uploads
        retry_files(|| self.upload_file_once(local_path, remote_path, app_handle.clone(), progress_key.clone())).await
    }

    async fn upload_file_once(
        &self,
        local_path: &str,
        remote_path: &str,
        app_handle: Option<tauri::AppHandle>,
        progress_key: Option<String>,
    ) -> Result<FileTransferProgress> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    log_error!(category: "SFTP", message: "SSH connection is no longer active");
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }
                log_info!(category: "SFTP", message: "Uploading file", details: "{} -> {}", local_path, remote_path);

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
                    let progress_key = progress_key.clone();
                    let start_time = std::time::Instant::now();

                    Box::new(move |bytes_transferred: u64, total_bytes: u64| {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let transfer_rate = if elapsed > 0.0 {
                            bytes_transferred as f64 / elapsed // bytes per second
                        } else {
                            0.0
                        };

                        let percentage = if total_bytes > 0 {
                            (bytes_transferred as f32 / total_bytes as f32) * 100.0
                        } else {
                            0.0
                        };

                        let progress = super::sftp::FileTransferProgress {
                            bytes_transferred,
                            total_bytes,
                            percentage,
                            transfer_rate,
                            file_name: progress_key.clone(),
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
                log_info!(category: "SFTP", message: "Upload complete", details: "{} bytes transferred", final_result.bytes_transferred);
                Ok(final_result)
            }
            None => {
                log_error!(category: "SFTP", message: "Not connected to cluster");
                Err(anyhow::anyhow!("Please connect to the cluster first"))
            }
        }
    }

    /// Download a file using the current connection
    pub async fn download_file(&self, remote_path: &str, local_path: &str) -> Result<FileTransferProgress> {
        // Use retry logic for file downloads
        retry_files(|| self.download_file_once(remote_path, local_path)).await
    }

    async fn download_file_once(&self, remote_path: &str, local_path: &str) -> Result<FileTransferProgress> {
        let mut conn = self.connection.lock().await;
        match conn.as_mut() {
            Some(connection) => {
                if !connection.is_connected() {
                    log_error!(category: "SFTP", message: "SSH connection is no longer active");
                    return Err(anyhow::anyhow!("SSH connection is no longer active"));
                }
                log_info!(category: "SFTP", message: "Downloading file", details: "{} -> {}", remote_path, local_path);

                // Set file transfer timeout before SFTP operation
                connection.set_file_transfer_timeout()?;

                let session = connection.get_session()?;
                let sftp = super::sftp::SFTPOperations::new(session);
                let result = sftp.download_file(remote_path, std::path::Path::new(local_path), None);

                // Reset to command timeout after operation (regardless of success/failure)
                connection.reset_command_timeout()?;

                let final_result = result?;
                log_info!(category: "SFTP", message: "Download complete", details: "{} bytes transferred", final_result.bytes_transferred);
                Ok(final_result)
            }
            None => {
                log_error!(category: "SFTP", message: "Not connected to cluster");
                Err(anyhow::anyhow!("Please connect to the cluster first"))
            }
        }
    }

    /// List files in a directory using the current connection
    /// If include_directories is false, only regular files are returned
    pub async fn list_files(&self, remote_path: &str, include_directories: bool) -> Result<Vec<SftpFileEntry>> {
        // Use retry logic for directory listing
        retry_quick(|| self.list_files_once(remote_path, include_directories)).await
    }

    async fn list_files_once(&self, remote_path: &str, include_directories: bool) -> Result<Vec<SftpFileEntry>> {
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
                let mut result = sftp.list_directory(remote_path)?;

                // Filter out directories if requested
                if !include_directories {
                    result.retain(|entry| !entry.is_directory);
                }

                // Reset to command timeout after operation
                connection.reset_command_timeout()?;

                Ok(result)
            }
            None => Err(anyhow::anyhow!("Please connect to the cluster first"))
        }
    }

    /// Create a directory using SSH mkdir -p command
    pub async fn create_directory(&self, remote_path: &str) -> Result<CommandResult> {
        // Use retry logic for directory creation
        retry_quick(|| self.create_directory_once(remote_path)).await
    }

    async fn create_directory_once(&self, remote_path: &str) -> Result<CommandResult> {
        // Use mkdir -p command for directory creation (matches delete_directory pattern)
        log_info!(category: "SSH", message: "Creating directory", details: "{}", remote_path);
        let mkdir_command = format!("mkdir -p -m 0755 {}", crate::security::shell::escape_parameter(remote_path));
        let result = self.execute_command(&mkdir_command, Some(crate::cluster::timeouts::QUICK_OPERATION)).await?;
        log_info!(category: "SSH", message: "Directory created successfully", details: "{}", remote_path);
        Ok(result)
    }

    /// Delete a directory and all its contents using SSH command
    pub async fn delete_directory(&self, remote_path: &str) -> Result<CommandResult> {
        // Use rm -rf command for directory deletion with retry logic
        log_info!(category: "SSH", message: "Deleting directory", details: "{}", remote_path);
        let rm_command = format!("rm -rf {}", crate::security::shell::escape_parameter(remote_path));
        let result = self.execute_command(&rm_command, Some(crate::cluster::timeouts::QUICK_OPERATION)).await?;
        log_info!(category: "SSH", message: "Directory deleted successfully", details: "{}", remote_path);
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
    /// ```ignore
    /// use crate::ssh::directory_structure::JobDirectoryStructure;
    /// // Mirror project to scratch using centralized path generation
    /// let source = format!("{}/", JobDirectoryStructure::project_dir("user", "job_123"));
    /// let dest = JobDirectoryStructure::scratch_dir("user", "job_123");
    /// manager.mirror_directory(&source, &dest).await?;
    /// ```
    pub async fn mirror_directory(&self, source: &str, destination: &str) -> Result<CommandResult> {
        log_info!(category: "SSH", message: "Syncing directory", details: "{} -> {}", source, destination);

        // Use rsync with archive mode and compression
        // -a: archive mode (preserves permissions, timestamps, etc.)
        // -z: compress during transfer
        let rsync_command = format!(
            "rsync -az {} {}",
            crate::security::shell::escape_parameter(source),
            crate::security::shell::escape_parameter(destination)
        );

        // Use default command timeout (rsync is efficient for cluster-side operations)
        let result = self.execute_command(&rsync_command, Some(crate::cluster::timeouts::DEFAULT_COMMAND)).await?;

        log_info!(category: "SSH", message: "Directory sync completed", details: "{} -> {}", source, destination);
        Ok(result)
    }

    /// Check if a file or directory exists
    pub async fn file_exists(&self, remote_path: &str) -> Result<bool> {
        // Use retry logic for existence checking
        retry_quick(|| self.file_exists_once(remote_path)).await
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

// =============================================================================
// Retry Logic
// =============================================================================
// Handles transient network failures with exponential backoff and jitter.
/// Retry quick operations (commands, queries, file checks)
/// Max 2 attempts, 200ms base delay, exponential backoff with jitter
/// Used by SSH operations and SLURM command execution
pub async fn retry_quick<T, F, Fut>(operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    retry_with_backoff(operation, 2, 200, 2000, 2.0).await
}

/// Retry file transfer operations (uploads, downloads)
/// Max 5 attempts, 2s base delay, patient retry for large file transfers
async fn retry_files<T, F, Fut>(operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    retry_with_backoff(operation, 5, 2000, 60000, 1.5).await
}

/// Core retry implementation with exponential backoff and jitter
async fn retry_with_backoff<T, F, Fut>(
    mut operation: F,
    max_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    backoff_multiplier: f64,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempts = 0;
    let mut delay_ms = base_delay_ms;

    loop {
        attempts += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempts >= max_attempts || !is_transient_error(&error) {
                    return Err(error);
                }

                log_debug!(
                    category: "Retry",
                    message: "Retrying operation",
                    details: "Attempt {}/{}, error: {}, delay: {}ms",
                    attempts, max_attempts, error, delay_ms
                );

                // Add jitter (±50%) to prevent thundering herd
                let jitter_factor = rand::Rng::gen_range(&mut rand::thread_rng(), 0.5..1.5);
                let jittered_delay = (delay_ms as f64 * jitter_factor) as u64;

                tokio::time::sleep(Duration::from_millis(jittered_delay)).await;

                // Calculate next delay with exponential backoff
                delay_ms = std::cmp::min(
                    (delay_ms as f64 * backoff_multiplier) as u64,
                    max_delay_ms
                );
            }
        }
    }
}

/// Determine if an error is transient and worth retrying
fn is_transient_error(error: &anyhow::Error) -> bool {
    let error_msg = error.to_string().to_lowercase();

    // Network-related errors that are often transient
    if error_msg.contains("timeout") ||
       error_msg.contains("connection") ||
       error_msg.contains("network") ||
       error_msg.contains("temporary") ||
       error_msg.contains("busy") ||
       error_msg.contains("unavailable") ||
       error_msg.contains("interrupted") ||
       error_msg.contains("broken pipe") {
        return true;
    }

    // Authentication and permission errors are permanent
    if error_msg.contains("authentication") ||
       error_msg.contains("permission") ||
       error_msg.contains("access denied") ||
       error_msg.contains("unauthorized") {
        return false;
    }

    // Default to not retrying for unknown errors
    false
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
        let upload_result = manager.upload_file("/local/file.txt", "/remote/file.txt", None, None).await;
        assert!(upload_result.is_err());
        assert!(upload_result.unwrap_err().to_string().contains("Please connect to the cluster"));

        // Test download without connection
        let download_result = manager.download_file("/remote/file.txt", "/local/file.txt").await;
        assert!(download_result.is_err());
        assert!(download_result.unwrap_err().to_string().contains("Please connect to the cluster"));

        // Test list files without connection
        let list_result = manager.list_files("/home/user", true).await;
        assert!(list_result.is_err());
        assert!(list_result.unwrap_err().to_string().contains("Please connect to the cluster"));

        // Test create directory without connection
        let mkdir_result = manager.create_directory("/remote/newdir").await;
        assert!(mkdir_result.is_err());
        assert!(mkdir_result.unwrap_err().to_string().contains("Please connect to the cluster"));
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

    // =========================================================================
    // Retry Logic Tests
    // =========================================================================

    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_successful_operation_no_retry() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok::<i32, anyhow::Error>(42)
            }
        }).await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Called only once
    }

    #[tokio::test]
    async fn test_retry_with_eventual_success() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                if count < 2 {
                    Err(anyhow::anyhow!("Temporary connection error"))
                } else {
                    Ok::<i32, anyhow::Error>(count as i32)
                }
            }
        }).await;

        assert_eq!(result.unwrap(), 2);
        assert_eq!(counter.load(Ordering::SeqCst), 2); // Called twice
    }

    #[tokio::test]
    async fn test_max_attempts_exceeded() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, anyhow::Error>(anyhow::anyhow!("Connection timeout"))
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2); // retry_quick has max 2 attempts
    }

    #[tokio::test]
    async fn test_permanent_error_no_retry() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, anyhow::Error>(anyhow::anyhow!("Authentication failed"))
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // No retry for auth errors
    }

    #[test]
    fn test_transient_error_detection() {
        assert!(is_transient_error(&anyhow::anyhow!("Connection timeout")));
        assert!(is_transient_error(&anyhow::anyhow!("Network error")));
        assert!(is_transient_error(&anyhow::anyhow!("Server busy")));
        assert!(is_transient_error(&anyhow::anyhow!("Temporarily unavailable")));

        assert!(!is_transient_error(&anyhow::anyhow!("Authentication failed")));
        assert!(!is_transient_error(&anyhow::anyhow!("Permission denied")));
        assert!(!is_transient_error(&anyhow::anyhow!("Access denied")));
        assert!(!is_transient_error(&anyhow::anyhow!("Invalid configuration")));
    }

    #[tokio::test]
    async fn test_backoff_progression() {
        // Verify exponential backoff happens (timing test)
        let start = std::time::Instant::now();
        let counter = Arc::new(AtomicU32::new(0));

        let _ = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, anyhow::Error>(anyhow::anyhow!("Timeout"))
            }
        }).await;

        let elapsed = start.elapsed();

        // Should take at least 100ms (one retry delay with jitter)
        assert!(elapsed.as_millis() >= 100);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_mirror_directory_without_connection() {
        let manager = ConnectionManager::new();

        // Test mirror directory without connection
        let result = manager.mirror_directory("/source/", "/destination/").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Please connect to the cluster"));
    }

    #[tokio::test]
    async fn test_list_files_filtering() {
        // This is a unit test for the business logic of filtering directories
        // The actual SSH operation is tested in integration tests
        let manager = ConnectionManager::new();

        // Without connection, should still fail predictably
        let result = manager.list_files("/test", true).await;
        assert!(result.is_err());

        let result = manager.list_files("/test", false).await;
        assert!(result.is_err());
    }
}