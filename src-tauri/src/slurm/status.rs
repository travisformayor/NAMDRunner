use crate::types::JobStatus;
use crate::ssh::get_connection_manager;
use super::commands::*;
use anyhow::{Result, anyhow};

pub struct SlurmStatusSync {
    username: String,
}

impl SlurmStatusSync {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
        }
    }

    pub async fn sync_job_status(&self, slurm_job_id: &str) -> Result<JobStatus> {
        // Use command builder for consistent SLURM command construction
        let squeue_cmd = job_status_command(slurm_job_id);

        let connection_manager = get_connection_manager();
        let result = connection_manager.execute_command(&squeue_cmd, Some(60)).await;

        match result {
            Ok(cmd_result) => {
                // If job found in active queue, parse its status
                if !cmd_result.stdout.trim().is_empty() {
                    return Self::parse_slurm_status(&cmd_result.stdout);
                }

                // If not in active queue, check completed jobs with sacct
                self.check_completed_job(slurm_job_id).await
            }
            Err(e) => {
                // If squeue fails, try sacct as fallback
                log::warn!("squeue failed for job {}: {}, trying sacct", slurm_job_id, e);
                self.check_completed_job(slurm_job_id).await
            }
        }
    }

    async fn check_completed_job(&self, slurm_job_id: &str) -> Result<JobStatus> {
        // Use command builder for consistent SLURM command construction
        let sacct_cmd = completed_job_status_command(slurm_job_id);

        let connection_manager = get_connection_manager();
        let result = connection_manager.execute_command(&sacct_cmd, Some(60)).await?;

        if result.stdout.trim().is_empty() {
            // Job not found in sacct either - might be older than 7 days or invalid job ID
            return Err(anyhow!("Job {} not found in SLURM queue or history", slurm_job_id));
        }

        Self::parse_slurm_status(&result.stdout)
    }

    fn parse_slurm_status(output: &str) -> Result<JobStatus> {
        // Handle all SLURM states from reference documentation
        let status = output.trim().to_uppercase();

        match status.as_str() {
            // Pending states
            "PD" | "PENDING" => Ok(JobStatus::Pending),

            // Running states
            "R" | "RUNNING" => Ok(JobStatus::Running),
            "CG" | "COMPLETING" => Ok(JobStatus::Running), // Still running, just cleaning up

            // Completed states
            "CD" | "COMPLETED" => Ok(JobStatus::Completed),

            // Failed states
            "F" | "FAILED" => Ok(JobStatus::Failed),
            "CA" | "CANCELLED" => Ok(JobStatus::Cancelled),
            "TO" | "TIMEOUT" => Ok(JobStatus::Failed),
            "NF" | "NODE_FAIL" => Ok(JobStatus::Failed),
            "PR" | "PREEMPTED" => Ok(JobStatus::Failed),

            // Handle unknown states
            _ => {
                log::warn!("Unknown SLURM status: {}", status);
                Err(anyhow!("Unknown SLURM status: {}", status))
            }
        }
    }

    pub async fn sync_all_jobs(&self, job_ids: &[String]) -> Result<Vec<(String, Result<JobStatus>)>> {
        let mut results = Vec::new();

        if job_ids.is_empty() {
            return Ok(results);
        }

        // For efficiency, try to batch query all jobs at once
        let batch_result = self.batch_query_jobs(job_ids).await;

        match batch_result {
            Ok(batch_statuses) => {
                // Return successful batch results
                for (job_id, status) in batch_statuses {
                    results.push((job_id, Ok(status)));
                }
            }
            Err(_) => {
                // If batch fails, fall back to individual queries
                log::warn!("Batch SLURM query failed, falling back to individual queries");

                for job_id in job_ids {
                    let status_result = self.sync_job_status(job_id).await;
                    results.push((job_id.clone(), status_result));
                }
            }
        }

        Ok(results)
    }

    async fn batch_query_jobs(&self, job_ids: &[String]) -> Result<Vec<(String, JobStatus)>> {
        if job_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Use command builder for batch query
        let squeue_cmd = batch_job_status_command(job_ids);

        let mut results = Vec::new();

        // Query active jobs
        let connection_manager = get_connection_manager();
        if let Ok(cmd_result) = connection_manager.execute_command(&squeue_cmd, Some(60)).await {
            for line in cmd_result.stdout.lines() {
                if let Some((job_id, status_str)) = line.split_once('|') {
                    if let Ok(status) = Self::parse_slurm_status(status_str) {
                        results.push((job_id.to_string(), status));
                    }
                }
            }
        }

        // For jobs not found in active queue, check sacct
        let found_job_ids: std::collections::HashSet<_> = results.iter().map(|(id, _)| id.as_str()).collect();
        let missing_jobs: Vec<_> = job_ids.iter().filter(|id| !found_job_ids.contains(id.as_str())).collect();

        if !missing_jobs.is_empty() {
            let missing_job_strings: Vec<String> = missing_jobs.iter().map(|s| s.to_string()).collect();
            let sacct_cmd = batch_completed_job_status_command(&missing_job_strings);

            if let Ok(cmd_result) = connection_manager.execute_command(&sacct_cmd, Some(60)).await {
                for line in cmd_result.stdout.lines() {
                    if let Some((job_id, status_str)) = line.split_once('|') {
                        if let Ok(status) = Self::parse_slurm_status(status_str) {
                            results.push((job_id.to_string(), status));
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub async fn cancel_job(&self, slurm_job_id: &str) -> Result<()> {
        let scancel_cmd = cancel_job_command(slurm_job_id);

        let connection_manager = get_connection_manager();
        let result = connection_manager.execute_command(&scancel_cmd, Some(60)).await?;

        if result.exit_code != 0 {
            return Err(anyhow!("Failed to cancel job {}: {}", slurm_job_id, result.stderr));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slurm_status_parsing() {
        // Test all documented SLURM status codes
        assert_eq!(SlurmStatusSync::parse_slurm_status("PD").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_slurm_status("PENDING").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_slurm_status("R").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_slurm_status("RUNNING").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_slurm_status("CG").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_slurm_status("COMPLETING").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_slurm_status("CD").unwrap(), JobStatus::Completed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("COMPLETED").unwrap(), JobStatus::Completed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("F").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("FAILED").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("CA").unwrap(), JobStatus::Cancelled);
        assert_eq!(SlurmStatusSync::parse_slurm_status("CANCELLED").unwrap(), JobStatus::Cancelled);
        assert_eq!(SlurmStatusSync::parse_slurm_status("TO").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("TIMEOUT").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("NF").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("NODE_FAIL").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("PR").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("PREEMPTED").unwrap(), JobStatus::Failed);

        // Test error cases
        assert!(SlurmStatusSync::parse_slurm_status("UNKNOWN").is_err());
        assert!(SlurmStatusSync::parse_slurm_status("").is_err());
    }

    #[test]
    fn test_slurm_command_construction() {
        // Test squeue command format matches reference documentation using command builder
        let job_id = "12345678";
        let cmd = crate::slurm::commands::job_status_command(job_id);

        // This tests that the command builder creates commands matching the reference docs
        assert!(cmd.contains("source /etc/profile"));
        assert!(cmd.contains("module load slurm/alpine"));
        assert!(cmd.contains("squeue -j"));
        assert!(cmd.contains("--format='%T'"));
        assert!(cmd.contains("--noheader"));
    }

    #[test]
    fn test_sacct_command_construction() {
        let job_id = "12345678";
        let cmd = crate::slurm::commands::completed_job_status_command(job_id);

        // Test that sacct command matches reference documentation
        assert!(cmd.contains("source /etc/profile"));
        assert!(cmd.contains("module load slurm/alpine"));
        assert!(cmd.contains("sacct -j"));
        assert!(cmd.contains("--starttime=$(date -d '7 days ago' +%Y-%m-%d)"));
        assert!(cmd.contains("--format=State"));
        assert!(cmd.contains("--parsable2"));
        assert!(cmd.contains("--noheader"));
    }

    #[test]
    fn test_case_insensitive_parsing() {
        // Test that status parsing is case insensitive
        assert_eq!(SlurmStatusSync::parse_slurm_status("pd").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_slurm_status("Pd").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_slurm_status("PD").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_slurm_status("running").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_slurm_status("RUNNING").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_slurm_status("completed").unwrap(), JobStatus::Completed);
        assert_eq!(SlurmStatusSync::parse_slurm_status("COMPLETED").unwrap(), JobStatus::Completed);
    }
}