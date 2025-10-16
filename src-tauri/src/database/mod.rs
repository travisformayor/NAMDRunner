use rusqlite::{Connection, Result as SqliteResult, Row};
use crate::types::{JobInfo, JobStatus, NAMDConfig, SlurmConfig};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct JobDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl JobDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        Self::initialize_schema(&conn)?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
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
                error_info TEXT,
                slurm_stdout TEXT,
                slurm_stderr TEXT,
                namd_config TEXT,
                slurm_config TEXT,
                input_files TEXT,
                remote_directory TEXT
            );

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
            CREATE INDEX IF NOT EXISTS idx_jobs_slurm_id ON jobs(slurm_job_id);
            CREATE INDEX IF NOT EXISTS idx_jobs_updated ON jobs(updated_at);
        "#)?;
        Ok(())
    }

    pub fn save_job(&self, job_info: &JobInfo) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let status_str = match &job_info.status {
            JobStatus::Created => "CREATED",
            JobStatus::Pending => "PENDING",
            JobStatus::Running => "RUNNING",
            JobStatus::Completed => "COMPLETED",
            JobStatus::Failed => "FAILED",
            JobStatus::Cancelled => "CANCELLED",
        };

        // Serialize complex fields to JSON
        let namd_config_json = serde_json::to_string(&job_info.namd_config)?;
        let slurm_config_json = serde_json::to_string(&job_info.slurm_config)?;
        let input_files_json = serde_json::to_string(&job_info.input_files)?;

        conn.execute(
            "INSERT OR REPLACE INTO jobs
             (job_id, job_name, status, slurm_job_id, created_at, updated_at, submitted_at, completed_at, project_dir, scratch_dir, error_info, slurm_stdout, slurm_stderr, namd_config, slurm_config, input_files, remote_directory)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            &[
                &job_info.job_id as &dyn rusqlite::ToSql,
                &job_info.job_name as &dyn rusqlite::ToSql,
                &status_str as &dyn rusqlite::ToSql,
                &job_info.slurm_job_id as &dyn rusqlite::ToSql,
                &job_info.created_at as &dyn rusqlite::ToSql,
                &job_info.updated_at as &dyn rusqlite::ToSql,
                &job_info.submitted_at as &dyn rusqlite::ToSql,
                &job_info.completed_at as &dyn rusqlite::ToSql,
                &job_info.project_dir as &dyn rusqlite::ToSql,
                &job_info.scratch_dir as &dyn rusqlite::ToSql,
                &job_info.error_info as &dyn rusqlite::ToSql,
                &job_info.slurm_stdout as &dyn rusqlite::ToSql,
                &job_info.slurm_stderr as &dyn rusqlite::ToSql,
                &namd_config_json as &dyn rusqlite::ToSql,
                &slurm_config_json as &dyn rusqlite::ToSql,
                &input_files_json as &dyn rusqlite::ToSql,
                &job_info.remote_directory as &dyn rusqlite::ToSql,
            ] as &[&dyn rusqlite::ToSql],
        )?;

        Ok(())
    }

    pub fn load_job(&self, job_id: &str) -> Result<Option<JobInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT job_id, job_name, status, slurm_job_id, created_at, updated_at, submitted_at, completed_at, project_dir, scratch_dir, error_info, slurm_stdout, slurm_stderr, namd_config, slurm_config, input_files, remote_directory
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
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT job_id, job_name, status, slurm_job_id, created_at, updated_at, submitted_at, completed_at, project_dir, scratch_dir, error_info, slurm_stdout, slurm_stderr, namd_config, slurm_config, input_files, remote_directory
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

    pub fn delete_job(&self, job_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        // Delete the job
        let rows_affected = conn.execute(
            "DELETE FROM jobs WHERE job_id = ?1",
            [job_id],
        )?;

        Ok(rows_affected > 0)
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

        // Deserialize JSON fields, fallback to defaults if missing or invalid
        let namd_config: NAMDConfig = row.get::<_, Option<String>>(13)?
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let slurm_config: SlurmConfig = row.get::<_, Option<String>>(14)?
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let input_files: Vec<crate::types::InputFile> = row.get::<_, Option<String>>(15)?
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

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
            slurm_stdout: row.get(11)?,
            slurm_stderr: row.get(12)?,
            namd_config,
            slurm_config,
            input_files,
            remote_directory: row.get::<_, Option<String>>(16)?.unwrap_or_else(|| row.get::<_, Option<String>>(8).unwrap_or(Some("/tmp".to_string())).unwrap_or_else(|| "/tmp".to_string())),
        })
    }
}

// Thread-safe global database instance
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

/// Synchronous database access
/// SQLite operations are fast (microseconds) and don't benefit from spawn_blocking overhead.
/// Desktop apps don't need the complexity of async database wrappers.
pub fn with_database<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&JobDatabase) -> Result<R>,
{
    let database_lock = DATABASE.lock().unwrap();
    let db = database_lock.as_ref()
        .ok_or_else(|| anyhow!("Database not initialized"))?;
    f(db)
}

