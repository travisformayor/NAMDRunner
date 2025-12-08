/// SLURM command builders for NAMDRunner
/// Simple, direct functions following CONTRIBUTING.md philosophy
///
/// All functions:
/// - Use input sanitization to prevent command injection
/// - Return Result for error handling
/// - Generate commands only (no execution)
use crate::validation::input;
use anyhow::Result;


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

// SLURM job status query commands

/// Get active job status using squeue
/// Accepts single job ID or multiple (single is just array of 1)
pub fn squeue_command(job_ids: &[String]) -> Result<String> {
    let sanitized_ids: Result<Vec<_>> = job_ids.iter()
        .map(|id| input::sanitize_job_id(id))
        .collect();
    let job_list = sanitized_ids?.join(",");

    // Use detailed format for single job, simpler format for batch
    if job_ids.len() == 1 {
        Ok(format!("squeue -j {} --format='%i|%T|%M|%l|%S|%e' --noheader", job_list))
    } else {
        Ok(format!("squeue -j {} --format='%i|%T' --noheader", job_list))
    }
}

/// Get completed job status using sacct
/// Accepts single job ID or multiple (single is just array of 1)
pub fn sacct_command(job_ids: &[String]) -> Result<String> {
    let sanitized_ids: Result<Vec<_>> = job_ids.iter()
        .map(|id| input::sanitize_job_id(id))
        .collect();
    let job_list = sanitized_ids?.join(",");

    // Use simpler format for single job, JobID+State for batch
    if job_ids.len() == 1 {
        Ok(format!("sacct -j {} --format=State --parsable2 --noheader", job_list))
    } else {
        Ok(format!("sacct -j {} --format=JobID,State --parsable2 --noheader", job_list))
    }
}

/// Cancel job command - single job
pub fn cancel_job_command(job_id: &str) -> Result<String> {
    let clean_id = input::sanitize_job_id(job_id)?;
    Ok(format!("scancel {}", clean_id))
}

#[cfg(test)]
mod tests {
    use super::*;


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
        assert!(squeue_command(&["12345; rm -rf /".to_string()]).is_err());
        assert!(cancel_job_command("../../etc/passwd").is_err());

        // Valid inputs should work
        assert!(squeue_command(&["12345".to_string()]).is_ok());
        assert!(cancel_job_command("12345").is_ok());
    }

    #[test]
    fn test_batch_commands() {
        let job_ids = vec!["12345".to_string(), "67890".to_string()];

        // Test squeue batch
        let cmd = squeue_command(&job_ids).unwrap();
        assert!(cmd.contains("squeue -j 12345,67890"));
        assert!(cmd.contains("--format='%i|%T'"));

        // Test sacct batch
        let cmd = sacct_command(&job_ids).unwrap();
        assert!(cmd.contains("sacct -j 12345,67890"));
    }

    #[test]
    fn test_single_job_commands() {
        let job_ids = vec!["12345".to_string()];

        assert!(squeue_command(&job_ids).is_ok());
        assert!(sacct_command(&job_ids).is_ok());
        assert!(cancel_job_command("12345").is_ok());
    }
}
