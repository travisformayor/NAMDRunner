use anyhow::{Result, anyhow};
use std::path::Path;

/// Maximum length for job IDs and usernames to prevent memory exhaustion
const MAX_IDENTIFIER_LENGTH: usize = 64;

/// Sanitize and validate a job ID
///
/// Job IDs must be:
/// - Alphanumeric characters, underscores, and hyphens only
/// - Between 1 and 64 characters
/// - No directory traversal sequences
/// - No absolute paths
pub fn sanitize_job_id(input: &str) -> Result<String> {
    // Basic length validation
    if input.is_empty() {
        return Err(anyhow!("Job ID cannot be empty"));
    }

    if input.len() > MAX_IDENTIFIER_LENGTH {
        return Err(anyhow!("Job ID too long (max {} characters)", MAX_IDENTIFIER_LENGTH));
    }

    // Check for directory traversal attempts
    if input.contains("..") || input.starts_with('/') || input.starts_with('\\') {
        return Err(anyhow!("Job ID contains invalid path sequences"));
    }

    // Check for null bytes and other dangerous characters
    if input.contains('\0') {
        return Err(anyhow!("Job ID contains null bytes"));
    }

    // Check for non-ASCII characters (Unicode) - reject entirely for security
    if !input.is_ascii() {
        return Err(anyhow!("Job ID contains non-ASCII characters"));
    }

    // Filter to safe characters only
    let sanitized: String = input.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    // Verify we didn't strip everything
    if sanitized.is_empty() {
        return Err(anyhow!("Job ID contains no valid characters"));
    }

    // Verify sanitized version matches original (no dangerous chars were present)
    if sanitized != input {
        return Err(anyhow!("Job ID contains invalid characters (only alphanumeric, underscore, and hyphen allowed)"));
    }

    Ok(sanitized)
}

/// Sanitize and validate a username
///
/// Similar rules to job IDs but may have slightly different constraints
pub fn sanitize_username(input: &str) -> Result<String> {
    // Basic length validation
    if input.is_empty() {
        return Err(anyhow!("Username cannot be empty"));
    }

    if input.len() > MAX_IDENTIFIER_LENGTH {
        return Err(anyhow!("Username too long (max {} characters)", MAX_IDENTIFIER_LENGTH));
    }

    // Check for directory traversal attempts
    if input.contains("..") || input.starts_with('/') || input.starts_with('\\') {
        return Err(anyhow!("Username contains invalid path sequences"));
    }

    // Check for null bytes and shell metacharacters
    if input.contains('\0') {
        return Err(anyhow!("Username contains null bytes"));
    }

    // Check for shell metacharacters that could enable command injection
    let dangerous_chars = ['$', '`', ';', '|', '&', '>', '<', '(', ')', '{', '}', '[', ']', '\'', '"', ' ', '\t', '\n', '\r'];
    if input.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err(anyhow!("Username contains shell metacharacters"));
    }

    // Filter to safe characters - allow dots for usernames like "john.doe"
    let sanitized: String = input.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect();

    // Verify we didn't strip everything
    if sanitized.is_empty() {
        return Err(anyhow!("Username contains no valid characters"));
    }

    // Verify sanitized version matches original
    if sanitized != input {
        return Err(anyhow!("Username contains invalid characters"));
    }

    Ok(sanitized)
}

/// Validate that a relative file path is safe (no traversal, no absolute paths, no null bytes)
///
/// Used for validating user-provided relative paths like file downloads
pub fn validate_relative_file_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("File path cannot be empty"));
    }

    if path.contains('\0') {
        return Err(anyhow!("File path contains null bytes"));
    }

    if path.starts_with('/') {
        return Err(anyhow!("File path must be relative, not absolute"));
    }

    if path.contains("..") {
        return Err(anyhow!("File path contains directory traversal"));
    }

    Ok(())
}

/// Validate that a constructed path is safe
///
/// This performs additional validation on complete paths to ensure
/// they don't escape intended directories through canonicalization
pub fn validate_path_safety(path: &str, allowed_prefixes: &[&str]) -> Result<()> {
    // Convert to Path for proper handling
    let path_obj = Path::new(path);

    // Check for absolute paths (should be relative to user directories)
    if path_obj.is_absolute() {
        // For our use case, absolute paths are actually expected
        // (e.g., /projects/user/{JOB_BASE_DIRECTORY}/job_001)
        // So we validate against allowed prefixes instead
    }

    // Normalize the path to resolve any .. sequences
    let canonical_path = match path_obj.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // Path doesn't exist yet, which is fine for directory creation
            // But we can still validate the components
            let mut normalized = std::path::PathBuf::new();
            for component in path_obj.components() {
                match component {
                    std::path::Component::ParentDir => {
                        // Don't allow going up directories in our validation
                        return Err(anyhow!("Path contains parent directory references"));
                    }
                    std::path::Component::CurDir => {
                        // Current dir is fine, just skip
                        continue;
                    }
                    _ => {
                        normalized.push(component);
                    }
                }
            }
            normalized
        }
    };

    // Check if the path starts with any allowed prefix
    let path_str = canonical_path.to_string_lossy();
    let is_allowed = allowed_prefixes.iter().any(|prefix| {
        path_str.starts_with(prefix)
    });

    if !is_allowed {
        return Err(anyhow!("Path '{}' is not within allowed directories", path_str));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_job_ids() {
        let valid_ids = vec!["job_001", "test-job", "MyJob123", "job_with_underscores"];

        for id in valid_ids {
            assert!(sanitize_job_id(id).is_ok(), "Should accept valid ID: {}", id);
        }
    }

    #[test]
    fn test_malicious_job_ids() {
        let long_string = "a".repeat(100);
        let malicious_ids = vec![
            "../../../etc/passwd",  // Directory traversal
            "/absolute/path",        // Absolute path
            "job; rm -rf /",        // Command injection
            "job\x00hidden",        // Null byte injection
            "job|malicious",        // Pipe injection
            "job&background",       // Background execution
            "job$(whoami)",         // Command substitution
            "job`whoami`",          // Command substitution
            "job\\escape",          // Backslash
            "job\"quote",           // Double quote
            "job'quote",            // Single quote
            &long_string,           // Length attack
            "",                     // Empty string
            "job with spaces",      // Spaces
        ];

        for id in malicious_ids {
            let result = sanitize_job_id(id);
            assert!(result.is_err(), "Should reject malicious ID: {}", id);
        }
    }

    #[test]
    fn test_valid_usernames() {
        let valid_usernames = vec!["testuser", "john.doe", "user_123", "user-name"];

        for username in valid_usernames {
            assert!(sanitize_username(username).is_ok(), "Should accept valid username: {}", username);
        }
    }

    #[test]
    fn test_malicious_usernames() {
        let malicious_usernames = vec![
            "../admin",              // Directory traversal
            "/root",                 // Absolute path
            "user; whoami",          // Command injection
            "user\x00",              // Null byte
            "user|admin",            // Pipe
            "user&admin",            // Background
            "user$(id)",             // Command substitution
            "user`id`",              // Command substitution
            "user'admin'",           // Single quotes
            "user\"admin\"",         // Double quotes
            "user with spaces",      // Spaces
            "user\ttab",             // Tab
            "user\nnewline",         // Newline
        ];

        for username in malicious_usernames {
            let result = sanitize_username(username);
            assert!(result.is_err(), "Should reject malicious username: {}", username);
        }
    }

    #[test]
    fn test_path_validation() {
        use crate::ssh::directory_structure::JobDirectoryStructure;
        // Valid paths - cross-platform
        if cfg!(windows) {
            assert!(validate_path_safety(&format!("C:\\Users\\user\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY), &["C:\\Users\\"]).is_ok());
            assert!(validate_path_safety(&format!("C:\\scratch\\user\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY), &["C:\\scratch\\"]).is_ok());
        } else {
            // Use centralized path generation for consistent testing
            let project_path = JobDirectoryStructure::project_dir("user", "job_001");
            let scratch_path = JobDirectoryStructure::scratch_dir("user", "job_001");
            assert!(validate_path_safety(&project_path, &JobDirectoryStructure::project_allowed_prefixes()).is_ok());
            assert!(validate_path_safety(&scratch_path, &JobDirectoryStructure::scratch_allowed_prefixes()).is_ok());
        }

        // Invalid paths (if they could be resolved)
        // Note: These tests assume the paths don't exist, so we're testing the component validation
    }

    #[test]
    fn test_centralized_validation_consistency() {
        // Test that all validation uses consistent patterns (business logic only)
        let test_inputs = vec!["normal_input", "test-123", "valid_file.txt"];

        for input in test_inputs {
            // All validation functions should use consistent character sets
            if let Ok(sanitized_id) = sanitize_job_id(input) {
                assert!(!sanitized_id.contains(' '), "Job IDs should not contain spaces");
                assert!(sanitized_id.is_ascii(), "Job IDs should be ASCII");
            }

            if let Ok(sanitized_username) = sanitize_username(input) {
                assert!(!sanitized_username.contains(' '), "Usernames should not contain spaces");
                assert!(sanitized_username.is_ascii(), "Usernames should be ASCII");
            }
        }
    }
}
