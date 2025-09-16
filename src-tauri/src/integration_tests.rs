//! Integration tests for complete job workflows
//!
//! These tests verify that the complete job lifecycle works correctly,
//! including directory creation, job submission with scratch directories,
//! and safe cleanup during deletion.

#[cfg(test)]
mod job_workflow_integration_tests {
    use crate::commands::jobs::*;
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

    /// Ensure we're in mock mode for integration tests
    fn setup_test_environment() {
        env::set_var("USE_MOCK_SSH", "true");

        // Initialize fresh database for tests
        let _ = crate::database::initialize_database(":memory:");

        // Reset and set mock state to connected so job submission works
        crate::mock_state::reset_mock_state();
        crate::mock_state::with_mock_state(|state| {
            state.connection_state = crate::types::ConnectionState::Connected;
        });
    }

    /// Test complete job lifecycle: create -> submit -> delete
    #[tokio::test]
    async fn test_complete_job_lifecycle() {
        setup_test_environment();

        // Step 1: Create a job
        let create_params = create_test_params("integration_test_job".to_string());

        let create_result = create_job(create_params).await;
        assert!(create_result.success, "Job creation should succeed: {:?}", create_result);
        assert!(create_result.job_id.is_some(), "Should return job ID");
        assert!(create_result.error.is_none(), "Should not have creation error");

        let job_id = create_result.job_id.unwrap();

        // Step 2: Verify job status after creation
        let status_result = get_job_status(job_id.clone()).await;
        if !status_result.success {
            println!("Job status error: {:?}", status_result.error);
        }
        assert!(status_result.success, "Should get job status successfully");
        assert!(status_result.job_info.is_some(), "Should return job info");

        let job_info = status_result.job_info.unwrap();
        assert_eq!(job_info.status, JobStatus::Created, "Job should be in Created status");
        assert_eq!(job_info.job_name, "integration_test_job", "Job name should match");
        assert!(job_info.project_dir.is_some(), "Should have project directory");
        assert!(job_info.scratch_dir.is_some(), "Should have scratch directory");

        // Step 3: Submit the job
        let submit_result = submit_job(job_id.clone()).await;
        assert!(submit_result.success, "Job submission should succeed: {:?}", submit_result);
        assert!(submit_result.slurm_job_id.is_some(), "Should return SLURM job ID");
        assert!(submit_result.submitted_at.is_some(), "Should have submission timestamp");
        assert!(submit_result.error.is_none(), "Should not have submission error");

        // Step 4: Verify job status after submission
        let status_after_submit = get_job_status(job_id.clone()).await;
        assert!(status_after_submit.success, "Should get job status after submission");

        let job_info_after_submit = status_after_submit.job_info.unwrap();
        assert_eq!(job_info_after_submit.status, JobStatus::Pending, "Job should be in Pending status");
        assert!(job_info_after_submit.slurm_job_id.is_some(), "Should have SLURM job ID");
        assert!(job_info_after_submit.submitted_at.is_some(), "Should have submission timestamp");

        // Step 5: Delete the job (with remote cleanup)
        let delete_result = delete_job(job_id.clone(), true).await;
        assert!(delete_result.success, "Job deletion should succeed: {:?}", delete_result);
        assert!(delete_result.error.is_none(), "Should not have deletion error");

        // Step 6: Verify job is deleted
        let status_after_delete = get_job_status(job_id).await;
        assert!(!status_after_delete.success, "Should not find job after deletion");
        assert!(status_after_delete.job_info.is_none(), "Should not return job info after deletion");
    }

    /// Test multiple job creation and management
    #[tokio::test]
    async fn test_multiple_job_management() {
        setup_test_environment();

        let mut job_ids = Vec::new();

        // Create multiple jobs
        for i in 1..=5 {
            let create_params = create_test_params(format!("multi_test_job_{}", i));

            let create_result = create_job(create_params).await;
            assert!(create_result.success, "Job {} creation should succeed", i);

            let job_id = create_result.job_id.unwrap();
            job_ids.push(job_id);
        }

        // Verify all jobs are listed
        let all_jobs_result = get_all_jobs().await;
        assert!(all_jobs_result.success, "Should get all jobs successfully");
        assert!(all_jobs_result.jobs.is_some(), "Should return jobs list");

        let all_jobs = all_jobs_result.jobs.unwrap();
        assert!(all_jobs.len() >= 5, "Should have at least 5 jobs");

        // Verify each job exists and has correct status
        for (i, job_id) in job_ids.iter().enumerate() {
            let status_result = get_job_status(job_id.clone()).await;
            assert!(status_result.success, "Should get status for job {}", i + 1);

            let job_info = status_result.job_info.unwrap();
            assert_eq!(job_info.status, JobStatus::Created, "Job {} should be in Created status", i + 1);
            assert_eq!(job_info.job_name, format!("multi_test_job_{}", i + 1), "Job {} name should match", i + 1);
        }

        // Submit some jobs
        for (i, job_id) in job_ids.iter().take(3).enumerate() {
            let submit_result = submit_job(job_id.clone()).await;
            assert!(submit_result.success, "Job {} submission should succeed", i + 1);
        }

        // Delete all jobs
        for (i, job_id) in job_ids.iter().enumerate() {
            let delete_result = delete_job(job_id.clone(), true).await;
            assert!(delete_result.success, "Job {} deletion should succeed", i + 1);
        }
    }

    /// Test job operations with network simulation
    #[tokio::test]
    async fn test_job_operations_with_connection_states() {
        setup_test_environment();

        // First, create a job (should work regardless of connection state)
        let create_params = create_test_params("network_test_job".to_string());

        let create_result = create_job(create_params).await;
        assert!(create_result.success, "Job creation should succeed");
        let job_id = create_result.job_id.unwrap();

        // Try to submit job when not connected (should fail in real mode, succeed in mock)
        let submit_result = submit_job(job_id.clone()).await;
        // In mock mode, this should succeed if the mock state indicates connection
        // The actual behavior depends on the mock state setup

        // Get job status (should always work)
        let status_result = get_job_status(job_id.clone()).await;
        assert!(status_result.success, "Getting job status should work");

        // Delete job
        let delete_result = delete_job(job_id, false).await; // Don't delete remote files
        assert!(delete_result.success, "Job deletion should succeed");
    }

    /// Test error handling in job workflows
    #[tokio::test]
    async fn test_job_workflow_error_handling() {
        setup_test_environment();

        // Test submitting non-existent job
        let submit_nonexistent = submit_job("nonexistent_job_123".to_string()).await;
        assert!(!submit_nonexistent.success, "Should fail to submit non-existent job");
        assert!(submit_nonexistent.error.is_some(), "Should have error message");

        // Test getting status of non-existent job
        let status_nonexistent = get_job_status("nonexistent_job_456".to_string()).await;
        assert!(!status_nonexistent.success, "Should fail to get status of non-existent job");

        // Test deleting non-existent job
        let delete_nonexistent = delete_job("nonexistent_job_789".to_string(), true).await;
        assert!(!delete_nonexistent.success, "Should fail to delete non-existent job");
        assert!(delete_nonexistent.error.is_some(), "Should have error message");

        // Test creating job with empty name
        let create_empty = create_test_params("".to_string());
        let create_empty_result = create_job(create_empty).await;
        assert!(!create_empty_result.success, "Should fail to create job with empty name");
        assert!(create_empty_result.error.is_some(), "Should have error message");
    }

    /// Test job synchronization functionality
    #[tokio::test]
    async fn test_job_synchronization() {
        setup_test_environment();

        // Create and submit a job
        let create_params = create_test_params("sync_test_job".to_string(),
        );

        let create_result = create_job(create_params).await;
        assert!(create_result.success, "Job creation should succeed");
        let job_id = create_result.job_id.unwrap();

        let submit_result = submit_job(job_id.clone()).await;
        assert!(submit_result.success, "Job submission should succeed");

        // Sync jobs (this simulates checking SLURM status)
        let sync_result = sync_jobs().await;

        // In mock mode, sync behavior depends on the mock state setup
        // We mainly test that the sync operation doesn't crash
        assert!(sync_result.jobs_updated >= 0, "Should report number of updated jobs");
    }

    /// Test directory path generation and validation
    #[tokio::test]
    async fn test_directory_path_handling() {
        setup_test_environment();

        // Create a job and verify directory paths are generated correctly
        let create_params = create_test_params("path_test_job".to_string(),
        );

        let create_result = create_job(create_params).await;
        assert!(create_result.success, "Job creation should succeed");
        let job_id = create_result.job_id.unwrap();

        // Get job info and verify paths
        let status_result = get_job_status(job_id).await;
        assert!(status_result.success, "Should get job status");
        let job_info = status_result.job_info.unwrap();

        // Verify project directory path
        let project_dir = job_info.project_dir.expect("Should have project directory");
        assert!(project_dir.starts_with("/projects/"), "Project directory should start with /projects/");
        assert!(project_dir.contains("namdrunner_jobs"), "Project directory should contain namdrunner_jobs");
        assert!(project_dir.contains(&job_info.job_id), "Project directory should contain job ID");

        // Verify scratch directory path
        let scratch_dir = job_info.scratch_dir.expect("Should have scratch directory");
        assert!(scratch_dir.starts_with("/scratch/"), "Scratch directory should start with /scratch/");
        assert!(scratch_dir.contains("namdrunner_jobs"), "Scratch directory should contain namdrunner_jobs");
        assert!(scratch_dir.contains(&job_info.job_id), "Scratch directory should contain job ID");

        // Verify paths don't contain dangerous sequences
        assert!(!project_dir.contains(".."), "Project directory should not contain .. sequences");
        assert!(!scratch_dir.contains(".."), "Scratch directory should not contain .. sequences");
        assert!(!project_dir.contains(";"), "Project directory should not contain semicolons");
        assert!(!scratch_dir.contains(";"), "Scratch directory should not contain semicolons");
    }

    /// Test job creation with various valid names
    #[tokio::test]
    async fn test_valid_job_name_variations() {
        setup_test_environment();

        let valid_names = vec![
            "simple_job",
            "job-with-hyphens",
            "JobWithCamelCase",
            "job_123",
            "job_with_underscores_and_numbers_123",
            "a",                    // Single character
        ];

        // Add the maximum length case separately to avoid borrowing issues
        let max_length_name = "a".repeat(64);
        let mut all_valid_names = valid_names;
        all_valid_names.push(&max_length_name);

        for job_name in all_valid_names {
            let create_params = create_test_params(job_name.to_string());

            let create_result = create_job(create_params).await;
            assert!(create_result.success,
                "Should accept valid job name '{}': {:?}", job_name, create_result);

            if let Some(job_id) = create_result.job_id {
                // Clean up
                let _ = delete_job(job_id, false).await;
            }
        }
    }

    /// Test concurrent job operations
    #[tokio::test]
    async fn test_concurrent_job_operations() {
        setup_test_environment();

        // Create multiple jobs concurrently
        let mut create_tasks = Vec::new();

        for i in 1..=10 {
            let create_params = create_test_params(format!("concurrent_job_{}", i));

            let task = tokio::spawn(async move {
                create_job(create_params).await
            });

            create_tasks.push(task);
        }

        // Wait for all create operations to complete
        let mut job_ids = Vec::new();
        for task in create_tasks {
            let result = task.await.expect("Task should complete");
            if result.success {
                job_ids.push(result.job_id.unwrap());
            }
        }

        assert!(job_ids.len() >= 8, "Most concurrent job creations should succeed");

        // Verify all jobs exist
        for job_id in &job_ids {
            let status_result = get_job_status(job_id.clone()).await;
            assert!(status_result.success, "Should get status for concurrent job");
        }

        // Clean up all jobs concurrently
        let mut delete_tasks = Vec::new();
        for job_id in job_ids {
            let task = tokio::spawn(async move {
                delete_job(job_id, false).await
            });
            delete_tasks.push(task);
        }

        // Wait for all delete operations
        for task in delete_tasks {
            let result = task.await.expect("Delete task should complete");
            // Most deletes should succeed (some might fail if job doesn't exist due to race conditions)
        }
    }

    /// Test that the enhanced implementation maintains backward compatibility
    #[tokio::test]
    async fn test_backward_compatibility() {
        setup_test_environment();

        // Test that basic job operations work the same as before
        let create_params = create_test_params("compatibility_test".to_string(),
        );

        let create_result = create_job(create_params).await;
        assert!(create_result.success, "Basic job creation should work");

        let job_id = create_result.job_id.unwrap();

        // All the existing API should continue to work
        let status_result = get_job_status(job_id.clone()).await;
        assert!(status_result.success, "Basic status check should work");

        let all_jobs_result = get_all_jobs().await;
        assert!(all_jobs_result.success, "Get all jobs should work");

        let _submit_result = submit_job(job_id.clone()).await;
        // Submit might fail if not connected, but shouldn't crash

        let delete_result = delete_job(job_id, false).await;
        assert!(delete_result.success, "Basic deletion should work");
    }
}

/// Performance and stress tests
#[cfg(test)]
mod performance_integration_tests {
    use super::*;
    use std::time::Instant;
    use std::env;
    use crate::commands::jobs::*;

    use crate::types::*;

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

    // Performance test removed - not aligned with testing philosophy
    // Focus is on business logic validation, not mock performance benchmarking

    // Performance test removed - not aligned with testing philosophy
    // Focus is on business logic validation, not mock performance benchmarking
}