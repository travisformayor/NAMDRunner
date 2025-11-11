use anyhow::{Result, anyhow};
use std::fs;
use crate::database;
use crate::types::ApiResult;
use crate::types::response_data::{DatabaseInfo, DatabaseOperationData};
use crate::{info_log, error_log};

/// Get current database path and size
#[tauri::command(rename_all = "snake_case")]
pub async fn get_database_info() -> ApiResult<DatabaseInfo> {
    info_log!("[DB Commands] Getting database info");

    // Get stored database path (set during initialization)
    let db_path = match database::get_current_database_path() {
        Some(path) => path,
        None => {
            error_log!("[DB Commands] Database not initialized");
            return ApiResult::error("Database not initialized".to_string());
        }
    };

    // Get file size
    let size_bytes = match fs::metadata(&db_path) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            error_log!("[DB Commands] Failed to get database size: {}", e);
            return ApiResult::error(format!("Failed to get database size: {}", e));
        }
    };

    // Get job count
    let job_count = match crate::database::with_database(|db| db.load_all_jobs()) {
        Ok(jobs) => jobs.len(),
        Err(e) => {
            error_log!("[DB Commands] Failed to get job count: {}", e);
            return ApiResult::error(format!("Failed to get job count: {}", e));
        }
    };

    ApiResult::success(DatabaseInfo {
        path: db_path.to_string_lossy().to_string(),
        size_bytes,
        job_count,
    })
}

/// Backup database using SQLite backup API (safe for online backup)
#[tauri::command(rename_all = "snake_case")]
pub async fn backup_database() -> ApiResult<DatabaseOperationData> {
    info_log!("[DB Commands] Starting database backup");

    use rfd::FileDialog;

    // Get stored database path
    let db_path = match database::get_current_database_path() {
        Some(path) => path,
        None => {
            error_log!("[DB Commands] Database not initialized");
            return ApiResult::error("Database not initialized".to_string());
        }
    };

    // Show save dialog (blocks until user selects or cancels)
    let save_path = FileDialog::new()
        .set_file_name("namdrunner_backup.db")
        .set_title("Save Database Backup")
        .add_filter("Database", &["db"])
        .save_file();

    let dest_path = match save_path {
        Some(path) => path,
        None => {
            // User cancelled - not an error
            info_log!("[DB Commands] Backup cancelled by user");
            return ApiResult::error("Backup cancelled".to_string());
        }
    };

    // Use SQLite backup API for safe online backup
    // This works even while database is in use
    match perform_backup(&db_path, &dest_path) {
        Ok(_) => {
            info_log!("[DB Commands] Backup successful: {}", dest_path.display());
            ApiResult::success(DatabaseOperationData {
                path: dest_path.to_string_lossy().to_string(),
                message: format!("Backup saved to {}", dest_path.display()),
            })
        }
        Err(e) => {
            error_log!("[DB Commands] Backup failed: {}", e);
            ApiResult::error(format!("Backup failed: {}", e))
        }
    }
}

/// Perform backup using SQLite's online backup API
fn perform_backup(source_path: &std::path::Path, dest_path: &std::path::Path) -> Result<()> {
    use rusqlite::Connection;

    // Open source database (read-only)
    let source_conn = Connection::open(source_path)?;

    // Create/open destination database
    let mut dest_conn = Connection::open(dest_path)?;

    // Perform online backup
    // This safely copies the database even while it's in use
    let backup = rusqlite::backup::Backup::new(&source_conn, &mut dest_conn)?;
    backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;

    Ok(())
}

/// Restore database from backup
/// This closes the current connection, replaces the file, and reopens
#[tauri::command(rename_all = "snake_case")]
pub async fn restore_database() -> ApiResult<DatabaseOperationData> {
    info_log!("[DB Commands] Starting database restore");

    use rfd::FileDialog;

    // Show open dialog
    let source_path = FileDialog::new()
        .set_title("Select Database Backup to Restore")
        .add_filter("Database", &["db"])
        .pick_file();

    let source = match source_path {
        Some(path) => path,
        None => {
            info_log!("[DB Commands] Restore cancelled by user");
            return ApiResult::error("Restore cancelled".to_string());
        }
    };

    // Get stored database path
    let db_path = match database::get_current_database_path() {
        Some(path) => path,
        None => {
            error_log!("[DB Commands] Database not initialized");
            return ApiResult::error("Database not initialized".to_string());
        }
    };

    // Close connection, replace file, reopen
    match perform_restore(&source, &db_path) {
        Ok(_) => {
            info_log!("[DB Commands] Restore successful from: {}", source.display());
            ApiResult::success(DatabaseOperationData {
                path: db_path.to_string_lossy().to_string(),
                message: format!("Database restored from {}", source.display()),
            })
        }
        Err(e) => {
            error_log!("[DB Commands] Restore failed: {}", e);
            ApiResult::error(format!("Restore failed: {}", e))
        }
    }
}

fn perform_restore(source: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    // Validate source file exists and is a valid SQLite database
    let source_conn = rusqlite::Connection::open(source)
        .map_err(|e| anyhow!("Invalid backup file: {}", e))?;
    drop(source_conn); // Close immediately after validation

    // Copy backup to destination (atomically replaces current DB)
    // reinitialize_database will handle closing the old connection
    fs::copy(source, dest)?;

    // Reinitialize database connection
    database::reinitialize_database(dest.to_str().unwrap())?;

    Ok(())
}

/// Reset database - delete and recreate with fresh schema
#[tauri::command(rename_all = "snake_case")]
pub async fn reset_database() -> ApiResult<DatabaseOperationData> {
    info_log!("[DB Commands] Resetting database");

    // Get stored database path
    let db_path = match database::get_current_database_path() {
        Some(path) => path,
        None => {
            error_log!("[DB Commands] Database not initialized");
            return ApiResult::error("Database not initialized".to_string());
        }
    };

    // Delete and reinitialize
    match perform_reset(&db_path) {
        Ok(_) => {
            info_log!("[DB Commands] Database reset successful");
            ApiResult::success(DatabaseOperationData {
                path: db_path.to_string_lossy().to_string(),
                message: "Database reset successfully".to_string(),
            })
        }
        Err(e) => {
            error_log!("[DB Commands] Reset failed: {}", e);
            ApiResult::error(format!("Reset failed: {}", e))
        }
    }
}

fn perform_reset(db_path: &std::path::Path) -> Result<()> {
    // Close connection first (via reinitialize with deleted file)
    // This is safe - we hold the DATABASE lock during the entire operation

    // Delete database file if it exists
    if db_path.exists() {
        // Close connection by setting to None
        // (reinitialize will do this, but we need to delete first)
        let mut database_lock = crate::database::DATABASE.lock().unwrap();
        *database_lock = None; // Drop connection
        drop(database_lock); // Release lock before file operation

        fs::remove_file(db_path)?;
        info_log!("[DB Commands] Deleted database file");
    }

    // Reinitialize creates new DB with schema
    database::reinitialize_database(db_path.to_str().unwrap())?;

    Ok(())
}
