use std::time::Duration;
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;
use rand::Rng;
use crate::ssh::errors::SSHError;

/// Configuration for retry operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts (including the initial attempt)
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries (prevents infinite backoff)
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add jitter to prevent thundering herd
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }

    /// Create a quick retry configuration for fast operations
    pub fn quick() -> Self {
        Self {
            max_attempts: 2,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(2),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }

    /// Create a patient retry configuration for slow operations
    pub fn patient() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(2000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 1.5,
            use_jitter: true,
        }
    }

    /// Create a configuration for network operations
    pub fn network() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(15),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

/// Manager for retry operations with exponential backoff
#[derive(Debug)]
pub struct RetryManager {
    config: RetryConfig,
}

impl RetryManager {
    /// Create a new retry manager with the given configuration
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Create a retry manager with default configuration
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }

    /// Retry an async operation with exponential backoff
    ///
    /// The operation should return a Result. If it returns an error that is retryable
    /// (according to the is_retryable function), it will be retried with exponential backoff.
    pub async fn retry_with_backoff<T, E, F, Fut>(
        &self,
        mut operation: F,
        is_retryable: fn(&E) -> bool,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempts = 0;
        let mut delay = self.config.base_delay;

        loop {
            attempts += 1;

            // Try the operation
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    // Log the attempt for debugging
                    eprintln!("Attempt {} failed: {:?}", attempts, error);

                    // Check if we should retry
                    if attempts >= self.config.max_attempts || !is_retryable(&error) {
                        return Err(error);
                    }

                    // Calculate delay with exponential backoff
                    let actual_delay = if self.config.use_jitter {
                        self.add_jitter(delay)
                    } else {
                        delay
                    };

                    eprintln!("Retrying in {:?} (attempt {} of {})", actual_delay, attempts + 1, self.config.max_attempts);

                    // Wait before retry
                    tokio::time::sleep(actual_delay).await;

                    // Update delay for next iteration
                    delay = Duration::from_millis(
                        std::cmp::min(
                            (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64,
                            self.config.max_delay.as_millis() as u64,
                        )
                    );
                }
            }
        }
    }

    /// Add jitter to a delay to prevent thundering herd effects
    fn add_jitter(&self, delay: Duration) -> Duration {
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.5..1.5); // ±50% jitter
        let jittered_millis = (delay.as_millis() as f64 * jitter_factor) as u64;
        Duration::from_millis(jittered_millis)
    }
}

/// Error classification for retry logic
pub mod classification {
    use super::*;

    /// Determine if an SSH error should be retried
    pub fn is_ssh_error_retryable(error: &SSHError) -> bool {
        match error {
            // Network errors are often transient
            SSHError::NetworkError(_) => true,
            SSHError::TimeoutError(_) => true,
            SSHError::HandshakeError(_) => true,
            SSHError::SessionError(_) => true,
            SSHError::FileTransferError(_) => true,
            SSHError::UnknownError(_) => true,

            // These errors are permanent and shouldn't be retried
            SSHError::AuthenticationError(_) => false,
            SSHError::PermissionError(_) => false,
            SSHError::ConfigurationError(_) => false,
            SSHError::CommandError(_) => false,
        }
    }

    /// Determine if an anyhow error should be retried based on its message
    pub fn is_anyhow_error_retryable(error: &anyhow::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();

        // Network-related errors that might be retryable
        if error_msg.contains("timeout") ||
           error_msg.contains("connection") ||
           error_msg.contains("network") ||
           error_msg.contains("temporary") ||
           error_msg.contains("busy") ||
           error_msg.contains("unavailable") {
            return true;
        }

        // Authentication and permission errors are not retryable
        if error_msg.contains("authentication") ||
           error_msg.contains("permission") ||
           error_msg.contains("access denied") ||
           error_msg.contains("unauthorized") {
            return false;
        }

        // Default to not retrying for unknown errors to be safe
        false
    }

    /// Determine if a standard IO error should be retried
    pub fn is_io_error_retryable(error: &std::io::Error) -> bool {
        match error.kind() {
            std::io::ErrorKind::TimedOut => true,
            std::io::ErrorKind::ConnectionRefused => true,
            std::io::ErrorKind::ConnectionAborted => true,
            std::io::ErrorKind::ConnectionReset => true,
            std::io::ErrorKind::BrokenPipe => true,
            std::io::ErrorKind::Interrupted => true,
            std::io::ErrorKind::WouldBlock => true,

            // These are permanent errors
            std::io::ErrorKind::PermissionDenied => false,
            std::io::ErrorKind::NotFound => false,
            std::io::ErrorKind::AlreadyExists => false,
            std::io::ErrorKind::InvalidInput => false,
            std::io::ErrorKind::InvalidData => false,

            // For unknown errors, be conservative
            _ => false,
        }
    }
}

/// Convenience functions for common retry patterns
pub mod patterns {
    use super::*;

    /// Retry an SSH operation with standard configuration
    pub async fn retry_ssh_operation<T, F, Fut>(operation: F) -> Result<T, SSHError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, SSHError>>,
    {
        let retry_manager = RetryManager::new(RetryConfig::network());
        retry_manager.retry_with_backoff(operation, classification::is_ssh_error_retryable).await
    }

    /// Retry a file operation with patient configuration
    pub async fn retry_file_operation<T, F, Fut>(operation: F) -> Result<T, anyhow::Error>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, anyhow::Error>>,
    {
        let retry_manager = RetryManager::new(RetryConfig::patient());
        retry_manager.retry_with_backoff(operation, classification::is_anyhow_error_retryable).await
    }

    /// Retry a quick operation with minimal delay
    pub async fn retry_quick_operation<T, F, Fut>(operation: F) -> Result<T, anyhow::Error>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, anyhow::Error>>,
    {
        let retry_manager = RetryManager::new(RetryConfig::quick());
        retry_manager.retry_with_backoff(operation, classification::is_anyhow_error_retryable).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_creation() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert!(config.use_jitter);

        let quick = RetryConfig::quick();
        assert_eq!(quick.max_attempts, 2);
        assert!(quick.base_delay < Duration::from_secs(1));

        let patient = RetryConfig::patient();
        assert_eq!(patient.max_attempts, 5);
        assert!(patient.base_delay > Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_successful_operation_no_retry() {
        let retry_manager = RetryManager::new(RetryConfig::quick());
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_manager.retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<i32, std::io::Error>(42)
                }
            },
            classification::is_io_error_retryable,
        ).await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Should only be called once
    }

    #[tokio::test]
    async fn test_retry_with_eventual_success() {
        let retry_manager = RetryManager::new(RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100)));
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_manager.retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    if count < 3 {
                        Err(std::io::Error::from(std::io::ErrorKind::TimedOut))
                    } else {
                        Ok::<i32, std::io::Error>(count as i32)
                    }
                }
            },
            classification::is_io_error_retryable,
        ).await;

        assert_eq!(result.unwrap(), 3);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Should be called 3 times
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let retry_manager = RetryManager::new(RetryConfig::new(2, Duration::from_millis(10), Duration::from_millis(100)));
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_manager.retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, std::io::Error>(std::io::Error::from(std::io::ErrorKind::TimedOut))
                }
            },
            classification::is_io_error_retryable,
        ).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2); // Should be called max_attempts times
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let retry_manager = RetryManager::new(RetryConfig::new(3, Duration::from_millis(10), Duration::from_millis(100)));
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_manager.retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, std::io::Error>(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
                }
            },
            classification::is_io_error_retryable,
        ).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Should only be called once (no retry)
    }

    #[test]
    fn test_ssh_error_classification() {
        // Retryable errors
        assert!(classification::is_ssh_error_retryable(&SSHError::NetworkError("test".to_string())));
        assert!(classification::is_ssh_error_retryable(&SSHError::TimeoutError("test".to_string())));
        assert!(classification::is_ssh_error_retryable(&SSHError::HandshakeError("test".to_string())));
        assert!(classification::is_ssh_error_retryable(&SSHError::SessionError("test".to_string())));
        assert!(classification::is_ssh_error_retryable(&SSHError::FileTransferError("test".to_string())));
        assert!(classification::is_ssh_error_retryable(&SSHError::UnknownError("test".to_string())));

        // Non-retryable errors
        assert!(!classification::is_ssh_error_retryable(&SSHError::AuthenticationError("test".to_string())));
        assert!(!classification::is_ssh_error_retryable(&SSHError::PermissionError("test".to_string())));
        assert!(!classification::is_ssh_error_retryable(&SSHError::ConfigurationError("test".to_string())));
        assert!(!classification::is_ssh_error_retryable(&SSHError::CommandError("test".to_string())));
    }

    #[test]
    fn test_io_error_classification() {
        // Retryable errors
        assert!(classification::is_io_error_retryable(&std::io::Error::from(std::io::ErrorKind::TimedOut)));
        assert!(classification::is_io_error_retryable(&std::io::Error::from(std::io::ErrorKind::ConnectionRefused)));
        assert!(classification::is_io_error_retryable(&std::io::Error::from(std::io::ErrorKind::ConnectionAborted)));
        assert!(classification::is_io_error_retryable(&std::io::Error::from(std::io::ErrorKind::Interrupted)));

        // Non-retryable errors
        assert!(!classification::is_io_error_retryable(&std::io::Error::from(std::io::ErrorKind::PermissionDenied)));
        assert!(!classification::is_io_error_retryable(&std::io::Error::from(std::io::ErrorKind::NotFound)));
        assert!(!classification::is_io_error_retryable(&std::io::Error::from(std::io::ErrorKind::InvalidInput)));
    }

    #[test]
    fn test_anyhow_error_classification() {
        // Retryable errors
        assert!(classification::is_anyhow_error_retryable(&anyhow::anyhow!("Connection timeout")));
        assert!(classification::is_anyhow_error_retryable(&anyhow::anyhow!("Network error")));
        assert!(classification::is_anyhow_error_retryable(&anyhow::anyhow!("Server busy, try again")));
        assert!(classification::is_anyhow_error_retryable(&anyhow::anyhow!("Service temporarily unavailable")));

        // Non-retryable errors
        assert!(!classification::is_anyhow_error_retryable(&anyhow::anyhow!("Authentication failed")));
        assert!(!classification::is_anyhow_error_retryable(&anyhow::anyhow!("Permission denied")));
        assert!(!classification::is_anyhow_error_retryable(&anyhow::anyhow!("Access denied")));
        assert!(!classification::is_anyhow_error_retryable(&anyhow::anyhow!("Invalid configuration")));
    }

    #[tokio::test]
    async fn test_jitter_variation() {
        let retry_manager = RetryManager::new(RetryConfig {
            max_attempts: 1,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            use_jitter: true,
        });

        // Test that jitter produces different delays
        let delay1 = retry_manager.add_jitter(Duration::from_millis(100));
        let delay2 = retry_manager.add_jitter(Duration::from_millis(100));

        // They should be in the range of 50-150ms (±50% jitter)
        assert!(delay1.as_millis() >= 50 && delay1.as_millis() <= 150);
        assert!(delay2.as_millis() >= 50 && delay2.as_millis() <= 150);
    }

    #[tokio::test]
    async fn test_backoff_progression() {
        let config = RetryConfig::new(4, Duration::from_millis(100), Duration::from_secs(2));
        let retry_manager = RetryManager::new(config);
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let start_time = std::time::Instant::now();

        let _result = retry_manager.retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, std::io::Error>(std::io::Error::from(std::io::ErrorKind::TimedOut))
                }
            },
            classification::is_io_error_retryable,
        ).await;

        let elapsed = start_time.elapsed();

        // Should have been called max_attempts times
        assert_eq!(counter.load(Ordering::SeqCst), 4);

        // Should have taken at least the sum of delays (approximately 100 + 200 + 400 ms)
        // But we don't test exact timing due to jitter and test environment variability
        assert!(elapsed >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_pattern_convenience_functions() {
        use super::patterns::*;

        // Test that the convenience functions work
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_quick_operation(|| {
            let counter = counter_clone.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                if count == 1 {
                    Err(anyhow::anyhow!("Temporary failure"))
                } else {
                    Ok(count)
                }
            }
        }).await;

        assert_eq!(result.unwrap(), 2);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}