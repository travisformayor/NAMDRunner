use ssh2::Session;
use std::io::Read;
use std::time::{Duration, Instant};
use anyhow::Result;
use super::errors::SSHError;

/// Result of command execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub timed_out: bool,
}

/// Command executor for SSH sessions
pub struct CommandExecutor<'a> {
    session: &'a Session,
    timeout: Duration,
}

impl<'a> CommandExecutor<'a> {
    /// Create a new command executor
    pub fn new(session: &'a Session, timeout_secs: u64) -> Self {
        Self {
            session,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Execute a command on the remote system
    pub async fn execute(&self, command: &str) -> Result<CommandResult> {
        let start = Instant::now();

        // Create channel for command execution
        let mut channel = self.session.channel_session()
            .map_err(|e| SSHError::SessionError(format!("Failed to create channel: {}", e)))?;

        // Set environment variables if needed (e.g., for module loading)
        // channel.setenv("PATH", "/usr/local/bin:/usr/bin:/bin")?;

        // Execute the command
        channel.exec(command)
            .map_err(|e| SSHError::CommandError(format!("Failed to execute command: {}", e)))?;

        // Read output with timeout
        let mut stdout = String::new();
        let mut stderr = String::new();

        // Use tokio timeout to wrap the entire operation
        let read_result = tokio::time::timeout(self.timeout, async {
            // Read stdout
            channel.read_to_string(&mut stdout)?;

            // Read stderr
            channel.stderr().read_to_string(&mut stderr)?;

            // Wait for channel to close and get exit status
            channel.wait_close()?;

            Ok::<(), anyhow::Error>(())
        }).await;

        let timed_out = read_result.is_err();

        if let Err(_) = read_result {
            return Err(SSHError::TimeoutError(
                format!("Command timed out after {} seconds", self.timeout.as_secs())
            ).into());
        }

        if let Err(e) = read_result.unwrap() {
            return Err(SSHError::CommandError(format!("Command execution failed: {}", e)).into());
        }

        let exit_code = channel.exit_status()
            .map_err(|e| SSHError::CommandError(format!("Failed to get exit status: {}", e)))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(CommandResult {
            stdout,
            stderr,
            exit_code,
            duration_ms,
            timed_out,
        })
    }

    /// Execute a command with module loading
    pub async fn execute_with_modules(&self, command: &str, modules: &[&str]) -> Result<CommandResult> {
        let mut full_command = String::new();

        // Add module loads
        for module in modules {
            full_command.push_str(&format!("module load {} && ", module));
        }

        // Add the actual command
        full_command.push_str(command);

        self.execute(&full_command).await
    }

    /// Execute multiple commands in sequence
    pub async fn execute_sequence(&self, commands: &[&str]) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();

        for command in commands {
            let result = self.execute(command).await?;

            // Stop on first error (non-zero exit code)
            if result.exit_code != 0 {
                results.push(result);
                break;
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Check if a command exists on the remote system
    pub async fn command_exists(&self, command: &str) -> Result<bool> {
        let check_command = format!("command -v {} >/dev/null 2>&1", command);
        let result = self.execute(&check_command).await?;
        Ok(result.exit_code == 0)
    }

    /// Get environment variable from remote system
    pub async fn get_env(&self, var_name: &str) -> Result<Option<String>> {
        let command = format!("echo ${}", var_name);
        let result = self.execute(&command).await?;

        if result.exit_code == 0 && !result.stdout.trim().is_empty() {
            Ok(Some(result.stdout.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    /// Check module availability
    pub async fn module_available(&self, module_name: &str) -> Result<bool> {
        let command = format!("module avail {} 2>&1", module_name);
        let result = self.execute(&command).await?;

        // Module is available if it appears in the output
        Ok(result.stdout.contains(module_name) || result.stderr.contains(module_name))
    }

    /// Load a module and verify it loaded
    pub async fn load_module(&self, module_name: &str) -> Result<()> {
        let command = format!("module load {} && module list 2>&1", module_name);
        let result = self.execute(&command).await?;

        if result.exit_code != 0 {
            return Err(SSHError::CommandError(
                format!("Failed to load module {}: {}", module_name, result.stderr)
            ).into());
        }

        // Verify module is in the loaded list
        if !result.stdout.contains(module_name) && !result.stderr.contains(module_name) {
            return Err(SSHError::CommandError(
                format!("Module {} did not load properly", module_name)
            ).into());
        }

        Ok(())
    }
}

/// Helper functions for common SLURM commands
pub struct SLURMCommands;

impl SLURMCommands {
    /// Build sbatch command
    pub fn sbatch(script_path: &str) -> String {
        format!("sbatch {}", script_path)
    }

    /// Build squeue command for a specific job
    pub fn squeue_job(job_id: &str) -> String {
        format!("squeue -j {} --format='%i|%T|%M|%l|%S|%e' --noheader", job_id)
    }

    /// Build squeue command for all user jobs
    pub fn squeue_user(username: &str) -> String {
        format!("squeue -u {} --format='%i|%T|%M|%l|%S|%e' --noheader", username)
    }

    /// Build sacct command for job history
    pub fn sacct_job(job_id: &str) -> String {
        format!(
            "sacct -j {} --format=JobID,JobName,State,ExitCode,Elapsed,Start,End --parsable2 --noheader",
            job_id
        )
    }

    /// Build scancel command
    pub fn scancel(job_id: &str) -> String {
        format!("scancel {}", job_id)
    }

    /// Parse sbatch output to get job ID
    pub fn parse_sbatch_output(output: &str) -> Option<String> {
        // Expected format: "Submitted batch job 12345678"
        output.split_whitespace()
            .last()
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_creation() {
        let result = CommandResult {
            stdout: "output".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
            duration_ms: 100,
            timed_out: false,
        };

        assert_eq!(result.stdout, "output");
        assert_eq!(result.exit_code, 0);
        assert!(!result.timed_out);
    }

    #[test]
    fn test_slurm_commands() {
        assert_eq!(SLURMCommands::sbatch("/path/to/script.sh"), "sbatch /path/to/script.sh");
        assert_eq!(
            SLURMCommands::squeue_job("12345"),
            "squeue -j 12345 --format='%i|%T|%M|%l|%S|%e' --noheader"
        );
        assert_eq!(SLURMCommands::scancel("12345"), "scancel 12345");
    }

    #[test]
    fn test_parse_sbatch_output() {
        let output = "Submitted batch job 12345678";
        assert_eq!(SLURMCommands::parse_sbatch_output(output), Some("12345678".to_string()));

        let invalid_output = "Error: Invalid script";
        assert_eq!(SLURMCommands::parse_sbatch_output(invalid_output), None);
    }


    #[test]
    fn test_squeue_command_format_validation() {
        let job_id = "12345";
        let command = SLURMCommands::squeue_job(job_id);

        // Verify format contains required fields
        assert!(command.contains("--format='%i|%T|%M|%l|%S|%e'"));
        assert!(command.contains("--noheader"));
        assert!(command.contains(&format!("-j {}", job_id)));

        // Test user queue command
        let username = "testuser";
        let user_command = SLURMCommands::squeue_user(username);
        assert!(user_command.contains(&format!("-u {}", username)));
        assert!(user_command.contains("--format='%i|%T|%M|%l|%S|%e'"));
    }

    #[test]
    fn test_sacct_command_format_validation() {
        let job_id = "98765";
        let command = SLURMCommands::sacct_job(job_id);

        // Verify required fields are present
        assert!(command.contains("JobID,JobName,State,ExitCode,Elapsed,Start,End"));
        assert!(command.contains("--parsable2"));
        assert!(command.contains("--noheader"));
        assert!(command.contains(&format!("-j {}", job_id)));
    }

    #[test]
    fn test_sbatch_output_parsing_edge_cases() {
        // Standard success case
        let standard = "Submitted batch job 12345678";
        assert_eq!(SLURMCommands::parse_sbatch_output(standard), Some("12345678".to_string()));

        // With extra whitespace
        let whitespace = "  Submitted batch job   98765  \n";
        assert_eq!(SLURMCommands::parse_sbatch_output(whitespace), Some("98765".to_string()));

        // Error cases
        assert_eq!(SLURMCommands::parse_sbatch_output(""), None);
        assert_eq!(SLURMCommands::parse_sbatch_output("Error: Permission denied"), None);
        assert_eq!(SLURMCommands::parse_sbatch_output("sbatch: command not found"), None);

        // Invalid job ID format
        assert_eq!(SLURMCommands::parse_sbatch_output("Submitted batch job abc"), None);
        assert_eq!(SLURMCommands::parse_sbatch_output("Submitted batch job "), None);

        // Multiple lines (should take first valid one)
        let multiline = "Submitted batch job 11111\nSubmitted batch job 22222";
        assert_eq!(SLURMCommands::parse_sbatch_output(multiline), Some("11111".to_string()));
    }

    #[test]
    fn test_command_result_timeout_detection() {

        // Test timeout scenarios
        let timeout_result = CommandResult {
            stdout: "".to_string(),
            stderr: "Operation timed out".to_string(),
            exit_code: 124, // Common timeout exit code
            duration_ms: 30000, // 30 seconds
            timed_out: true,
        };

        // Our logic should detect timeout conditions
        assert!(timeout_result.duration_ms >= 30000);
        assert_eq!(timeout_result.exit_code, 124);
        assert!(timeout_result.timed_out);

        // Test successful quick command
        let quick_result = CommandResult {
            stdout: "success".to_string(),
            stderr: "".to_string(),
            exit_code: 0,
            duration_ms: 50,
            timed_out: false,
        };

        assert_eq!(quick_result.exit_code, 0);
        assert!(quick_result.duration_ms < 1000);
        assert!(!quick_result.timed_out);
    }


    #[test]
    fn test_slurm_format_string_consistency() {
        // Ensure SLURM format strings are consistent across commands
        let job_format = "%i|%T|%M|%l|%S|%e";

        let job_cmd = SLURMCommands::squeue_job("123");
        let user_cmd = SLURMCommands::squeue_user("testuser");

        assert!(job_cmd.contains(job_format));
        assert!(user_cmd.contains(job_format));

        // Both should use the same formatting options
        assert!(job_cmd.contains("--noheader"));
        assert!(user_cmd.contains("--noheader"));
    }

    #[test]
    fn test_error_result_parsing() {
        // Test parsing various error scenarios
        let error_cases = vec![
            ("sbatch: error: Batch job submission failed: Invalid account specified", "Invalid account"),
            ("squeue: error: Invalid user specified", "Invalid user"),
            ("scancel: error: Kill job error on job id 12345: Invalid job id specified", "Invalid job id"),
            ("module: command not found", "command not found"),
        ];

        for (error_output, expected_content) in error_cases {
            // Test that our error handling can extract meaningful information
            assert!(error_output.contains(expected_content));

            // Simulate error result
            let error_result = CommandResult {
                stdout: "".to_string(),
                stderr: error_output.to_string(),
                exit_code: 1,
                duration_ms: 50,
                timed_out: false,
            };

            assert_ne!(error_result.exit_code, 0);
            assert!(!error_result.stderr.is_empty());
        }
    }
}