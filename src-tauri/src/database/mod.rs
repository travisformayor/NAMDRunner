use rusqlite::Connection;
use crate::types::JobInfo;
use crate::templates::{Template, TemplateSummary};
use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::{log_info, log_debug};
use tauri::Manager;

/// Simple document-store database for jobs and templates
/// Stores JobInfo and Template as JSON - no complex schema, no migrations needed
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
            -- Jobs table - stores JobInfo as JSON
            CREATE TABLE IF NOT EXISTS jobs (
                job_id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );

            -- Index on status for filtering (uses JSON extraction)
            CREATE INDEX IF NOT EXISTS idx_jobs_status
            ON jobs(json_extract(data, '$.status'));

            -- Templates table - stores Template as JSON
            CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                namd_config_template TEXT NOT NULL,
                variables TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
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

    // Template CRUD operations

    pub fn save_template(&self, template: &Template) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Serialize variables to JSON
        let variables_json = serde_json::to_string(&template.variables)?;

        conn.execute(
            "INSERT OR REPLACE INTO templates (id, name, description, namd_config_template, variables, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                &template.id,
                &template.name,
                &template.description,
                &template.namd_config_template,
                &variables_json,
                &template.created_at,
                &template.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn load_template(&self, id: &str) -> Result<Option<Template>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, description, namd_config_template, variables, created_at, updated_at FROM templates WHERE id = ?1"
        )?;

        let mut rows = stmt.query([id])?;

        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let namd_config_template: String = row.get(3)?;
            let variables_json: String = row.get(4)?;
            let created_at: String = row.get(5)?;
            let updated_at: String = row.get(6)?;

            let variables = serde_json::from_str(&variables_json)?;

            Ok(Some(Template {
                id,
                name,
                description,
                namd_config_template,
                variables,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_templates(&self) -> Result<Vec<TemplateSummary>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, description FROM templates ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TemplateSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;

        let mut templates = Vec::new();
        for row_result in rows {
            templates.push(row_result?);
        }

        Ok(templates)
    }

    pub fn delete_template(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            "DELETE FROM templates WHERE id = ?1",
            [id],
        )?;

        Ok(rows_affected > 0)
    }

    /// Count how many jobs use a specific template
    pub fn count_jobs_using_template(&self, template_id: &str) -> Result<u32> {
        let conn = self.conn.lock().unwrap();

        // Jobs stored as JSON - use json_extract to query template_id field
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE json_extract(data, '$.template_id') = ?1",
            [template_id],
            |row| row.get(0),
        )?;

        Ok(count)
    }
}

// Thread-safe global database instance
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;

lazy_static! {
    pub static ref DATABASE: Arc<Mutex<Option<JobDatabase>>> = Arc::new(Mutex::new(None));
    static ref DATABASE_PATH: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
}

// Track if we've attempted to load default templates this session
static DEFAULTS_LOADED: AtomicBool = AtomicBool::new(false);

/// Get the database file path (development vs production)
pub fn get_database_path(app_handle: &tauri::AppHandle) -> Result<PathBuf> {
    if cfg!(debug_assertions) {
        // Development: current directory
        Ok(PathBuf::from("./namdrunner_dev.db"))
    } else {
        // Production: OS-specific app data directory
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| anyhow!("Failed to resolve app data directory: {}", e))?;

        // Ensure directory exists
        std::fs::create_dir_all(&app_data_dir)?;

        Ok(app_data_dir.join("namdrunner.db"))
    }
}

pub fn initialize_database(db_path: &str) -> Result<()> {
    log_info!(
        category: "Database",
        message: "Initializing database",
        details: "{}", db_path
    );

    let db = JobDatabase::new(db_path)?;

    let mut database_lock = DATABASE.lock().unwrap();
    *database_lock = Some(db);

    // Track current path
    let mut path_lock = DATABASE_PATH.lock().unwrap();
    *path_lock = Some(PathBuf::from(db_path));

    Ok(())
}

/// Close and reinitialize database connection
/// Used for restore and reset operations
pub fn reinitialize_database(db_path: &str) -> Result<()> {
    log_info!(
        category: "Database",
        message: "Reinitializing database connection",
        details: "{}", db_path
    );

    let mut database_lock = DATABASE.lock().unwrap();

    // Drop existing connection (closes SQLite connection)
    if database_lock.is_some() {
        log_debug!(
            category: "Database",
            message: "Closing existing connection"
        );
        *database_lock = None;
        // JobDatabase is dropped here, closing the Connection
    }

    // Create new connection
    log_debug!(
        category: "Database",
        message: "Opening new connection"
    );
    let db = JobDatabase::new(db_path)?;
    *database_lock = Some(db);

    // Update tracked path
    let mut path_lock = DATABASE_PATH.lock().unwrap();
    *path_lock = Some(PathBuf::from(db_path));

    // Reset default templates flag (force reload for new DB)
    DEFAULTS_LOADED.store(false, Ordering::Relaxed);

    log_info!(
        category: "Database",
        message: "Database reinitialized successfully"
    );
    Ok(())
}

/// Get current database file path (for displaying in UI)
/// Returns None if database not initialized
pub fn get_current_database_path() -> Option<PathBuf> {
    DATABASE_PATH.lock().unwrap().clone()
}

/// Ensure default templates are loaded (idempotent - safe to call multiple times)
/// Called on first template list to ensure defaults exist
/// Uses atomic flag to only attempt loading once per app session
pub fn ensure_default_templates_loaded() -> Result<()> {
    // Check if we've already attempted to load defaults this session
    if DEFAULTS_LOADED.load(Ordering::Relaxed) {
        // Already loaded (or attempted) this session
        return Ok(());
    }

    // Mark as attempted (do this first to prevent concurrent loads)
    DEFAULTS_LOADED.store(true, Ordering::Relaxed);

    // Load defaults from JSON files
    log_info!(
        category: "Templates",
        message: "First template access - checking for default templates"
    );
    with_database(load_default_templates)
}

/// Load default templates embedded in the binary at compile time
fn load_default_templates(db: &JobDatabase) -> Result<()> {
    // Embed template JSON files at compile time using include_str!
    // Path is relative to this file: src-tauri/src/database/mod.rs
    // Templates are at: src-tauri/templates/
    const TEMPLATE_FILES: &[(&str, &str)] = &[
        ("vacuum_optimization_v1", include_str!("../../templates/vacuum_optimization_v1.json")),
        ("explicit_solvent_npt_v1", include_str!("../../templates/explicit_solvent_npt_v1.json")),
    ];

    log_info!(
        category: "Templates",
        message: "Loading default templates from embedded data",
        details: "Loading {} default template(s)", TEMPLATE_FILES.len()
    );

    for (template_id, json_content) in TEMPLATE_FILES {
        // Parse template JSON
        let template: Template = serde_json::from_str(json_content)
            .map_err(|e| anyhow!("Failed to parse embedded template {}: {}", template_id, e))?;

        // Check if template already exists in database
        if db.load_template(&template.id)?.is_none() {
            // Template doesn't exist, insert it
            db.save_template(&template)?;
            log_info!(
                category: "Templates",
                message: "Loaded default template",
                details: "{}", template.name
            );
        }
    }

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
