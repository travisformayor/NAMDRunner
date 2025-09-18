pub mod connection;
pub mod sftp;
pub mod commands;
pub mod errors;
pub mod manager;

#[cfg(test)]
pub mod test_utils;

pub use connection::{SSHConnection, ConnectionConfig, ConnectionInfo};
pub use sftp::{SFTPOperations, FileTransferProgress, RemoteFileInfo, ProgressCallback};
pub use commands::{CommandExecutor, CommandResult, SLURMCommands};
pub use errors::{SSHError, map_ssh_error, ConnectionError};
pub use manager::ConnectionManager;

lazy_static::lazy_static! {
    /// Global connection manager for Tauri commands
    /// Provides proper lifecycle management and cleanup
    static ref CONNECTION_MANAGER: ConnectionManager = ConnectionManager::new();
}

/// Get the global connection manager instance
/// Use this to access SSH operations directly
pub fn get_connection_manager() -> &'static ConnectionManager {
    &CONNECTION_MANAGER
}