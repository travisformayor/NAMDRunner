use rusqlite::Connection;
use crate::types::JobInfo;
use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Simple document-store database for job caching
/// Stores entire JobInfo as JSON - no complex schema, no migrations needed
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
            -- Simple two-column document store
            CREATE TABLE IF NOT EXISTS jobs (
                job_id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );

            -- Index on status for filtering (uses JSON extraction)
            CREATE INDEX IF NOT EXISTS idx_jobs_status
            ON jobs(json_extract(data, '$.status'));
        "#)?;
        Ok(())
    }

    pub fn save_job(&self, job_info: &JobInfo) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Serialize entire JobInfo to JSON
        let json_data = serde_json::to_string(job_info)?;

        conn.execute(
            "INSERT OR REPLACE INTO jobs (job_id, data) VALUES (?1, ?2)",
            rusqlite::params![&job_info.job_id, &json_data],
        )?;

        Ok(())
    }

    pub fn load_job(&self, job_id: &str) -> Result<Option<JobInfo>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT data FROM jobs WHERE job_id = ?1")?;

        let mut rows = stmt.query([job_id])?;

        if let Some(row) = rows.next()? {
            let json_data: String = row.get(0)?;
            let job_info: JobInfo = serde_json::from_str(&json_data)?;
            Ok(Some(job_info))
        } else {
            Ok(None)
        }
    }

    pub fn load_all_jobs(&self) -> Result<Vec<JobInfo>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT data FROM jobs ORDER BY json_extract(data, '$.created_at') DESC"
        )?;

        let rows = stmt.query_map([], |row| {
            let json_data: String = row.get(0)?;
            Ok(json_data)
        })?;

        let mut jobs = Vec::new();
        for row_result in rows {
            let json_data = row_result?;
            let job_info: JobInfo = serde_json::from_str(&json_data)?;
            jobs.push(job_info);
        }

        Ok(jobs)
    }

    pub fn delete_job(&self, job_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            "DELETE FROM jobs WHERE job_id = ?1",
            [job_id],
        )?;

        Ok(rows_affected > 0)
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
