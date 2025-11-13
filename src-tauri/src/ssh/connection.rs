use ssh2::{Session, DisconnectCode};
use std::time::Duration;
use anyhow::Result;
use super::errors::SSHError;
use crate::{debug_log, info_log, error_log};

/// Configuration for SSH connections
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Command execution timeout in seconds
    pub command_timeout: u64,
    /// File transfer timeout in seconds (for TCP socket write operations during SFTP)
    pub file_transfer_timeout: u64,
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
            file_transfer_timeout: 300,  // 5 minutes for large file transfers
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
        info_log!("[SSH] Starting connection to {}:{} as {}", self.host, self.port, self.username);

        // Clear any existing session
        self.disconnect().await?;

        // Create TCP connection with explicit DNS resolution and detailed logging
        let tcp_addr_string = format!("{}:{}", self.host, self.port);

        // First, try to resolve the hostname explicitly
        let socket_addrs = match tokio::net::lookup_host(&tcp_addr_string).await {
            Ok(addrs) => {
                let addr_list: Vec<std::net::SocketAddr> = addrs.collect();
                debug_log!("[SSH] DNS resolved {} addresses", addr_list.len());
                addr_list
            }
            Err(e) => {
                let error_msg = format!("DNS resolution failed for {}: {}", self.host, e);
                error_log!("[SSH] ERROR: {}", error_msg);
                return Err(SSHError::NetworkError(error_msg).into());
            }
        };

        if socket_addrs.is_empty() {
            let error_msg = format!("No IP addresses found for hostname: {}", self.host);
            error_log!("[SSH] ERROR: {}", error_msg);
            return Err(SSHError::NetworkError(error_msg).into());
        }

        // Try connecting to each resolved address
        let mut last_error = None;
        for socket_addr in socket_addrs {

            match std::net::TcpStream::connect_timeout(
                &socket_addr,
                Duration::from_secs(self.config.timeout)
            ) {
                Ok(tcp) => {
                    debug_log!("[SSH] Connected to {}", socket_addr);

                    // Configure TCP
                    if let Err(e) = tcp.set_nodelay(self.config.tcp_nodelay) {
                        debug_log!("[SSH] Warning: Failed to set TCP nodelay: {}", e);
                    }
                    if let Err(e) = tcp.set_read_timeout(Some(Duration::from_secs(self.config.timeout))) {
                        debug_log!("[SSH] Warning: Failed to set read timeout: {}", e);
                    }
                    if let Err(e) = tcp.set_write_timeout(Some(Duration::from_secs(self.config.timeout))) {
                        debug_log!("[SSH] Warning: Failed to set write timeout: {}", e);
                    }

                    // Create and configure SSH session
                    return self.establish_ssh_session(tcp, password).await;
                }
                Err(e) => {
                    let error_msg = format!("TCP connection failed to {}: {}", socket_addr, e);
                    debug_log!("[SSH] {}", error_msg);
                    last_error = Some(error_msg);
                    continue;
                }
            }
        }

        // If we get here, all connection attempts failed
        let final_error = last_error.unwrap_or_else(|| "Unknown connection error".to_string());
        Err(SSHError::NetworkError(format!("Failed to connect to any resolved address for {}: {}", self.host, final_error)).into())
    }

    /// Establish SSH session over an existing TCP connection
    async fn establish_ssh_session(&mut self, tcp: std::net::TcpStream, password: &str) -> Result<()> {
        info_log!("[SSH] Establishing SSH session...");

        // Create SSH session
        let mut session = Session::new().map_err(|e| {
            let error_msg = format!("Failed to create SSH session: {}", e);
            error_log!("[SSH] ERROR: {}", error_msg);
            SSHError::HandshakeError(error_msg)
        })?;

        session.set_tcp_stream(tcp);

        // Configure blocking mode with appropriate timeout
        // The ssh2 library has its own blocking timeout (defaults to 1 second!)
        // We need to set this BEFORE handshake to use our configured timeout
        session.set_blocking(true);

        // Use connection timeout for initial handshake and commands
        // This will be updated to file_transfer_timeout when doing SFTP operations
        let timeout_ms = (self.config.timeout * 1000) as u32;
        session.set_timeout(timeout_ms);
        debug_log!("[SSH] Set session blocking timeout to {} ms ({} seconds)",
                   timeout_ms, self.config.timeout);

        info_log!("[SSH] Starting SSH handshake...");
        session.handshake().map_err(|e| {
            let error_msg = format!("SSH handshake failed: {}", e);
            error_log!("[SSH] ERROR: {}", error_msg);
            SSHError::HandshakeError(error_msg)
        })?;

        info_log!("[SSH] SSH handshake successful");

        // Log SSH server information
        if let Some(remote) = session.banner() {
            debug_log!("[SSH] Server: {}", remote);
        }

        // Set keepalive if configured
        if self.config.keepalive_interval > 0 {
            debug_log!("[SSH] Setting keepalive interval to {} seconds", self.config.keepalive_interval);
            session.set_keepalive(true, self.config.keepalive_interval);
        }

        // Log available authentication methods
        if let Ok(methods) = session.auth_methods(&self.username) {
            debug_log!("[SSH] Available auth methods: {}", methods);
        }

        // Attempt password authentication
        let pwd_string = password.to_string();
        info_log!("[SSH] Authenticating user '{}'", self.username);
        session.userauth_password(&self.username, &pwd_string).map_err(|e| {
            let error_msg = format!("Authentication failed for user {}: {}", self.username, e);
            error_log!("[SSH] {}", error_msg);
            debug_log!("[SSH] SSH2 Error code: {:?}", e.code());
            SSHError::AuthenticationError(error_msg)
        })?;

        // Verify authentication succeeded
        if !session.authenticated() {
            let error_msg = "Authentication failed - session not authenticated".to_string();
            error_log!("[SSH] ERROR: {}", error_msg);
            return Err(SSHError::AuthenticationError(error_msg).into());
        }

        info_log!("[SSH] Authentication successful");
        self.session = Some(session);
        info_log!("[SSH] Connection fully established");
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
        self.session.as_ref().is_some_and(|s| s.authenticated())
    }

    /// Get the username for this connection
    pub fn get_username(&self) -> &str {
        &self.username
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

    /// Set the session timeout for file transfer operations
    /// This should be called before SFTP operations to use the longer file_transfer_timeout
    pub fn set_file_transfer_timeout(&mut self) -> Result<()> {
        if let Some(session) = &mut self.session {
            let timeout_ms = (self.config.file_transfer_timeout * 1000) as u32;
            session.set_timeout(timeout_ms);
            debug_log!("[SSH] Updated session timeout to {} ms ({} seconds) for file transfers",
                       timeout_ms, self.config.file_transfer_timeout);
        }
        Ok(())
    }

    /// Reset the session timeout to the default command timeout
    /// This should be called after SFTP operations to restore normal timeout
    pub fn reset_command_timeout(&mut self) -> Result<()> {
        if let Some(session) = &mut self.session {
            let timeout_ms = (self.config.timeout * 1000) as u32;
            session.set_timeout(timeout_ms);
            debug_log!("[SSH] Reset session timeout to {} ms ({} seconds) for commands",
                       timeout_ms, self.config.timeout);
        }
        Ok(())
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
        assert_eq!(config.file_transfer_timeout, 300);
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

    #[test]
    fn test_tcp_address_string_formatting() {
        // Test our logic for creating "host:port" strings for DNS resolution
        let config = ConnectionConfig::default();

        // Test with hostname
        let conn_hostname = SSHConnection::new(
            "login.rc.colorado.edu".to_string(),
            22,
            "testuser".to_string(),
            config.clone()
        );

        // Test with IP address
        let conn_ip = SSHConnection::new(
            "192.168.1.100".to_string(),
            2222,
            "testuser".to_string(),
            config.clone()
        );

        // Test with localhost
        let conn_localhost = SSHConnection::new(
            "localhost".to_string(),
            22,
            "testuser".to_string(),
            config
        );

        // These should format correctly for DNS resolution
        // The actual formatting logic is: format!("{}:{}", self.host, self.port)
        assert_eq!(conn_hostname.host, "login.rc.colorado.edu");
        assert_eq!(conn_hostname.port, 22);

        assert_eq!(conn_ip.host, "192.168.1.100");
        assert_eq!(conn_ip.port, 2222);

        assert_eq!(conn_localhost.host, "localhost");
        assert_eq!(conn_localhost.port, 22);

        // Verify formatting would create correct strings for DNS lookup
        let hostname_addr = format!("{}:{}", conn_hostname.host, conn_hostname.port);
        let ip_addr = format!("{}:{}", conn_ip.host, conn_ip.port);
        let localhost_addr = format!("{}:{}", conn_localhost.host, conn_localhost.port);

        assert_eq!(hostname_addr, "login.rc.colorado.edu:22");
        assert_eq!(ip_addr, "192.168.1.100:2222");
        assert_eq!(localhost_addr, "localhost:22");
    }

    #[test]
    fn test_hostname_vs_ip_address_handling() {
        let config = ConnectionConfig::default();

        // These are all valid inputs that should work with DNS resolution
        let valid_hosts = vec![
            ("example.com", 22),
            ("sub.example.com", 443),
            ("localhost", 22),
            ("127.0.0.1", 22),
            ("192.168.1.1", 2222),
            ("::1", 22),
            ("2001:db8::1", 22),
        ];

        for (host, port) in valid_hosts {
            let conn = SSHConnection::new(
                host.to_string(),
                port,
                "testuser".to_string(),
                config.clone()
            );

            // Verify the connection object stores the values correctly
            assert_eq!(conn.host, host);
            assert_eq!(conn.port, port);
            assert_eq!(conn.username, "testuser");
            assert!(!conn.is_connected());

            // Verify the TCP address string would be formatted correctly
            let tcp_addr_string = format!("{}:{}", conn.host, conn.port);
            assert!(tcp_addr_string.contains(host));
            assert!(tcp_addr_string.contains(&port.to_string()));
        }
    }

    #[test]
    fn test_socket_addr_parsing_logic() {
        // This test verifies that we DON'T try to parse hostnames as SocketAddr directly

        // These strings would work with DNS resolution but NOT with SocketAddr::parse()
        let hostname_strings = vec![
            "login.rc.colorado.edu:22",
            "example.com:443",
            "localhost:22",
        ];

        // These strings would work with both DNS resolution AND SocketAddr::parse()
        let ip_strings = vec![
            "127.0.0.1:22",
            "192.168.1.1:443",
            "[::1]:22",
            "[2001:db8::1]:80",
        ];

        // Verify that hostname strings cannot be parsed as SocketAddr
        for addr_str in hostname_strings {
            let parse_result: Result<std::net::SocketAddr, _> = addr_str.parse();
            assert!(parse_result.is_err(),
                "Hostname '{}' should NOT parse as SocketAddr directly", addr_str);
        }

        // Verify that IP strings CAN be parsed as SocketAddr
        for addr_str in ip_strings {
            let parse_result: Result<std::net::SocketAddr, _> = addr_str.parse();
            assert!(parse_result.is_ok(),
                "IP address '{}' SHOULD parse as SocketAddr", addr_str);
        }

        // This demonstrates why we need DNS resolution instead of direct parsing
        // Our fix uses tokio::net::lookup_host() which handles both hostnames and IPs
    }

    #[test]
    fn test_edge_cases_in_address_handling() {
        let config = ConnectionConfig::default();

        // Test edge cases that should still work
        let edge_cases = vec![
            ("a.b", 1),           // Short hostname
            ("x", 65535),         // Single character hostname, max port
            ("127.0.0.1", 1),     // Min port
            ("test-host.domain-name.com", 8080), // Hyphenated hostname
        ];

        for (host, port) in edge_cases {
            let conn = SSHConnection::new(
                host.to_string(),
                port,
                "user".to_string(),
                config.clone()
            );

            assert_eq!(conn.host, host);
            assert_eq!(conn.port, port);

            // Verify formatting still works correctly
            let tcp_addr_string = format!("{}:{}", conn.host, conn.port);
            assert_eq!(tcp_addr_string, format!("{}:{}", host, port));
        }
    }
}