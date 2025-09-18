use ssh2::{Session, DisconnectCode};
use std::time::Duration;
use anyhow::{Result, Context};
use super::errors::SSHError;

/// Configuration for SSH connections
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Command execution timeout in seconds
    pub command_timeout: u64,
    /// Keepalive interval in seconds (0 to disable)
    pub keepalive_interval: u32,
    /// Maximum authentication attempts
    pub max_auth_attempts: u32,
    /// TCP nodelay setting
    pub tcp_nodelay: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            command_timeout: 120,
            keepalive_interval: 30,
            max_auth_attempts: 3,
            tcp_nodelay: true,
        }
    }
}

/// SSH connection manager
pub struct SSHConnection {
    session: Option<Session>,
    config: ConnectionConfig,
    host: String,
    port: u16,
    username: String,
}

impl std::fmt::Debug for SSHConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SSHConnection")
            .field("config", &self.config)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("session", &self.session.is_some())
            .finish()
    }
}

impl SSHConnection {
    /// Create a new SSH connection instance
    pub fn new(host: String, port: u16, username: String, config: ConnectionConfig) -> Self {
        Self {
            session: None,
            config,
            host,
            port,
            username,
        }
    }

    /// Connect to the SSH server with password authentication
    pub async fn connect(&mut self, password: &str) -> Result<()> {
        // Clear any existing session
        self.disconnect().await?;

        // Create TCP connection with timeout
        let tcp_addr = format!("{}:{}", self.host, self.port);
        let tcp = std::net::TcpStream::connect_timeout(
            &tcp_addr.parse().context("Invalid host address")?,
            Duration::from_secs(self.config.timeout)
        ).map_err(|e| {
            SSHError::NetworkError(format!("Failed to connect to {}: {}", tcp_addr, e))
        })?;

        // Configure TCP
        tcp.set_nodelay(self.config.tcp_nodelay)?;
        tcp.set_read_timeout(Some(Duration::from_secs(self.config.timeout)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(self.config.timeout)))?;

        // Create SSH session
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake().map_err(|e| {
            SSHError::HandshakeError(format!("SSH handshake failed: {}", e))
        })?;

        // Set keepalive if configured
        if self.config.keepalive_interval > 0 {
            session.set_keepalive(true, self.config.keepalive_interval);
        }

        // Authenticate with password
        session.userauth_password(&self.username, password).map_err(|e| {
            SSHError::AuthenticationError(format!("Authentication failed for user {}: {}", self.username, e))
        })?;

        // Verify authentication succeeded
        if !session.authenticated() {
            return Err(SSHError::AuthenticationError("Authentication failed".to_string()).into());
        }

        self.session = Some(session);
        Ok(())
    }

    /// Disconnect from the SSH server
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            session.disconnect(Some(DisconnectCode::ByApplication), "Closing connection", None)?;
        }
        Ok(())
    }

    /// Check if the connection is active
    pub fn is_connected(&self) -> bool {
        self.session.as_ref().map_or(false, |s| s.authenticated())
    }

    /// Get a reference to the SSH session
    pub fn get_session(&self) -> Result<&Session> {
        self.session.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Not connected to SSH server")
        })
    }

    /// Get a mutable reference to the SSH session
    pub fn get_session_mut(&mut self) -> Result<&mut Session> {
        self.session.as_mut().ok_or_else(|| {
            anyhow::anyhow!("Not connected to SSH server")
        })
    }

    /// Execute a keepalive to maintain the connection
    pub async fn keepalive(&self) -> Result<()> {
        if let Some(session) = &self.session {
            session.keepalive_send()?;
        }
        Ok(())
    }

    /// Get connection information
    pub fn get_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            connected: self.is_connected(),
            connected_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Drop for SSHConnection {
    fn drop(&mut self) {
        // Best-effort cleanup when dropping
        // Note: This is synchronous cleanup since Drop can't be async
        if let Some(session) = self.session.take() {
            // The SSH session will be automatically closed when dropped
            let _ = session.disconnect(Some(ssh2::DisconnectCode::ByApplication), "Closing connection", None);
        }
    }
}

/// Information about an SSH connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
    pub connected_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.timeout, 30);
        assert_eq!(config.command_timeout, 120);
        assert_eq!(config.keepalive_interval, 30);
        assert_eq!(config.max_auth_attempts, 3);
        assert!(config.tcp_nodelay);
    }

    #[test]
    fn test_connection_creation() {
        let config = ConnectionConfig::default();
        let conn = SSHConnection::new(
            "test.example.com".to_string(),
            22,
            "testuser".to_string(),
            config
        );

        assert!(!conn.is_connected());
        assert_eq!(conn.host, "test.example.com");
        assert_eq!(conn.port, 22);
        assert_eq!(conn.username, "testuser");
    }

    #[test]
    fn test_connection_info() {
        let config = ConnectionConfig::default();
        let conn = SSHConnection::new(
            "test.example.com".to_string(),
            22,
            "testuser".to_string(),
            config
        );

        let info = conn.get_info();
        assert_eq!(info.host, "test.example.com");
        assert_eq!(info.port, 22);
        assert_eq!(info.username, "testuser");
        assert!(!info.connected);
    }
}