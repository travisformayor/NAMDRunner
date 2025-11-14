use log::{Level, Log, Metadata, Record};
use std::sync::Once;
use tauri::AppHandle;

static LOGGER_INIT: Once = Once::new();

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
            // Extract category and details from the message
            let message = record.args().to_string();

            // Parse the message format: [category] {details or message}
            let (category, actual_message) = if message.starts_with('[') {
                if let Some(end_bracket) = message.find(']') {
                    let cat = message[1..end_bracket].to_string();
                    let rest = message[end_bracket + 1..].trim().to_string();
                    (cat, rest)
                } else {
                    ("General".to_string(), message)
                }
            } else {
                ("General".to_string(), message)
            };

            // Format for console: [LEVEL] [category] message
            println!(
                "[{}] [{}] {}",
                record.level().to_string().to_uppercase(),
                category,
                actual_message
            );
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

/// Unified logging macro with named parameters
#[macro_export]
macro_rules! app_log {
    // Full form: level, category, message, details, toast
    (level: $level:ident, category: $category:expr, message: $message:expr, details: $details:expr, toast: $toast:expr) => {{
        let details_str = $details.to_string();
        let log_msg = $crate::types::core::AppLogMessage {
            level: stringify!($level).to_lowercase(),
            category: $category.to_string(),
            message: $message.to_string(),
            details: Some(details_str.clone()),
            show_toast: $toast,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Log with details for console
        match stringify!($level) {
            "info" => log::info!("[{}] {}", $category, details_str),
            "error" => log::error!("[{}] {}", $category, details_str),
            "warn" => log::warn!("[{}] {}", $category, details_str),
            "debug" => log::debug!("[{}] {}", $category, details_str),
            _ => log::info!("[{}] {}", $category, details_str),
        }

        // Emit to frontend
        if let Some(app_handle) = $crate::logging::get_app_handle() {
            use tauri::Emitter;
            let _ = app_handle.emit("app-log", &log_msg);
        }
    }};

    // With formatted details but no toast
    (level: $level:ident, category: $category:expr, message: $message:expr, details: $($details_args:tt)+) => {
        $crate::app_log!(level: $level, category: $category, message: $message, details: format!($($details_args)+), toast: false)
    };

    // With details and toast
    (level: $level:ident, category: $category:expr, message: $message:expr, details: $details:expr, toast: $toast:expr) => {{
        let log_msg = $crate::types::core::AppLogMessage {
            level: stringify!($level).to_lowercase(),
            category: $category.to_string(),
            message: $message.to_string(),
            details: Some($details.to_string()),
            show_toast: $toast,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Log with details for console
        match stringify!($level) {
            "info" => log::info!("[{}] {}", $category, $details),
            "error" => log::error!("[{}] {}", $category, $details),
            "warn" => log::warn!("[{}] {}", $category, $details),
            "debug" => log::debug!("[{}] {}", $category, $details),
            _ => log::info!("[{}] {}", $category, $details),
        }

        // Emit to frontend
        if let Some(app_handle) = $crate::logging::get_app_handle() {
            use tauri::Emitter;
            let _ = app_handle.emit("app-log", &log_msg);
        }
    }};

    // With toast but no details
    (level: $level:ident, category: $category:expr, message: $message:expr, toast: $toast:expr) => {{
        let log_msg = $crate::types::core::AppLogMessage {
            level: stringify!($level).to_lowercase(),
            category: $category.to_string(),
            message: $message.to_string(),
            details: None,
            show_toast: $toast,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Log with message for console
        match stringify!($level) {
            "info" => log::info!("[{}] {}", $category, $message),
            "error" => log::error!("[{}] {}", $category, $message),
            "warn" => log::warn!("[{}] {}", $category, $message),
            "debug" => log::debug!("[{}] {}", $category, $message),
            _ => log::info!("[{}] {}", $category, $message),
        }

        // Emit to frontend
        if let Some(app_handle) = $crate::logging::get_app_handle() {
            use tauri::Emitter;
            let _ = app_handle.emit("app-log", &log_msg);
        }
    }};

    // Basic form: level, category, message (no details, no toast)
    (level: $level:ident, category: $category:expr, message: $message:expr) => {
        $crate::app_log!(level: $level, category: $category, message: $message, toast: false)
    };
}

/// Convenience macro for info logging
#[macro_export]
macro_rules! log_info {
    (category: $category:expr, message: $message:expr, details: $($details_args:tt)+) => {
        $crate::app_log!(level: info, category: $category, message: $message, details: $($details_args)+)
    };
    (category: $category:expr, message: $message:expr) => {
        $crate::app_log!(level: info, category: $category, message: $message)
    };
}

/// Convenience macro for error logging
#[macro_export]
macro_rules! log_error {
    (category: $category:expr, message: $message:expr, details: $($details_args:tt)+) => {
        $crate::app_log!(level: error, category: $category, message: $message, details: $($details_args)+)
    };
    (category: $category:expr, message: $message:expr) => {
        $crate::app_log!(level: error, category: $category, message: $message)
    };
}

/// Convenience macro for warning logging
#[macro_export]
macro_rules! log_warn {
    (category: $category:expr, message: $message:expr, details: $($details_args:tt)+) => {
        $crate::app_log!(level: warn, category: $category, message: $message, details: $($details_args)+)
    };
    (category: $category:expr, message: $message:expr) => {
        $crate::app_log!(level: warn, category: $category, message: $message)
    };
}

/// Convenience macro for debug logging
#[macro_export]
macro_rules! log_debug {
    (category: $category:expr, message: $message:expr, details: $($details_args:tt)+) => {
        $crate::app_log!(level: debug, category: $category, message: $message, details: $($details_args)+)
    };
    (category: $category:expr, message: $message:expr) => {
        $crate::app_log!(level: debug, category: $category, message: $message)
    };
}

/// Convenience macro for toast notifications (info level with toast enabled)
#[macro_export]
macro_rules! toast_log {
    (category: $category:expr, message: $message:expr, details: $($details_args:tt)+) => {
        $crate::app_log!(level: info, category: $category, message: $message, details: format!($($details_args)+), toast: true)
    };
    (category: $category:expr, message: $message:expr) => {
        $crate::app_log!(level: info, category: $category, message: $message, toast: true)
    };
}

/// Get the current app handle (for internal use by macros)
pub fn get_app_handle() -> Option<AppHandle> {
    unsafe {
        #[allow(static_mut_refs)]
        LOGGER.app_handle.clone()
    }
}
