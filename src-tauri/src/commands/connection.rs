use crate::types::*;
use crate::ssh::get_connection_manager;
use crate::{info_log, debug_log, error_log};

#[tauri::command(rename_all = "snake_case")]
pub async fn connect_to_cluster(params: ConnectParams) -> ConnectResult {
    info_log!("[CONNECT] Starting connection to {} as {}", params.host, params.username);
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

