use anyhow::Result;
use crate::types::{JobInfo, JobStatus};

/// Repository trait for job data access operations
pub trait JobRepository {
    /// Save a job to the database
    fn save_job(&self, job: &JobInfo) -> Result<()>;

    /// Load a job by ID from the database
    fn load_job(&self, job_id: &str) -> Result<Option<JobInfo>>;

    /// Load all jobs from the database
    fn load_all_jobs(&self) -> Result<Vec<JobInfo>>;

    /// Load jobs filtered by status
    fn load_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobInfo>>;

    /// Update job status in the database
    fn update_job_status(&self, job_id: &str, status: JobStatus, source: &str) -> Result<()>;

    /// Delete a job from the database
    fn delete_job(&self, job_id: &str) -> Result<bool>;
}

/// Default implementation using the existing database module
pub struct DefaultJobRepository;

impl JobRepository for DefaultJobRepository {
    fn save_job(&self, job: &JobInfo) -> Result<()> {
        crate::database::with_database(|db| db.save_job(job))
    }

    fn load_job(&self, job_id: &str) -> Result<Option<JobInfo>> {
        crate::database::with_database(|db| db.load_job(job_id))
    }

    fn load_all_jobs(&self) -> Result<Vec<JobInfo>> {
        crate::database::with_database(|db| db.load_all_jobs())
    }

    fn load_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobInfo>> {
        crate::database::with_database(|db| db.get_jobs_by_status(status))
    }

    fn update_job_status(&self, job_id: &str, status: JobStatus, source: &str) -> Result<()> {
        crate::database::with_database(|db| db.update_job_status(job_id, status, source))
    }

    fn delete_job(&self, job_id: &str) -> Result<bool> {
        crate::database::with_database(|db| db.delete_job(job_id))
    }
}

impl DefaultJobRepository {
    pub fn update_job_with_slurm_id(&self, job: &mut JobInfo, slurm_job_id: String) -> Result<()> {
        job.slurm_job_id = Some(slurm_job_id.clone());
        job.updated_at = Some(chrono::Utc::now().to_rfc3339());

        if job.status == JobStatus::Created {
            job.status = JobStatus::Pending;
            job.submitted_at = job.updated_at.clone();
        }

        self.save_job(job)
    }

    pub fn update_job_status_with_timestamps(&self, job: &mut JobInfo, new_status: JobStatus, source: &str) -> Result<()> {
        job.status = new_status.clone();
        job.updated_at = Some(chrono::Utc::now().to_rfc3339());

        if new_status == JobStatus::Completed || new_status == JobStatus::Failed || new_status == JobStatus::Cancelled {
            job.completed_at = job.updated_at.clone();
        }

        self.update_job_status(&job.job_id, new_status, source)
    }
}

/// Global default job repository instance
pub fn default_job_repository() -> DefaultJobRepository {
    DefaultJobRepository
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mock repository for testing
    struct MockJobRepository {
        jobs: Arc<Mutex<HashMap<String, JobInfo>>>,
    }

    impl MockJobRepository {
        fn new() -> Self {
            Self {
                jobs: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl JobRepository for MockJobRepository {
        fn save_job(&self, job: &JobInfo) -> Result<()> {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.insert(job.job_id.clone(), job.clone());
            Ok(())
        }

        fn load_job(&self, job_id: &str) -> Result<Option<JobInfo>> {
            let jobs = self.jobs.lock().unwrap();
            Ok(jobs.get(job_id).cloned())
        }

        fn load_all_jobs(&self) -> Result<Vec<JobInfo>> {
            let jobs = self.jobs.lock().unwrap();
            Ok(jobs.values().cloned().collect())
        }

        fn load_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobInfo>> {
            let jobs = self.jobs.lock().unwrap();
            Ok(jobs.values()
                .filter(|job| job.status == status)
                .cloned()
                .collect())
        }

        fn update_job_status(&self, job_id: &str, status: JobStatus, _source: &str) -> Result<()> {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = status;
                job.updated_at = Some(chrono::Utc::now().to_rfc3339());
            }
            Ok(())
        }

        fn delete_job(&self, job_id: &str) -> Result<bool> {
            let mut jobs = self.jobs.lock().unwrap();
            Ok(jobs.remove(job_id).is_some())
        }
    }

    #[test]
    fn test_job_repository_operations() {
        let repo = MockJobRepository::new();

        // Create test job using the constructor
        let job = JobInfo::new(
            "test_job".to_string(),
            "Test Job".to_string(),
            crate::types::NAMDConfig {
                steps: 1000,
                temperature: 300.0,
                timestep: 2.0,
                outputname: "output".to_string(),
                dcd_freq: Some(100),
                restart_freq: Some(500),
            },
            crate::types::SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            Vec::new(),
            "/scratch/test".to_string(),
        );

        // Test save and load
        repo.save_job(&job).unwrap();
        let loaded = repo.load_job("test_job").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().job_id, "test_job");

        // Test load all
        let all_jobs = repo.load_all_jobs().unwrap();
        assert_eq!(all_jobs.len(), 1);

        // Test load by status
        let created_jobs = repo.load_jobs_by_status(JobStatus::Created).unwrap();
        assert_eq!(created_jobs.len(), 1);

        // Test update status
        repo.update_job_status("test_job", JobStatus::Running, "test").unwrap();
        let updated_job = repo.load_job("test_job").unwrap().unwrap();
        assert_eq!(updated_job.status, JobStatus::Running);

        // Test delete
        let deleted = repo.delete_job("test_job").unwrap();
        assert!(deleted);
        let after_delete = repo.load_job("test_job").unwrap();
        assert!(after_delete.is_none());
    }
}