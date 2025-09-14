use crate::types::*;
use crate::mock_state::{with_mock_state, get_mock_state};
use chrono::Utc;

#[tauri::command]
pub async fn connect_to_cluster(params: ConnectParams) -> ConnectResult {
    // Enhanced mock implementation with realistic behavior
    // In Phase 2, this will use ssh2 crate for real SSH connection
    
    // Validate parameters
    if params.host.is_empty() || params.username.is_empty() || params.password.is_empty() {
        return ConnectResult {
            success: false,
            session_info: None,
            error: Some("Missing required connection parameters".to_string()),
        };
    }
    
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
    if params.host.contains("invalid") {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_with_valid_params() {
        let params = ConnectParams {
            host: "test.cluster.com".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
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
        let params = ConnectParams {
            host: "invalid.cluster.com".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
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
            password: "testpass".to_string(),
        };

        let result = connect_to_cluster(params).await;

        assert!(!result.success);
        assert!(result.session_info.is_none());
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Missing required connection parameters"));
    }
}

#[tauri::command] 
pub async fn disconnect() -> DisconnectResult {
    // Enhanced mock implementation - simulate realistic disconnection
    
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

#[tauri::command]
pub async fn get_connection_status() -> ConnectionStatusResult {
    // Return current connection state from enhanced mock state manager
    
    get_mock_state(|state| {
        ConnectionStatusResult {
            state: state.connection_state.clone(),
            session_info: state.session_info.clone(),
        }
    }).unwrap_or(ConnectionStatusResult {
        state: ConnectionState::Disconnected,
        session_info: None,
    })
}