use std::env;
use std::future::Future;

/// Centralized demo mode detection
pub fn is_demo_mode() -> bool {
    if let Ok(demo_env) = env::var("USE_MOCK_SSH") {
        return demo_env.to_lowercase() == "true";
    }

    #[cfg(debug_assertions)]
    {
        true
    }

    #[cfg(not(debug_assertions))]
    {
        false
    }
}

/// Set demo mode via environment variable
pub fn set_demo_mode(is_demo: bool) {
    env::set_var("USE_MOCK_SSH", if is_demo { "true" } else { "false" });
}

/// Generic mode switcher for async operations
pub async fn execute_with_mode<T, M, R>(
    demo_impl: M,
    real_impl: R,
) -> T
where
    M: Future<Output = T>,
    R: Future<Output = T>,
{
    if is_demo_mode() {
        demo_impl.await
    } else {
        real_impl.await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_mode_environment_control() {
        set_demo_mode(true);
        assert!(is_demo_mode(), "Should be in demo mode when USE_MOCK_SSH=true");

        set_demo_mode(false);
        assert!(!is_demo_mode(), "Should be in real mode when USE_MOCK_SSH=false");

        env::remove_var("USE_MOCK_SSH");
    }

    #[test]
    fn test_development_defaults() {
        env::remove_var("USE_MOCK_SSH");

        #[cfg(debug_assertions)]
        {
            assert!(is_demo_mode(), "Should default to demo in debug builds");
        }

        #[cfg(not(debug_assertions))]
        {
            assert!(!is_demo_mode(), "Should default to real SSH in release builds");
        }
    }

    #[test]
    fn test_environment_variable_precedence() {
        set_demo_mode(false);
        assert!(!is_demo_mode(), "Environment variable should override build defaults");

        set_demo_mode(true);
        assert!(is_demo_mode(), "Environment variable should override build defaults");

        env::remove_var("USE_MOCK_SSH");
    }

    #[tokio::test]
    async fn test_execute_with_mode() {
        set_demo_mode(true);

        let result = execute_with_mode(
            async { "demo_result" },
            async { "real_result" }
        ).await;

        assert_eq!(result, "demo_result");

        set_demo_mode(false);

        let result = execute_with_mode(
            async { "demo_result" },
            async { "real_result" }
        ).await;

        assert_eq!(result, "real_result");

        env::remove_var("USE_MOCK_SSH");
    }
}
