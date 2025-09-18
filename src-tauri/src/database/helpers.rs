use anyhow::Result;
use crate::types::{JobInfo, JobStatus};
use chrono::Utc;

/// Helper function to update a job with SLURM ID
pub fn update_job_with_slurm_id(job: &mut JobInfo, slurm_job_id: String) -> Result<()> {
    job.slurm_job_id = Some(slurm_job_id.clone());
    job.updated_at = Some(Utc::now().to_rfc3339());

    if job.status == JobStatus::Created {
        job.status = JobStatus::Pending;
        job.submitted_at = job.updated_at.clone();
    }

    super::with_database(|db| db.save_job(job))
}

/// Helper function to update job status with timestamps
pub fn update_job_status_with_timestamps(job: &mut JobInfo, new_status: JobStatus, source: &str) -> Result<()> {
    job.status = new_status.clone();
    job.updated_at = Some(Utc::now().to_rfc3339());

    if new_status == JobStatus::Completed || new_status == JobStatus::Failed || new_status == JobStatus::Cancelled {
        job.completed_at = job.updated_at.clone();
    }

    super::with_database(|db| db.update_job_status(&job.job_id, new_status, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NAMDConfig, SlurmConfig};

    #[test]
    fn test_update_job_with_slurm_id() {
        // Create test job
        let mut job = JobInfo::new(
            "test_job".to_string(),
            "Test Job".to_string(),
            NAMDConfig {
                steps: 1000,
                temperature: 300.0,
                timestep: 2.0,
                outputname: "output".to_string(),
                dcd_freq: Some(100),
                restart_freq: Some(500),
            },
            SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            Vec::new(),
            "/scratch/test".to_string(),
        );

        // Update with SLURM ID
        let result = update_job_with_slurm_id(&mut job, "12345".to_string());

        // In test mode, this might fail due to database not being initialized
        // But we can still test the job modification
        assert_eq!(job.slurm_job_id, Some("12345".to_string()));
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.submitted_at.is_some());
    }
}