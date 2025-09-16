use anyhow::Result;
use crate::ssh::get_connection_manager;
use crate::retry::patterns;

/// Utilities for working with SSH connections with retry logic and validation
///
/// This module provides high-level wrappers around SSH operations that include:
/// - Automatic connection validation before operations
/// - Built-in retry logic with exponential backoff
/// - Consistent error handling across all SSH operations
/// - Direct async/await patterns without Box::pin allocations
///
/// This is the preferred interface for SSH operations in job management.
/// Contains only the methods actually needed by the job lifecycle:
/// - Directory creation (for job setup)
/// - Directory deletion (for job cleanup)
/// - Connection validation (for safety checks)
pub struct ConnectionUtils;

impl ConnectionUtils {
    /// Check if we're connected to the cluster, returning appropriate error if not
    pub async fn ensure_connected() -> Result<()> {
        let connection_manager = get_connection_manager();
        if !connection_manager.is_connected().await {
            return Err(anyhow::anyhow!("Not connected to cluster"));
        }
        Ok(())
    }

    /// Create a directory with retry logic
    pub async fn create_directory_with_retry(path: &str) -> Result<()> {
        let path = path.to_string();
        patterns::retry_file_operation(|| {
            let path = path.clone();
            async move {
                let connection_manager = get_connection_manager();

                // Check if connected first
                if !connection_manager.is_connected().await {
                    return Err(anyhow::anyhow!("Not connected to cluster"));
                }

                connection_manager.create_directory(&path).await
            }
        }).await
    }


    /// Execute a command to delete directories with retry logic
    pub async fn delete_directory_with_retry(dir_path: &str) -> Result<crate::ssh::commands::CommandResult> {
        let dir_path = dir_path.to_string();
        patterns::retry_quick_operation(|| {
            let dir_path = dir_path.clone();
            async move {
                let connection_manager = get_connection_manager();

                // Check if connected first
                if !connection_manager.is_connected().await {
                    return Err(anyhow::anyhow!("Not connected to cluster"));
                }

                let rm_command = format!("rm -rf {}", crate::validation::shell::escape_parameter(&dir_path));
                connection_manager.execute_command(&rm_command, Some(30)).await
            }
        }).await
    }
}