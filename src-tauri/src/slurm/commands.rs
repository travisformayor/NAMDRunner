/// Simple SLURM command builder that consolidates common patterns
/// Based on slurm-commands-reference.md

pub struct SlurmCommand {
    command: String,
}

impl SlurmCommand {
    /// Create a new SLURM command with the standard module loading prefix
    pub fn new() -> Self {
        Self {
            command: "source /etc/profile && module load slurm/alpine".to_string(),
        }
    }

    /// Add squeue command with specified format
    pub fn squeue(mut self, job_ids: &str, format: &str) -> Self {
        self.command.push_str(&format!(" && squeue -j {} --format='{}' --noheader", job_ids, format));
        self
    }

    /// Add sacct command for completed jobs
    pub fn sacct(mut self, job_ids: &str, format: &str, days_back: u32) -> Self {
        self.command.push_str(&format!(
            " && sacct -j {} --starttime=$(date -d '{} days ago' +%Y-%m-%d) --format={} --parsable2 --noheader",
            job_ids, days_back, format
        ));
        self
    }

    /// Add sbatch command
    pub fn sbatch(mut self, script_path: &str) -> Self {
        self.command.push_str(&format!(" && sbatch {}", script_path));
        self
    }

    /// Add scancel command
    pub fn scancel(mut self, job_ids: &str) -> Self {
        self.command.push_str(&format!(" && scancel {}", job_ids));
        self
    }

    /// Add scontrol command
    pub fn scontrol(mut self, action: &str, job_id: &str) -> Self {
        self.command.push_str(&format!(" && scontrol {} {}", action, job_id));
        self
    }

    /// Build the final command string
    pub fn build(self) -> String {
        self.command
    }
}

/// Helper functions for common SLURM command patterns

/// Get job status command - single job
pub fn job_status_command(job_id: &str) -> String {
    SlurmCommand::new()
        .squeue(job_id, "%T")
        .build()
}

/// Get job status command - multiple jobs
pub fn batch_job_status_command(job_ids: &[String]) -> String {
    let job_list = job_ids.join(",");
    SlurmCommand::new()
        .squeue(&job_list, "%i|%T")
        .build()
}

/// Get completed job status command
pub fn completed_job_status_command(job_id: &str) -> String {
    SlurmCommand::new()
        .sacct(job_id, "State", 7)
        .build()
}

/// Get completed job status command - multiple jobs
pub fn batch_completed_job_status_command(job_ids: &[String]) -> String {
    let job_list = job_ids.join(",");
    SlurmCommand::new()
        .sacct(&job_list, "JobID,State", 7)
        .build()
}

/// Submit job command
pub fn submit_job_command(script_path: &str) -> String {
    SlurmCommand::new()
        .sbatch(script_path)
        .build()
}

/// Cancel job command - single job
pub fn cancel_job_command(job_id: &str) -> String {
    SlurmCommand::new()
        .scancel(job_id)
        .build()
}

/// Cancel multiple jobs command
pub fn cancel_jobs_command(job_ids: &[String]) -> String {
    let job_list = job_ids.join(",");
    SlurmCommand::new()
        .scancel(&job_list)
        .build()
}

/// Hold job command
pub fn hold_job_command(job_id: &str) -> String {
    SlurmCommand::new()
        .scontrol("hold", job_id)
        .build()
}

/// Release job command
pub fn release_job_command(job_id: &str) -> String {
    SlurmCommand::new()
        .scontrol("release", job_id)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_builder() {
        let cmd = SlurmCommand::new()
            .squeue("12345", "%T")
            .build();

        assert_eq!(cmd, "source /etc/profile && module load slurm/alpine && squeue -j 12345 --format='%T' --noheader");
    }

    #[test]
    fn test_job_status_command() {
        let cmd = job_status_command("12345");
        assert!(cmd.contains("source /etc/profile"));
        assert!(cmd.contains("module load slurm/alpine"));
        assert!(cmd.contains("squeue -j 12345"));
        assert!(cmd.contains("--format='%T'"));
        assert!(cmd.contains("--noheader"));
    }

    #[test]
    fn test_batch_commands() {
        let job_ids = vec!["12345".to_string(), "67890".to_string()];

        let cmd = batch_job_status_command(&job_ids);
        assert!(cmd.contains("12345,67890"));
        assert!(cmd.contains("--format='%i|%T'"));

        let cmd2 = batch_completed_job_status_command(&job_ids);
        assert!(cmd2.contains("12345,67890"));
        assert!(cmd2.contains("--format=JobID,State"));
        assert!(cmd2.contains("--starttime=$(date -d '7 days ago' +%Y-%m-%d)"));
    }

    #[test]
    fn test_submit_command() {
        let cmd = submit_job_command("/path/to/job.sbatch");
        assert!(cmd.contains("sbatch /path/to/job.sbatch"));
    }

    #[test]
    fn test_cancel_commands() {
        let cmd = cancel_job_command("12345");
        assert!(cmd.contains("scancel 12345"));

        let job_ids = vec!["12345".to_string(), "67890".to_string()];
        let cmd2 = cancel_jobs_command(&job_ids);
        assert!(cmd2.contains("scancel 12345,67890"));
    }

    #[test]
    fn test_control_commands() {
        let cmd = hold_job_command("12345");
        assert!(cmd.contains("scontrol hold 12345"));

        let cmd2 = release_job_command("12345");
        assert!(cmd2.contains("scontrol release 12345"));
    }
}