/// SLURM command builders for NAMDRunner
/// Simple, direct functions following CONTRIBUTING.md philosophy
///
/// All functions:
/// - Use input sanitization to prevent command injection
/// - Return Result for error handling
/// - Generate commands only (no execution)

use crate::validation::input;
use anyhow::Result;

/// Build squeue command for a specific job
#[allow(dead_code)]
pub fn squeue_job(job_id: &str) -> Result<String> {
    let clean_id = input::sanitize_job_id(job_id)?;
    Ok(format!("squeue -j {} --format='%i|%T|%M|%l|%S|%e' --noheader", clean_id))
}

/// Build squeue command for all user jobs
#[allow(dead_code)]
pub fn squeue_user(username: &str) -> Result<String> {
    let clean_username = input::sanitize_username(username)?;
    Ok(format!("squeue -u {} --format='%i|%T|%M|%l|%S|%e' --noheader", clean_username))
}

/// Build sacct command for single job
#[allow(dead_code)]
pub fn sacct_job(job_id: &str) -> Result<String> {
    let clean_id = input::sanitize_job_id(job_id)?;
    Ok(format!(
        "sacct -j {} --format=JobID,JobName,State,ExitCode,Elapsed,Start,End --parsable2 --noheader",
        clean_id
    ))
}

/// Build sacct command for multiple jobs (batch query)
#[allow(dead_code)]
pub fn sacct_batch(job_ids: &[String]) -> Result<String> {
    let sanitized_ids: Result<Vec<_>> = job_ids.iter()
        .map(|id| input::sanitize_job_id(id))
        .collect();
    let job_list = sanitized_ids?.join(",");
    Ok(format!(
        "sacct -j {} --format=JobID,State --parsable2 --noheader",
        job_list
    ))
}

/// Build scancel command for job cancellation
#[allow(dead_code)]
pub fn scancel_job(job_id: &str) -> Result<String> {
    let clean_id = input::sanitize_job_id(job_id)?;
    Ok(format!("scancel {}", clean_id))
}

/// Build sbatch submit command with directory change
///
/// This uses safe shell escaping for the directory path and submits
/// the specified script in that directory.
pub fn submit_job_command(scratch_dir: &str, script_name: &str) -> Result<String> {
    // Use safe shell escaping for directory
    Ok(crate::validation::shell::safe_cd_and_run(scratch_dir, &format!("sbatch {}", script_name)))
}

/// Parse sbatch output to extract SLURM job ID
///
/// Expected format: "Submitted batch job 12345678"
/// Returns None if output doesn't match expected format
pub fn parse_sbatch_output(output: &str) -> Option<String> {
    output.lines()
        .find(|line| line.trim().starts_with("Submitted batch job"))
        .and_then(|line| line.split_whitespace().last())
        .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
        .map(String::from)
}

// Helper functions for slurm/status.rs compatibility
// These wrap the simple functions above to match what status.rs expects

/// Get job status command - single job
#[allow(dead_code)]
pub fn job_status_command(job_id: &str) -> Result<String> {
    squeue_job(job_id)
}

/// Get job status command - multiple jobs
#[allow(dead_code)]
pub fn batch_job_status_command(job_ids: &[String]) -> Result<String> {
    let sanitized_ids: Result<Vec<_>> = job_ids.iter()
        .map(|id| input::sanitize_job_id(id))
        .collect();
    let job_list = sanitized_ids?.join(",");
    Ok(format!("squeue -j {} --format='%i|%T' --noheader", job_list))
}

/// Get completed job status command
#[allow(dead_code)]
pub fn completed_job_status_command(job_id: &str) -> Result<String> {
    let clean_id = input::sanitize_job_id(job_id)?;
    Ok(format!("sacct -j {} --format=State --parsable2 --noheader", clean_id))
}

/// Get completed job status command - multiple jobs
#[allow(dead_code)]
pub fn batch_completed_job_status_command(job_ids: &[String]) -> Result<String> {
    sacct_batch(job_ids)
}

/// Cancel job command - single job
#[allow(dead_code)]
pub fn cancel_job_command(job_id: &str) -> Result<String> {
    scancel_job(job_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squeue_commands() {
        let cmd = squeue_job("12345").unwrap();
        assert!(cmd.contains("squeue -j 12345"));
        assert!(cmd.contains("--format='%i|%T|%M|%l|%S|%e'"));
        assert!(cmd.contains("--noheader"));

        let cmd2 = squeue_user("testuser").unwrap();
        assert!(cmd2.contains("squeue -u testuser"));
        assert!(cmd2.contains("--format='%i|%T|%M|%l|%S|%e'"));
    }

    #[test]
    fn test_sacct_commands() {
        let cmd = sacct_job("12345").unwrap();
        assert!(cmd.contains("sacct -j 12345"));
        assert!(cmd.contains("--format=JobID,JobName,State,ExitCode,Elapsed,Start,End"));
        assert!(cmd.contains("--parsable2"));
        assert!(cmd.contains("--noheader"));
    }

    #[test]
    fn test_sacct_batch() {
        let job_ids = vec!["12345".to_string(), "67890".to_string()];
        let cmd = sacct_batch(&job_ids).unwrap();
        assert!(cmd.contains("sacct -j 12345,67890"));
        assert!(cmd.contains("--format=JobID,State"));
    }

    #[test]
    fn test_scancel() {
        let cmd = scancel_job("12345").unwrap();
        assert_eq!(cmd, "scancel 12345");
    }

    #[test]
    fn test_submit_job_command() {
        let cmd = submit_job_command("/scratch/test", "job.sbatch").unwrap();
        assert!(cmd.contains("cd '/scratch/test'"));
        assert!(cmd.contains("sbatch job.sbatch"));
    }

    #[test]
    fn test_parse_sbatch() {
        assert_eq!(
            parse_sbatch_output("Submitted batch job 12345678"),
            Some("12345678".to_string())
        );

        assert_eq!(
            parse_sbatch_output("  Submitted batch job   98765  \n"),
            Some("98765".to_string())
        );

        assert_eq!(parse_sbatch_output("Error: permission denied"), None);
        assert_eq!(parse_sbatch_output(""), None);
        assert_eq!(parse_sbatch_output("Submitted batch job abc"), None);
    }

    #[test]
    fn test_input_sanitization() {
        // Malicious inputs should be rejected
        assert!(squeue_job("12345; rm -rf /").is_err());
        assert!(scancel_job("../../etc/passwd").is_err());
        assert!(squeue_user("user$(whoami)").is_err());

        // Valid inputs should work
        assert!(squeue_job("12345").is_ok());
        assert!(squeue_user("testuser").is_ok());
    }

    #[test]
    fn test_batch_job_status_command() {
        let job_ids = vec!["12345".to_string(), "67890".to_string()];
        let cmd = batch_job_status_command(&job_ids).unwrap();
        assert!(cmd.contains("squeue -j 12345,67890"));
        assert!(cmd.contains("--format='%i|%T'"));
    }

    #[test]
    fn test_compatibility_functions() {
        // Test that compatibility functions work correctly
        assert!(job_status_command("12345").is_ok());
        assert!(completed_job_status_command("12345").is_ok());
        assert!(cancel_job_command("12345").is_ok());

        let job_ids = vec!["12345".to_string()];
        assert!(batch_job_status_command(&job_ids).is_ok());
        assert!(batch_completed_job_status_command(&job_ids).is_ok());
    }
}
