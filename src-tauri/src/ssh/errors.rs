use std::fmt;
use std::error::Error;

/// SSH-specific error types
#[derive(Debug, Clone)]
pub enum SSHError {
    /// Network connectivity errors
    NetworkError(String),
    /// Authentication failures
    AuthenticationError(String),
    /// SSH handshake failures
    HandshakeError(String),
    /// Command execution errors
    CommandError(String),
    /// File transfer errors
    FileTransferError(String),
    /// Timeout errors
    TimeoutError(String),
    /// Permission errors
    PermissionError(String),
    /// Configuration errors
    ConfigurationError(String),
    /// Session errors
    SessionError(String),
    /// Unknown errors
    UnknownError(String),
}

impl fmt::Display for SSHError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SSHError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SSHError::AuthenticationError(msg) => write!(f, "Authentication failed: {}", msg),
            SSHError::HandshakeError(msg) => write!(f, "SSH handshake failed: {}", msg),
            SSHError::CommandError(msg) => write!(f, "Command execution failed: {}", msg),
            SSHError::FileTransferError(msg) => write!(f, "File transfer failed: {}", msg),
            SSHError::TimeoutError(msg) => write!(f, "Operation timed out: {}", msg),
            SSHError::PermissionError(msg) => write!(f, "Permission denied: {}", msg),
            SSHError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            SSHError::SessionError(msg) => write!(f, "Session error: {}", msg),
            SSHError::UnknownError(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl Error for SSHError {}

/// Error mapping for frontend compatibility
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionError {
    pub category: String,
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
    pub suggestions: Vec<String>,
}

/// Map SSH errors to frontend-compatible error format
pub fn map_ssh_error(error: &SSHError) -> ConnectionError {
    match error {
        SSHError::NetworkError(msg) => ConnectionError {
            category: "Network".to_string(),
            code: "NET_001".to_string(),
            message: "Cannot reach cluster host".to_string(),
            details: Some(msg.clone()),
            retryable: true,
            suggestions: vec![
                "Check your network connection".to_string(),
                "Verify the cluster hostname is correct".to_string(),
                "Check if you need to be on VPN".to_string(),
                "Try again in a moment".to_string(),
            ],
        },
        SSHError::AuthenticationError(msg) => ConnectionError {
            category: "Authentication".to_string(),
            code: "AUTH_001".to_string(),
            message: "Authentication failed".to_string(),
            details: Some(msg.clone()),
            retryable: false,
            suggestions: vec![
                "Check your username and password".to_string(),
                "Verify your account is active".to_string(),
                "Check if your password has expired".to_string(),
                "Contact system administrator if you cannot log in".to_string(),
            ],
        },
        SSHError::HandshakeError(msg) => ConnectionError {
            category: "Network".to_string(),
            code: "NET_003".to_string(),
            message: "Connection refused by server".to_string(),
            details: Some(msg.clone()),
            retryable: true,
            suggestions: vec![
                "Verify the cluster is accepting SSH connections".to_string(),
                "Check if SSH service is running on the cluster".to_string(),
                "Verify the port number (default is 22)".to_string(),
                "Contact system administrator if issue persists".to_string(),
            ],
        },
        SSHError::CommandError(msg) => ConnectionError {
            category: "Validation".to_string(),
            code: "VAL_001".to_string(),
            message: "Command execution failed".to_string(),
            details: Some(msg.clone()),
            retryable: false,
            suggestions: vec![
                "Check the command syntax".to_string(),
                "Verify you have permission to run this command".to_string(),
                "Check if required modules are loaded".to_string(),
            ],
        },
        SSHError::FileTransferError(msg) => ConnectionError {
            category: "FileOperation".to_string(),
            code: "FILE_002".to_string(),
            message: "File transfer failed".to_string(),
            details: Some(msg.clone()),
            retryable: true,
            suggestions: vec![
                "Check available disk space on cluster".to_string(),
                "Verify network connection stability".to_string(),
                "Check file permissions".to_string(),
                "Try transferring smaller files".to_string(),
            ],
        },
        SSHError::TimeoutError(msg) => ConnectionError {
            category: "Timeout".to_string(),
            code: "NET_002".to_string(),
            message: "Connection timed out".to_string(),
            details: Some(msg.clone()),
            retryable: true,
            suggestions: vec![
                "Check your network speed".to_string(),
                "The cluster may be under heavy load".to_string(),
                "Try increasing the timeout setting".to_string(),
                "Try again in a few moments".to_string(),
            ],
        },
        SSHError::PermissionError(msg) => ConnectionError {
            category: "Permission".to_string(),
            code: "PERM_001".to_string(),
            message: "Permission denied".to_string(),
            details: Some(msg.clone()),
            retryable: false,
            suggestions: vec![
                "Check you have access to this resource".to_string(),
                "Verify your user permissions".to_string(),
                "Contact system administrator for access".to_string(),
            ],
        },
        SSHError::ConfigurationError(msg) => ConnectionError {
            category: "Configuration".to_string(),
            code: "CFG_001".to_string(),
            message: "Invalid configuration".to_string(),
            details: Some(msg.clone()),
            retryable: false,
            suggestions: vec![
                "Check your connection settings".to_string(),
                "Verify all required fields are filled".to_string(),
                "Review the configuration for errors".to_string(),
            ],
        },
        SSHError::SessionError(msg) => ConnectionError {
            category: "Authentication".to_string(),
            code: "AUTH_002".to_string(),
            message: "Session has expired".to_string(),
            details: Some(msg.clone()),
            retryable: true,
            suggestions: vec![
                "Your session has timed out".to_string(),
                "Please reconnect to continue".to_string(),
                "This is normal after extended inactivity".to_string(),
            ],
        },
        SSHError::UnknownError(msg) => ConnectionError {
            category: "Unknown".to_string(),
            code: "UNK_001".to_string(),
            message: "An unexpected error occurred".to_string(),
            details: Some(msg.clone()),
            retryable: true,
            suggestions: vec![
                "Try the operation again".to_string(),
                "Check the logs for more details".to_string(),
                "Contact support if the issue persists".to_string(),
            ],
        },
    }
}

/// Convert from standard errors to SSH errors
impl From<std::io::Error> for SSHError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::TimedOut => {
                SSHError::TimeoutError(error.to_string())
            }
            std::io::ErrorKind::PermissionDenied => {
                SSHError::PermissionError(error.to_string())
            }
            std::io::ErrorKind::ConnectionRefused |
            std::io::ErrorKind::ConnectionAborted |
            std::io::ErrorKind::ConnectionReset => {
                SSHError::NetworkError(error.to_string())
            }
            _ => SSHError::UnknownError(error.to_string())
        }
    }
}

/// Convert from ssh2 errors to SSH errors
impl From<ssh2::Error> for SSHError {
    fn from(error: ssh2::Error) -> Self {
        // Parse SSH2 error messages to determine error type
        let error_msg = error.to_string();

        if error_msg.contains("authentication") || error_msg.contains("publickey") {
            SSHError::AuthenticationError(error_msg)
        } else if error_msg.contains("timeout") {
            SSHError::TimeoutError(error_msg)
        } else if error_msg.contains("permission") || error_msg.contains("denied") {
            SSHError::PermissionError(error_msg)
        } else if error_msg.contains("handshake") {
            SSHError::HandshakeError(error_msg)
        } else if error_msg.contains("session") {
            SSHError::SessionError(error_msg)
        } else {
            SSHError::UnknownError(error_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::test_utils::MockErrorBuilder;

    #[test]
    fn test_ssh_error_display() {
        let error = SSHError::NetworkError("Connection failed".to_string());
        assert_eq!(error.to_string(), "Network error: Connection failed");

        let error = SSHError::AuthenticationError("Invalid password".to_string());
        assert_eq!(error.to_string(), "Authentication failed: Invalid password");
    }

    #[test]
    fn test_error_mapping() {
        let ssh_error = SSHError::NetworkError("Host unreachable".to_string());
        let conn_error = map_ssh_error(&ssh_error);

        assert_eq!(conn_error.category, "Network");
        assert_eq!(conn_error.code, "NET_001");
        assert!(conn_error.retryable);
        assert!(!conn_error.suggestions.is_empty());
    }

    #[test]
    fn test_auth_error_mapping() {
        let ssh_error = SSHError::AuthenticationError("Wrong password".to_string());
        let conn_error = map_ssh_error(&ssh_error);

        assert_eq!(conn_error.category, "Authentication");
        assert_eq!(conn_error.code, "AUTH_001");
        assert!(!conn_error.retryable);
        assert!(!conn_error.suggestions.is_empty());
    }

    #[test]
    fn test_all_error_type_display_formats() {
        // Test display formatting for all error types
        let test_cases = vec![
            (SSHError::NetworkError("test".to_string()), "Network error: test"),
            (SSHError::AuthenticationError("test".to_string()), "Authentication failed: test"),
            (SSHError::HandshakeError("test".to_string()), "SSH handshake failed: test"),
            (SSHError::CommandError("test".to_string()), "Command execution failed: test"),
            (SSHError::FileTransferError("test".to_string()), "File transfer failed: test"),
            (SSHError::TimeoutError("test".to_string()), "Operation timed out: test"),
            (SSHError::PermissionError("test".to_string()), "Permission denied: test"),
            (SSHError::ConfigurationError("test".to_string()), "Configuration error: test"),
            (SSHError::SessionError("test".to_string()), "Session error: test"),
            (SSHError::UnknownError("test".to_string()), "Unknown error: test"),
        ];

        for (error, expected) in test_cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_comprehensive_error_mapping() {
        // Test all error types get mapped correctly
        let error_mappings = vec![
            (SSHError::NetworkError("host down".to_string()), "Network", "NET_001", true),
            (SSHError::AuthenticationError("bad auth".to_string()), "Authentication", "AUTH_001", false),
            (SSHError::HandshakeError("handshake fail".to_string()), "Network", "NET_003", true),
            (SSHError::CommandError("cmd fail".to_string()), "Validation", "VAL_001", false),
            (SSHError::FileTransferError("transfer fail".to_string()), "FileOperation", "FILE_002", true),
            (SSHError::TimeoutError("timeout".to_string()), "Timeout", "NET_002", true),
            (SSHError::PermissionError("no access".to_string()), "Permission", "PERM_001", false),
            (SSHError::ConfigurationError("bad config".to_string()), "Configuration", "CFG_001", false),
            (SSHError::SessionError("session dead".to_string()), "Authentication", "AUTH_002", true),
            (SSHError::UnknownError("mystery".to_string()), "Unknown", "UNK_001", true),
        ];

        for (ssh_error, expected_category, expected_code, expected_retryable) in error_mappings {
            let conn_error = map_ssh_error(&ssh_error);

            assert_eq!(conn_error.category, expected_category);
            assert_eq!(conn_error.code, expected_code);
            assert_eq!(conn_error.retryable, expected_retryable);
            assert!(!conn_error.suggestions.is_empty());
            assert!(conn_error.details.is_some());
        }
    }

    #[test]
    fn test_error_suggestion_quality() {
        // Test that error suggestions are meaningful
        let network_error = SSHError::NetworkError("Connection refused".to_string());
        let network_mapped = map_ssh_error(&network_error);

        assert!(network_mapped.suggestions.iter().any(|s| s.contains("network")));
        assert!(network_mapped.suggestions.iter().any(|s| s.contains("connection")));

        let auth_error = SSHError::AuthenticationError("Login failed".to_string());
        let auth_mapped = map_ssh_error(&auth_error);

        assert!(auth_mapped.suggestions.iter().any(|s| s.contains("password") || s.contains("username")));
        assert!(auth_mapped.suggestions.iter().any(|s| s.contains("account")));

        let timeout_error = SSHError::TimeoutError("Operation timed out".to_string());
        let timeout_mapped = map_ssh_error(&timeout_error);

        assert!(timeout_mapped.suggestions.iter().any(|s| s.contains("timeout") || s.contains("slow")));
    }

    #[test]
    fn test_retryable_vs_non_retryable_logic() {
        // Retryable errors
        let retryable_errors = vec![
            SSHError::NetworkError("temp fail".to_string()),
            SSHError::TimeoutError("slow connection".to_string()),
            SSHError::FileTransferError("network hiccup".to_string()),
            SSHError::SessionError("session expired".to_string()),
            SSHError::UnknownError("mystery".to_string()),
            SSHError::HandshakeError("handshake timeout".to_string()),
        ];

        for error in retryable_errors {
            let mapped = map_ssh_error(&error);
            assert!(mapped.retryable, "Error should be retryable: {:?}", error);
        }

        // Non-retryable errors
        let non_retryable_errors = vec![
            SSHError::AuthenticationError("bad password".to_string()),
            SSHError::PermissionError("access denied".to_string()),
            SSHError::ConfigurationError("invalid config".to_string()),
            SSHError::CommandError("command not found".to_string()),
        ];

        for error in non_retryable_errors {
            let mapped = map_ssh_error(&error);
            assert!(!mapped.retryable, "Error should not be retryable: {:?}", error);
        }
    }

    #[test]
    fn test_io_error_conversion() {
        // Test conversion from std::io::Error to SSHError
        let timeout_io = std::io::Error::from(std::io::ErrorKind::TimedOut);
        let ssh_timeout: SSHError = timeout_io.into();
        if let SSHError::TimeoutError(_) = ssh_timeout {
            // Expected
        } else {
            panic!("Expected TimeoutError, got: {:?}", ssh_timeout);
        }

        let permission_io = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        let ssh_permission: SSHError = permission_io.into();
        if let SSHError::PermissionError(_) = ssh_permission {
            // Expected
        } else {
            panic!("Expected PermissionError, got: {:?}", ssh_permission);
        }

        let connection_io = std::io::Error::from(std::io::ErrorKind::ConnectionRefused);
        let ssh_network: SSHError = connection_io.into();
        if let SSHError::NetworkError(_) = ssh_network {
            // Expected
        } else {
            panic!("Expected NetworkError, got: {:?}", ssh_network);
        }
    }

    #[test]
    fn test_mock_error_builder() {
        // Test our mock error builder utility
        let network_err = MockErrorBuilder::network_error("Connection lost");
        assert!(matches!(network_err, SSHError::NetworkError(_)));
        assert_eq!(network_err.to_string(), "Network error: Connection lost");

        let auth_err = MockErrorBuilder::auth_error("Invalid credentials");
        assert!(matches!(auth_err, SSHError::AuthenticationError(_)));

        let timeout_err = MockErrorBuilder::timeout_error("Request timed out");
        assert!(matches!(timeout_err, SSHError::TimeoutError(_)));

        let perm_err = MockErrorBuilder::permission_error("Access denied");
        assert!(matches!(perm_err, SSHError::PermissionError(_)));

        let transfer_err = MockErrorBuilder::file_transfer_error("Upload failed");
        assert!(matches!(transfer_err, SSHError::FileTransferError(_)));
    }

    #[test]
    fn test_error_clone_functionality() {
        // Test that all errors can be cloned (needed for ApiResult)
        let original = SSHError::NetworkError("test error".to_string());
        let cloned = original.clone();

        assert_eq!(original.to_string(), cloned.to_string());

        // Test with all error types
        let errors = vec![
            SSHError::NetworkError("net".to_string()),
            SSHError::AuthenticationError("auth".to_string()),
            SSHError::HandshakeError("handshake".to_string()),
            SSHError::CommandError("cmd".to_string()),
            SSHError::FileTransferError("file".to_string()),
            SSHError::TimeoutError("timeout".to_string()),
            SSHError::PermissionError("perm".to_string()),
            SSHError::ConfigurationError("config".to_string()),
            SSHError::SessionError("session".to_string()),
            SSHError::UnknownError("unknown".to_string()),
        ];

        for error in errors {
            let cloned = error.clone();
            assert_eq!(error.to_string(), cloned.to_string());
        }
    }

    #[test]
    fn test_connection_error_serialization() {
        // Test that ConnectionError can be serialized/deserialized for frontend
        let ssh_error = SSHError::NetworkError("Connection failed".to_string());
        let conn_error = map_ssh_error(&ssh_error);

        // Test serialization (for sending to frontend)
        let json = serde_json::to_string(&conn_error).expect("Should serialize");
        assert!(json.contains("Network"));
        assert!(json.contains("NET_001"));

        // Test deserialization (for receiving from frontend)
        let deserialized: ConnectionError = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.category, conn_error.category);
        assert_eq!(deserialized.code, conn_error.code);
        assert_eq!(deserialized.retryable, conn_error.retryable);
    }

    #[test]
    fn test_error_message_consistency() {
        // Test that error messages are consistent and helpful
        let base_message = "Connection failed";
        let error = SSHError::NetworkError(base_message.to_string());
        let mapped = map_ssh_error(&error);

        // Original message should be preserved in details
        assert_eq!(mapped.details.as_ref().unwrap(), base_message);

        // Main message should be user-friendly
        assert!(!mapped.message.is_empty());
        assert!(mapped.message != base_message); // Should be more user-friendly

        // Suggestions should be actionable
        for suggestion in &mapped.suggestions {
            assert!(!suggestion.is_empty());
            assert!(suggestion.len() > 10); // Should be meaningful, not just "retry"
        }
    }

    #[test]
    fn test_empty_and_edge_case_error_messages() {
        // Test behavior with empty or unusual error messages
        let empty_error = SSHError::NetworkError("".to_string());
        let mapped = map_ssh_error(&empty_error);
        assert!(!mapped.message.is_empty()); // Should still have a meaningful message

        let long_error = SSHError::NetworkError("x".repeat(1000));
        let mapped_long = map_ssh_error(&long_error);
        assert!(mapped_long.details.is_some());
        assert!(!mapped_long.message.is_empty());

        let special_chars = SSHError::NetworkError("Error: \n\t\r\0".to_string());
        let mapped_special = map_ssh_error(&special_chars);
        assert!(!mapped_special.message.is_empty());
    }
}