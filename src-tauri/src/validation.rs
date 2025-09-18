use anyhow::{Result, anyhow};
use std::path::Path;

/// Input validation and sanitization for security-critical operations
pub mod input {
    use super::*;

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
            // (e.g., /projects/user/namdrunner_jobs/job_001)
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
}

/// Shell command construction utilities
pub mod shell {
    use super::*;

    /// Safely escape a parameter for shell commands
    ///
    /// This uses proper shell escaping to prevent command injection
    pub fn escape_parameter(param: &str) -> String {
        // Use single quotes for strong quoting, escaping any single quotes in the string
        if param.contains('\'') {
            // If the string contains single quotes, we need to escape them
            format!("'{}'", param.replace('\'', "'\"'\"'"))
        } else {
            // Simple case - just wrap in single quotes
            format!("'{}'", param)
        }
    }

    /// Build a shell command safely with escaped parameters
    ///
    /// Takes a command template with {} placeholders and replaces them with escaped parameters
    pub fn build_command_safely(template: &str, params: &[&str]) -> Result<String> {
        let mut result = template.to_string();

        for param in params {
            // Find the first {} and replace it with the escaped parameter
            if let Some(pos) = result.find("{}") {
                let escaped = escape_parameter(param);
                result.replace_range(pos..pos+2, &escaped);
            } else {
                return Err(anyhow!("More parameters provided than placeholders in template"));
            }
        }

        // Check if there are any unreplaced placeholders
        if result.contains("{}") {
            return Err(anyhow!("Template has more placeholders than parameters provided"));
        }

        Ok(result)
    }
}

/// Directory path utilities for NAMDRunner
pub mod paths {
    use super::*;

    /// Generate a safe project directory path for a user and job
    pub fn project_directory(username: &str, job_id: &str) -> Result<String> {
        let clean_username = super::input::sanitize_username(username)?;
        let clean_job_id = super::input::sanitize_job_id(job_id)?;

        let path = format!("/projects/{}/namdrunner_jobs/{}", clean_username, clean_job_id);

        // Validate the path is within allowed directories
        super::input::validate_path_safety(&path, &["/projects/"])?;

        Ok(path)
    }

    /// Generate a safe scratch directory path for a user and job
    pub fn scratch_directory(username: &str, job_id: &str) -> Result<String> {
        let clean_username = super::input::sanitize_username(username)?;
        let clean_job_id = super::input::sanitize_job_id(job_id)?;

        let path = format!("/scratch/alpine/{}/namdrunner_jobs/{}", clean_username, clean_job_id);

        // Validate the path is within allowed directories
        super::input::validate_path_safety(&path, &["/scratch/"])?;

        Ok(path)
    }

    /// Get the standard subdirectories that should be created for a job
    pub fn job_subdirectories() -> Vec<&'static str> {
        vec!["inputs", "outputs", "scripts"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod input_tests {
        use super::*;

        #[test]
        fn test_valid_job_ids() {
            let valid_ids = vec!["job_001", "test-job", "MyJob123", "job_with_underscores"];

            for id in valid_ids {
                assert!(input::sanitize_job_id(id).is_ok(), "Should accept valid ID: {}", id);
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
                let result = input::sanitize_job_id(id);
                assert!(result.is_err(), "Should reject malicious ID: {}", id);
            }
        }

        #[test]
        fn test_valid_usernames() {
            let valid_usernames = vec!["testuser", "john.doe", "user_123", "user-name"];

            for username in valid_usernames {
                assert!(input::sanitize_username(username).is_ok(), "Should accept valid username: {}", username);
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
                let result = input::sanitize_username(username);
                assert!(result.is_err(), "Should reject malicious username: {}", username);
            }
        }

        #[test]
        fn test_path_validation() {
            // Valid paths
            assert!(input::validate_path_safety("/projects/user/namdrunner_jobs/job_001", &["/projects/"]).is_ok());
            assert!(input::validate_path_safety("/scratch/alpine/user/namdrunner_jobs/job_001", &["/scratch/"]).is_ok());

            // Invalid paths (if they could be resolved)
            // Note: These tests assume the paths don't exist, so we're testing the component validation
        }
    }

    mod shell_tests {
        use super::*;

        #[test]
        fn test_shell_escaping() {
            assert_eq!(shell::escape_parameter("simple"), "'simple'");
            assert_eq!(shell::escape_parameter("with spaces"), "'with spaces'");
            assert_eq!(shell::escape_parameter("with'quote"), "'with'\"'\"'quote'");
            assert_eq!(shell::escape_parameter("dangerous;command"), "'dangerous;command'");
        }

        #[test]
        fn test_safe_command_building() {
            let template = "mkdir -p {} && cd {}";
            let params = vec!["test_dir", "test_dir"];
            let result = shell::build_command_safely(template, &params).unwrap();
            assert_eq!(result, "mkdir -p 'test_dir' && cd 'test_dir'");
        }

        #[test]
        fn test_command_building_errors() {
            // Too many parameters
            let result = shell::build_command_safely("mkdir {}", &["dir1", "dir2"]);
            assert!(result.is_err());

            // Too few parameters
            let result = shell::build_command_safely("mkdir {} {}", &["dir1"]);
            assert!(result.is_err());
        }
    }

    mod path_tests {
        use super::*;

        #[test]
        fn test_project_directory_generation() {
            let result = paths::project_directory("testuser", "job_001").unwrap();
            assert_eq!(result, "/projects/testuser/namdrunner_jobs/job_001");
        }

        #[test]
        fn test_scratch_directory_generation() {
            let result = paths::scratch_directory("testuser", "job_001").unwrap();
            assert_eq!(result, "/scratch/alpine/testuser/namdrunner_jobs/job_001");
        }

        #[test]
        fn test_malicious_path_generation() {
            // Should fail with malicious inputs
            assert!(paths::project_directory("../admin", "job_001").is_err());
            assert!(paths::project_directory("testuser", "../../../etc").is_err());
        }

        #[test]
        fn test_subdirectories() {
            let subdirs = paths::job_subdirectories();
            assert!(subdirs.contains(&"inputs"));
            assert!(subdirs.contains(&"outputs"));
            assert!(subdirs.contains(&"scripts"));
        }
    }

    #[test]
    fn test_comprehensive_security_scenarios() {
        // Test the complete workflow with malicious inputs
        let malicious_scenarios = vec![
            ("../../../root", "job_001"),
            ("testuser", "; rm -rf /"),
            ("test\x00user", "job_001"),
            ("testuser", "job$(whoami)"),
            ("user|admin", "job_001"),
        ];

        for (username, job_id) in malicious_scenarios {
            // Both project and scratch directory generation should fail
            assert!(paths::project_directory(username, job_id).is_err(),
                    "Should reject malicious combo: {} / {}", username, job_id);
            assert!(paths::scratch_directory(username, job_id).is_err(),
                    "Should reject malicious combo: {} / {}", username, job_id);
        }
    }
}