use serde::{Deserialize, Serialize};
use std::sync::Once;
use tauri::{AppHandle, Emitter};
use log::{Level, Log, Metadata, Record};

static LOGGER_INIT: Once = Once::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: String,
    pub message: String,
    pub target: String,
    pub timestamp: String,
}

pub struct TauriLogger {
    app_handle: Option<AppHandle>,
}

impl TauriLogger {
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }
}

impl Log for TauriLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_message = LogMessage {
                level: record.level().to_string().to_lowercase(),
                message: record.args().to_string(),
                target: record.target().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            // Send to frontend if we have an app handle
            if let Some(app_handle) = &self.app_handle {
                let _ = app_handle.emit("rust-log", &log_message);
            }

            // Also print to console for development
            println!("[{}] [{}] {}", log_message.level.to_uppercase(), log_message.target, log_message.message);
        }
    }

    fn flush(&self) {}
}

static mut LOGGER: TauriLogger = TauriLogger { app_handle: None };

/// Initialize the logging system
pub fn init_logging() {
    LOGGER_INIT.call_once(|| {
        unsafe {
            #[allow(static_mut_refs)]
            log::set_logger(&LOGGER)
                .map(|()| log::set_max_level(log::LevelFilter::Debug))
                .expect("Failed to initialize logger");
        }
    });
}

/// Set the app handle for the logger (call this after Tauri app is initialized)
pub fn set_app_handle(app_handle: AppHandle) {
    unsafe {
        #[allow(static_mut_refs)]
        LOGGER.set_app_handle(app_handle);
    }
}

/// Macro for debug logging that shows up in frontend console
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

/// Macro for info logging that shows up in frontend console
#[macro_export]
macro_rules! info_log {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

/// Macro for error logging that shows up in frontend console
#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
    };
}

/// Macro for warning logging that shows up in frontend console
#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        log::warn!($($arg)*);
    };
}