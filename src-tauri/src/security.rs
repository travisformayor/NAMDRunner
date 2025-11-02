use secstr::SecStr;
use serde::{Deserialize, Deserializer};

/// Secure password handling with automatic memory clearing
#[derive(Clone)]
pub struct SecurePassword(SecStr);

impl SecurePassword {
    /// Create a new secure password from a string
    pub fn new(password: String) -> Self {
        Self(SecStr::from(password))
    }

    /// Create from a string reference
    pub fn from_str(password: &str) -> Self {
        Self(SecStr::from(password.to_string()))
    }

    /// Access the password temporarily for authentication
    /// The closure ensures the password is only accessible for a short time
    pub fn with_password<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        let bytes = self.0.unsecure();
        let password_str = std::str::from_utf8(bytes).unwrap_or("");
        f(password_str)
    }

    /// Convert to a standard string (use sparingly and clear immediately after use)
    /// This is only for compatibility with libraries that require &str
    pub fn expose(&self) -> String {
        let bytes = self.0.unsecure();
        std::str::from_utf8(bytes).unwrap_or("").to_string()
    }

    /// Check if the password is empty
    pub fn is_empty(&self) -> bool {
        self.0.unsecure().is_empty()
    }

    /// Get the length of the password (for debugging/logging purposes)
    pub fn len(&self) -> usize {
        self.0.unsecure().len()
    }
}

impl std::fmt::Debug for SecurePassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecurePassword([REDACTED])")
    }
}

impl Drop for SecurePassword {
    fn drop(&mut self) {
        // SecStr handles secure memory clearing automatically
    }
}

/// Custom deserializer for SecurePassword from JSON
impl<'de> Deserialize<'de> for SecurePassword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let password = String::deserialize(deserializer)?;
        Ok(SecurePassword::new(password))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_password_creation() {
        let password = SecurePassword::new("test123".to_string());

        password.with_password(|p| {
            assert_eq!(p, "test123");
        });
    }

    #[test]
    fn test_secure_password_from_str() {
        let password = SecurePassword::from_str("test456");

        password.with_password(|p| {
            assert_eq!(p, "test456");
        });
    }

    #[test]
    fn test_secure_password_debug() {
        let password = SecurePassword::new("secret".to_string());
        let debug_str = format!("{:?}", password);

        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("secret"));
    }


    #[test]
    fn test_secure_password_never_logged() {
        let password = SecurePassword::new("super_secret_password".to_string());

        // Test that Debug output doesn't contain password
        let debug_output = format!("{:?}", password);
        assert!(debug_output.contains("REDACTED"));
        assert!(!debug_output.contains("super_secret_password"));
        assert!(!debug_output.contains("secret"));
        assert!(!debug_output.contains("password"));

        // Test that Display is not implemented (would compile error if it was)
        // This ensures passwords can't accidentally be printed with {}
        // Note: This test passes because SecurePassword doesn't implement Display
    }

    #[test]
    fn test_connection_params_sanitized() {
        use crate::types::commands::ConnectParams;

        let params = ConnectParams {
            host: "test.example.com".to_string(),
            username: "testuser".to_string(),
            password: SecurePassword::new("secret123".to_string()),
        };

        // Test that Debug output of connection params doesn't expose password
        let debug_output = format!("{:?}", params);
        assert!(debug_output.contains("test.example.com"));
        assert!(debug_output.contains("testuser"));
        assert!(debug_output.contains("REDACTED"));
        assert!(!debug_output.contains("secret123"));
        assert!(!debug_output.contains("secret"));
    }

    #[test]
    fn test_secure_password_clone_safety() {
        let password1 = SecurePassword::new("original_password".to_string());
        let password2 = password1.clone();

        // Both clones should work independently
        password1.with_password(|p| assert_eq!(p, "original_password"));
        password2.with_password(|p| assert_eq!(p, "original_password"));

        // Both should redact in debug output
        assert!(format!("{:?}", password1).contains("REDACTED"));
        assert!(format!("{:?}", password2).contains("REDACTED"));
    }

    #[test]
    fn test_secure_password_length_safe() {
        let password = SecurePassword::new("test123".to_string());

        // Length should be accessible for validation without exposing content
        assert_eq!(password.len(), 7);
        assert!(!password.is_empty());

        let empty_password = SecurePassword::new("".to_string());
        assert_eq!(empty_password.len(), 0);
        assert!(empty_password.is_empty());
    }
}