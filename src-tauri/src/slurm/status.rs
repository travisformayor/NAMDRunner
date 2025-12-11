use crate::types::JobStatus;
use crate::ssh::{get_connection_manager, retry_quick};
use super::commands::*;
use crate::log_warn;
use anyhow::{Result, anyhow};

pub struct SlurmStatusSync {}

impl SlurmStatusSync {
    pub fn new(_username: &str) -> Self {
        Self {}
    }

    /// Query SLURM for job statuses
    /// Returns Vec of (job_id, Result<JobStatus>) for each queried job
    ///
    /// Uses consistent job_id|status format from both squeue (active) and sacct (completed)
    pub async fn query_job_statuses(&self, job_ids: &[String]) -> Result<Vec<(String, Result<JobStatus>)>> {
        if job_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();

        // Query active jobs with squeue
        let squeue_cmd = squeue_command(job_ids)?;
        let squeue_result = retry_quick(|| {
            let cmd = squeue_cmd.clone();
            async move {
                let connection_manager = get_connection_manager();
                connection_manager.execute_command(&cmd, Some(crate::cluster::timeouts::SLURM_OPERATION)).await
                    .map_err(|e| anyhow!("SLURM squeue failed: {}", e))
            }
        }).await?;

        // Parse squeue output (format: job_id|status per line)
        for line in squeue_result.stdout.lines() {
            if let Some((job_id, status)) = Self::parse_status_line(line) {
                results.push((job_id, Ok(status)));
            }
        }

        // Find jobs not in active queue (need sacct for completed jobs)
        let found_job_ids: std::collections::HashSet<_> = results.iter().map(|(id, _)| id.as_str()).collect();
        let missing_jobs: Vec<String> = job_ids.iter()
            .filter(|id| !found_job_ids.contains(id.as_str()))
            .cloned()
            .collect();

        if !missing_jobs.is_empty() {
            // Query completed jobs with sacct
            let sacct_cmd = sacct_command(&missing_jobs)?;
            let sacct_result = retry_quick(|| {
                let cmd = sacct_cmd.clone();
                async move {
                    let connection_manager = get_connection_manager();
                    connection_manager.execute_command(&cmd, Some(crate::cluster::timeouts::SLURM_OPERATION)).await
                        .map_err(|e| anyhow!("SLURM sacct failed: {}", e))
                }
            }).await?;

            // Parse sacct output (format: job_id|status per line with --parsable2)
            for line in sacct_result.stdout.lines() {
                if let Some((job_id, status)) = Self::parse_status_line(line) {
                    results.push((job_id, Ok(status)));
                }
            }
        }

        // For any jobs still not found, add error results
        let final_found: std::collections::HashSet<String> = results.iter().map(|(id, _)| id.clone()).collect();
        for job_id in job_ids {
            if !final_found.contains(job_id) {
                results.push((job_id.clone(), Err(anyhow!("Job {} not found in SLURM queue or history", job_id))));
            }
        }

        Ok(results)
    }

    /// Parse a single line of job_id|status format
    fn parse_status_line(line: &str) -> Option<(String, JobStatus)> {
        let (job_id, status_str) = line.split_once('|')?;
        let status = Self::parse_status_code(status_str).ok()?;
        Some((job_id.to_string(), status))
    }

    /// Parse SLURM status code to JobStatus
    fn parse_status_code(status: &str) -> Result<JobStatus> {
        let status = status.trim().to_uppercase();

        match status.as_str() {
            // Pending states
            "PD" | "PENDING" => Ok(JobStatus::Pending),

            // Running states
            "R" | "RUNNING" => Ok(JobStatus::Running),
            "CG" | "COMPLETING" => Ok(JobStatus::Running),

            // Completed states
            "CD" | "COMPLETED" => Ok(JobStatus::Completed),

            // Failed states
            "F" | "FAILED" => Ok(JobStatus::Failed),
            "CA" | "CANCELLED" => Ok(JobStatus::Cancelled),
            "TO" | "TIMEOUT" => Ok(JobStatus::Failed),
            "NF" | "NODE_FAIL" => Ok(JobStatus::Failed),
            "PR" | "PREEMPTED" => Ok(JobStatus::Failed),

            // Memory/Resource failures
            "OOM" | "OUT_OF_MEMORY" | "OUT_OF_ME+" => Ok(JobStatus::Failed),

            // System failures
            "BF" | "BOOT_FAIL" => Ok(JobStatus::Failed),
            "DL" | "DEADLINE" => Ok(JobStatus::Failed),

            // Handle unknown states
            _ => {
                log_warn!(
                    category: "SLURM",
                    message: "Unknown SLURM status",
                    details: "Unknown SLURM status: {}", status
                );
                Err(anyhow!("Unknown SLURM status: {}", status))
            }
        }
    }

    pub async fn cancel_job(&self, slurm_job_id: &str) -> Result<()> {
        let scancel_cmd = cancel_job_command(slurm_job_id)?;

        let result = retry_quick(|| {
            let cmd = scancel_cmd.clone();
            async move {
                let connection_manager = get_connection_manager();
                connection_manager.execute_command(&cmd, Some(crate::cluster::timeouts::SLURM_OPERATION)).await
                    .map_err(|e| anyhow!("SLURM scancel failed: {}", e))
            }
        }).await?;

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
    fn test_status_code_parsing() {
        // Test all documented SLURM status codes
        assert_eq!(SlurmStatusSync::parse_status_code("PD").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_status_code("PENDING").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_status_code("R").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_status_code("RUNNING").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_status_code("CG").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_status_code("COMPLETING").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_status_code("CD").unwrap(), JobStatus::Completed);
        assert_eq!(SlurmStatusSync::parse_status_code("COMPLETED").unwrap(), JobStatus::Completed);
        assert_eq!(SlurmStatusSync::parse_status_code("F").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("FAILED").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("CA").unwrap(), JobStatus::Cancelled);
        assert_eq!(SlurmStatusSync::parse_status_code("CANCELLED").unwrap(), JobStatus::Cancelled);
        assert_eq!(SlurmStatusSync::parse_status_code("TO").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("TIMEOUT").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("NF").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("NODE_FAIL").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("PR").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("PREEMPTED").unwrap(), JobStatus::Failed);

        // Memory/resource failures
        assert_eq!(SlurmStatusSync::parse_status_code("OOM").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("OUT_OF_MEMORY").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("OUT_OF_ME+").unwrap(), JobStatus::Failed);

        // System failures
        assert_eq!(SlurmStatusSync::parse_status_code("BF").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("BOOT_FAIL").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("DL").unwrap(), JobStatus::Failed);
        assert_eq!(SlurmStatusSync::parse_status_code("DEADLINE").unwrap(), JobStatus::Failed);

        // Error cases
        assert!(SlurmStatusSync::parse_status_code("UNKNOWN").is_err());
        assert!(SlurmStatusSync::parse_status_code("").is_err());
    }

    #[test]
    fn test_status_line_parsing() {
        // Test the job_id|status format parsing
        let result = SlurmStatusSync::parse_status_line("12345678|RUNNING");
        assert!(result.is_some());
        let (job_id, status) = result.unwrap();
        assert_eq!(job_id, "12345678");
        assert_eq!(status, JobStatus::Running);

        // Test completed status
        let result = SlurmStatusSync::parse_status_line("99999|COMPLETED");
        assert!(result.is_some());
        let (job_id, status) = result.unwrap();
        assert_eq!(job_id, "99999");
        assert_eq!(status, JobStatus::Completed);

        // Test invalid format
        assert!(SlurmStatusSync::parse_status_line("no-pipe-here").is_none());
        assert!(SlurmStatusSync::parse_status_line("12345|INVALID_STATUS").is_none());
    }

    #[test]
    fn test_case_insensitive_parsing() {
        assert_eq!(SlurmStatusSync::parse_status_code("pd").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_status_code("Pd").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_status_code("PD").unwrap(), JobStatus::Pending);
        assert_eq!(SlurmStatusSync::parse_status_code("running").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_status_code("RUNNING").unwrap(), JobStatus::Running);
        assert_eq!(SlurmStatusSync::parse_status_code("completed").unwrap(), JobStatus::Completed);
        assert_eq!(SlurmStatusSync::parse_status_code("COMPLETED").unwrap(), JobStatus::Completed);
    }

    #[test]
    fn test_command_format_consistency() {
        // Verify squeue uses consistent format
        let cmd = squeue_command(&["12345".to_string()]).unwrap();
        assert!(cmd.contains("--format='%i|%T'"));
        assert!(cmd.contains("--noheader"));

        // Verify sacct uses consistent format
        let cmd = sacct_command(&["12345".to_string()]).unwrap();
        assert!(cmd.contains("--format=JobID,State"));
        assert!(cmd.contains("--parsable2"));
        assert!(cmd.contains("--noheader"));
    }
}
