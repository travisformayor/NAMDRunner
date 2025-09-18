use rusqlite::{Connection, Result as SqliteResult, Row};
use crate::types::{JobInfo, JobStatus, NAMDConfig, SlurmConfig};
use anyhow::{Result, anyhow};
use std::path::Path;

pub mod helpers;

pub struct JobDatabase {
    conn: Connection,
}

impl JobDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        Self::initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    fn initialize_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(r#"
            -- Jobs table for local cache
            CREATE TABLE IF NOT EXISTS jobs (
                job_id TEXT PRIMARY KEY,
                job_name TEXT NOT NULL,
                status TEXT NOT NULL,
                slurm_job_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT,
                submitted_at TEXT,
                completed_at TEXT,
                project_dir TEXT,
                scratch_dir TEXT,
                error_info TEXT
            );

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
            CREATE INDEX IF NOT EXISTS idx_jobs_slurm_id ON jobs(slurm_job_id);
            CREATE INDEX IF NOT EXISTS idx_jobs_updated ON jobs(updated_at);

            -- Job status history for debugging and user feedback
            CREATE TABLE IF NOT EXISTS job_status_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                status TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                source TEXT NOT NULL,
                FOREIGN KEY (job_id) REFERENCES jobs (job_id)
            );

            -- Application metadata table
            CREATE TABLE IF NOT EXISTS app_metadata (
                key TEXT PRIMARY KEY,
                value TEXT,
                updated_at TEXT
            );

            -- Insert schema version if not exists
            INSERT OR IGNORE INTO app_metadata (key, value, updated_at)
            VALUES ('schema_version', '1.0', datetime('now'));
        "#)?;
        Ok(())
    }

    pub fn save_job(&self, job_info: &JobInfo) -> Result<()> {
        let status_str = match &job_info.status {
            JobStatus::Created => "CREATED",
            JobStatus::Pending => "PENDING",
            JobStatus::Running => "RUNNING",
            JobStatus::Completed => "COMPLETED",
            JobStatus::Failed => "FAILED",
            JobStatus::Cancelled => "CANCELLED",
        };

        self.conn.execute(
            "INSERT OR REPLACE INTO jobs
             (job_id, job_name, status, slurm_job_id, created_at, updated_at, submitted_at, completed_at, project_dir, scratch_dir, error_info)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                &job_info.job_id,
                &job_info.job_name,
                status_str,
                &job_info.slurm_job_id,
                &job_info.created_at,
                &job_info.updated_at,
                &job_info.submitted_at,
                &job_info.completed_at,
                &job_info.project_dir,
                &job_info.scratch_dir,
                &job_info.error_info,
            ),
        )?;

        // Add status history entry
        self.add_status_history(&job_info.job_id, status_str, "local")?;

        Ok(())
    }

    pub fn load_job(&self, job_id: &str) -> Result<Option<JobInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT job_id, job_name, status, slurm_job_id, created_at, updated_at, submitted_at, completed_at, project_dir, scratch_dir, error_info
             FROM jobs WHERE job_id = ?1"
        )?;

        let job_iter = stmt.query_map([job_id], |row| {
            self.row_to_job_info(row)
        })?;

        for job in job_iter {
            return Ok(Some(job?));
        }

        Ok(None)
    }

    pub fn load_all_jobs(&self) -> Result<Vec<JobInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT job_id, job_name, status, slurm_job_id, created_at, updated_at, submitted_at, completed_at, project_dir, scratch_dir, error_info
             FROM jobs ORDER BY created_at DESC"
        )?;

        let job_iter = stmt.query_map([], |row| {
            self.row_to_job_info(row)
        })?;

        let mut jobs = Vec::new();
        for job in job_iter {
            jobs.push(job?);
        }

        Ok(jobs)
    }

    pub fn update_job_status(&self, job_id: &str, new_status: JobStatus, source: &str) -> Result<()> {
        let status_str = match new_status {
            JobStatus::Created => "CREATED",
            JobStatus::Pending => "PENDING",
            JobStatus::Running => "RUNNING",
            JobStatus::Completed => "COMPLETED",
            JobStatus::Failed => "FAILED",
            JobStatus::Cancelled => "CANCELLED",
        };

        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "UPDATE jobs SET status = ?1, updated_at = ?2 WHERE job_id = ?3",
            (status_str, &now, job_id),
        )?;

        // Add status history entry
        self.add_status_history(job_id, status_str, source)?;

        Ok(())
    }

    pub fn update_slurm_job_id(&self, job_id: &str, slurm_job_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "UPDATE jobs SET slurm_job_id = ?1, updated_at = ?2 WHERE job_id = ?3",
            (slurm_job_id, &now, job_id),
        )?;

        Ok(())
    }

    pub fn delete_job(&self, job_id: &str) -> Result<bool> {
        // Delete status history first (foreign key constraint)
        self.conn.execute(
            "DELETE FROM job_status_history WHERE job_id = ?1",
            [job_id],
        )?;

        // Delete the job
        let rows_affected = self.conn.execute(
            "DELETE FROM jobs WHERE job_id = ?1",
            [job_id],
        )?;

        Ok(rows_affected > 0)
    }

    pub fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<JobInfo>> {
        let status_str = match status {
            JobStatus::Created => "CREATED",
            JobStatus::Pending => "PENDING",
            JobStatus::Running => "RUNNING",
            JobStatus::Completed => "COMPLETED",
            JobStatus::Failed => "FAILED",
            JobStatus::Cancelled => "CANCELLED",
        };

        let mut stmt = self.conn.prepare(
            "SELECT job_id, job_name, status, slurm_job_id, created_at, updated_at, submitted_at, completed_at, project_dir, scratch_dir, error_info
             FROM jobs WHERE status = ?1 ORDER BY created_at DESC"
        )?;

        let job_iter = stmt.query_map([status_str], |row| {
            self.row_to_job_info(row)
        })?;

        let mut jobs = Vec::new();
        for job in job_iter {
            jobs.push(job?);
        }

        Ok(jobs)
    }

    fn add_status_history(&self, job_id: &str, status: &str, source: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO job_status_history (job_id, status, timestamp, source) VALUES (?1, ?2, ?3, ?4)",
            (job_id, status, &now, source),
        )?;

        Ok(())
    }

    fn row_to_job_info(&self, row: &Row) -> SqliteResult<JobInfo> {
        let status_str: String = row.get(2)?;
        let status = match status_str.as_str() {
            "CREATED" => JobStatus::Created,
            "PENDING" => JobStatus::Pending,
            "RUNNING" => JobStatus::Running,
            "COMPLETED" => JobStatus::Completed,
            "FAILED" => JobStatus::Failed,
            "CANCELLED" => JobStatus::Cancelled,
            _ => JobStatus::Created, // Default fallback
        };

        Ok(JobInfo {
            job_id: row.get(0)?,
            job_name: row.get(1)?,
            status,
            slurm_job_id: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            submitted_at: row.get(6)?,
            completed_at: row.get(7)?,
            project_dir: row.get(8)?,
            scratch_dir: row.get(9)?,
            error_info: row.get(10)?,
            // Provide default values for new fields
            namd_config: NAMDConfig::default(),
            slurm_config: SlurmConfig::default(),
            input_files: Vec::new(),
            remote_directory: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "/tmp".to_string()),
        })
    }
}

// Thread-safe global database instance
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref DATABASE: Arc<Mutex<Option<JobDatabase>>> = Arc::new(Mutex::new(None));
}

pub fn initialize_database(db_path: &str) -> Result<()> {
    let db = JobDatabase::new(db_path)?;
    let mut database_lock = DATABASE.lock().unwrap();
    *database_lock = Some(db);
    Ok(())
}

pub fn with_database<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&JobDatabase) -> Result<R>,
{
    let database_lock = DATABASE.lock().unwrap();
    match database_lock.as_ref() {
        Some(db) => f(db),
        None => Err(anyhow!("Database not initialized")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_database() -> JobDatabase {
        // Use in-memory database for tests
        JobDatabase::new(":memory:").unwrap()
    }

    fn create_test_job_info() -> JobInfo {
        let now = Utc::now().to_rfc3339();
        JobInfo {
            job_id: "test_job_001".to_string(),
            job_name: "test_simulation".to_string(),
            status: JobStatus::Created,
            slurm_job_id: None,
            created_at: now.clone(),
            updated_at: Some(now),
            submitted_at: None,
            completed_at: None,
            project_dir: Some("/projects/testuser/namdrunner_jobs/test_job_001".to_string()),
            scratch_dir: Some("/scratch/alpine/testuser/namdrunner_jobs/test_job_001".to_string()),
            error_info: None,
            namd_config: NAMDConfig {
                steps: 1000,
                temperature: 300.0,
                timestep: 2.0,
                outputname: "output".to_string(),
                dcd_freq: Some(100),
                restart_freq: Some(500),
            },
            slurm_config: SlurmConfig {
                cores: 4,
                memory: "4GB".to_string(),
                walltime: "01:00:00".to_string(),
                partition: Some("compute".to_string()),
                qos: None,
            },
            input_files: Vec::new(),
            remote_directory: "/projects/testuser/namdrunner_jobs/test_job_001".to_string(),
        }
    }

    #[test]
    fn test_database_creation_and_schema() {
        let db = create_test_database();

        // Verify tables exist by querying them
        let mut stmt = db.conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let table_names: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert!(table_names.contains(&"jobs".to_string()));
        assert!(table_names.contains(&"job_status_history".to_string()));
        assert!(table_names.contains(&"app_metadata".to_string()));

        // Verify schema version is set
        let mut stmt = db.conn.prepare("SELECT value FROM app_metadata WHERE key = 'schema_version'").unwrap();
        let version: String = stmt.query_row([], |row| {
            Ok(row.get(0)?)
        }).unwrap();
        assert_eq!(version, "1.0");
    }

    #[test]
    fn test_save_and_load_job() {
        let db = create_test_database();
        let job = create_test_job_info();

        // Test save
        assert!(db.save_job(&job).is_ok());

        // Test load
        let loaded_job = db.load_job("test_job_001").unwrap();
        assert!(loaded_job.is_some());
        let loaded_job = loaded_job.unwrap();
        assert_eq!(loaded_job.job_id, job.job_id);
        assert_eq!(loaded_job.job_name, job.job_name);
        assert_eq!(loaded_job.status, job.status);
        assert_eq!(loaded_job.project_dir, job.project_dir);
        assert_eq!(loaded_job.scratch_dir, job.scratch_dir);
    }

    #[test]
    fn test_load_nonexistent_job() {
        let db = create_test_database();

        let result = db.load_job("nonexistent_job").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_job_status() {
        let db = create_test_database();
        let job = create_test_job_info();

        // Save initial job
        db.save_job(&job).unwrap();

        // Update status
        db.update_job_status("test_job_001", JobStatus::Running, "test").unwrap();

        // Verify update
        let loaded_job = db.load_job("test_job_001").unwrap().unwrap();
        assert_eq!(loaded_job.status, JobStatus::Running);
        assert!(loaded_job.updated_at.is_some());

        // Verify status history was recorded
        let mut stmt = db.conn.prepare("SELECT status, source FROM job_status_history WHERE job_id = ? ORDER BY timestamp DESC").unwrap();
        let history: Vec<(String, String)> = stmt.query_map(["test_job_001"], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(history.len(), 2); // Initial save + update
        assert_eq!(history[0], ("RUNNING".to_string(), "test".to_string()));
        assert_eq!(history[1], ("CREATED".to_string(), "local".to_string()));
    }

    #[test]
    fn test_update_slurm_job_id() {
        let db = create_test_database();
        let job = create_test_job_info();

        // Save initial job
        db.save_job(&job).unwrap();

        // Update SLURM job ID
        db.update_slurm_job_id("test_job_001", "12345678").unwrap();

        // Verify update
        let loaded_job = db.load_job("test_job_001").unwrap().unwrap();
        assert_eq!(loaded_job.slurm_job_id, Some("12345678".to_string()));
        assert!(loaded_job.updated_at.is_some());
    }

    #[test]
    fn test_load_all_jobs() {
        let db = create_test_database();

        // Create multiple jobs
        let mut job1 = create_test_job_info();
        job1.job_id = "job_001".to_string();
        let mut job2 = create_test_job_info();
        job2.job_id = "job_002".to_string();
        job2.status = JobStatus::Running;

        db.save_job(&job1).unwrap();
        db.save_job(&job2).unwrap();

        // Load all jobs
        let all_jobs = db.load_all_jobs().unwrap();
        assert_eq!(all_jobs.len(), 2);

        // Verify jobs are sorted by created_at DESC (most recent first)
        let job_ids: Vec<_> = all_jobs.iter().map(|j| j.job_id.as_str()).collect();
        assert!(job_ids.contains(&"job_001"));
        assert!(job_ids.contains(&"job_002"));
    }

    #[test]
    fn test_get_jobs_by_status() {
        let db = create_test_database();

        // Create jobs with different statuses
        let mut job1 = create_test_job_info();
        job1.job_id = "job_001".to_string();
        job1.status = JobStatus::Created;

        let mut job2 = create_test_job_info();
        job2.job_id = "job_002".to_string();
        job2.status = JobStatus::Running;

        let mut job3 = create_test_job_info();
        job3.job_id = "job_003".to_string();
        job3.status = JobStatus::Running;

        db.save_job(&job1).unwrap();
        db.save_job(&job2).unwrap();
        db.save_job(&job3).unwrap();

        // Get running jobs
        let running_jobs = db.get_jobs_by_status(JobStatus::Running).unwrap();
        assert_eq!(running_jobs.len(), 2);

        // Get created jobs
        let created_jobs = db.get_jobs_by_status(JobStatus::Created).unwrap();
        assert_eq!(created_jobs.len(), 1);
        assert_eq!(created_jobs[0].job_id, "job_001");

        // Get completed jobs (should be empty)
        let completed_jobs = db.get_jobs_by_status(JobStatus::Completed).unwrap();
        assert_eq!(completed_jobs.len(), 0);
    }

    #[test]
    fn test_delete_job() {
        let db = create_test_database();
        let job = create_test_job_info();

        // Save job
        db.save_job(&job).unwrap();

        // Verify it exists
        assert!(db.load_job("test_job_001").unwrap().is_some());

        // Delete job
        let deleted = db.delete_job("test_job_001").unwrap();
        assert!(deleted);

        // Verify it's gone
        assert!(db.load_job("test_job_001").unwrap().is_none());

        // Verify status history is also deleted
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM job_status_history WHERE job_id = ?").unwrap();
        let count: i64 = stmt.query_row(["test_job_001"], |row| {
            Ok(row.get(0)?)
        }).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_delete_nonexistent_job() {
        let db = create_test_database();

        let deleted = db.delete_job("nonexistent_job").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_job_status_mapping() {
        let db = create_test_database();

        // Test all job status variants
        let statuses = vec![
            JobStatus::Created,
            JobStatus::Pending,
            JobStatus::Running,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ];

        for (i, status) in statuses.iter().enumerate() {
            let mut job = create_test_job_info();
            job.job_id = format!("job_{:03}", i);
            job.status = status.clone();

            // Save and reload
            db.save_job(&job).unwrap();
            let loaded_job = db.load_job(&job.job_id).unwrap().unwrap();
            assert_eq!(loaded_job.status, *status);
        }
    }

    #[test]
    fn test_sequential_operations() {
        let db = create_test_database();

        // Test multiple sequential operations
        for i in 0..5 {
            let mut job = create_test_job_info();
            job.job_id = format!("sequential_job_{}", i);

            // Save job
            assert!(db.save_job(&job).is_ok());

            // Update status
            assert!(db.update_job_status(&job.job_id, JobStatus::Running, "test").is_ok());

            // Load job back
            let loaded = db.load_job(&job.job_id).unwrap();
            assert!(loaded.is_some());
            assert_eq!(loaded.unwrap().status, JobStatus::Running);
        }

        // Verify all jobs were created
        let all_jobs = db.load_all_jobs().unwrap();
        assert_eq!(all_jobs.len(), 5);
    }

    #[test]
    fn test_status_history_tracking() {
        let db = create_test_database();
        let job = create_test_job_info();

        // Save initial job
        db.save_job(&job).unwrap();

        // Update status multiple times
        db.update_job_status("test_job_001", JobStatus::Pending, "slurm").unwrap();
        db.update_job_status("test_job_001", JobStatus::Running, "slurm").unwrap();
        db.update_job_status("test_job_001", JobStatus::Completed, "slurm").unwrap();

        // Check status history
        let mut stmt = db.conn.prepare(
            "SELECT status, source FROM job_status_history WHERE job_id = ? ORDER BY timestamp ASC"
        ).unwrap();
        let history: Vec<(String, String)> = stmt.query_map(["test_job_001"], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(history.len(), 4);
        assert_eq!(history[0], ("CREATED".to_string(), "local".to_string()));
        assert_eq!(history[1], ("PENDING".to_string(), "slurm".to_string()));
        assert_eq!(history[2], ("RUNNING".to_string(), "slurm".to_string()));
        assert_eq!(history[3], ("COMPLETED".to_string(), "slurm".to_string()));
    }

    #[test]
    fn test_job_persistence_methods() {
        // Test the JobInfo persistence methods
        use std::sync::Mutex;
        lazy_static::lazy_static! {
            static ref TEST_DB: Mutex<Option<JobDatabase>> = Mutex::new(None);
        }

        // Initialize test database in global context
        {
            let mut db_lock = TEST_DB.lock().unwrap();
            *db_lock = Some(create_test_database());
        }

        // Override the global database for testing
        let original_db = DATABASE.lock().unwrap().take();

        {
            let mut db_lock = DATABASE.lock().unwrap();
            *db_lock = TEST_DB.lock().unwrap().take();
        }

        // Test JobInfo methods
        let job = create_test_job_info();

        // Test direct database functions
        assert!(with_database(|db| db.save_job(&job)).is_ok());

        // Test load_job
        let loaded = with_database(|db| db.load_job("test_job_001")).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().job_id, job.job_id);

        // Test load_all_jobs
        let all_jobs = with_database(|db| db.load_all_jobs()).unwrap();
        assert_eq!(all_jobs.len(), 1);

        // Test load_jobs_by_status
        let created_jobs = with_database(|db| db.get_jobs_by_status(JobStatus::Created)).unwrap();
        assert_eq!(created_jobs.len(), 1);

        // Test update_job_status
        assert!(with_database(|db| db.update_job_status(&job.job_id, JobStatus::Running, "test")).is_ok());
        let updated_job = with_database(|db| db.load_job(&job.job_id)).unwrap().unwrap();
        assert_eq!(updated_job.status, JobStatus::Running);

        // Test delete_job
        assert!(with_database(|db| db.delete_job(&job.job_id)).unwrap());

        // Restore original database
        {
            let mut db_lock = DATABASE.lock().unwrap();
            *db_lock = original_db;
        }
    }
}