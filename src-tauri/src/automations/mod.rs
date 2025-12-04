// Simplified automation functions for NAMDRunner workflows
// Provides progress tracking through callbacks while maintaining direct function patterns

pub mod job_creation;
pub mod job_submission;
pub mod job_completion;
pub mod job_deletion;
pub mod job_sync;
pub mod file_operations;
pub mod errors;
pub mod progress;
pub mod common;

// Re-export simplified automation functions with progress reporting
pub use job_creation::execute_job_creation_with_progress;
pub use job_submission::execute_job_submission_with_progress;
pub use job_completion::execute_job_completion_internal;  // Internal automatic completion
pub use job_deletion::execute_job_deletion;
pub use job_sync::{sync_all_jobs, fetch_slurm_logs_if_needed, refetch_slurm_logs, JobSyncResult};
pub use file_operations::{
    upload_files_to_job, download_job_file, download_all_files_as_zip,
    list_job_files, validate_upload_file, classify_file_type
};

// Re-export error types for structured error handling
pub use errors::{AutomationError, AutomationResult};

// Re-export progress types for structured progress reporting
pub use progress::{ProgressInfo, ProgressCallback, ProgressTracker};