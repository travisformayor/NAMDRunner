/// SLURM command builders for NAMDRunner
/// Simple, direct functions following CONTRIBUTING.md philosophy
///
/// All functions:
/// - Use input sanitization to prevent command injection
/// - Return Result for error handling
/// - Generate commands only (no execution)
use crate::security::input;
use crate::security::shell;
use anyhow::Result;


/// Build sbatch submit command with directory change
///
/// This uses safe shell escaping for the directory path and submits
/// the specified script in that directory.
pub fn submit_job_command(scratch_dir: &str, script_name: &str) -> Result<String> {
    // Use safe shell escaping for directory
    Ok(shell::safe_cd_and_run(scratch_dir, &format!("sbatch {}", script_name)))
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
/// Always uses consistent job_id|status format for reliable parsing
pub fn squeue_command(job_ids: &[String]) -> Result<String> {
    let sanitized_ids: Result<Vec<_>> = job_ids.iter()
        .map(|id| input::sanitize_job_id(id))
        .collect();
    let job_list = sanitized_ids?.join(",");
    Ok(format!("squeue -j {} --format='%i|%T' --noheader", job_list))
}

/// Get completed job status using sacct
/// Always uses consistent job_id|status format for reliable parsing
pub fn sacct_command(job_ids: &[String]) -> Result<String> {
    let sanitized_ids: Result<Vec<_>> = job_ids.iter()
        .map(|id| input::sanitize_job_id(id))
        .collect();
    let job_list = sanitized_ids?.join(",");
    Ok(format!("sacct -j {} --format=JobID,State --parsable2 --noheader", job_list))
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
    fn test_squeue_consistent_format() {
        // Single and batch queries use identical format for reliable parsing
        let single = squeue_command(&["12345".to_string()]).unwrap();
        let batch = squeue_command(&["12345".to_string(), "67890".to_string()]).unwrap();

        // Both use job_id|status format
        assert!(single.contains("--format='%i|%T'"));
        assert!(batch.contains("--format='%i|%T'"));
        assert!(single.contains("squeue -j 12345"));
        assert!(batch.contains("squeue -j 12345,67890"));
    }

    #[test]
    fn test_sacct_consistent_format() {
        // Single and batch queries use identical format for reliable parsing
        let single = sacct_command(&["12345".to_string()]).unwrap();
        let batch = sacct_command(&["12345".to_string(), "67890".to_string()]).unwrap();

        // Both use JobID,State format with parsable2 delimiter
        assert!(single.contains("--format=JobID,State"));
        assert!(batch.contains("--format=JobID,State"));
        assert!(single.contains("--parsable2"));
        assert!(batch.contains("--parsable2"));
    }

    #[test]
    fn test_cancel_job_command() {
        assert!(cancel_job_command("12345").is_ok());
        let cmd = cancel_job_command("12345").unwrap();
        assert!(cmd.contains("scancel 12345"));
    }
}
