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
}

/// Generate a zip command for archiving output files
/// Creates zip in /tmp/ and returns the temp file path
pub fn zip_outputs_command(project_dir: &str, job_id: &str) -> Result<(String, String)> {
    use crate::validation::shell;

    // Validate inputs
    let clean_project_dir = shell::escape_parameter(project_dir);
    let clean_job_id = shell::escape_parameter(job_id);

    // Temp zip file path
    let temp_zip = format!("/tmp/namdrunner_outputs_{}.zip", clean_job_id);
    let clean_temp_zip = shell::escape_parameter(&temp_zip);

    // Build command: cd to job dir and zip outputs subdirectory
    let command = format!(
        "cd {} && zip -r {} {}",
        clean_project_dir,
        clean_temp_zip,
        shell::escape_parameter(super::JobDirectoryStructure::OUTPUTS)
    );

    Ok((command, temp_zip))
}

/// Generate a command to remove a temporary file
pub fn remove_temp_file_command(file_path: &str) -> Result<String> {
    use crate::validation::shell;

    let clean_path = shell::escape_parameter(file_path);
    Ok(format!("rm -f {}", clean_path))
}