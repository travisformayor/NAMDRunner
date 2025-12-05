// Simplified automation functions for NAMDRunner workflows
// Provides progress tracking through callbacks while maintaining direct function patterns

pub mod job_creation;
pub mod job_submission;
pub mod job_completion;
pub mod job_deletion;
pub mod job_sync;
pub mod file_operations;
pub mod common;

// Re-export simplified automation functions with progress reporting
pub use job_creation::execute_job_creation_with_progress;
pub use job_submission::execute_job_submission_with_progress;
pub use job_completion::execute_job_completion_internal;  // Internal automatic completion
pub use job_deletion::execute_job_deletion;
pub use job_sync::{sync_all_jobs, fetch_slurm_logs, JobSyncResult};
pub use file_operations::{
    download_job_file, download_all_files_as_zip, validate_upload_file
};