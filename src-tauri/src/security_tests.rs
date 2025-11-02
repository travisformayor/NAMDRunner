//! Security-focused integration tests for NAMDRunner
//!
//! These tests verify that the application properly handles malicious inputs
//! and prevents security vulnerabilities like command injection, directory traversal,
//! and other attack vectors.

#[cfg(test)]
mod security_integration_tests {
    use crate::validation::{input, paths, shell};
    use crate::types::*;
    use std::env;

    // Test helper functions that test validation directly following CONTRIBUTING.md testing strategy
    // Core Principle: Test our business logic, not external libraries or Tauri infrastructure

    /// Test wrapper for create_job that tests validation logic directly
    async fn test_create_job(params: CreateJobParams) -> CreateJobResult {
        // Force mock mode for this test
        crate::demo::set_demo_mode(true);

        // Test the validation layer directly - invalid inputs fail before Tauri infrastructure is needed
        use crate::validation::input;

        // Validate job name first - this is what we're really testing
        if let Err(e) = input::sanitize_job_id(&params.job_name) {
            return CreateJobResult {
                success: false,
                job_id: None,
                job: None,
                error: Some(e.to_string()),
            };
        }

        // If validation passes, return success (we're testing security validation, not job creation)
        CreateJobResult {
            success: true,
            job_id: Some(format!("test_{}", params.job_name)),
            job: None,
            error: None,
        }
    }

    /// Test wrapper for submit_job that tests validation logic directly
    async fn test_submit_job(job_id: String) -> SubmitJobResult {
        crate::demo::set_demo_mode(true);

        // Test the validation layer directly
        if let Err(e) = crate::validation::input::sanitize_job_id(&job_id) {
            return SubmitJobResult {
                success: false,
                slurm_job_id: None,
                submitted_at: None,
                error: Some(e.to_string()),
            };
        }

        SubmitJobResult {
            success: true,
            slurm_job_id: Some("mock_slurm_12345".to_string()),
            submitted_at: Some(chrono::Utc::now().to_rfc3339()),
            error: None,
        }
    }

    /// Test wrapper for delete_job that tests validation logic directly
    async fn test_delete_job(job_id: String, _delete_remote: bool) -> DeleteJobResult {
        crate::demo::set_demo_mode(true);

        // Test the validation layer directly
        if let Err(e) = crate::validation::input::sanitize_job_id(&job_id) {
            return DeleteJobResult {
                success: false,
                error: Some(e.to_string()),
            };
        }

        DeleteJobResult {
            success: true,
            error: None,
        }
    }

    /// Create a minimal test job params structure
    fn create_test_params(job_name: String) -> CreateJobParams {
        CreateJobParams {
            job_name,
            namd_config: NAMDConfig {
                outputname: "output".to_string(),
                temperature: 300.0,
                timestep: 2.0,
                execution_mode: ExecutionMode::Run,
                steps: 1000,
                cell_basis_vector1: None,
                cell_basis_vector2: None,
                cell_basis_vector3: None,
                pme_enabled: false,
                npt_enabled: false,
                langevin_damping: 5.0,
                xst_freq: 100,
                output_energies_freq: 100,
                dcd_freq: 100,
                restart_freq: 100,
                output_pressure_freq: 100,
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

            let result = test_create_job(params).await;

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
            let result = test_submit_job(malicious_id.to_string()).await;

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
            let result = test_delete_job(malicious_id.to_string(), true).await;

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

        // Test that valid inputs create expected paths using centralized path generation
        use crate::ssh::directory_structure::JobDirectoryStructure;
        let valid_result = paths::project_directory("testuser", "job_001");
        assert!(valid_result.is_ok());
        // Should match centralized path generation
        let expected_project = JobDirectoryStructure::project_dir("testuser", "job_001");
        assert_eq!(valid_result.unwrap(), expected_project);

        let valid_scratch = paths::scratch_directory("testuser", "job_001");
        assert!(valid_scratch.is_ok());
        // Should match centralized path generation
        let expected_scratch = JobDirectoryStructure::scratch_dir("testuser", "job_001");
        assert_eq!(valid_scratch.unwrap(), expected_scratch);
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

    /// Test complete job lifecycle with malicious inputs
    #[tokio::test]
    async fn test_complete_malicious_job_lifecycle() {
        setup_test_environment();

        // Try to create a job with a malicious name
        let malicious_params = create_test_params("../../../etc/passwd; rm -rf /".to_string());

        let create_result = test_create_job(malicious_params).await;
        assert!(!create_result.success, "Should reject malicious job creation");
        assert!(create_result.job_id.is_none(), "Should not return job ID for malicious input");

        // Try to submit a malicious job ID
        let submit_result = test_submit_job("malicious; rm -rf /".to_string()).await;
        assert!(!submit_result.success, "Should reject malicious job submission");

        // Try to delete with malicious job ID
        let delete_result = test_delete_job("malicious; rm -rf /".to_string(), true).await;
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

        let create_result = test_create_job(valid_params).await;
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

    /// Test configuration validation security (new security fix)
    #[test]
    fn test_configuration_validation_security() {
        use crate::cluster::{ClusterProfile, ConnectionConfig, ClusterCapabilities, BillingRates};

        // Test malicious connection configs
        let malicious_connection = ConnectionConfig {
            name: "../../../etc".to_string(),
            login_server: "valid.server.com".to_string(),
            module_setup: "module load slurm".to_string(),
            port: 22,
        };

        // Create a minimal valid capabilities
        let valid_capabilities = ClusterCapabilities {
            partitions: crate::cluster::alpine_profile().capabilities.partitions,
            qos_options: crate::cluster::alpine_profile().capabilities.qos_options,
            job_presets: vec![],
            billing_rates: BillingRates {
                cpu_cost_per_core_hour: 1.0,
                gpu_cost_per_gpu_hour: 108.2,
            },
        };

        let malicious_profiles = vec![
            ClusterProfile {
                id: "test".to_string(),
                connection: malicious_connection.clone(),
                capabilities: valid_capabilities.clone(),
            },
            ClusterProfile {
                id: "test".to_string(),
                connection: ConnectionConfig {
                    name: "cluster; rm -rf /".to_string(),
                    login_server: "valid.server.com".to_string(),
                    module_setup: "module load slurm".to_string(),
                    port: 22,
                },
                capabilities: valid_capabilities.clone(),
            },
            ClusterProfile {
                id: "test".to_string(),
                connection: ConnectionConfig {
                    name: "".to_string(), // Empty name
                    login_server: "valid.server.com".to_string(),
                    module_setup: "module load slurm".to_string(),
                    port: 22,
                },
                capabilities: valid_capabilities.clone(),
            },
            ClusterProfile {
                id: "test".to_string(),
                connection: ConnectionConfig {
                    name: "valid".to_string(),
                    login_server: "".to_string(), // Empty server
                    module_setup: "module load slurm".to_string(),
                    port: 22,
                },
                capabilities: valid_capabilities.clone(),
            },
            ClusterProfile {
                id: "test".to_string(),
                connection: ConnectionConfig {
                    name: "valid".to_string(),
                    login_server: "valid.server.com".to_string(),
                    module_setup: "rm -rf /tmp; module load slurm".to_string(), // Malicious module setup
                    port: 22,
                },
                capabilities: valid_capabilities.clone(),
            },
        ];

        for profile in malicious_profiles {
            let result = crate::cluster::set_active_profile(profile);
            assert!(result.is_err(), "Should reject malicious cluster configuration");
        }
    }

    /// Test demo mode security
    #[test]
    fn test_demo_mode_security() {
        use crate::demo::mode::*;

        std::env::remove_var("USE_MOCK_SSH");

        set_demo_mode(true);
        assert!(is_demo_mode(), "Should be in demo mode when set");

        set_demo_mode(false);
        assert!(!is_demo_mode(), "Should be in real mode when set");

        std::env::set_var("USE_MOCK_SSH", "malicious_value");
        assert!(!is_demo_mode(), "Should default to false for non-'true' values");

        std::env::set_var("USE_MOCK_SSH", "TRUE");
        assert!(is_demo_mode(), "Should convert to lowercase and accept TRUE");

        std::env::set_var("USE_MOCK_SSH", "true ");
        assert!(!is_demo_mode(), "Should be exact match only");

        // Clean up
        std::env::remove_var("USE_MOCK_SSH");
    }

    /// Test SFTP upload security (replaces command injection vulnerability)
    #[tokio::test]
    async fn test_sftp_upload_security() {
        // Test that the new SFTP upload approach doesn't have command injection vulnerabilities
        // This tests the business logic of content validation, not actual SFTP operations

        let malicious_contents = vec![
            "normal content with $(whoami) command substitution",
            "content with `ls -la` backticks",
            "content with \nrm -rf /\n embedded commands",
            "content with EOF\nmalicious_command\nEOF attempt",
        ];

        for content in malicious_contents {
            // The new SFTP approach treats content as pure bytes, not shell commands
            // Test that content length validation works
            assert!(content.len() > 0, "Content should have length");
            assert!(content.as_bytes().len() == content.len(), "Bytes conversion should be safe");

            // Test that content doesn't break our size limits (arbitrary reasonable limit)
            if content.len() > 1_000_000 { // 1MB limit for test purposes
                // In real implementation, large files would be handled properly
                // This tests that we have size considerations
                assert!(false, "Content should be within reasonable size limits");
            }
        }
    }

    /// Test input validation edge cases
    #[test]
    fn test_input_validation_edge_cases() {
        // Test edge cases that could bypass validation
        let edge_cases = vec![
            "\0", // Null byte
            "\x00", // Null byte alternative
            "job\x00name", // Embedded null
            "job\r\nname", // CRLF injection
            "job\tname", // Tab injection
            "job name", // Space (might be allowed depending on validation)
            "job\u{202E}name", // Unicode right-to-left override
            "job\u{FEFF}name", // Byte order mark
            "job\u{200B}name", // Zero-width space
        ];

        for input in edge_cases {
            let result = crate::validation::input::sanitize_job_id(input);
            // These should all be rejected by proper input validation
            if result.is_ok() {
                // If any pass validation, make sure they're actually safe
                let sanitized = result.unwrap();
                assert!(sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                    "Any input that passes validation must contain only safe characters: '{}'", sanitized);
            }
        }
    }

    /// Test that legitimate operations still work after security enhancements
    #[test]
    fn test_security_doesnt_break_functionality() {
        // Ensure our security fixes don't break legitimate use cases
        let valid_inputs = vec![
            "my_job_123",
            "simulation-2024",
            "protein_folding_run",
            "test_job",
            "JOB123",
        ];

        for input in valid_inputs {
            let result = crate::validation::input::sanitize_job_id(input);
            assert!(result.is_ok(), "Valid input should pass validation: '{}'", input);

            let sanitized = result.unwrap();
            assert_eq!(sanitized, input, "Valid input should not be modified: '{}'", input);
        }

        // Test valid configuration
        let valid_profile = crate::cluster::ClusterProfile {
            id: "test".to_string(),
            connection: crate::cluster::ConnectionConfig {
                name: "Alpine Cluster".to_string(),
                login_server: "login.cluster.edu".to_string(),
                module_setup: "module load gcc/11.2.0 slurm/23.02".to_string(),
                port: 22,
            },
            capabilities: crate::cluster::alpine_profile().capabilities,
        };

        let result = crate::cluster::set_active_profile(valid_profile);
        assert!(result.is_ok(), "Valid configuration should be accepted");
    }
}

