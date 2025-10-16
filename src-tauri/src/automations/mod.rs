// Simplified automation functions for NAMDRunner workflows
// Provides progress tracking through callbacks while maintaining direct function patterns

pub mod job_creation;
pub mod job_submission;
pub mod job_completion;
pub mod job_sync;
pub mod errors;
pub mod progress;

// Re-export simplified automation functions with progress reporting
pub use job_creation::execute_job_creation_with_progress;
pub use job_submission::execute_job_submission_with_progress;
pub use job_completion::execute_job_completion_with_progress;
pub use job_sync::{sync_all_jobs, fetch_slurm_logs_if_needed, JobSyncResult};

// Re-export error types for structured error handling
pub use errors::{AutomationError, AutomationResult};

// Re-export progress types for structured progress reporting
pub use progress::{ProgressInfo, ProgressCallback, ProgressTracker};