use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use super::{SSHConnection, ConnectionConfig, ConnectionInfo};
use super::commands::CommandResult;
use super::sftp::{FileTransferProgress, RemoteFileInfo};
use crate::security::SecurePassword;
use crate::retry::patterns;

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
        patterns::retry_quick_operation(|| {
            let connection = self.connection.clone();
            let command = command.to_string();
            let timeout = timeout;
            Box::pin(async move {
                let conn = connection.lock().await;
                match conn.as_ref() {
                    Some(connection) => {
                        let session = connection.get_session()?;
                        let executor = super::commands::CommandExecutor::new(session, timeout.unwrap_or(120));
                        executor.execute(&command).await
                    }
                    None => Err(anyhow::anyhow!("No SSH connection available"))
                }
            })
        }).await
    }

    /// Upload a file using the current connection
    pub async fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<FileTransferProgress> {
        // Use retry logic for file uploads
        patterns::retry_file_operation(|| {
            let connection = self.connection.clone();
            let local_path = local_path.to_string();
            let remote_path = remote_path.to_string();
            Box::pin(async move {
                let conn = connection.lock().await;
                match conn.as_ref() {
                    Some(connection) => {
                        let session = connection.get_session()?;
                        let sftp = super::sftp::SFTPOperations::new(session);
                        sftp.upload_file(std::path::Path::new(&local_path), &remote_path, None)
                    }
                    None => Err(anyhow::anyhow!("No SSH connection available"))
                }
            })
        }).await
    }

    /// Download a file using the current connection
    pub async fn download_file(&self, remote_path: &str, local_path: &str) -> Result<FileTransferProgress> {
        // Use retry logic for file downloads
        patterns::retry_file_operation(|| {
            let connection = self.connection.clone();
            let remote_path = remote_path.to_string();
            let local_path = local_path.to_string();
            Box::pin(async move {
                let conn = connection.lock().await;
                match conn.as_ref() {
                    Some(connection) => {
                        let session = connection.get_session()?;
                        let sftp = super::sftp::SFTPOperations::new(session);
                        sftp.download_file(&remote_path, std::path::Path::new(&local_path), None)
                    }
                    None => Err(anyhow::anyhow!("No SSH connection available"))
                }
            })
        }).await
    }

    /// List files in a directory using the current connection
    pub async fn list_files(&self, remote_path: &str) -> Result<Vec<RemoteFileInfo>> {
        // Use retry logic for directory listing
        patterns::retry_quick_operation(|| {
            let connection = self.connection.clone();
            let remote_path = remote_path.to_string();
            Box::pin(async move {
                let conn = connection.lock().await;
                match conn.as_ref() {
                    Some(connection) => {
                        let session = connection.get_session()?;
                        let sftp = super::sftp::SFTPOperations::new(session);
                        sftp.list_directory(&remote_path)
                    }
                    None => Err(anyhow::anyhow!("No SSH connection available"))
                }
            })
        }).await
    }

    /// Create a directory using native SFTP
    pub async fn create_directory(&self, remote_path: &str) -> Result<()> {
        // Use retry logic for directory creation
        patterns::retry_file_operation(|| {
            let connection = self.connection.clone();
            let remote_path = remote_path.to_string();
            Box::pin(async move {
                let conn = connection.lock().await;
                match conn.as_ref() {
                    Some(connection) => {
                        let session = connection.get_session()?;
                        let sftp = super::sftp::SFTPOperations::new(session);
                        sftp.create_directory_recursive(&remote_path, 0o755)
                    }
                    None => Err(anyhow::anyhow!("No SSH connection available"))
                }
            })
        }).await
    }

    /// Get file information using native SFTP
    pub async fn get_file_info(&self, remote_path: &str) -> Result<RemoteFileInfo> {
        // Use retry logic for file info retrieval
        patterns::retry_quick_operation(|| {
            let connection = self.connection.clone();
            let remote_path = remote_path.to_string();
            Box::pin(async move {
                let conn = connection.lock().await;
                match conn.as_ref() {
                    Some(connection) => {
                        let session = connection.get_session()?;
                        let sftp = super::sftp::SFTPOperations::new(session);
                        sftp.stat(&remote_path)
                    }
                    None => Err(anyhow::anyhow!("No SSH connection available"))
                }
            })
        }).await
    }

    /// Check if a file or directory exists
    pub async fn file_exists(&self, remote_path: &str) -> Result<bool> {
        // Use retry logic for existence checking
        patterns::retry_quick_operation(|| {
            let connection = self.connection.clone();
            let remote_path = remote_path.to_string();
            Box::pin(async move {
                let conn = connection.lock().await;
                match conn.as_ref() {
                    Some(connection) => {
                        let session = connection.get_session()?;
                        let sftp = super::sftp::SFTPOperations::new(session);
                        // Try to stat the file - if it succeeds, the file exists
                        match sftp.stat(&remote_path) {
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
                        }
                    }
                    None => Err(anyhow::anyhow!("No SSH connection available"))
                }
            })
        }).await
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
        assert!(result.unwrap_err().to_string().contains("No SSH connection"));
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
        assert!(upload_result.unwrap_err().to_string().contains("No SSH connection"));

        // Test download without connection
        let download_result = manager.download_file("/remote/file.txt", "/local/file.txt").await;
        assert!(download_result.is_err());
        assert!(download_result.unwrap_err().to_string().contains("No SSH connection"));

        // Test list files without connection
        let list_result = manager.list_files("/home/user").await;
        assert!(list_result.is_err());
        assert!(list_result.unwrap_err().to_string().contains("No SSH connection"));

        // Test create directory without connection
        let mkdir_result = manager.create_directory("/remote/newdir").await;
        assert!(mkdir_result.is_err());
        assert!(mkdir_result.unwrap_err().to_string().contains("No SSH connection"));

        // Test get file info without connection
        let stat_result = manager.get_file_info("/remote/file.txt").await;
        assert!(stat_result.is_err());
        assert!(stat_result.unwrap_err().to_string().contains("No SSH connection"));
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