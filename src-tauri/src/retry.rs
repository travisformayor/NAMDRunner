use std::time::Duration;
use anyhow::Result;
use std::future::Future;
use rand::Rng;

/// Retry quick operations (SSH commands, queries, file checks)
/// Max 2 attempts, 200ms base delay, exponential backoff with jitter
pub async fn retry_quick<T, F, Fut>(operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    retry_with_backoff(
        operation,
        2,              // max_attempts
        200,            // base_delay_ms
        2000,           // max_delay_ms
        2.0,            // backoff_multiplier
    ).await
}

/// Retry file operations (uploads, downloads)
/// Max 5 attempts, 2s base delay, patient retry for large file transfers
pub async fn retry_files<T, F, Fut>(operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    retry_with_backoff(
        operation,
        5,              // max_attempts
        2000,           // base_delay_ms
        60000,          // max_delay_ms (60 seconds)
        1.5,            // backoff_multiplier (gentler)
    ).await
}

/// Core retry implementation with exponential backoff and jitter
async fn retry_with_backoff<T, F, Fut>(
    mut operation: F,
    max_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    backoff_multiplier: f64,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempts = 0;
    let mut delay_ms = base_delay_ms;

    loop {
        attempts += 1;

        // Try the operation
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                // Check if we should retry
                if attempts >= max_attempts || !is_transient_error(&error) {
                    return Err(error);
                }

                // Log retry attempt
                crate::log_debug!(
                    category: "Retry",
                    message: "Retrying operation",
                    details: "Attempt {}/{}, error: {}, delay: {}ms",
                    attempts,
                    max_attempts,
                    error,
                    delay_ms
                );

                // Add jitter (±50%) to prevent thundering herd
                let jitter_factor = rand::thread_rng().gen_range(0.5..1.5);
                let jittered_delay = (delay_ms as f64 * jitter_factor) as u64;

                // Wait before retry
                tokio::time::sleep(Duration::from_millis(jittered_delay)).await;

                // Calculate next delay with exponential backoff
                delay_ms = std::cmp::min(
                    (delay_ms as f64 * backoff_multiplier) as u64,
                    max_delay_ms
                );
            }
        }
    }
}

/// Determine if an error is transient and worth retrying
fn is_transient_error(error: &anyhow::Error) -> bool {
    let error_msg = error.to_string().to_lowercase();

    // Network-related errors that are often transient
    if error_msg.contains("timeout") ||
       error_msg.contains("connection") ||
       error_msg.contains("network") ||
       error_msg.contains("temporary") ||
       error_msg.contains("busy") ||
       error_msg.contains("unavailable") ||
       error_msg.contains("interrupted") ||
       error_msg.contains("broken pipe") {
        return true;
    }

    // Authentication and permission errors are permanent
    if error_msg.contains("authentication") ||
       error_msg.contains("permission") ||
       error_msg.contains("access denied") ||
       error_msg.contains("unauthorized") {
        return false;
    }

    // Default to not retrying for unknown errors
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_successful_operation_no_retry() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok::<i32, anyhow::Error>(42)
            }
        }).await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Called only once
    }

    #[tokio::test]
    async fn test_retry_with_eventual_success() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                if count < 2 {
                    Err(anyhow::anyhow!("Temporary connection error"))
                } else {
                    Ok::<i32, anyhow::Error>(count as i32)
                }
            }
        }).await;

        assert_eq!(result.unwrap(), 2);
        assert_eq!(counter.load(Ordering::SeqCst), 2); // Called twice
    }

    #[tokio::test]
    async fn test_max_attempts_exceeded() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, anyhow::Error>(anyhow::anyhow!("Connection timeout"))
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2); // retry_quick has max 2 attempts
    }

    #[tokio::test]
    async fn test_permanent_error_no_retry() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, anyhow::Error>(anyhow::anyhow!("Authentication failed"))
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // No retry for auth errors
    }

    #[test]
    fn test_transient_error_detection() {
        assert!(is_transient_error(&anyhow::anyhow!("Connection timeout")));
        assert!(is_transient_error(&anyhow::anyhow!("Network error")));
        assert!(is_transient_error(&anyhow::anyhow!("Server busy")));
        assert!(is_transient_error(&anyhow::anyhow!("Temporarily unavailable")));

        assert!(!is_transient_error(&anyhow::anyhow!("Authentication failed")));
        assert!(!is_transient_error(&anyhow::anyhow!("Permission denied")));
        assert!(!is_transient_error(&anyhow::anyhow!("Access denied")));
        assert!(!is_transient_error(&anyhow::anyhow!("Invalid configuration")));
    }

    #[tokio::test]
    async fn test_backoff_progression() {
        // Verify exponential backoff happens (timing test)
        let start = std::time::Instant::now();
        let counter = Arc::new(AtomicU32::new(0));

        let _ = retry_quick(|| {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, anyhow::Error>(anyhow::anyhow!("Timeout"))
            }
        }).await;

        let elapsed = start.elapsed();

        // Should take at least 200ms (one retry delay)
        // With jitter it could be 100-300ms
        assert!(elapsed.as_millis() >= 100);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_file_operation_retry_patience() {
        let counter = Arc::new(AtomicU32::new(0));

        let result = retry_files(|| {
            let counter = counter.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                if count < 3 {
                    Err(anyhow::anyhow!("Network timeout"))
                } else {
                    Ok::<i32, anyhow::Error>(count as i32)
                }
            }
        }).await;

        assert_eq!(result.unwrap(), 3);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // File retry is more patient
    }
}
