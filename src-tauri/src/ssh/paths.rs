use anyhow::Result;
use crate::ssh::directory_structure::JobDirectoryStructure;
use crate::security::input;

/// Directory type for job operations
pub enum DirectoryType {
    Project,
    Scratch,
}

/// Generate a safe job directory path (project or scratch)
/// Validates username, job_id, and ensures path is within allowed prefixes
fn safe_job_directory(
    username: &str,
    job_id: &str,
    dir_type: DirectoryType,
) -> Result<String> {
    let clean_username = input::sanitize_username(username)?;
    let clean_job_id = input::sanitize_job_id(job_id)?;

    // Build path and get allowed prefixes based on directory type
    let (path, allowed_prefixes) = match dir_type {
        DirectoryType::Project => (
            JobDirectoryStructure::project_dir(&clean_username, &clean_job_id),
            JobDirectoryStructure::project_allowed_prefixes(),
        ),
        DirectoryType::Scratch => (
            JobDirectoryStructure::scratch_dir(&clean_username, &clean_job_id),
            JobDirectoryStructure::scratch_allowed_prefixes(),
        ),
    };

    // Validate the path is within allowed directories
    input::validate_path_safety(&path, &allowed_prefixes)?;

    Ok(path)
}

/// Generate a safe project directory path for a user and job
pub fn project_directory(username: &str, job_id: &str) -> Result<String> {
    safe_job_directory(username, job_id, DirectoryType::Project)
}

/// Generate a safe scratch directory path for a user and job
pub fn scratch_directory(username: &str, job_id: &str) -> Result<String> {
    safe_job_directory(username, job_id, DirectoryType::Scratch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_directory_generation() {
        use crate::ssh::directory_structure::JobDirectoryStructure;
        let result = project_directory("testuser", "job_001").unwrap();
        if cfg!(windows) {
            assert_eq!(result, format!("C:\\Users\\testuser\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY));
        } else {
            // Should match centralized path generation
            let expected = JobDirectoryStructure::project_dir("testuser", "job_001");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_scratch_directory_generation() {
        use crate::ssh::directory_structure::JobDirectoryStructure;
        let result = scratch_directory("testuser", "job_001").unwrap();
        if cfg!(windows) {
            assert_eq!(result, format!("C:\\scratch\\testuser\\{}\\job_001", crate::ssh::directory_structure::JOB_BASE_DIRECTORY));
        } else {
            // Should match centralized path generation
            let expected = JobDirectoryStructure::scratch_dir("testuser", "job_001");
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_malicious_path_generation() {
        // Should fail with malicious inputs
        assert!(project_directory("../admin", "job_001").is_err());
        assert!(project_directory("testuser", "../../../etc").is_err());
    }

    #[test]
    fn test_comprehensive_security_scenarios() {
        // Test the complete workflow with malicious inputs
        let malicious_scenarios = vec![
            ("../../../root", "job_001"),
            ("testuser", "; rm -rf /"),
            ("test\x00user", "job_001"),
            ("testuser", "job$(whoami)"),
            ("user|admin", "job_001"),
        ];

        for (username, job_id) in malicious_scenarios {
            // Both project and scratch directory generation should fail
            assert!(project_directory(username, job_id).is_err(),
                    "Should reject malicious combo: {} / {}", username, job_id);
            assert!(scratch_directory(username, job_id).is_err(),
                    "Should reject malicious combo: {} / {}", username, job_id);
        }
    }
}
