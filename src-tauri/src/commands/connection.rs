use crate::types::*;
use crate::types::response_data::ConnectionStatus;
use crate::ssh::get_connection_manager;
use crate::{log_info, log_debug, log_error};

#[tauri::command(rename_all = "snake_case")]
pub async fn connect_to_cluster(params: ConnectParams) -> ApiResult<SessionInfo> {
    log_info!(category: "Connection", message: "Starting connection", details: "Host: {}, User: {}", params.host, params.username);
    let port = 22;

    match get_connection_manager().connect(params.host.clone(), port, params.username.clone(), &params.password).await {
        Ok(connection_info) => {
            log_info!(category: "Connection", message: "Successfully connected to cluster", show_toast: true);

            let session_info = SessionInfo {
                host: connection_info.host,
                username: connection_info.username,
                connected_at: connection_info.connected_at,
            };

            ApiResult::success(session_info)
        }
        Err(e) => {
            log_error!(category: "Connection", message: "Connection failed", details: "Error: {}", e);

            let error_message = if let Some(ssh_err) = e.downcast_ref::<crate::ssh::errors::SSHError>() {
                let conn_error = crate::ssh::errors::map_ssh_error(ssh_err);
                log_debug!(category: "Connection", message: "Error details", details: "Category: {} (Code: {})", conn_error.category, conn_error.code);

                format!("{} (Code: {}) - {}. Suggestions: {}",
                    conn_error.message,
                    conn_error.code,
                    conn_error.details.unwrap_or_else(|| "No additional details".to_string()),
                    conn_error.suggestions.join("; ")
                )
            } else {
                let generic_msg = format!("Connection failed: {}", e);
                log_error!(category: "Connection", message: "Connection error", details: "{}", generic_msg);
                generic_msg
            };

            ApiResult::error(error_message)
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn disconnect() -> ApiResult<()> {
    match get_connection_manager().disconnect().await {
        Ok(_) => ApiResult::success(()),
        Err(e) => ApiResult::error(format!("Disconnect failed: {}", e)),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_connection_status() -> ApiResult<ConnectionStatus> {
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

    let status = ConnectionStatus {
        state,
        session_info,
    };

    ApiResult::success(status)
}

