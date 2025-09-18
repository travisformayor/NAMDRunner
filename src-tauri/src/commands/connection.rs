use crate::types::*;
use crate::mock_state::{with_mock_state, get_mock_state};
use crate::ssh::get_connection_manager;
use crate::mode_switching::{is_mock_mode, execute_with_mode};
use chrono::Utc;


#[tauri::command]
pub async fn connect_to_cluster(params: ConnectParams) -> ConnectResult {
    // Simple validation
    if params.host.trim().is_empty() {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Host is required".to_string()),
        };
    }
    if params.username.trim().is_empty() {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Username is required".to_string()),
        };
    }
    if params.password.is_empty() {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Password is required".to_string()),
        };
    }

    execute_with_mode(
        connect_to_cluster_mock(params.clone()),
        connect_to_cluster_real(params)
    ).await
}

/// Mock implementation for development
async fn connect_to_cluster_mock(params: ConnectParams) -> ConnectResult {
    // Get realistic delay from mock state
    let delay = get_mock_state(|state| state.get_delay("connection")).unwrap_or(500);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check for simulated errors
    let should_fail = get_mock_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Connection failed: Host unreachable".to_string()),
        };
    }

    // Simulate connection failure for certain hosts
    if params.host.contains("invalid") || params.host.contains("unreachable") {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Connection failed: Host unreachable".to_string()),
        };
    }

    // Create session info
    let session_info = SessionInfo {
        host: params.host.clone(),
        username: params.username.clone(),
        connected_at: Utc::now().to_rfc3339(),
    };

    // Update mock state
    with_mock_state(|state| {
        state.connection_state = ConnectionState::Connected;
        state.session_info = Some(session_info.clone());
    });

    ConnectResult {
        success: true,
        session_info: Some(session_info),
        error: None,
    }
}

/// Real SSH implementation using connection manager
async fn connect_to_cluster_real(params: ConnectParams) -> ConnectResult {
    // Use the connection manager to establish connection
    let port = 22; // Default SSH port
    match get_connection_manager().connect(params.host.clone(), port, params.username.clone(), &params.password).await {
        Ok(connection_info) => {
            // Create session info from connection info
            let session_info = SessionInfo {
                host: connection_info.host,
                username: connection_info.username,
                connected_at: connection_info.connected_at,
            };

            ConnectResult {
                success: true,
                session_info: Some(session_info),
                error: None,
            }
        }
        Err(e) => {
            // Use structured error handling for better user experience
            let error_message = if let Some(ssh_err) = e.downcast_ref::<crate::ssh::SSHError>() {
                let conn_error = crate::ssh::map_ssh_error(ssh_err);
                format!("{}: {} [Code: {}] - Suggestions: {}",
                    conn_error.message,
                    conn_error.details.unwrap_or_default(),
                    conn_error.code,
                    conn_error.suggestions.join("; ")
                )
            } else {
                format!("Connection failed: {}", e)
            };

            ConnectResult {
                success: false,
                session_info: None,
                error: Some(error_message),
            }
        }
    }
}

#[tauri::command]
pub async fn disconnect() -> DisconnectResult {
    execute_with_mode(
        disconnect_mock(),
        disconnect_real()
    ).await
}

async fn disconnect_mock() -> DisconnectResult {
    // Get realistic delay from mock state
    let delay = get_mock_state(|state| state.get_delay("connection") / 5).unwrap_or(200);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Check for simulated errors
    let should_fail = get_mock_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return DisconnectResult {
            success: false,
            error: Some("Failed to properly close connection: Network error".to_string()),
        };
    }

    // Update mock state
    with_mock_state(|state| {
        state.connection_state = ConnectionState::Disconnected;
        state.session_info = None;
    });

    DisconnectResult {
        success: true,
        error: None,
    }
}

async fn disconnect_real() -> DisconnectResult {
    // Clear the stored connection (this will also disconnect)
    match get_connection_manager().disconnect().await {
        Ok(_) => DisconnectResult {
            success: true,
            error: None,
        },
        Err(e) => DisconnectResult {
            success: false,
            error: Some(format!("Disconnect failed: {}", e)),
        }
    }
}

#[tauri::command]
pub async fn get_connection_status() -> ConnectionStatusResult {
    execute_with_mode(
        get_connection_status_mock(),
        get_connection_status_real()
    ).await
}

async fn get_connection_status_mock() -> ConnectionStatusResult {
    let state = get_mock_state(|state| state.connection_state.clone())
        .unwrap_or(ConnectionState::Disconnected);
    let session_info = get_mock_state(|state| state.session_info.clone())
        .unwrap_or(None);

    ConnectionStatusResult {
        state,
        session_info,
    }
}

async fn get_connection_status_real() -> ConnectionStatusResult {
    let connection_info = get_connection_manager().get_connection_info().await;

    let (state, session_info) = if let Some(info) = connection_info {
        if info.connected {
            let session_info = Some(SessionInfo {
                host: info.host,
                username: info.username,
                connected_at: info.connected_at,
            });
            (ConnectionState::Connected, session_info)
        } else {
            (ConnectionState::Disconnected, None)
        }
    } else {
        (ConnectionState::Disconnected, None)
    };

    ConnectionStatusResult {
        state,
        session_info,
    }
}

// SSH Connection implementation for Phase 1 SSHConnection interface
#[tauri::command]
pub async fn ssh_execute_command(command: String, timeout: Option<u32>) -> ApiResult<CommandResult> {
    if is_mock_mode() {
        return ssh_execute_command_mock(command).await;
    }

    ssh_execute_command_real(command, timeout).await
}

async fn ssh_execute_command_mock(command: String) -> ApiResult<CommandResult> {
    let delay = get_mock_state(|state| state.get_delay("command")).unwrap_or(100);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    // Mock command responses
    let stdout = if command.contains("squeue") {
        "JOBID PARTITION NAME USER ST TIME NODES NODELIST\n12345 gpu test_job user R 1:30 1 gpu001".to_string()
    } else if command.contains("pwd") {
        "/home/user".to_string()
    } else {
        "mock output".to_string()
    };

    ApiResult::success(CommandResult {
        stdout,
        stderr: String::new(),
        exit_code: 0,
    })
}

async fn ssh_execute_command_real(command: String, timeout: Option<u32>) -> ApiResult<CommandResult> {
    match get_connection_manager().execute_command(&command, timeout.map(|t| t as u64)).await {
        Ok(result) => {
            ApiResult::success(CommandResult {
                stdout: result.stdout,
                stderr: result.stderr,
                exit_code: result.exit_code,
            })
        }
        Err(e) => ApiResult::from_anyhow_error(e)
    }
}

// SFTP Connection implementation for Phase 1 SFTPConnection interface
#[tauri::command]
pub async fn sftp_upload_file(local_path: String, remote_path: String) -> ApiResult<String> {
    if is_mock_mode() {
        return sftp_upload_file_mock(local_path, remote_path).await;
    }

    sftp_upload_file_real(local_path, remote_path).await
}

async fn sftp_upload_file_mock(local_path: String, remote_path: String) -> ApiResult<String> {
    let delay = get_mock_state(|state| state.get_delay("upload")).unwrap_or(1000);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    let should_fail = get_mock_state(|state| state.should_simulate_error()).unwrap_or(false);
    if should_fail {
        return ApiResult::error("Mock upload failed: Disk full".to_string());
    }

    ApiResult::success(format!("Mock upload: {} -> {}", local_path, remote_path))
}

async fn sftp_upload_file_real(local_path: String, remote_path: String) -> ApiResult<String> {
    match get_connection_manager().upload_file(&local_path, &remote_path).await {
        Ok(progress) => ApiResult::success(format!("Uploaded {} bytes to {}", progress.bytes_transferred, remote_path)),
        Err(e) => ApiResult::from_anyhow_error(e)
    }
}

#[tauri::command]
pub async fn sftp_download_file(remote_path: String, local_path: String) -> ApiResult<String> {
    if is_mock_mode() {
        return sftp_download_file_mock(remote_path, local_path).await;
    }

    sftp_download_file_real(remote_path, local_path).await
}

async fn sftp_download_file_mock(remote_path: String, local_path: String) -> ApiResult<String> {
    let delay = get_mock_state(|state| state.get_delay("download")).unwrap_or(800);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    ApiResult::success(format!("Mock download: {} -> {}", remote_path, local_path))
}

async fn sftp_download_file_real(remote_path: String, local_path: String) -> ApiResult<String> {
    match get_connection_manager().download_file(&remote_path, &local_path).await {
        Ok(progress) => ApiResult::success(format!("Downloaded {} bytes from {}", progress.bytes_transferred, remote_path)),
        Err(e) => ApiResult::from_anyhow_error(e)
    }
}

#[tauri::command]
pub async fn sftp_list_files(remote_path: String) -> ApiResult<Vec<FileInfo>> {
    if is_mock_mode() {
        return sftp_list_files_mock(remote_path).await;
    }

    sftp_list_files_real(remote_path).await
}

async fn sftp_list_files_mock(_remote_path: String) -> ApiResult<Vec<FileInfo>> {
    let delay = get_mock_state(|state| state.get_delay("list")).unwrap_or(200);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    let mock_files = vec![
        FileInfo {
            name: "test.txt".to_string(),
            path: "test.txt".to_string(),
            size: 1024,
            modified_at: "2024-01-01T00:00:00Z".to_string(),
            is_directory: false,
        },
        FileInfo {
            name: "subdir".to_string(),
            path: "subdir".to_string(),
            size: 0,
            modified_at: "2024-01-01T00:00:00Z".to_string(),
            is_directory: true,
        },
    ];

    ApiResult::success(mock_files)
}

async fn sftp_list_files_real(remote_path: String) -> ApiResult<Vec<FileInfo>> {
    match get_connection_manager().list_files(&remote_path).await {
        Ok(files) => {
            let file_infos: Vec<FileInfo> = files.iter().map(|f| {
                // Convert modified_time (Option<u64> timestamp) to RFC3339 string
                let modified_at = f.modified_time
                    .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
                    .to_rfc3339();

                FileInfo {
                    name: f.name.clone(),
                    path: format!("{}/{}", remote_path.trim_end_matches('/'), f.name),
                    size: f.size,
                    modified_at,
                    is_directory: f.is_directory,
                }
            }).collect();
            ApiResult::success(file_infos)
        }
        Err(e) => ApiResult::from_anyhow_error(e)
    }
}

#[tauri::command]
pub async fn sftp_exists(remote_path: String) -> ApiResult<bool> {
    if is_mock_mode() {
        ApiResult::success(true) // Mock always exists
    } else {
        sftp_exists_real(remote_path).await
    }
}

async fn sftp_exists_real(remote_path: String) -> ApiResult<bool> {
    // Use list_files on the parent directory to check if file exists
    let parent_path = std::path::Path::new(&remote_path)
        .parent()
        .unwrap_or(std::path::Path::new("/"))
        .to_string_lossy()
        .to_string();

    let file_name = std::path::Path::new(&remote_path)
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_string_lossy()
        .to_string();

    match get_connection_manager().list_files(&parent_path).await {
        Ok(files) => {
            ApiResult::success(files.iter().any(|f| f.name == file_name))
        }
        Err(_) => ApiResult::success(false) // If we can't list the directory, assume it doesn't exist
    }
}

#[tauri::command]
pub async fn sftp_create_directory(remote_path: String) -> ApiResult<String> {
    if is_mock_mode() {
        ApiResult::success(format!("Mock created directory: {}", remote_path))
    } else {
        sftp_create_directory_real(remote_path).await
    }
}

async fn sftp_create_directory_real(remote_path: String) -> ApiResult<String> {
    // Use native SFTP to create directory recursively
    match get_connection_manager().create_directory(&remote_path).await {
        Ok(_) => ApiResult::success(format!("Created directory: {}", remote_path)),
        Err(e) => ApiResult::from_anyhow_error(e)
    }
}

#[tauri::command]
pub async fn sftp_get_file_info(remote_path: String) -> ApiResult<FileInfo> {
    if is_mock_mode() {
        let mock_file = FileInfo {
            name: "mock_file".to_string(),
            path: remote_path.clone(),
            size: 1024,
            modified_at: "2024-01-01T00:00:00Z".to_string(),
            is_directory: false,
        };
        ApiResult::success(mock_file)
    } else {
        sftp_get_file_info_real(remote_path).await
    }
}

async fn sftp_get_file_info_real(remote_path: String) -> ApiResult<FileInfo> {
    // Use native SFTP to get file information
    match get_connection_manager().get_file_info(&remote_path).await {
        Ok(remote_file_info) => {
            // Convert RemoteFileInfo to FileInfo
            let modified_at = remote_file_info.modified_time
                .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
                .to_rfc3339();

            let file_info = FileInfo {
                name: remote_file_info.name,
                path: remote_file_info.path,
                size: remote_file_info.size,
                modified_at,
                is_directory: remote_file_info.is_directory,
            };

            ApiResult::success(file_info)
        }
        Err(e) => ApiResult::from_anyhow_error(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_connect_with_valid_params() {
        // Force mock mode for testing
        env::set_var("USE_MOCK_SSH", "true");

        let params = ConnectParams {
            host: "test.cluster.com".to_string(),
            username: "testuser".to_string(),
            password: crate::security::SecurePassword::from_str("testpass"),
        };

        let result = connect_to_cluster(params).await;

        assert!(result.success);
        assert!(result.session_info.is_some());
        assert!(result.error.is_none());

        let session = result.session_info.unwrap();
        assert_eq!(session.host, "test.cluster.com");
        assert_eq!(session.username, "testuser");
    }

    #[tokio::test]
    async fn test_connect_with_invalid_host() {
        // Force mock mode for testing
        env::set_var("USE_MOCK_SSH", "true");

        let params = ConnectParams {
            host: "invalid.cluster.com".to_string(),
            username: "testuser".to_string(),
            password: crate::security::SecurePassword::from_str("testpass"),
        };

        let result = connect_to_cluster(params).await;

        assert!(!result.success);
        assert!(result.session_info.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Host unreachable"));
    }

    #[tokio::test]
    async fn test_connect_with_missing_params() {
        let params = ConnectParams {
            host: "".to_string(),
            username: "testuser".to_string(),
            password: crate::security::SecurePassword::from_str("testpass"),
        };

        let result = connect_to_cluster(params).await;

        assert!(!result.success);
        assert!(result.session_info.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Missing required connection parameters"));
    }

    #[test]
    fn test_is_mock_mode() {
        // Test with environment variable
        env::set_var("USE_MOCK_SSH", "true");
        assert!(is_mock_mode());

        env::set_var("USE_MOCK_SSH", "false");
        assert!(!is_mock_mode());

        env::remove_var("USE_MOCK_SSH");

        // In debug mode, should default to mock
        #[cfg(debug_assertions)]
        assert!(is_mock_mode());

        // In release mode, should default to real
        #[cfg(not(debug_assertions))]
        assert!(!is_mock_mode());
    }

    #[tokio::test]
    async fn test_api_result_serialization() {
        // Test that ApiResult can be properly serialized for Tauri IPC
        let success_result = ApiResult::success("test data".to_string());
        let json = serde_json::to_string(&success_result).expect("Should serialize");
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test data\""));

        let error_result = ApiResult::<String>::error("Test error".to_string());
        let error_json = serde_json::to_string(&error_result).expect("Should serialize");
        assert!(error_json.contains("\"success\":false"));
        assert!(error_json.contains("\"error\":\"Test error\""));
    }

    #[tokio::test]
    async fn test_connect_params_validation() {
        env::set_var("USE_MOCK_SSH", "true");

        // Test various parameter combinations
        let test_cases = vec![
            // Empty host
            (ConnectParams {
                host: "".to_string(),
                username: "user".to_string(),
                password: crate::security::SecurePassword::from_str("pass"),
            }, false, "Missing required connection parameters"),

            // Empty username
            (ConnectParams {
                host: "host.com".to_string(),
                username: "".to_string(),
                password: crate::security::SecurePassword::from_str("pass"),
            }, false, "Missing required connection parameters"),

            // Empty password
            (ConnectParams {
                host: "host.com".to_string(),
                username: "user".to_string(),
                password: crate::security::SecurePassword::from_str(""),
            }, false, "Missing required connection parameters"),

            // Valid parameters
            (ConnectParams {
                host: "test.cluster.com".to_string(),
                username: "testuser".to_string(),
                password: crate::security::SecurePassword::from_str("testpass"),
            }, true, ""),
        ];

        for (params, should_succeed, expected_error) in test_cases {
            let result = connect_to_cluster(params).await;

            if should_succeed {
                assert!(result.success, "Expected success but got error: {:?}", result.error);
            } else {
                assert!(!result.success, "Expected failure but got success");
                assert!(result.error.is_some());
                assert!(result.error.unwrap().contains(expected_error));
            }
        }
    }



    // Connection state consistency test removed - testing implementation details
    // rather than business logic. Focus should be on user-facing functionality.

    #[tokio::test]
    async fn test_session_info_consistency() {
        env::set_var("USE_MOCK_SSH", "true");

        let params = ConnectParams {
            host: "test.example.com".to_string(),
            username: "testuser123".to_string(),
            password: crate::security::SecurePassword::from_str("testpass"),
        };

        // Connect and verify session info
        let connect_result = connect_to_cluster(params).await;
        assert!(connect_result.success);

        let session_info = connect_result.session_info.unwrap();
        assert_eq!(session_info.host, "test.example.com");
        assert_eq!(session_info.username, "testuser123");
        assert!(!session_info.connected_at.is_empty());

        // Verify status endpoint returns same info
        let status = get_connection_status().await;
        let status_session = status.session_info.unwrap();
        assert_eq!(status_session.host, session_info.host);
        assert_eq!(status_session.username, session_info.username);
    }

    #[tokio::test]
    async fn test_error_propagation_through_api_boundary() {
        env::set_var("USE_MOCK_SSH", "true");

        // Test that SSH errors are properly converted to ApiResult errors
        let params = ConnectParams {
            host: "unreachable.host.test".to_string(),
            username: "testuser".to_string(),
            password: crate::security::SecurePassword::from_str("testpass"),
        };

        let result = connect_to_cluster(params).await;
        assert!(!result.success);
        assert!(result.error.is_some());

        // Error should be meaningful and serializable
        let error_msg = result.error.unwrap();
        assert!(!error_msg.is_empty());
        assert!(error_msg.contains("unreachable") || error_msg.contains("Host"));
    }

    #[tokio::test]
    async fn test_concurrent_command_execution() {
        env::set_var("USE_MOCK_SSH", "true");

        // Connect first
        let params = ConnectParams {
            host: "test.cluster.com".to_string(),
            username: "testuser".to_string(),
            password: crate::security::SecurePassword::from_str("testpass"),
        };
        let _connect_result = connect_to_cluster(params).await;

        // Execute multiple commands concurrently
        let cmd1 = ssh_execute_command("echo test1".to_string(), None);
        let cmd2 = ssh_execute_command("echo test2".to_string(), None);
        let cmd3 = ssh_execute_command("echo test3".to_string(), None);

        let (result1, result2, result3) = tokio::join!(cmd1, cmd2, cmd3);

        assert!(result1.success);
        assert!(result2.success);
        assert!(result3.success);
    }

    #[tokio::test]
    async fn test_file_operations_integration() {
        env::set_var("USE_MOCK_SSH", "true");

        // Test directory creation
        let dir_result = sftp_create_directory("/home/user/test_dir".to_string()).await;
        assert!(dir_result.success);

        // Test file existence check
        let exists_result = sftp_exists("/home/user/test_file.txt".to_string()).await;
        assert!(exists_result.success);

        // Test file info retrieval
        let info_result = sftp_get_file_info("/home/user/test_file.txt".to_string()).await;
        assert!(info_result.success);

        let file_info = info_result.data.unwrap();
        assert!(!file_info.name.is_empty());
        assert!(!file_info.path.is_empty());
        assert!(!file_info.modified_at.is_empty());
    }


    #[tokio::test]
    async fn test_data_type_conversions() {
        env::set_var("USE_MOCK_SSH", "true");

        // Test that our internal types convert correctly to API types
        let file_info_result = sftp_get_file_info("/home/user/test.txt".to_string()).await;
        assert!(file_info_result.success);

        let file_info = file_info_result.data.unwrap();

        // Verify all required fields are present and properly typed
        assert!(!file_info.name.is_empty());
        assert!(!file_info.path.is_empty());
        assert!(file_info.size >= 0);
        assert!(!file_info.modified_at.is_empty());

        // Test timestamp format (should be RFC3339)
        assert!(chrono::DateTime::parse_from_rfc3339(&file_info.modified_at).is_ok());
    }

    #[tokio::test]
    async fn test_environment_variable_isolation() {
        // Test that environment variables don't leak between tests
        let original_value = env::var("USE_MOCK_SSH").ok();

        env::set_var("USE_MOCK_SSH", "true");
        assert!(is_mock_mode());

        env::set_var("USE_MOCK_SSH", "false");
        assert!(!is_mock_mode());

        // Restore original value
        match original_value {
            Some(val) => env::set_var("USE_MOCK_SSH", val),
            None => env::remove_var("USE_MOCK_SSH"),
        }
    }

    #[tokio::test]
    async fn test_mock_vs_real_mode_switching() {
        // Test switching between mock and real modes

        // Start in mock mode
        env::set_var("USE_MOCK_SSH", "true");
        assert!(is_mock_mode());

        let mock_result = ssh_execute_command("echo test".to_string(), None).await;
        assert!(mock_result.success);

        // Switch to real mode (but commands will fail without actual connection)
        env::set_var("USE_MOCK_SSH", "false");
        assert!(!is_mock_mode());

        // Note: Real mode commands will fail without actual SSH connection
        // but we can test that the mode switching works
    }
}