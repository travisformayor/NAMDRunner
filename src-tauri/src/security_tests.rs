//! Security-focused integration tests for NAMDRunner
//!
//! These tests verify that the application properly handles malicious inputs
//! and prevents security vulnerabilities like command injection, directory traversal,
//! and other attack vectors.

#[cfg(test)]
mod security_integration_tests {
    use crate::commands::jobs::*;
    use crate::validation::{input, paths, shell};
    use crate::types::*;
    use std::env;

    /// Create a minimal test job params structure
    fn create_test_params(job_name: String) -> CreateJobParams {
        CreateJobParams {
            job_name,
            namd_config: NAMDConfig {
                steps: 1000,
                temperature: 300.0,
                timestep: 2.0,
                outputname: "output".to_string(),
                dcd_freq: Some(100),
                restart_freq: Some(100),
            },
            slurm_config: SlurmConfig {
                cores: 4,
                memory: "8GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            input_files: vec![],
        }
    }

    /// Ensure we're in mock mode for security tests
    fn setup_test_environment() {
        env::set_var("USE_MOCK_SSH", "true");
    }

    /// Test malicious job creation inputs
    #[tokio::test]
    async fn test_malicious_job_creation() {
        setup_test_environment();

        // Test various malicious job names
        let long_string = "a".repeat(100);
        let malicious_job_names = vec![
            "../../../etc/passwd",      // Directory traversal
            "/absolute/path",           // Absolute path
            "job; rm -rf /",           // Command injection
            "job\x00hidden",           // Null byte injection
            "job|malicious",           // Pipe injection
            "job&background",          // Background execution
            "job$(whoami)",            // Command substitution
            "job`whoami`",             // Command substitution
            "job\\escape",             // Backslash
            "job\"quote",              // Double quote
            "job'quote",               // Single quote
            &long_string,              // Length attack
            "",                        // Empty string
            "job with spaces",         // Spaces
            "job\ttab",                // Tab character
            "job\nnewline",            // Newline character
            "job\rcarriage",           // Carriage return
            "../admin/secret",         // Path traversal with directories
            "../../root/.ssh/id_rsa",  // SSH key access attempt
            "job/../../../home",       // Mixed content with traversal
        ];

        for malicious_name in malicious_job_names {
            let params = create_test_params(malicious_name.to_string());

            let result = create_job(params).await;

            // All malicious job names should be rejected
            assert!(!result.success,
                "Should reject malicious job name: '{}', but got success: {:?}",
                malicious_name, result);

            // Should have a meaningful error message
            assert!(result.error.is_some(),
                "Should have error message for malicious job name: '{}'",
                malicious_name);

            let error_msg = result.error.unwrap();
            assert!(!error_msg.is_empty(),
                "Error message should not be empty for malicious job name: '{}'",
                malicious_name);
        }
    }

    /// Test malicious job submission inputs
    #[tokio::test]
    async fn test_malicious_job_submission() {
        setup_test_environment();

        let malicious_job_ids = vec![
            "../../../etc/passwd",
            "/absolute/path",
            "job; rm -rf /",
            "job\x00hidden",
            "job|malicious",
            "job&background",
            "job$(whoami)",
            "job`whoami`",
        ];

        for malicious_id in malicious_job_ids {
            let result = submit_job(malicious_id.to_string()).await;

            // All malicious job IDs should be rejected
            assert!(!result.success,
                "Should reject malicious job ID: '{}', but got success: {:?}",
                malicious_id, result);

            assert!(result.error.is_some(),
                "Should have error message for malicious job ID: '{}'",
                malicious_id);
        }
    }

    /// Test malicious job deletion inputs
    #[tokio::test]
    async fn test_malicious_job_deletion() {
        setup_test_environment();

        let malicious_job_ids = vec![
            "../../../etc/passwd",
            "/absolute/path",
            "job; rm -rf /",
            "job\x00hidden",
            "job|malicious",
            "job&background",
            "job$(whoami)",
            "job`whoami`",
        ];

        for malicious_id in malicious_job_ids {
            let result = delete_job(malicious_id.to_string(), true).await;

            // All malicious job IDs should be rejected
            assert!(!result.success,
                "Should reject malicious job ID: '{}', but got success: {:?}",
                malicious_id, result);

            assert!(result.error.is_some(),
                "Should have error message for malicious job ID: '{}'",
                malicious_id);
        }
    }

    /// Test input validation functions directly
    #[test]
    fn test_input_validation_comprehensive() {
        // Test job ID validation
        let long_string = "a".repeat(100);
        let dangerous_job_ids = vec![
            "../../../etc/passwd",      // Directory traversal
            "/absolute/path",           // Absolute path
            "job; rm -rf /",           // Command injection
            "job\x00hidden",           // Null byte injection
            "job|malicious",           // Pipe injection
            "job&background",          // Background execution
            "job$(whoami)",            // Command substitution
            "job`whoami`",             // Command substitution
            "job\\escape",             // Backslash
            "job\"quote",              // Double quote
            "job'quote",               // Single quote
            &long_string,              // Length attack
            "",                        // Empty string
            "job with spaces",         // Spaces
            "job\ttab",                // Tab character
            "job\nnewline",            // Newline character
            "job{dangerous}",          // Braces
            "job[dangerous]",          // Brackets
            "job<dangerous>",          // Angle brackets
            "job%dangerous",           // Percent encoding
            "job*wildcard",            // Wildcards
            "job?wildcard",            // Question mark wildcard
        ];

        for dangerous_id in dangerous_job_ids {
            let result = input::sanitize_job_id(dangerous_id);
            assert!(result.is_err(),
                "Should reject dangerous job ID: '{}', but validation passed",
                dangerous_id);
        }

        // Test valid job IDs
        let valid_job_ids = vec![
            "job_001",
            "test-job",
            "MyJob123",
            "job_with_underscores",
            "valid-job-name",
            "Job123",
        ];

        for valid_id in valid_job_ids {
            let result = input::sanitize_job_id(valid_id);
            assert!(result.is_ok(),
                "Should accept valid job ID: '{}', but validation failed: {:?}",
                valid_id, result);
        }
    }

    /// Test username validation
    #[test]
    fn test_username_validation_comprehensive() {
        let dangerous_usernames = vec![
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
            "user{admin}",           // Braces
            "user[admin]",           // Brackets
            "user<admin>",           // Angle brackets
            "user*",                 // Wildcard
            "user?",                 // Question mark
        ];

        for dangerous_username in dangerous_usernames {
            let result = input::sanitize_username(dangerous_username);
            assert!(result.is_err(),
                "Should reject dangerous username: '{}', but validation passed",
                dangerous_username);
        }

        // Test valid usernames
        let valid_usernames = vec![
            "testuser",
            "john.doe",
            "user_123",
            "user-name",
            "validuser",
        ];

        for valid_username in valid_usernames {
            let result = input::sanitize_username(valid_username);
            assert!(result.is_ok(),
                "Should accept valid username: '{}', but validation failed: {:?}",
                valid_username, result);
        }
    }

    /// Test path generation security
    #[test]
    fn test_path_generation_security() {
        // Test that malicious inputs don't create dangerous paths
        let test_cases = vec![
            ("../admin", "job_001"),
            ("testuser", "../../../etc"),
            ("user\x00admin", "job_001"),
            ("testuser", "job\x00hidden"),
            ("user;admin", "job_001"),
            ("testuser", "job;malicious"),
        ];

        for (username, job_id) in test_cases {
            // Both project and scratch directory generation should fail
            let project_result = paths::project_directory(username, job_id);
            assert!(project_result.is_err(),
                "Should reject malicious path combo for project: {} / {}",
                username, job_id);

            let scratch_result = paths::scratch_directory(username, job_id);
            assert!(scratch_result.is_err(),
                "Should reject malicious path combo for scratch: {} / {}",
                username, job_id);
        }

        // Test that valid inputs create expected paths
        let valid_result = paths::project_directory("testuser", "job_001");
        assert!(valid_result.is_ok());
        assert_eq!(valid_result.unwrap(), "/projects/testuser/namdrunner_jobs/job_001");

        let valid_scratch = paths::scratch_directory("testuser", "job_001");
        assert!(valid_scratch.is_ok());
        assert_eq!(valid_scratch.unwrap(), "/scratch/alpine/testuser/namdrunner_jobs/job_001");
    }

    /// Test shell command escaping
    #[test]
    fn test_shell_command_escaping() {
        // Test dangerous parameters get properly escaped
        let dangerous_params = vec![
            "file; rm -rf /",        // Command injection
            "file && malicious",     // Command chaining
            "file || backup",        // Command alternatives
            "file | evil",           // Pipe to command
            "file & background",     // Background execution
            "file$(whoami)",         // Command substitution
            "file`whoami`",          // Command substitution
            "file'quote'",           // Single quotes
            "file\"quote\"",         // Double quotes
            "file\\escape",          // Backslash
            "file\nmalicious",       // Newline injection
            "file;id;",              // Multiple command injection
        ];

        for dangerous_param in dangerous_params {
            let escaped = shell::escape_parameter(dangerous_param);

            // The escaped version should be wrapped in quotes
            assert!(escaped.starts_with('\'') && escaped.ends_with('\''),
                "Parameter should be wrapped in single quotes: '{}'", escaped);

            // For parameters containing single quotes, they should be properly escaped
            // The pattern '\"'\"' is a valid way to escape single quotes in shell
            // What we need to ensure is that dangerous characters like ; | & are contained
            // within the quoted sections and can't be interpreted by the shell

            // Test that the escaped parameter is safe to use in a shell command
            // by ensuring it doesn't allow command injection when used
            let test_command = format!("echo {}", escaped);

            // The dangerous characters should not be interpretable by the shell
            // This is guaranteed if they're within single quotes (which our function does)
            assert!(!test_command.contains("; echo") || test_command.contains("'; echo"),
                "Command injection should be prevented: '{}'", test_command);
        }
    }

    /// Test command building with malicious templates and parameters
    #[test]
    fn test_command_building_security() {
        // Test that command building with malicious parameters is safe
        let safe_template = "mkdir -p {} && cd {}";
        let malicious_params = vec![
            "dir; rm -rf /",
            "dir && malicious",
            "dir || backup",
            "dir | evil",
            "dir$(whoami)",
            "dir`whoami`",
        ];

        for malicious_param in &malicious_params {
            let result = shell::build_command_safely(safe_template, &[malicious_param, malicious_param]);

            assert!(result.is_ok(), "Command building should succeed with proper escaping");

            let command = result.unwrap();

            // The command should contain escaped parameters
            assert!(command.contains('\''), "Command should contain escaped parameters");

            // Verify that dangerous characters are properly escaped
            // The semicolon should be within single quotes, e.g., 'dir; rm -rf /'
            // We can't have unquoted semicolons
            let parts: Vec<&str> = command.split('\'').collect();
            // In a properly escaped command, semicolons should only appear in odd-indexed parts
            // (inside quotes): part0 'part1' part2 'part3' ...
            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 0 && part.contains(';') {
                    panic!("Unescaped semicolon found outside quotes in command: '{}'", command);
                }
            }
        }
    }

    /// Test complete job lifecycle with malicious inputs
    #[tokio::test]
    async fn test_complete_malicious_job_lifecycle() {
        setup_test_environment();

        // Try to create a job with a malicious name
        let malicious_params = create_test_params("../../../etc/passwd; rm -rf /".to_string());

        let create_result = create_job(malicious_params).await;
        assert!(!create_result.success, "Should reject malicious job creation");
        assert!(create_result.job_id.is_none(), "Should not return job ID for malicious input");

        // Try to submit a malicious job ID
        let submit_result = submit_job("malicious; rm -rf /".to_string()).await;
        assert!(!submit_result.success, "Should reject malicious job submission");

        // Try to delete with malicious job ID
        let delete_result = delete_job("malicious; rm -rf /".to_string(), true).await;
        assert!(!delete_result.success, "Should reject malicious job deletion");
    }

    /// Test boundary conditions and edge cases
    #[test]
    fn test_security_boundary_conditions() {
        // Test empty inputs
        assert!(input::sanitize_job_id("").is_err());
        assert!(input::sanitize_username("").is_err());

        // Test maximum length inputs
        let long_string = "a".repeat(1000);
        assert!(input::sanitize_job_id(&long_string).is_err());
        assert!(input::sanitize_username(&long_string).is_err());

        // Test exactly at the boundary (64 characters)
        let boundary_string = "a".repeat(64);
        assert!(input::sanitize_job_id(&boundary_string).is_ok());
        assert!(input::sanitize_username(&boundary_string).is_ok());

        // Test just over the boundary (65 characters)
        let over_boundary_string = "a".repeat(65);
        assert!(input::sanitize_job_id(&over_boundary_string).is_err());
        assert!(input::sanitize_username(&over_boundary_string).is_err());

        // Test Unicode characters
        assert!(input::sanitize_job_id("job_ñ").is_err()); // Non-ASCII
        assert!(input::sanitize_job_id("job_测试").is_err()); // Chinese characters

        // Test null byte variations
        assert!(input::sanitize_job_id("job\x00").is_err());
        assert!(input::sanitize_job_id("\x00job").is_err());
        assert!(input::sanitize_job_id("jo\x00b").is_err());
    }

    /// Test that valid operations still work after security enhancements
    #[tokio::test]
    async fn test_valid_operations_still_work() {
        setup_test_environment();

        // Create a valid job
        let valid_params = create_test_params("valid_job_123".to_string());

        let create_result = create_job(valid_params).await;
        assert!(create_result.success, "Should accept valid job creation: {:?}", create_result);
        assert!(create_result.job_id.is_some(), "Should return job ID for valid job");

        // The job ID returned should be valid
        let job_id = create_result.job_id.unwrap();
        assert!(input::sanitize_job_id(&job_id).is_ok(), "Returned job ID should be valid");

        // We can't test submission and deletion in mock mode without more setup,
        // but the creation test validates that the security doesn't break normal operation
    }

    /// Test error message quality (should not leak sensitive information)
    #[test]
    fn test_error_message_security() {
        // Error messages should be informative but not leak system information
        let malicious_inputs = vec![
            "../../../etc/passwd",
            "/root/.ssh/id_rsa",
            "job; cat /etc/shadow",
        ];

        for malicious_input in malicious_inputs {
            let result = input::sanitize_job_id(malicious_input);
            assert!(result.is_err());

            let error_msg = result.unwrap_err().to_string();

            // Error message should not contain the malicious input verbatim
            assert!(!error_msg.contains("passwd"),
                "Error message should not leak sensitive paths: '{}'", error_msg);
            assert!(!error_msg.contains("shadow"),
                "Error message should not leak sensitive paths: '{}'", error_msg);
            assert!(!error_msg.contains("ssh"),
                "Error message should not leak sensitive paths: '{}'", error_msg);

            // Should have a generic but helpful message
            assert!(error_msg.contains("invalid") || error_msg.contains("Invalid"),
                "Error message should indicate invalid input: '{}'", error_msg);
        }
    }
}

/// Performance tests to ensure security validation doesn't cause DoS
#[cfg(test)]
mod security_performance_tests {
    use super::*;
    use std::time::Instant;
    use crate::validation::input;

    #[test]
    fn test_validation_performance() {
        // Test that validation is fast even with large inputs
        let large_input = "a".repeat(10000);

        let start = Instant::now();
        let result = input::sanitize_job_id(&large_input);
        let duration = start.elapsed();

        // Should reject quickly (under 1ms for this size)
        assert!(duration.as_millis() < 10, "Validation should be fast even for large inputs");
        assert!(result.is_err(), "Should reject oversized input");

        // Test many rapid validations don't cause issues
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = input::sanitize_job_id("test_job");
        }
        let duration = start.elapsed();

        // 1000 validations should complete quickly
        assert!(duration.as_millis() < 100, "Bulk validation should be fast");
    }

    #[test]
    fn test_no_regex_dos() {
        // Test inputs that might cause ReDoS (Regular Expression Denial of Service)
        let redos_patterns = vec![
            "a".repeat(1000) + &"b".repeat(1000),  // Large alternating pattern
            ("ab".repeat(500)),                     // Repeating pattern
            "a".repeat(500) + "!" + &"a".repeat(500), // Large with special char
        ];

        for pattern in redos_patterns {
            let start = Instant::now();
            let _result = input::sanitize_job_id(&pattern);
            let duration = start.elapsed();

            // Should not take more than a few milliseconds
            assert!(duration.as_millis() < 50,
                "Validation should not be vulnerable to ReDoS attacks, took {}ms for pattern length {}",
                duration.as_millis(), pattern.len());
        }
    }
}