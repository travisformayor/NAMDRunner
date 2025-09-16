use std::env;
use std::future::Future;


/// Centralized mock mode detection
pub fn is_mock_mode() -> bool {
    // Check explicit environment variable first
    if let Ok(mock_env) = env::var("USE_MOCK_SSH") {
        return mock_env.to_lowercase() == "true";
    }

    // Development mode defaults
    #[cfg(debug_assertions)]
    {
        true // Default to mock in debug
    }

    // In release builds, default to real SSH
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

/// Generic mode switcher that can be used without implementing the trait
pub async fn execute_with_mode<T, M, R>(
    mock_impl: M,
    real_impl: R,
) -> T
where
    M: Future<Output = T>,
    R: Future<Output = T>,
{
    if is_mock_mode() {
        mock_impl.await
    } else {
        real_impl.await
    }
}

/// Macro to simplify creating mode-switched implementations
#[macro_export]
macro_rules! mode_switch {
    ($mock_expr:expr, $real_expr:expr) => {
        $crate::mode_switching::execute_with_mode($mock_expr, $real_expr).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_mock_mode_detection() {
        // Test explicit environment variable
        env::set_var("USE_MOCK_SSH", "true");
        assert!(is_mock_mode());

        env::set_var("USE_MOCK_SSH", "false");
        assert!(!is_mock_mode());

        env::remove_var("USE_MOCK_SSH");

        // Test default behavior based on build mode
        #[cfg(debug_assertions)]
        assert!(is_mock_mode());

        #[cfg(not(debug_assertions))]
        assert!(!is_mock_mode());
    }

    #[tokio::test]
    async fn test_generic_mode_switcher() {
        env::set_var("USE_MOCK_SSH", "true");

        let result = execute_with_mode(
            async { "mock" },
            async { "real" }
        ).await;

        assert_eq!(result, "mock");

        env::set_var("USE_MOCK_SSH", "false");

        let result = execute_with_mode(
            async { "mock" },
            async { "real" }
        ).await;

        assert_eq!(result, "real");
    }

}