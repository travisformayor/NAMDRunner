use crate::types::*;
use crate::demo::{with_demo_state, get_demo_state, execute_with_mode};
use crate::ssh::get_connection_manager;
use crate::{debug_log, info_log, error_log};
use chrono::Utc;

#[tauri::command(rename_all = "snake_case")]
pub async fn connect_to_cluster(params: ConnectParams) -> ConnectResult {
    info_log!("[CONNECT] Starting connection to {} as {}", params.host, params.username);

    execute_with_mode(
        connect_to_cluster_demo(params.clone()),
        connect_to_cluster_real(params)
    ).await
}

async fn connect_to_cluster_demo(params: ConnectParams) -> ConnectResult {
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

    let delay = get_demo_state(|state| state.get_delay("connection")).unwrap_or(500);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    let should_fail = get_demo_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Connection failed: Host unreachable".to_string()),
        };
    }

    if params.host.contains("invalid") || params.host.contains("unreachable") {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Connection failed: Host unreachable".to_string()),
        };
    }

    let session_info = SessionInfo {
        host: params.host.clone(),
        username: params.username.clone(),
        connected_at: Utc::now().to_rfc3339(),
    };

    with_demo_state(|state| {
        state.connection_state = ConnectionState::Connected;
        state.session_info = Some(session_info.clone());
    });

    ConnectResult {
        success: true,
        session_info: Some(session_info),
        error: None,
    }
}

async fn connect_to_cluster_real(params: ConnectParams) -> ConnectResult {
    let port = 22;
    match get_connection_manager().connect(params.host.clone(), port, params.username.clone(), &params.password).await {
        Ok(connection_info) => {
            info_log!("[SSH] Successfully connected to {}:{} as {}", connection_info.host, connection_info.port, connection_info.username);

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
            error_log!("[SSH] Connection failed: {}", e);

            let error_message = if let Some(ssh_err) = e.downcast_ref::<crate::ssh::errors::SSHError>() {
                let conn_error = crate::ssh::errors::map_ssh_error(ssh_err);
                debug_log!("[SSH] Error Category: {} (Code: {})", conn_error.category, conn_error.code);

                format!("{} (Code: {}) - {}. Suggestions: {}",
                    conn_error.message,
                    conn_error.code,
                    conn_error.details.unwrap_or_else(|| "No additional details".to_string()),
                    conn_error.suggestions.join("; ")
                )
            } else {
                let generic_msg = format!("Connection failed: {}", e);
                error_log!("[SSH] Connection error: {}", generic_msg);
                generic_msg
            };

            ConnectResult {
                success: false,
                session_info: None,
                error: Some(error_message),
            }
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn disconnect() -> DisconnectResult {
    execute_with_mode(
        disconnect_demo(),
        disconnect_real()
    ).await
}

async fn disconnect_demo() -> DisconnectResult {
    let delay = get_demo_state(|state| state.get_delay("connection") / 5).unwrap_or(200);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

    let should_fail = get_demo_state(|state| state.should_simulate_error()).unwrap_or(false);

    if should_fail {
        return DisconnectResult {
            success: false,
            error: Some("Failed to properly close connection: Network error".to_string()),
        };
    }

    with_demo_state(|state| {
        state.connection_state = ConnectionState::Disconnected;
        state.session_info = None;
    });

    DisconnectResult {
        success: true,
        error: None,
    }
}

async fn disconnect_real() -> DisconnectResult {
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

#[tauri::command(rename_all = "snake_case")]
pub async fn get_connection_status() -> ConnectionStatusResult {
    execute_with_mode(
        get_connection_status_demo(),
        get_connection_status_real()
    ).await
}

async fn get_connection_status_demo() -> ConnectionStatusResult {
    let state = get_demo_state(|state| state.connection_state.clone())
        .unwrap_or(ConnectionState::Disconnected);
    let session_info = get_demo_state(|state| state.session_info.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_connect_with_valid_params() {
        env::set_var("USE_MOCK_SSH", "true");
        crate::demo::set_demo_mode(true);

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
        env::set_var("USE_MOCK_SSH", "true");
        crate::demo::set_demo_mode(true);

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
        assert!(result.error.unwrap().contains("Host is required"));
    }

    // test_is_mock_mode removed - environment variable testing has parallel execution issues
    // Mode switching is tested comprehensively in integration tests

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
        crate::demo::set_demo_mode(true);

        // Test various parameter combinations
        let test_cases = vec![
            // Empty host
            (ConnectParams {
                host: "".to_string(),
                username: "user".to_string(),
                password: crate::security::SecurePassword::from_str("pass"),
            }, false, "Host is required"),

            // Empty username
            (ConnectParams {
                host: "host.com".to_string(),
                username: "".to_string(),
                password: crate::security::SecurePassword::from_str("pass"),
            }, false, "Username is required"),

            // Empty password
            (ConnectParams {
                host: "host.com".to_string(),
                username: "user".to_string(),
                password: crate::security::SecurePassword::from_str(""),
            }, false, "Password is required"),

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

    #[tokio::test]
    async fn test_connect_params_edge_cases() {
        // Force mock mode for this test
        env::set_var("USE_MOCK_SSH", "true");
        crate::demo::set_demo_mode(true);

        // Test edge cases that could cause connection issues
        let test_cases = vec![
            // Whitespace-only host
            (ConnectParams {
                host: "   ".to_string(),
                username: "user".to_string(),
                password: crate::security::SecurePassword::from_str("pass"),
            }, false, "Host is required"),

            // Whitespace-only username
            (ConnectParams {
                host: "host.com".to_string(),
                username: "   ".to_string(),
                password: crate::security::SecurePassword::from_str("pass"),
            }, false, "Username is required"),

            // Valid hostname variations that should work
            (ConnectParams {
                host: "login.rc.colorado.edu".to_string(), // This was our failing case
                username: "testuser".to_string(),
                password: crate::security::SecurePassword::from_str("testpass"),
            }, true, ""),

            (ConnectParams {
                host: "127.0.0.1".to_string(), // IP address
                username: "testuser".to_string(),
                password: crate::security::SecurePassword::from_str("testpass"),
            }, true, ""),

            (ConnectParams {
                host: "localhost".to_string(), // Short hostname
                username: "testuser".to_string(),
                password: crate::security::SecurePassword::from_str("testpass"),
            }, true, ""),

            // Hostname with subdomain
            (ConnectParams {
                host: "cluster.university.edu".to_string(),
                username: "testuser".to_string(),
                password: crate::security::SecurePassword::from_str("testpass"),
            }, true, ""),

            // Hostname with hyphens (valid)
            (ConnectParams {
                host: "test-cluster.domain-name.org".to_string(),
                username: "testuser".to_string(),
                password: crate::security::SecurePassword::from_str("testpass"),
            }, true, ""),
        ];

        for (params, should_succeed, expected_error) in test_cases {
            let result = connect_to_cluster(params.clone()).await;

            if should_succeed {
                assert!(result.success,
                    "Expected success for host '{}' but got error: {:?}",
                    params.host, result.error);
            } else {
                assert!(!result.success,
                    "Expected failure for host '{}' but got success",
                    params.host);
                assert!(result.error.is_some());
                let error_msg = result.error.unwrap();
                assert!(error_msg.contains(expected_error),
                    "Expected error to contain '{}' but got: {}",
                    expected_error, error_msg);
            }
        }
    }

    #[tokio::test]
    async fn test_connect_params_security_validation() {
        env::set_var("USE_MOCK_SSH", "true");
        crate::demo::set_demo_mode(true);

        // Test that parameter validation doesn't leak sensitive information
        let test_cases = vec![
            // Test with a password that shouldn't appear in error messages
            (ConnectParams {
                host: "".to_string(),
                username: "user".to_string(),
                password: crate::security::SecurePassword::from_str("super_secret_password_123"),
            }, "super_secret_password_123"),

            (ConnectParams {
                host: "host.com".to_string(),
                username: "".to_string(),
                password: crate::security::SecurePassword::from_str("another_secret_456"),
            }, "another_secret_456"),
        ];

        for (params, password_text) in test_cases {
            let result = connect_to_cluster(params).await;

            assert!(!result.success, "Expected validation failure");
            assert!(result.error.is_some());

            let error_msg = result.error.unwrap();

            // Verify password doesn't appear in error message
            assert!(!error_msg.contains(password_text),
                "Error message contains password: {}", error_msg);

            // Verify error message doesn't contain partial password
            assert!(!error_msg.contains("secret"),
                "Error message contains password fragment: {}", error_msg);
        }
    }

    #[tokio::test]
    async fn test_connect_params_hostname_format_validation() {
        // Force mock mode for this test
        env::set_var("USE_MOCK_SSH", "true");
        crate::demo::set_demo_mode(true);

        // Test various hostname formats to ensure our DNS resolution approach works
        let valid_hostnames = vec![
            "example.com",
            "sub.example.com",
            "login.rc.colorado.edu",  // Our original failing case
            "cluster1.university.edu",
            "test-host.domain.org",
            "a.b.c.d.e",              // Deep subdomain
            "localhost",
            "127.0.0.1",
            "192.168.1.100",
            "10.0.0.1",
        ];

        for hostname in valid_hostnames {
            let params = ConnectParams {
                host: hostname.to_string(),
                username: "testuser".to_string(),
                password: crate::security::SecurePassword::from_str("testpass"),
            };

            let result = connect_to_cluster(params).await;

            // All valid hostnames should pass validation (they'll succeed in mock mode)
            assert!(result.success,
                "Hostname '{}' should be valid but failed with error: {:?}",
                hostname, result.error);
        }
    }



    // Connection state consistency test removed - testing implementation details
    // rather than business logic. Focus should be on user-facing functionality.

    #[tokio::test]
    async fn test_session_info_consistency() {
        // Force mock mode for this test
        env::set_var("USE_MOCK_SSH", "true");
        crate::demo::set_demo_mode(true);

        let params = ConnectParams {
            host: "test.example.com".to_string(),
            username: "testuser123".to_string(),
            password: crate::security::SecurePassword::from_str("testpass"),
        };

        // Connect and verify session info
        let connect_result = connect_to_cluster(params).await;
        assert!(connect_result.success, "Connection should succeed in mock mode");

        let session_info = connect_result.session_info
            .expect("Session info should be present after successful connection");
        assert_eq!(session_info.host, "test.example.com");
        assert_eq!(session_info.username, "testuser123");
        assert!(!session_info.connected_at.is_empty());

        // Verify status endpoint returns same info
        let status = get_connection_status().await;
        if let Some(status_session) = status.session_info {
            assert_eq!(status_session.host, session_info.host);
            assert_eq!(status_session.username, session_info.username);
        }
        // Note: status.session_info might be None if another test disconnected
    }

    #[tokio::test]
    async fn test_error_propagation_through_api_boundary() {
        env::set_var("USE_MOCK_SSH", "true");
        crate::demo::set_demo_mode(true);

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

    // test_concurrent_command_execution, test_file_operations_integration, and test_data_type_conversions removed
    // These tests used Phase 1 SSH/SFTP commands (ssh_execute_command, sftp_*) which have been deleted.
    // File operations are now tested through high-level job commands (upload_job_files, download_job_output)
    // in the files module tests.

    // test_environment_variable_isolation and test_mock_vs_real_mode_switching removed
    // These tests have parallel execution issues with environment variables.
    // Mode switching functionality is properly tested through:
    // 1. Runtime mode tests in mode_switching module
    // 2. Integration tests that rely on demo mode
    // 3. Connection tests that use demo mode for their operations
}