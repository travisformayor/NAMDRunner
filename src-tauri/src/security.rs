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

/// Utility functions for secure memory operations
pub mod memory {
    /// Clear a string's memory by overwriting with zeros
    pub fn clear_string(s: &mut String) {
        // Convert to bytes and zero them
        unsafe {
            let bytes = s.as_bytes_mut();
            for byte in bytes.iter_mut() {
                *byte = 0;
            }
        }
        s.clear();
    }

    /// Clear a vector's memory by overwriting with zeros
    pub fn clear_bytes(bytes: &mut [u8]) {
        for byte in bytes.iter_mut() {
            *byte = 0;
        }
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
    fn test_memory_clear_string() {
        let mut test_string = "sensitive_data".to_string();
        memory::clear_string(&mut test_string);

        assert!(test_string.is_empty());
    }

    #[test]
    fn test_memory_clear_bytes() {
        let mut test_bytes = b"sensitive".to_vec();
        memory::clear_bytes(&mut test_bytes);

        assert!(test_bytes.iter().all(|&b| b == 0));
    }
}