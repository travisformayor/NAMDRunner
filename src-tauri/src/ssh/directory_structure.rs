/// Standard job directory structure for NAMDRunner jobs
///
/// This module defines the canonical directory structure used across all job operations:
/// - Job creation creates these subdirectories
/// - Script generation references these paths in NAMD configs
/// - File operations use these paths for uploads/downloads
///
/// Centralizing this knowledge prevents inconsistencies and makes the structure easy to change.
pub struct JobDirectoryStructure;

impl JobDirectoryStructure {
    /// Subdirectory for user-uploaded input files (PSF, PDB, PRM files)
    pub const INPUT_FILES: &'static str = "input_files";

    /// Subdirectory for generated job scripts (job.sbatch, config.namd)
    pub const SCRIPTS: &'static str = "scripts";

    /// Subdirectory for NAMD output files (coordinates, velocities, trajectories)
    pub const OUTPUTS: &'static str = "outputs";

    /// Get all subdirectories that should be created for a job
    pub fn subdirectories() -> Vec<&'static str> {
        vec![Self::INPUT_FILES, Self::SCRIPTS, Self::OUTPUTS]
    }

    /// Get the input file path for use in NAMD config (relative to working directory)
    ///
    /// Returns: "input_files/{filename}"
    pub fn input_path(filename: &str) -> String {
        format!("{}/{}", Self::INPUT_FILES, filename)
    }

    /// Get the output path for use in NAMD config (relative to working directory)
    ///
    /// Returns: "outputs/{filename}"
    pub fn output_path(filename: &str) -> String {
        format!("{}/{}", Self::OUTPUTS, filename)
    }

    /// Get the script path for a given script file (relative to working directory)
    ///
    /// Returns: "scripts/{filename}"
    pub fn script_path(filename: &str) -> String {
        format!("{}/{}", Self::SCRIPTS, filename)
    }

    /// Get the full input file path (project_dir + input_files/ + filename)
    ///
    /// Returns: "{project_dir}/input_files/{filename}"
    pub fn full_input_path(project_dir: &str, filename: &str) -> String {
        format!("{}/{}/{}", project_dir, Self::INPUT_FILES, filename)
    }

    /// Get the full output file path (project_dir + outputs/ + filename)
    ///
    /// Returns: "{project_dir}/outputs/{filename}"
    pub fn full_output_path(project_dir: &str, filename: &str) -> String {
        format!("{}/{}/{}", project_dir, Self::OUTPUTS, filename)
    }

    /// Get the full script file path (project_dir + scripts/ + filename)
    ///
    /// Returns: "{project_dir}/scripts/{filename}"
    pub fn full_script_path(project_dir: &str, filename: &str) -> String {
        format!("{}/{}/{}", project_dir, Self::SCRIPTS, filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subdirectories_contains_all() {
        let subdirs = JobDirectoryStructure::subdirectories();
        assert_eq!(subdirs.len(), 3);
        assert!(subdirs.contains(&"input_files"));
        assert!(subdirs.contains(&"scripts"));
        assert!(subdirs.contains(&"outputs"));
    }

    #[test]
    fn test_input_path() {
        assert_eq!(
            JobDirectoryStructure::input_path("structure.psf"),
            "input_files/structure.psf"
        );
    }

    #[test]
    fn test_output_path() {
        assert_eq!(
            JobDirectoryStructure::output_path("sim_output"),
            "outputs/sim_output"
        );
    }

    #[test]
    fn test_script_path() {
        assert_eq!(
            JobDirectoryStructure::script_path("job.sbatch"),
            "scripts/job.sbatch"
        );
    }

    #[test]
    fn test_full_input_path() {
        assert_eq!(
            JobDirectoryStructure::full_input_path("/projects/user/namdrunner_jobs/job_001", "structure.psf"),
            "/projects/user/namdrunner_jobs/job_001/input_files/structure.psf"
        );
    }

    #[test]
    fn test_full_output_path() {
        assert_eq!(
            JobDirectoryStructure::full_output_path("/projects/user/namdrunner_jobs/job_001", "sim.dcd"),
            "/projects/user/namdrunner_jobs/job_001/outputs/sim.dcd"
        );
    }

    #[test]
    fn test_full_script_path() {
        assert_eq!(
            JobDirectoryStructure::full_script_path("/projects/user/namdrunner_jobs/job_001", "config.namd"),
            "/projects/user/namdrunner_jobs/job_001/scripts/config.namd"
        );
    }
}
