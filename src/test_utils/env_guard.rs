#![allow(clippy::disallowed_methods)]

use std::sync::Mutex;

/// A guard for safely managing environment variables in tests.
/// When dropped, it restores the original value of the environment variable.
pub struct EnvGuard {
    key: String,
    original_value: Option<String>,
}

impl EnvGuard {
    /// Set an environment variable for the duration of the test.
    /// The original value is restored when the guard is dropped.
    pub fn set(key: &str, value: &str) -> Self {
        let original_value = std::env::var(key).ok();
        // Use a mutex to ensure thread safety
        static ENV_MUTEX: Mutex<()> = Mutex::new(());
        let _lock = ENV_MUTEX.lock().unwrap();

        // SAFETY: We're in a test context and using a mutex to ensure thread safety
        unsafe {
            std::env::set_var(key, value);
        }

        Self { key: key.to_string(), original_value }
    }

    /// Remove an environment variable for the duration of the test.
    /// The original value is restored when the guard is dropped.
    pub fn remove(key: &str) -> Self {
        let original_value = std::env::var(key).ok();
        // Use a mutex to ensure thread safety
        static ENV_MUTEX: Mutex<()> = Mutex::new(());
        let _lock = ENV_MUTEX.lock().unwrap();

        // SAFETY: We're in a test context and using a mutex to ensure thread safety
        unsafe {
            std::env::remove_var(key);
        }

        Self { key: key.to_string(), original_value }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // Use a mutex to ensure thread safety
        static ENV_MUTEX: Mutex<()> = Mutex::new(());
        let _lock = ENV_MUTEX.lock().unwrap();

        // SAFETY: We're in a test context and using a mutex to ensure thread safety
        unsafe {
            match &self.original_value {
                Some(value) => std::env::set_var(&self.key, value),
                None => std::env::remove_var(&self.key),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_guard_set() {
        let original = std::env::var("TEST_ENV_VAR").ok();
        {
            let _guard = EnvGuard::set("TEST_ENV_VAR", "test_value");
            assert_eq!(std::env::var("TEST_ENV_VAR").unwrap(), "test_value");
        }
        // Guard is dropped, original value should be restored
        assert_eq!(std::env::var("TEST_ENV_VAR").ok(), original);
    }

    #[test]
    fn test_env_guard_remove() {
        // First set a value
        std::env::set_var("TEST_ENV_VAR_2", "initial_value");
        {
            let _guard = EnvGuard::remove("TEST_ENV_VAR_2");
            assert!(std::env::var("TEST_ENV_VAR_2").is_err());
        }
        // Guard is dropped, original value should be restored
        assert_eq!(std::env::var("TEST_ENV_VAR_2").unwrap(), "initial_value");
        // Clean up
        std::env::remove_var("TEST_ENV_VAR_2");
    }
}
